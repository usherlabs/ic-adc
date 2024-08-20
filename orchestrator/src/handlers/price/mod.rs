use crate::{
    config::Config,
    helpers::logs::{ic::get_canister_logs, types::EventLog},
};
use anyhow::Result;
use chrono::prelude::*;
use poller::LogPollerState;
use types::Response;

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

    if latest_valid_logs.len() == 0 {
        return Ok(());
    };
    println!("Processing {} valid logs", latest_valid_logs.len());

    // generate proofs using redstone api and pyth api
    let responses = fetch_pricing_data(latest_valid_logs).await;
    println!("Processed {} valid logs", responses.len());

    let agent = config.get_agent().await?;
    for response in responses {
        // let payload = serde_json::to_string(&response)?;
        agent
            .update(&config.canister, "receive_orchestrator_response")
            .with_arg(candid::encode_args((response,))?)
            .call_and_wait()
            .await?;
    }

    let updated_state = LogPollerState::default();
    updated_state.save_state()?;

    println!("Responses pushed to canister\n");
    Ok(())
}

pub async fn fetch_pricing_data(event_logs: Vec<EventLog>) -> Vec<Response> {
    let mut responses: Vec<Response> = vec![];

    for event in event_logs {
        let request = event.logs.clone();
        let request_options: types::RequestOpts = request.clone().opts;
        let mut price_response = Response::from(request);

        if request_options.price {
            // TODO: error handling for when the price fails to process
            if price_response.process_prices().await.is_ok() {
                responses.push(price_response);
            } else {
                println!("Failed to process pricing data:{:?}", event)
            }
        }

        //TODO: check for other request options to fetch other details
    }

    responses
}
