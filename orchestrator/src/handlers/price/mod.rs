use crate::{
    config::{Config, NotaryInformation},
    helpers::{logs::types::EventLog, utils::get_utc_timestamp},
};
// use anyhow::Result;
use poller::LogPollerState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    result::Result::{self, Ok},
    sync::Arc,
};
use tracing::{debug, error, info};
use types::{ErrorResponse, Response};
use utils::process_proofs;

pub mod poller;
pub mod sources;
pub mod traits;
pub mod utils;

/// Define a default base currency for the price pair when one is nor provided
pub const DEFAULT_BASE_CURRENCY: &str = "USDT";
/// Define a global variable to track whether the program is running already
pub static IS_RUNNING: AtomicBool = AtomicBool::new(false);

pub type ResponseResult = Result<Response, ErrorResponse>;

pub async fn handler(notary_information: Arc<NotaryInformation>, latest_valid_logs: Vec<EventLog>) {
    // if program is already running then return
    if IS_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    // set the running state to true to prevent further instances until this is complete
    IS_RUNNING.store(true, Ordering::SeqCst);

    let fetch_logs_response = process_canister_logs(notary_information, latest_valid_logs).await;

    if let Err(e) = fetch_logs_response {
        error!("Failed to fetch canister logs: {}", e)
    } else {
        // if theres no error then update the last timestamp
        let updated_state = LogPollerState::new(fetch_logs_response.unwrap());
        updated_state.save_state().unwrap();
    }

    // set the running state to false to enable further instances until this is complete
    IS_RUNNING.store(false, Ordering::SeqCst);
}

/// register handlers for several orchestrator programs
pub async fn process_canister_logs(
    notary_information: Arc<NotaryInformation>,
    latest_valid_logs: Vec<EventLog>,
) -> anyhow::Result<u64> {
    let config = Config::env();

    if latest_valid_logs.len() == 0 {
        return Ok(get_utc_timestamp());
    };

    // generate proofs using redstone api and pyth api
    let responses = fetch_pricing_data(latest_valid_logs.clone()).await;

    info!("Processed {} valid price logs", responses.len(),);

    let agent = config.get_agent().await?;
    let notary_pubkey = &notary_information.public_key;

    // TODO: Could be wiser to batch these responses into a single IC update call
    for response in &responses {
        debug!("Pushing response {:?}", response);
        agent
            .update(&config.canister, "receive_orchestrator_response")
            // .with_arg(candid::encode_args((response,))?)
            .with_arg(candid::encode_args((response, notary_pubkey))?)
            .call_and_wait()
            .await?;
    }
    info!("Pushed {} responses to the canister", responses.len());

    // get the latest timestamp from the logs and resume from there
    let latest_log_timestamp = latest_valid_logs.last().unwrap();
    Ok(latest_log_timestamp.timestamp)
}

pub async fn fetch_pricing_data(event_logs: Vec<EventLog>) -> Vec<ResponseResult> {
    let mut responses: Vec<ResponseResult> = vec![];

    for event in event_logs {
        debug!("Processing log #{}: {:?}", event.index, event.logs);

        let request = event.logs.clone();
        let request_options = request.clone().opts;
        let mut price_response = Response::from(request.clone());

        // if the price option is set to true then we should fetch price data
        if request_options.price {
            let process_status = process_proofs(&mut price_response).await;
            match process_status {
                Err(msg) => {
                    error!("Failed to process pricing data:{:?}", msg);
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
