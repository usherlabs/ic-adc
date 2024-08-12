use crate::{config::Config, helpers::logs::ic::get_canister_logs};
use chrono::prelude::*;
use poller::LogPollerState;

pub mod poller;
pub mod types;

/// register handlers for several orchestrator programs
pub async fn fetch_canister_logs() {
    println!(
        "Running 'fetch_canister_logs' at {}",
        Utc::now().to_string()
    );
    let state = LogPollerState::load_state().unwrap();
    let config = Config::get_and_persist(&None).unwrap();

    let latest_logs = get_canister_logs( &config, Some(state.start_timestamp))
        .await
        .unwrap();

    // get all the logs which meet this criteria
    println!("logs: {:?}", latest_logs);
    // TODO: parse the logs to extract pricing information
    // TODO: generate proofs using redstone api and other api
    // TODO: send proofs to canister

    let updated_state = LogPollerState::default();
    updated_state.save_state().unwrap();
    // println!("updated_state: {:?}", updated_state);
    // println!("\n\n\n");
}
