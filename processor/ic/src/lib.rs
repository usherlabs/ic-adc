use candid::Principal;
use core::panic;
use hex;
use ic_cdk::api::time;
use ic_cdk::{println, storage};
use state::REQUEST_RESPONSE_BUFFER;
use std::collections::HashMap;
use types::{ADCResponse, ErrorResponse, Request, RequestOpts, Response};
use utils::{get_currency_pair_price, send_adc_response};
use verity_dp_ic::{owner, whitelist};

pub mod sources;
pub mod state;
pub mod utils;

/// use this variable to control the max number of currency pairs
/// that can be contained in one request
const REQUEST_CURRENCY_PAIR_LIMIT: usize = 10;

// @dev testing command
#[ic_cdk::query]
fn name() -> String {
    format!("adc")
}

#[ic_cdk::init]
async fn init(verifier_canister: Option<Principal>) {
    owner::init_owner();
    state::set_verifier_canister(verifier_canister);
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
async fn set_verifier_canister(verifier_canister_principal: Principal) {
    state::set_verifier_canister(Some(verifier_canister_principal));
}

#[ic_cdk::query]
async fn get_verifier_canister() -> Option<Principal> {
    state::get_verifier_canister()
}

#[ic_cdk::update]
/// requests prices from the orchestrator
/// where `currency_pairs` is a comma separated list of pairs
/// e.g "BTC,ETH/USDT"
/// @dev? is the person requesting the prices supposed to provide the prices
async fn request_data(currency_pairs: String, opts: RequestOpts) -> String {
    assert!(
        state::get_verifier_canister().is_some(),
        "VERIFIER_CANISTER_NOT_SET"
    );
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

    if !whitelist::is_whitelisted(caller_principal) {
        panic!(
            "canister with principal:{} is not allowed to call this method",
            caller_principal
        );
    }
    // creates a price request object with an arb id
    // include the caller canister's id to let adc know where to send a response to
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
async fn receive_orchestrator_response(response: ADCResponse, notary_pubkey: String) {
    assert!(
        state::get_verifier_canister().is_some(),
        "VERIFIER_CANISTER_NOT_SET"
    );
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

    // if we get an error response then return that
    if response.is_err() {
        send_adc_response(response_owner, response).unwrap();
        return;
    }

    // otherwise get the request and process it
    let mut response = response.unwrap();

    // iterate through each of the currency pairs and then get the price consensus
    // or errors (if any), and attach it to the object
    // and return the response to the calling canister
    let mut processed_pairs = vec![];
    for mut currency_pair in response.pairs.clone() {
        // only get the price of a particular pair if it does not have any existing errors
        if currency_pair.error.is_none() {
            let pair_price = get_currency_pair_price(&currency_pair, &notary_pubkey).await;
            match pair_price {
                Ok(price) => {
                    currency_pair.price = Some(price);
                }
                Err(err) => currency_pair.error = Some(err.to_string()),
            }
        }
        processed_pairs.push(currency_pair);
    }

    response.pairs = processed_pairs.clone();

    send_adc_response(response_owner, Ok(response)).unwrap();
}

#[ic_cdk::query]
/// Check if this canister is whitelisted
async fn is_canister_whitelisted(principal: Principal) -> bool {
    owner::only_owner();
    whitelist::is_whitelisted(principal)
}

// --------------------------- upgrade hooks ------------------------- //
#[ic_cdk::pre_upgrade]
/// backup state variables from canister
fn pre_upgrade() {
    let cloned_buffer = state::get_buffer();
    let cloned_verifier = state::get_verifier_canister();
    let cloned_whitelist = whitelist::WHITE_LIST.with(|rc| rc.borrow().clone());

    storage::stable_save((cloned_buffer, cloned_whitelist, cloned_verifier)).unwrap()
}
#[ic_cdk::post_upgrade]
/// restore state variables from backup
async fn post_upgrade() {
    let (cached_buffer, cached_whitelist, cached_verifier): (
        HashMap<String, bool>,
        HashMap<Principal, bool>,
        Option<Principal>,
    ) = storage::stable_restore().unwrap();

    owner::init_owner();
    whitelist::WHITE_LIST.with(|store| *store.borrow_mut() = cached_whitelist);

    state::set_buffer(cached_buffer);
    state::set_verifier_canister(cached_verifier);
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
