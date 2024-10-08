use std::str::FromStr;

use ic_cdk::println;
use candid::Principal;
use types::{ADCResponse, RequestOpts,ADCErrorResponse};

pub mod types;

#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_adc_request(currency_pairs: String) -> String {
    let adc_canister_request_method = "request_data";
    //TODO: change the principal to that of the processor's
    let adc_canister_principal = Principal::from_str("brbgg-kaaaa-aaaan-qmvzq-cai").unwrap();
    let options = RequestOpts::default();

    let (request_id,): (String,) = ic_cdk::call(
        adc_canister_principal,
        adc_canister_request_method,
        (currency_pairs, options, ),
    )
    .await
    .unwrap();

    // println!("{:?}", request_id)
    return request_id;
}

#[ic_cdk::update]
fn receive_adc_response(response: Result<ADCResponse, ADCErrorResponse>) {
    println!("receive_adc_response: {:?}", response);
}
