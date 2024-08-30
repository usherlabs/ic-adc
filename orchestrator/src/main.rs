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

#[cfg(test)]
mod tests {
    use handlers::price::fetch_canister_logs;

    use super::*; // Import everything from the parent module

    #[tokio::test]
    async fn test_fetch_canister_logs() {
        //dev: need to be able to test the function without waiting for the cronjob
        let _ = fetch_canister_logs().await;
    }
}
