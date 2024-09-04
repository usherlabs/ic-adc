use crate::{
    config::Config,
    helpers::logs::{ic::get_canister_logs, types::EventLog},
};
// use anyhow::Result;
use chrono::prelude::*;
use poller::LogPollerState;
use std::result::Result::{self, Ok};
use std::sync::atomic::{AtomicBool, Ordering};
use types::{ErrorResponse, Response};

pub mod poller;
pub mod sources;
pub mod traits;
pub mod types;

/// Define a default base currency for the price pair when one is nor provided
pub const DEFAULT_BASE_CURRENCY: &str = "USDT";
/// Define a global variable to track whether the program is running already
pub static IS_RUNNING: AtomicBool = AtomicBool::new(false);

pub type ResponseResult = Result<Response, ErrorResponse>;

pub async fn handler() {
    // if program is already running then return
    if IS_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    // set the running state to true to prevent further instances untill this is complete
    IS_RUNNING.store(true, Ordering::SeqCst);

    let fetch_logs_response = fetch_canister_logs().await;

    if let Err(e) = fetch_logs_response {
        println!("Error fetching canister logs: {}", e)
    } else {
        // if theres no error then update the last timestamp
        let updated_state = LogPollerState::default();
        updated_state.save_state().unwrap();
    }

    // set the running state to false to enable further instances untill this is complete
    IS_RUNNING.store(false, Ordering::SeqCst);
}

/// register handlers for several orchestrator programs
pub async fn fetch_canister_logs() -> anyhow::Result<()> {
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
    // TODO: Could be wiser to batch these responses into a single IC update call
    for response in responses {
        agent
            .update(&config.canister, "receive_orchestrator_response")
            .with_arg(candid::encode_args((response,))?)
            .call_and_wait()
            .await?;
    }

    println!("Responses pushed to canister\n");
    Ok(())
}

pub async fn fetch_pricing_data(event_logs: Vec<EventLog>) -> Vec<ResponseResult> {
    let mut responses: Vec<ResponseResult> = vec![];

    for event in event_logs {
        let request = event.logs.clone();
        let request_options: types::RequestOpts = request.clone().opts;
        let mut price_response = Response::from(request.clone());

        // if the price option is set to true then we should
        if request_options.price {
            let process_status = price_response.process_prices().await;
            match process_status {
                Err(msg) => {
                    println!("Failed to process pricing data:{:?}", msg);
                    // on error we push an error response to the canister
                    responses.push(Err(ErrorResponse::new(
                        request.id,
                        request.owner,
                        msg.to_string(),
                    )));
                }
                Ok(_) => responses.push(Ok(price_response)),
            };
        }

        //TODO: check for other request options to fetch other details
    }

    responses
}
