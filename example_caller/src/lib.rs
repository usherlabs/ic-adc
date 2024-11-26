use std::str::FromStr;

use candid::Principal;
use types::{ADCResponse, RequestOpts};

#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_adc_request(currency_pairs: String) -> String {
    let adc_canister_request_method = "request_data";
    //TODO: change the principal to that of the processor's
    let adc_canister_principal = Principal::from_str("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap();
    let options = RequestOpts { price: true };

    let (request_id,): (String,) = ic_cdk::call(
        adc_canister_principal,
        adc_canister_request_method,
        (currency_pairs, options),
    )
    .await
    .unwrap();

    // println!("{:?}", request_id)
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
