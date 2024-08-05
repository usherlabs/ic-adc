use triggers::cron::load_cron;
use tokio_cron_scheduler::JobSchedulerError;

pub mod utils;
pub mod  handlers;
pub mod triggers;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    let mut cron = load_cron().await.unwrap();
    
    
    // start the cronjob
    cron.start().await?;

    Ok(())
}
