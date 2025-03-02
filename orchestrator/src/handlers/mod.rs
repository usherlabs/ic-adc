use price::poller::LogPollerState;

pub mod price;
pub mod url;
use std::sync::Arc;

use crate::{config::{Config, NotaryInformation}, helpers::logs::ic::get_canister_logs};


pub async fn batch_handler(notary_information: Arc<NotaryInformation>)->() {
    let notary_information = Arc::clone(&notary_information);
    let state = LogPollerState::load_state().expect("State Load");

    let config = Config::env();

    // get all the logs which meet this criteria
    let (latest_valid_logs, latest_valid_url_logs) = get_canister_logs(&config, Some(state.start_timestamp)).await.expect("canister Log");

    // println!("{:?}\n{:?}",&latest_valid_logs, &latest_valid_url_logs);
    url::handler(notary_information.clone(), latest_valid_url_logs).await;
    price::handler(notary_information,latest_valid_logs).await;
}