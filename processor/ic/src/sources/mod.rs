use pyth::Pyth;
use redstone::Redstone;
use types::ProofTypes;
use verity_ic::verify::types::ProofResponse;

use crate::state;

pub mod pyth;
pub mod redstone;

pub trait PricingDataSource {
    fn get_price(http_body: String) -> anyhow::Result<f64>;
}

/// Given a proof type and a corresponding proof response
/// parse the http response json and
/// get the actual price for the asset which is contained in the response
pub fn get_asset_price_from_proofs(
    proof_types: &Vec<ProofTypes>,
    verification_response_proofs: &Vec<ProofResponse>,
) -> anyhow::Result<f64> {
    let source_prices: Vec<anyhow::Result<f64>> = verification_response_proofs
        .iter()
        .enumerate()
        .map(|(index, res)| {
            let http_body = res.get_http_response_body();

            // get the proof's request source to know how to parse its response
            let proof_type = &proof_types[index];
            let price = match proof_type {
                ProofTypes::Pyth(_) => Pyth::get_price(http_body),
                ProofTypes::Redstone(_) => Redstone::get_price(http_body),
            };

            price
        })
        .collect();

    if source_prices.iter().any(|res| res.is_err()) {
        anyhow::bail!("Error getting assset price")
    }

    // calculate the average of these responses
    let source_prices_values: Vec<f64> = source_prices
        .iter()
        .map(|p| p.as_ref().unwrap().to_owned())
        .collect();
    let source_prices_sum: f64 = source_prices_values.iter().sum();
    let count = source_prices_values.len() as f64;
    let average_asset_price = source_prices_sum / count;

    Ok(average_asset_price)
}

/// Request verification from the managed verifier response
pub async fn request_proof_verification(
    stringified_proofs: &Vec<String>,
    notary_pubkey: &String,
) -> Vec<ProofResponse> {
    let verifier_canister = state::get_verifier_canister().unwrap();

    // make a request to the managed verifier canister
    // to get a response which would contain the verified/decrypted proofs sent
    let (response,): (Vec<ProofResponse>,) = ic_cdk::call(
        verifier_canister,
        "verify_proof_async",
        (stringified_proofs, notary_pubkey),
    )
    .await
    .unwrap();

    response
}
