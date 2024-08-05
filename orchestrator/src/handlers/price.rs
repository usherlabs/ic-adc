use std::fs;
use uuid::Uuid;
use tokio_cron_scheduler::JobScheduler;


// register handlers for several orchestrator programs
pub fn fetch_price_data(_uuid: Uuid, _l: JobScheduler){
    println!("I run every 10 seconds");
    fs::write("example.txt", b"This is an example.").unwrap();
}