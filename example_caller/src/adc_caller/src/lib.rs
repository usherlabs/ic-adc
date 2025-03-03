use std::str::FromStr;
use ic_cdk::api::call::call_with_payment128;
use candid::Principal;
use types::{ADCResponse, ADCResponseV2, RequestOpts};
use verity_ic::owner;

pub mod  state;


#[ic_cdk::init]
async fn init(adc_canister: Option<Principal>) {
    owner::init_owner();
    state::set_adc_address(adc_canister);
    state::set_transaction_fee(1_000_000_000_000);
}


#[ic_cdk::update]
async fn set_transaction_fee(transaction_fee: u128) {
    owner::only_owner();
    state::set_transaction_fee(transaction_fee);
}


#[ic_cdk::query]
async fn get_transaction_fee() -> u128 {
    state::get_transaction_fee()
}


#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_adc_request(currency_pairs: String,fee: u128) -> String {
    assert!(
        state::get_adc_address().is_some(),
        "ADC_ADDRESS_NOT_SET"
    );

    let adc_canister_request_method = "request_data";
    //TODO: change the principal to that of the processor's
    let adc_canister_principal = state::get_adc_address().unwrap();
    let options = RequestOpts { price: true };

    let (request_id,): (String,) = call_with_payment128(
        adc_canister_principal,
        adc_canister_request_method,
        (currency_pairs, options),
        fee
    )
    .await
    .unwrap();

    return request_id;
}

#[ic_cdk::update]
/// recieve a response form the ADC canister
fn receive_adc_response(response: ADCResponse) {
    // log the price and name of each asset recieved
    for currency_pair in response.unwrap().pairs {
        // if there was an error fetching the currency pair then log an error
        if currency_pair.error.is_some() {
            ic_cdk::println!(
                "There was an error:{} fetching {}",
                currency_pair.error.unwrap(),
                currency_pair.repr
            )
        // otherwise log the price
        } else {
            ic_cdk::println!(
                "Currency Pair: {} has a price of : {}",
                currency_pair.repr,
                currency_pair.price.unwrap()
            );
        }
    }
}

#[ic_cdk::update]
/// recieve a response form the ADC canister
fn receive_adc_response_v2(response: ADCResponseV2) {
    // log the price and name of each asset recieved
    for content in response.unwrap().contents {
        // if there was an error fetching the currency pair then log an error
            ic_cdk::println!(
                "Content:\n {}",
                content,
            );
    }
}
