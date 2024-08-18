use crate::{
    config::Config,
    helpers::logs::{ic::get_canister_logs, types::EventLog},
};
use anyhow::Result;
use chrono::prelude::*;
use poller::LogPollerState;
use types::PriceResponse;

pub mod poller;
pub mod sources;
pub mod traits;
pub mod types;

pub const DEFAULT_BASE_CURRENCY: &str = "USDT";

/// register handlers for several orchestrator programs
pub async fn fetch_canister_logs() -> Result<()> {
    println!(
        "Running 'fetch_canister_logs' at {}",
        Utc::now().to_string()
    );
    let state = LogPollerState::load_state()?;
    let config = Config::get_and_persist(&None)?;

    // get all the logs which meet this criteria
    let latest_valid_logs: Vec<EventLog> =
        get_canister_logs(&config, Some(state.start_timestamp)).await?;
    println!("Processing {} valid logs", latest_valid_logs.len());

    // generate proofs using redstone api and pyth api
    let responses = fetch_pricing_data(latest_valid_logs).await;
    println!("Processed {} valid logs", responses.len());

    if responses.len() == 0 {return Ok(())}

    let agent = config.get_agent().await?;
    for response in responses {
        // let payload = serde_json::to_string(&response)?;
        agent
            .update(&response.owner, "receive_price_response")
            .with_arg(candid::encode_args((response,))?)
            .call_and_wait()
            .await?;
    }

    let updated_state = LogPollerState::default();
    updated_state.save_state()?;

    //TODO: use tracing instead of println
    println!("Responses pushed to canister");
    Ok(())
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
            println!("Failed to process pricing data:{:?}", event)
        }
    }

    responses
}
