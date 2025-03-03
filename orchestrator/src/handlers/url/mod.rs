use crate::{
    config::{Config, NotaryInformation},
    handlers::price::poller::LogPollerState,
    helpers::{
        logs::types::EventUrlLog,
        utils::get_utc_timestamp,
        verity::get_verity_client,
    },
};
use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}};
use std::{
    result::Result::{self, Ok},
    sync::Arc,
};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, error, info};
use types::{ErrorResponse, ResponseV2};

/// Define a default base currency for the price pair when one is nor provided
pub const DEFAULT_BASE_CURRENCY: &str = "USDT";
/// Define a global variable to track whether the program is running already
pub static IS_RUNNING: AtomicBool = AtomicBool::new(false);

pub type ResponseResult = Result<ResponseV2, ErrorResponse>;

pub async fn handler(
    notary_information: Arc<NotaryInformation>,
    latest_valid_logs: Vec<EventUrlLog>,
) {
    // if program is already running then return
    if IS_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    // set the running state to true to prevent further instances until this is complete
    IS_RUNNING.store(true, Ordering::SeqCst);

    let fetch_logs_response = process_canister_logs(notary_information, latest_valid_logs).await;

    if let Err(e) = fetch_logs_response {
        error!("Failed to fetch canister logs: {}", e)
    }

    // set the running state to false to enable further instances until this is complete
    IS_RUNNING.store(false, Ordering::SeqCst);
}

/// register handlers for several orchestrator programs
pub async fn process_canister_logs(
    notary_information: Arc<NotaryInformation>,
    latest_valid_logs: Vec<EventUrlLog>,
) -> anyhow::Result<u64> {
    let state = LogPollerState::load_state()?;

    let config = Config::env();
    let start_timestamp =
        chrono::DateTime::from_timestamp(i64::try_from(state.start_timestamp)?, 0).unwrap();
    debug!("Fetching canister logs since {:?}", start_timestamp);

    debug!("Fetched {} valid logs", latest_valid_logs.len());

    if latest_valid_logs.len() == 0 {
        return Ok(get_utc_timestamp());
    };

    // generate proofs using redstone api and pyth api
    let responses = resolve_data(latest_valid_logs.clone()).await;

    info!("Processed {} valid logs", responses.len(),);

    let agent = config.get_agent().await?;
    let notary_pubkey = &notary_information.public_key;

    // TODO: Could be wiser to batch these responses into a single IC update call
    for response in &responses {
        debug!("Pushing response {:?}", response);
        agent
            .update(&config.canister, "receive_orchestrator_data")
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

pub async fn resolve_data(event_logs: Vec<EventUrlLog>) -> Vec<ResponseResult> {
    let mut responses: Vec<ResponseResult> = vec![];

    for event in event_logs {
        debug!("Processing log #{}: {:?}", event.index, event.logs);

        let request = event.logs.clone();
        let request_options = request.clone();
        let mut price_response = ResponseV2::from(request.clone());

        // if the price option is set to true then we should fetch price data
        if !request_options.target_url.is_empty() {
            let verity = get_verity_client();

            let mut headers = HeaderMap::new();

            for header in request.headers {
                
                // Parse the key into a HeaderName and the value into a HeaderValue
                if let (Ok(header_name), Ok(header_value)) = (header.key.as_str().parse::<HeaderName>(), header.value.parse::<HeaderValue>()) {
                    headers.insert(header_name, header_value);
                } else {
                    eprintln!("Invalid header: {} -> {}", header.key, header.value);
                }
            }
            let process_status = if request.method.to_lowercase() == "get" {
                verity.get(request.target_url).body(request.body).redact(request.redacted).headers(headers).send().await
            } else {
                verity.post(request.target_url).body(request.body).redact(request.redacted).headers(headers).send().await
            };
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
                Ok(verify_response) => {
                    price_response.proof_requests=vec![verify_response.proof];

                    responses.push(Ok(price_response))
                },
            };
        }

        //TODO: check for other request options to fetch other details
    }

    responses
}
