use crate::{sources::{get_asset_price_from_proofs, request_proof_verification}, state};
use candid::Principal;
use ic_cdk::api::call::{msg_cycles_accept128, RejectionCode};
use types::{ADCResponse, ADCResponseV2, CurrencyPair, ProofTypes, Token};
use ic_cdk::api::time;
use verity_ic::verify::types::ProofResponse;

// send a response to the canister who requested pricing data
pub fn send_adc_response(
    recipient: Principal,
    adc_response: ADCResponse,
)-> Result<(), RejectionCode> {
    let canister_response = ic_cdk::notify(recipient, "receive_adc_response", (adc_response,));
    return canister_response;
}

pub fn send_adc_response_v2(
    recipient: Principal,
    adc_response: ADCResponseV2,
)-> Result<(), RejectionCode> {
    let canister_response = ic_cdk::notify(recipient, "receive_adc_response_v2", (adc_response,));
    return canister_response;
}

pub async fn get_proof_response(proof_types: &Vec<ProofTypes>,notary_pubkey: &String)->Vec<ProofResponse>{
    let stringified_proofs: Vec<String> = proof_types
    .iter()
    .map(|pt: &ProofTypes| pt.to_string())
    .collect();

    // request proof response from verification canister
    request_proof_verification(&stringified_proofs, notary_pubkey).await
}
// Provided a token and a notary public key for the notary used to generate the proofs attached to the tokens
// verify/decrypt the proofs and come to a concensus on the token price
pub async fn get_token_price(token: &Token, notary_pubkey: &String) -> anyhow::Result<f64> {
    // get the proofs in a stringified form
    let proof_types = token.proofs.as_ref().unwrap();

    let verification_response_proofs=get_proof_response(proof_types,notary_pubkey).await;
    // parse the proof response based on the corresponding proof type
    // to get the price in the response of the http response body of the verified request
    // use the proof type to parse the json as either a pyth or redstone proof
    let asset_price = get_asset_price_from_proofs(proof_types, &verification_response_proofs);

    asset_price
}

/// Derive the prices for both the base and quote token(if exists)
/// And calculate the pair price as a whole
pub async fn get_currency_pair_price(
    currency_pair: &CurrencyPair,
    notary_pubkey: &String,
) -> anyhow::Result<f64> {
    // get base price
    let base_token = &currency_pair.base;
    let mut token_price = get_token_price(base_token, notary_pubkey).await?;

    // get quote price and potentially divide the base price by it
    if let Some(quote_token) = &currency_pair.quote {
        let quote_token_price = get_token_price(quote_token, notary_pubkey).await?;

        token_price /= quote_token_price;
    }

    Ok(token_price)
}



pub async fn generate_request_url()-> String {
        // derive the request id
    let (random_bytes,): (Vec<u8>,) =
        ic_cdk::call(Principal::management_canister(), "raw_rand", ())
            .await
            .unwrap();
    let random_hex_byte: String = hex::encode(random_bytes)
        .get(0..5)
        .unwrap_or_default()
        .to_string();
    format!("{}_{}", time().to_string(), random_hex_byte)
}

pub async fn check_gas(){
    // Define the fee in cycles (for example, 1 trillion cycles)
    let fee: u128 = state::get_transaction_fee();

    // Accept up to `fee` cycles from the attached call
    let accepted_cycles =msg_cycles_accept128(fee);

    // If not enough cycles were attached, abort the call
    if accepted_cycles < fee {
        ic_cdk::api::trap("Insufficient cycles attached to cover fee.");
    }
}