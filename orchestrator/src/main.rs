use dotenv::dotenv;
use tokio_cron_scheduler::JobSchedulerError;
use tracing::info;
use triggers::cron::load_cron;

pub mod config;
pub mod handlers;
pub mod helpers;
pub mod triggers;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    tracing_subscriber::fmt::init();

    info!("Orchestrator Started");

    // load env vars
    dotenv().ok();

    // load the cron and its handlers
    let mut cron = load_cron().await.unwrap();

    // start the cronjob
    cron.start().await?;
    Ok(())
}
