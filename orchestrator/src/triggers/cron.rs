use crate::{
    config::Config,
    handlers::price::{self},
    helpers::cron::CronJob,
};
use tokio_cron_scheduler::{Job, JobSchedulerError};

pub async fn load_cron() -> Result<CronJob, JobSchedulerError> {
    let cronjob = CronJob::new().await;
    let config: Config = Config::env();

    // add jobs to the cronjob
    cronjob
        .add_job(Job::new_async(&config.job_schedule[..], |_, _| {
            Box::pin(async { price::handler().await })
        })?)
        .await?;
    // add jobs to the cronjob

    return Ok(cronjob);
}
