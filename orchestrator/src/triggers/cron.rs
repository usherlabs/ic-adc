use std::sync::Arc;

use crate::{
    config::Config,
    handlers::price::{self},
    helpers::cron::CronJob,
};
use tokio_cron_scheduler::Job;

pub async fn load_cron() -> anyhow::Result<CronJob> {
    let cronjob = CronJob::new().await;
    let config: Config = Config::env();
    // get the connected notary public key here and pass it to the price handler
    let notary_information = Arc::new(config.get_connected_notary().await?.clone());

    // add jobs to the cronjob
    cronjob
        .add_job(Job::new_async(&config.job_schedule[..], move |_, _| {
            let notary_information = Arc::clone(&notary_information);
            Box::pin(async { price::handler(notary_information).await })
        })?)
        .await?;
    // add jobs to the cronjob

    return Ok(cronjob);
}
