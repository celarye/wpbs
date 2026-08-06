/* SPDX-License-Identifier: GPL-3.0-or-later */
/* Copyright © 2026 Eduard Smet */

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc::UnboundedSender, oneshot::channel};

use crate::{
    TASKS,
    config::plugins::permissions::services::job_scheduler::PluginPermissionsJobScheduler,
    runtime::{
        internal::InternalRuntime,
        plugins::{
            RuntimePluginMetadata,
            wpbs::plugin::{
                core_import_functions::HostError,
                job_scheduler_import_types::{
                    Host as JobSchedulerImportTypesHost, JobSchedulerRegistrations,
                    JobSchedulerRegistrationsResult,
                },
            },
        },
    },
    utils::channels::{CoreMessages, JobSchedulerMessages},
};

impl InternalRuntime {
    pub async fn register_job_scheduler(
        core_tx: UnboundedSender<CoreMessages>,
        plugin_metadata: Arc<RuntimePluginMetadata>,
        job_scheduler_registrations: JobSchedulerRegistrations,
    ) -> Result<JobSchedulerRegistrationsResult, HostError> {
        if TASKS.read().await.services.job_scheduler.is_none() {
            return Err(HostError::from("The job scheduler service is disabled"));
        }

        let scheduled_jobs_registrations_result =
            if let Some(scheduled_job_registrations) = job_scheduler_registrations.scheduled_jobs {
                if plugin_metadata
                    .permissions
                    .services
                    .job_scheduler
                    .contains(&PluginPermissionsJobScheduler::ScheduledJobs)
                {
                    let mut scheduled_job_registrations_result = HashMap::new();

                    for scheduled_job_registration in scheduled_job_registrations {
                        let (sender, receiver) = channel();

                        core_tx
                            .send(CoreMessages::JobScheduler(JobSchedulerMessages::AddJob(
                                plugin_metadata.plugin_uuid,
                                scheduled_job_registration.clone(),
                                sender,
                            )))
                            .unwrap();

                        let job_scheduler_result = receiver
                            .await
                            .unwrap()
                            .map(|job_uuid| job_uuid.to_string())
                            .map_err(|err| err.to_string());

                        scheduled_job_registrations_result
                            .insert(scheduled_job_registration, job_scheduler_result);
                    }

                    Some(Ok(scheduled_job_registrations_result))
                } else {
                    Some(Err(HostError::from(
                        "Plugin is not allowed to register scheduled jobs",
                    )))
                }
            } else {
                None
            };

        Ok(JobSchedulerRegistrationsResult {
            scheduled_jobs: scheduled_jobs_registrations_result,
        })
    }
}

impl JobSchedulerImportTypesHost for InternalRuntime {}
