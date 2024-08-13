use crate::{
    config::Config,
    helpers::logs::{ic::get_canister_logs, types::EventLog},
};
use chrono::prelude::*;
use poller::LogPollerState;
use types::{CurrencyPair, PriceResponse};

pub mod poller;
pub mod sources;
pub mod traits;
pub mod types;

pub const DEFAULT_BASE_CURRENCY: &str = "USDT";

/// register handlers for several orchestrator programs
pub async fn fetch_canister_logs() {
    println!(
        "Running 'fetch_canister_logs' at {}",
        Utc::now().to_string()
    );
    let state = LogPollerState::load_state().unwrap();
    let config = Config::get_and_persist(&None).unwrap();

    let latest_valid_logs: Vec<EventLog> = get_canister_logs(&config, Some(state.start_timestamp))
        .await
        .unwrap();

    // get all the logs which meet this criteria
    println!("logs: {:?}", latest_valid_logs);
    // generate proofs using redstone api and pyth api
    let responses = fetch_pricing_data(latest_valid_logs).await;
    println!("responses{:?}", responses);
    // TODO: send proofs to canister

    let updated_state = LogPollerState::default();
    updated_state.save_state().unwrap();
}

pub async fn fetch_pricing_data(event_logs: Vec<EventLog>) -> Vec<PriceResponse> {
    let mut responses: Vec<PriceResponse> = vec![];

    for event in event_logs {
        let price_request = event.logs.clone();
        let mut price_response = PriceResponse::from(price_request);

        // TODO: error handling for when the price fails to process
        if price_response.process_prices().await.is_ok() {
            responses.push(price_response);
        } else {
            // TODO: each request coming from the canister should have an identifier
            println!("Failed to process pricing data:{:?}", event)
        }
    }

    responses
}
