use candid::Principal;
use core::panic;
use hex;
use ic_cdk::api::time;
use ic_cdk::{println, storage};
use std::{cell::RefCell, collections::HashMap};
use types::{ErrorResponse, Request, RequestOpts, Response};
use verity_dp_ic::{owner, whitelist};

pub mod types;

thread_local! {
    static REQUEST_RESPONSE_BUFFER: RefCell<HashMap<String, bool>> = RefCell::default();
}

/// use this variable to control the max number of currencypairs
/// that can be contained in one request
const REQUEST_CURRENCY_PAIR_LIMIT: usize = 10;

// @dev testing command
#[ic_cdk::query]
fn name() -> String {
    format!("adc")
}

#[ic_cdk::init]
async fn init() {
    owner::init_owner()
}

#[ic_cdk::update]
async fn add_to_whitelist(principal: Principal) {
    owner::only_owner();
    whitelist::add_to_whitelist(principal)
}

#[ic_cdk::update]
async fn remove_from_whitelist(principal: Principal) {
    owner::only_owner();
    whitelist::add_to_whitelist(principal)
}

#[ic_cdk::update]
/// requests prices from the orchestrator
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT"
/// @dev? is the person requesting the prices supposed to provide the prices
async fn request_data(currency_pairs: String, opts: RequestOpts) -> String {
    let caller_principal = ic_cdk::caller();

    // derive the request id
    let (random_bytes,): (Vec<u8>,) =
        ic_cdk::call(Principal::management_canister(), "raw_rand", ())
            .await
            .unwrap();
    let random_hex_byte: String = hex::encode(random_bytes)
        .get(0..5)
        .unwrap_or_default()
        .to_string();
    let request_id = format!("{}_{}", time().to_string(), random_hex_byte);

    println!("{request_id}");
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

    // validate that this request for data contains a maximum of 10 pairs
    if price_request.pairs.len() > REQUEST_CURRENCY_PAIR_LIMIT {
        panic!(
            "Number of pairs requested must not be more than {}",
            REQUEST_CURRENCY_PAIR_LIMIT
        );
    };
    let price_request_stringified = serde_json::to_string(&price_request).unwrap();
    // log the price request to be picked up by the orchestrator
    println!("{}", price_request_stringified);

    REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow_mut().insert(request_id.clone(), true));

    return request_id;
}

#[ic_cdk::update]
/// this function is going to be called by the orchestrator which would be authenticated with the 'owner' keys
/// it would receive the response for a request made and forward it to the requesting canister
async fn receive_orchestrator_response(response: Result<Response, ErrorResponse>) {
    // only owner(orchestrator) can call
    owner::only_owner();

    let (response_owner, id) = match response.clone() {
        Ok(Response { owner, id, .. }) => (owner, id),
        Err(ErrorResponse { owner, id, .. }) => (owner, id),
    };

    // validate that id is present in buffer
    if !REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow().contains_key(&id)) {
        panic!("invalid response")
    }
    // remove ID from buffer
    REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow_mut().remove(&id));

    // call function and get response
    let _call_result: Result<(), _> =
        ic_cdk::call(response_owner, "receive_adc_response", (response,)).await;
}

#[ic_cdk::query]
async fn is_canister_whitelisted(principal: Principal) -> bool {
    owner::only_owner();
    whitelist::is_whitelisted(principal)
}

// --------------------------- upgrade hooks ------------------------- //
#[ic_cdk::pre_upgrade]
/// backuo
fn pre_upgrade() {
    let cloned_buffer = REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow().clone());
    let cloned_whitelist = whitelist::WHITE_LIST.with(|rc| rc.borrow().clone());

    storage::stable_save((cloned_buffer, cloned_whitelist)).unwrap()
}
#[ic_cdk::post_upgrade]
async fn post_upgrade() {
    let (cached_buffer, cached_whitelist): (HashMap<String, bool>, HashMap<Principal, bool>) =
        storage::stable_restore().unwrap();

    REQUEST_RESPONSE_BUFFER.with(|store| *store.borrow_mut() = cached_buffer);
    whitelist::WHITE_LIST.with(|store| *store.borrow_mut() = cached_whitelist);

    owner::init_owner();
}
// --------------------------- upgrade hooks ------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist() {
        let dummy_principal = Principal::anonymous();

        whitelist::add_to_whitelist(dummy_principal);

        let is_whitelisted = whitelist::is_whitelisted(dummy_principal);
        assert_eq!(is_whitelisted, true)
    }
}
