/* SPDX-License-Identifier: GPL-3.0-or-later */
/* Copyright © 2026 Eduard Smet */

use std::{collections::HashMap, fmt::Write};

use fjall::{Guard, KeyspaceCreateOptions};
use tokio::sync::oneshot::channel;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::{
    Shutdown,
    config::plugins::permissions::core::PluginPermissionsCore,
    runtime::{
        internal::InternalRuntime,
        plugins::wpbs::plugin::{
            core_import_functions::Host as CoreImportFunctionsHost,
            core_import_types::{
                CoreRegistrations, CoreRegistrationsResult, Deregistrations, DeregistrationsResult,
                Host as CoreImportTypesHost, LogLevels, Registrations, RegistrationsResult,
            },
            core_types::{Host as CoreTypesHost, HostError},
        },
    },
    utils::channels::{CoreMessages, RuntimeMessages, RuntimeMessagesCore},
};

impl CoreTypesHost for InternalRuntime {}
impl CoreImportTypesHost for InternalRuntime {}

impl CoreImportFunctionsHost for InternalRuntime {
    async fn log(&mut self, level: LogLevels, message: String) {
        match level {
            LogLevels::Trace => trace!("[{}]: {message}", self.metadata.user_id),
            LogLevels::Debug => debug!("[{}]: {message}", self.metadata.user_id),
            LogLevels::Info => info!("[{}]: {message}", self.metadata.user_id),
            LogLevels::Warn => warn!("[{}]: {message}", self.metadata.user_id),
            LogLevels::Error => error!("[{}]: {message}", self.metadata.user_id),
        }
    }

    async fn get_state(&mut self, key: String) -> Result<Option<Vec<u8>>, HostError> {
        let key = format!("{}:{key}", self.metadata.plugin_uuid);

        let plugin_store_keyspace = self
            .database
            .keyspace("plugin_store", KeyspaceCreateOptions::default)
            .map_err(|err| err.to_string())?;

        plugin_store_keyspace
            .get(&key)
            .map_err(|err| err.to_string())
            .map(|r| r.map(|s| s.to_vec()))
    }

    async fn set_state(&mut self, key: String, value: Vec<u8>) -> Result<(), HostError> {
        let key = format!("{}:{key}", self.metadata.plugin_uuid);

        let plugin_store_keyspace = self
            .database
            .keyspace("plugin_store", KeyspaceCreateOptions::default)
            .map_err(|err| err.to_string())?;

        plugin_store_keyspace
            .insert(&key, &value)
            .map_err(|err| err.to_string())
    }

    async fn clear_state(&mut self) -> Result<(), HostError> {
        let plugin_store_keyspace = self
            .database
            .keyspace("plugin_store", KeyspaceCreateOptions::default)
            .map_err(|err| err.to_string())?;

        let entries = plugin_store_keyspace.prefix(self.metadata.plugin_uuid.as_bytes());

        for entry in entries {
            plugin_store_keyspace
                .remove(entry.key().map_err(|err| err.to_string())?)
                .map_err(|err| err.to_string())?;
        }

        Ok(())
    }

    async fn register(&mut self, registrations: Registrations) -> RegistrationsResult {
        let core_registrations_result = registrations.core.map(|cr| self.register_core(cr));

        let services_registrations_result =
            if let Some(services_registrations) = registrations.services {
                Some(
                    Self::register_services(
                        self.database.clone(),
                        self.core_tx.clone(),
                        self.metadata.clone(),
                        services_registrations,
                    )
                    .await,
                )
            } else {
                None
            };

        RegistrationsResult {
            core: core_registrations_result,
            services: services_registrations_result,
        }
    }

    // TODO: Implement
    async fn deregister(&mut self, _deregistrations: Deregistrations) -> DeregistrationsResult {
        DeregistrationsResult {
            core: None,
            services: None,
        }
    }

    async fn remove(&mut self, reason: String) {
        if self
            .core_tx
            .send(CoreMessages::Runtime(RuntimeMessages::Core(
                RuntimeMessagesCore::RemovePlugin(self.metadata.plugin_uuid),
            )))
            .is_ok()
        {
            info!(
                "The {} plugin has unloaded itself, reason: {reason}",
                self.metadata.user_id
            );
        }
    }

    async fn shutdown(&mut self, restart: bool) -> Result<(), HostError> {
        if !self
            .metadata
            .permissions
            .core
            .contains(&PluginPermissionsCore::Shutdown)
        {
            return Err(HostError::from("Not allowed to call shutdown"));
        }

        let shutdown_kind = if restart {
            Shutdown::Restart
        } else {
            Shutdown::Normal
        };

        self.core_tx
            .send(CoreMessages::Shutdown(shutdown_kind))
            .unwrap();

        Ok(())
    }

    async fn dependency_function(
        &mut self,
        registry_id: String,
        plugin_id: String,
        function_id: String,
        plugin_version: Option<String>,
        params: Vec<u8>,
    ) -> Result<Vec<u8>, HostError> {
        let mut signature = format!("{registry_id}:{plugin_id}:{function_id}@");

        if let Some(plugin_version) = plugin_version {
            write!(signature, "{plugin_version}").unwrap();
        }

        let dependency_functions_keyspace = self
            .database
            .keyspace("dependency_functions", KeyspaceCreateOptions::default)
            .map_err(|err| err.to_string())?;

        let Some(plugin_uuid_bytes) = dependency_functions_keyspace
            .prefix(&signature)
            .next()
            .map(Guard::value)
            .transpose()
            .map_err(|err| err.to_string())?
        else {
            return Err(format!("The {signature} dependency function was not found"));
        };

        let (sender, receiver) = channel();

        let _ = self
            .core_tx
            .send(CoreMessages::Runtime(RuntimeMessages::Core(
                RuntimeMessagesCore::CallDependencyFunction(
                    Uuid::from_slice(&plugin_uuid_bytes).unwrap(),
                    signature,
                    params,
                    sender,
                ),
            )));

        receiver
            .await
            .unwrap_or(Err(HostError::from("Runtime is shutting down")))
    }
}

impl InternalRuntime {
    fn register_core(&self, core_registrations: CoreRegistrations) -> CoreRegistrationsResult {
        let dependency_functions = core_registrations
            .dependency_functions
            .map(|dfr| self.register_dependency_functions(dfr));

        CoreRegistrationsResult {
            dependency_functions,
        }
    }

    fn register_dependency_functions(
        &self,
        dependency_function_registrations: Vec<String>,
    ) -> Result<HashMap<String, String>, HostError> {
        if !self
            .metadata
            .permissions
            .core
            .contains(&PluginPermissionsCore::DependencyFunctions)
        {
            return Err(HostError::from(
                "Plugin is not allowed to register dependency functions",
            ));
        }

        let mut dependency_function_registrations_result = HashMap::new();

        let dependency_functions_keyspace = self
            .database
            .keyspace("dependency_functions", KeyspaceCreateOptions::default)
            .map_err(|err| err.to_string())?;

        for dependency_function_registration in dependency_function_registrations {
            let signature = format!(
                "{}:{}:{dependency_function_registration}@{}",
                self.metadata.namespace_id, self.metadata.plugin_id, self.metadata.version
            );

            dependency_functions_keyspace
                .insert(&signature, self.metadata.plugin_uuid.as_bytes())
                .map_err(|err| err.to_string())?;

            dependency_function_registrations_result
                .insert(dependency_function_registration, signature);
        }

        Ok(dependency_function_registrations_result)
    }
}
