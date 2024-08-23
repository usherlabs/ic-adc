use crate::{
    config::Config,
    handlers::price::{fetch_canister_logs, poller::LogPollerState},
    helpers::cron::CronJob,
};
use tokio_cron_scheduler::{Job, JobSchedulerError};

pub async fn load_cron() -> Result<CronJob, JobSchedulerError> {
    let cronjob = CronJob::new().await;
    let config: Config = Config::env();

    // add jobs to the cronjob
    cronjob
        .add_job(Job::new_async(&config.job_schedule[..], |_, _| {
            Box::pin(async {
                let res = fetch_canister_logs().await;
                if let Err(e) = res {
                    println!("Error fetching canister logs: {}", e);
                    LogPollerState::load_state()
                        .unwrap()
                        .unlock_state()
                        .unwrap();
                }
                // if there is an error then the state would still be locked.
                // so we unlock it here
            })
        })?)
        .await?;
    // add jobs to the cronjob

    return Ok(cronjob);
}
