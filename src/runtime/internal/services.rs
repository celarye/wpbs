/* SPDX-License-Identifier: GPL-3.0-or-later */
/* Copyright © 2026 Eduard Smet */

use std::sync::Arc;

use fjall::Database;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    runtime::{
        internal::InternalRuntime,
        plugins::{
            RuntimePluginMetadata,
            wpbs::plugin::core_import_types::{ServicesRegistrations, ServicesRegistrationsResult},
        },
    },
    utils::channels::CoreMessages,
};

mod discord;
mod job_scheduler;

impl InternalRuntime {
    pub async fn register_services(
        database: Database,
        core_tx: UnboundedSender<CoreMessages>,
        plugin_metadata: Arc<RuntimePluginMetadata>,
        services_registrations: ServicesRegistrations,
    ) -> ServicesRegistrationsResult {
        let job_scheduler =
            if let Some(job_scheduler_registrations) = services_registrations.job_scheduler {
                Some(
                    Self::register_job_scheduler(
                        core_tx,
                        plugin_metadata.clone(),
                        job_scheduler_registrations,
                    )
                    .await,
                )
            } else {
                None
            };

        let discord = if let Some(discord_registrations) = services_registrations.discord {
            Some(Self::register_discord(database, plugin_metadata, discord_registrations).await)
        } else {
            None
        };

        ServicesRegistrationsResult {
            job_scheduler,
            discord,
        }
    }
}
