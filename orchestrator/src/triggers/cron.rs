use tokio_cron_scheduler::{JobSchedulerError, Job};
use crate::{handlers::price::fetch_price_data, utils::cron::CronJob};

pub async fn load_cron() -> Result<CronJob, JobSchedulerError> {
    let cronjob = CronJob::new().await;

    // add jobs to the cronjob
    cronjob
        .add_job(Job::new("1/10 * * * * *", fetch_price_data)?)
        .await?;
    // add jobs to the cronjob

    return Ok(cronjob)
}
