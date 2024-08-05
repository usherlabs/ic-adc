use std::time::Duration;

use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

#[derive(Clone)]
pub struct CronJob {
    pub scheduler: JobScheduler,
}

impl CronJob {
    pub async fn new() -> Self {
        CronJob {
            scheduler: JobScheduler::new().await.unwrap(),
        }
    }

    pub async fn add_job(&self, new_job: Job) -> Result<String, JobSchedulerError> {
        let res = self.scheduler.add(new_job).await?;

        Ok(res.to_string())
    }

    pub async fn start(&mut self) -> Result<(), JobSchedulerError> {
        // add a shutdown hook
        self.scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                println!("shutting down scheduler");
            })
        }));

        // Start the scheduler
        self.scheduler.start().await?;

        // start an infinite loop to keep the scheduler running
        // TODO: Figure out a way to keep the program alive or move this loop to a more centralized location
        loop {
            sleep(Duration::from_secs(60)).await; 
        };
        
    }
}
