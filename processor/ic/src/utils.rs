use crate::sources::{get_asset_price_from_proofs, request_proof_verification};
use candid::Principal;
use ic_cdk::api::call::RejectionCode;
use verity_dp_ic::verify::types::{ADCResponse, CurrencyPair, ProofTypes, Token};

// send a response to the canister who requested pricing daya
pub fn send_adc_response(
    recipient: Principal,
    adc_response: ADCResponse,
) -> Result<(), RejectionCode> {
    let canister_response = ic_cdk::notify(recipient, "receive_adc_response", (adc_response,));

    return canister_response;
}

// Provided a token and a notary public key for the notary used to generate the proofs attached to the tokens
// verify/decrypt the proofs and come to a concensus on the token price
pub async fn get_token_price(token: &Token, notary_pubkey: &String) -> anyhow::Result<f64> {
    // get the proofs in a stringified form
    let proof_types = token.proofs.as_ref().unwrap();
    let stringified_proofs: Vec<String> = proof_types
        .iter()
        .map(|pt: &ProofTypes| pt.to_string())
        .collect();

    // request proof response from verification canister
    let verification_response =
        request_proof_verification(&stringified_proofs, notary_pubkey).await;

    // get the verificaction response results
    let verification_response_proofs = verification_response.unwrap().results;

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
