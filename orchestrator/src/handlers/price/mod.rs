use crate::{
    config::Config,
    helpers::logs::{ic::get_canister_logs, types::EventLog},
};
use anyhow::Result;
use chrono::prelude::*;
use poller::LogPollerState;
use std::result::Result::Ok;
use std::sync::atomic::{AtomicBool, Ordering};
use types::Response;

pub mod poller;
pub mod sources;
pub mod traits;
pub mod types;

/// Define a default base currency for the price pair when one is nor provided
pub const DEFAULT_BASE_CURRENCY: &str = "USDT";
/// Define a global variable to track whether the program is running already
pub static IS_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn handler() {
    // if program is already running then return
    if IS_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    // set the running state to true to prevent further instances untill this is complete
    IS_RUNNING.store(true, Ordering::SeqCst);

    let fetch_logs_response = fetch_canister_logs().await;
    // set the running state to false to enable further instances untill this is complete
    IS_RUNNING.store(false, Ordering::SeqCst);

    if let Err(e) = fetch_logs_response {
        println!("Error fetching canister logs: {}", e)
    }
}

/// register handlers for several orchestrator programs
pub async fn fetch_canister_logs() -> Result<()> {
    let state = LogPollerState::load_state()?;

    let config = Config::env();
    if config.is_dev {
        println!(
            "Running 'fetch_canister_logs' at {}",
            Utc::now().to_string()
        );
    }

    // get all the logs which meet this criteria
    let latest_valid_logs: Vec<EventLog> =
        get_canister_logs(&config, Some(state.start_timestamp)).await?;

    if latest_valid_logs.len() == 0 {
        return Ok(());
    };
    println!(
        "Processing {} valid logs at {}",
        latest_valid_logs.len(),
        Utc::now().to_string()
    );

    // generate proofs using redstone api and pyth api
    let responses = fetch_pricing_data(latest_valid_logs).await;
    println!(
        "Processed {} valid logs at {}",
        responses.len(),
        Utc::now().to_string()
    );

    let agent = config.get_agent().await?;
    for response in responses {
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
            let process_status = price_response.process_prices().await;
            match process_status {
                // TODO: error handling for when the price fails to process
                Err(msg) => {
                    println!("Failed to process pricing data:{:?}", msg)
                }
                Ok(_) => responses.push(price_response),
            };
        }

        //TODO: check for other request options to fetch other details
    }

    responses
}
