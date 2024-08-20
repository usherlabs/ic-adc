use candid::Principal;
use core::panic;
use ic_cdk::{api::time, println};
use ic_cdk_macros::*;
use std::{cell::RefCell, collections::HashMap};
use types::{Request, RequestOpts, Response};
use verity_dp_ic::{owner, whitelist};

pub mod types;

thread_local! {
    static REQUEST_RESPONSE_BUFFER: RefCell<HashMap<String, bool>> = RefCell::default();
}

// @dev testing command
#[query]
fn name() -> String {
    format!("processor canister")
}

#[init]
async fn init() {
    owner::init_owner()
}

#[update]
async fn add_to_whitelist(principal: Principal) {
    owner::only_owner();
    whitelist::add_to_whitelist(principal)
}

#[update]
async fn remove_from_whitelist(principal: Principal) {
    owner::only_owner();
    whitelist::add_to_whitelist(principal)
}

#[update]
/// requests prices from the orchestrator
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT"
/// @dev? is the person requesting the prices supposed to provide the prices
fn request_data(currency_pairs: String, opts: RequestOpts) -> String {
    let caller_principal = ic_cdk::caller();
    let request_id = time().to_string();
    if !whitelist::is_whitelisted(caller_principal) {
        panic!(
            "canister with principal:{} is not allowed to call this method",
            caller_principal
        );
    }
    // creates a price request object with an arb id
    // attach a buffer with valid pending id's
    // include the caller canister's id to know who to send a response to
    let price_request = Request::new(request_id.clone(), caller_principal, currency_pairs, opts);

    let price_request_stringified = serde_json::to_string(&price_request).unwrap();
    // log the price request to be picked up by the orchestrator
    println!("{}", price_request_stringified);

    REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow_mut().insert(request_id.clone(), true));

    return request_id;
}

#[update]
/// this function is going to be called by the orchestrator which would be authenticated with the 'owner' keys
/// it would receive the response for a request made and forward it to the requesting canister
async fn receive_orchestrator_response(response: Response) {
    // only owner(orchestrator) can call
    owner::only_owner();
    // validate that id is present in buffer
    if !REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow().contains_key(&response.id)) {
        panic!("invalid response")
    }
    // remove ID from buffer
    REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow_mut().remove(&response.id));

    // call function and get response
    let _call_result: Result<(), _> =
        ic_cdk::call(response.owner, "receive_adc_response", (response,)).await;
}

#[query]
async fn is_canister_whitelisted(principal: Principal) -> bool {
    owner::only_owner();
    whitelist::is_whitelisted(principal)
}
