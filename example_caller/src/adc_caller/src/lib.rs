use candid::Principal;
use ic_cdk::{
    api::{call::call_with_payment128, management_canister::http_request::TransformFunc},
    storage,
};
use state::{get_request_value, set_request_value};
use types::{ADCResponse, ADCResponseV2, Headers, RequestOpts};
use verity_ic::{owner, verify::types::ProofResponse};

pub mod state;

//1. IMPORT IC MANAGEMENT CANISTER
//This includes all methods and types needed
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

use ic_cdk_macros::query;
use serde::{Deserialize, Serialize};
use serde_json;

// This struct is legacy code and is not really used in the code.
#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

//Update method using the HTTPS outcalls feature
#[ic_cdk::update]
async fn send_http_request(
    target_url: String,
    method: String,
    redacted: String,
    headers: Vec<Headers>,
    body: String,
) -> String {
    let request_headers: Vec<HttpHeader> = headers
        .into_iter()
        .map(|x| HttpHeader {
            name: x.key,
            value: x.value,
        })
        .collect();

    //note: here, r#""# is used for raw strings in Rust, which allows you to include characters like " and \ without needing to escape them.
    //We could have used "serde_json" as well.
    let json_utf8: Vec<u8> = body.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);

    // This struct is legacy code and is not really used in the code. Need to be removed in the future
    // The "TransformContext" function does need a CONTEXT parameter, but this implementation is not necessary
    // the TransformContext(transform, context) below accepts this "context", but it does nothing with it in this implementation.
    // bucket_start_time_index and closing_price_index are meaninglesss
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let _method = if method.to_uppercase() == "POST" {
        HttpMethod::POST
    } else {
        HttpMethod::GET
    };

    let request = CanisterHttpRequestArgument {
        url: target_url.to_string(),
        max_response_bytes: None, //optional for request
        method: _method,
        headers: request_headers,
        body: request_body,
        transform: Some(TransformContext {
            context: serde_json::to_vec(&context).unwrap(),
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
        }),
        // transform: None, //optional for request
    };

    //3. MAKE HTTPS REQUEST AND WAIT FOR RESPONSE

    //Note: in Rust, `http_request()` already sends the cycles needed
    //so no need for explicit Cycles.add() as in Motoko
    match http_request(request, state::get_transaction_fee()).await {
        //4. DECODE AND RETURN THE RESPONSE

        //See:https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.HttpResponse.html
        Ok((response,)) => {
            //if successful, `HttpResponse` has this structure:
            // pub struct HttpResponse {
            //     pub status: Nat,
            //     pub headers: Vec<HttpHeader>,
            //     pub body: Vec<u8>,
            // }

            //We need to decode that Vec<u8> that is the body into readable text.
            //To do this, we:
            //  1. Call `String::from_utf8()` on response.body
            //  3. We use a switch to explicitly call out both cases of decoding the Blob into ?Text
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::api::print(format!("{:?}", str_body));

            //The API response will looks like this:
            // { successful: true }

            //Return the body as a string and end the method
            // let result: String = format!(
            //     "{}. See more info of the request sent at: {}",
            //     str_body, target_url
            // );
            str_body
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");

            //Return the error as a string and end the method
            message
        }
    }
}

// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200u32 {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error: err = {:?}", raw));
    }
    res
}

#[ic_cdk::init]
async fn init(adc_canister: Option<Principal>) {
    owner::init_owner();
    state::set_adc_address(adc_canister);
    state::set_transaction_fee(2_000_000_000);
}

#[ic_cdk::update]
async fn set_transaction_fee(transaction_fee: u128) {
    owner::only_owner();
    state::set_transaction_fee(transaction_fee);
}

#[ic_cdk::update]
async fn set_adc_address(adc_address_principal: Principal) {
    state::set_adc_address(Some(adc_address_principal));
}

#[ic_cdk::query]
async fn get_adc_address() -> Option<Principal> {
    state::get_adc_address()
}

#[ic_cdk::query]
async fn get_transaction_fee() -> u128 {
    state::get_transaction_fee()
}

#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_adc_request(currency_pairs: String) -> String {
    assert!(state::get_adc_address().is_some(), "ADC_ADDRESS_NOT_SET");

    let adc_canister_request_method = "request_data";
    //TODO: change the principal to that of the processor's
    let adc_canister_principal = state::get_adc_address().unwrap();
    let options = RequestOpts { price: true };

    let (request_id,): (String,) = call_with_payment128(
        adc_canister_principal,
        adc_canister_request_method,
        (currency_pairs, options),
        state::get_transaction_fee(),
    )
    .await
    .unwrap();

    return request_id;
}

#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_http_request(
    target_url: String,
    method: String,
    redacted: String,
    headers: Vec<Headers>,
    body: String,
) -> String {
    assert!(state::get_adc_address().is_some(), "ADC_ADDRESS_NOT_SET");

    let adc_canister_request_method = "request_data_url";
    //TODO: change the principal to that of the processor's
    let adc_canister_principal = state::get_adc_address().unwrap();

    let (request_id,): (String,) = call_with_payment128(
        adc_canister_principal,
        adc_canister_request_method,
        (target_url, method, redacted, headers, body),
        state::get_transaction_fee(),
    )
    .await
    .unwrap();

    return request_id;
}

#[ic_cdk::update]
/// receive a response form the ADC canister
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
/// receive a response form the ADC canister
fn get_adc_response(request_id: String) -> Option<String> {
    get_request_value(&request_id)
}

#[ic_cdk::update]
/// receive a response form the ADC canister
fn receive_adc_response_v2(response: ADCResponseV2) {
    let adc_response = response.unwrap();
    // log the price and name of each asset received
    for content in adc_response.clone().contents {
        // if there was an error fetching the currency pair then log an error
        let full_proof = ProofResponse::FullProof(content).get_http_response_body();
        ic_cdk::println!("\n\nContent:\n {}", &full_proof);
        set_request_value(&adc_response.id, full_proof);
    }
}

// --------------------------- upgrade hooks ------------------------- //
#[ic_cdk::pre_upgrade]
/// backup state variables from canister
fn pre_upgrade() {
    let cloned_verifier = state::get_adc_address().clone();
    let clone_fee = state::get_transaction_fee();

    storage::stable_save((cloned_verifier, clone_fee)).unwrap()
}
#[ic_cdk::post_upgrade]
/// restore state variables from backup
async fn post_upgrade() {
    let (cached_adc, cache_fee): (Option<Principal>, u128) = storage::stable_restore().unwrap();

    owner::init_owner();

    state::set_adc_address(cached_adc);
    state::set_transaction_fee(cache_fee);
}
// --------------------------- upgrade hooks ------------------------- //
