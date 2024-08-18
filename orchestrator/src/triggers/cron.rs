use crate::{handlers::price::fetch_canister_logs, helpers::cron::CronJob};
use tokio_cron_scheduler::{Job, JobSchedulerError};

pub async fn load_cron() -> Result<CronJob, JobSchedulerError> {
    let cronjob = CronJob::new().await;

    // add jobs to the cronjob
    cronjob
        .add_job(Job::new_async("1/60 * * * * *", |_, _| {
            Box::pin(async {
                let res = fetch_canister_logs().await;
                if let Err(e) = res {
                    println!("Error fetching canister logs: {}", e);
                }
            })
        })?)
        .await?;
    // add jobs to the cronjob

    return Ok(cronjob);
}
