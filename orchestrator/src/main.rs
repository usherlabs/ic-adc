use chrono::Utc;
use dotenv::dotenv;
use tokio_cron_scheduler::JobSchedulerError;
use triggers::cron::load_cron;

pub mod config;
pub mod handlers;
pub mod helpers;
pub mod triggers;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    println!("Orchestrator Started at {}", Utc::now().to_string());

    // load env vars
    dotenv().ok();

    // load the cron and its handlers
    let mut cron = load_cron().await.unwrap();

    // start the cronjob
    cron.start().await?;
    Ok(())
}
