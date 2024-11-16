use verity_dp_ic::verify::types::{CurrencyPair, ProofTypes, Response, Token};

use super::{
    sources::{pyth::Pyth, redstone::Redstone},
    traits::PricingDataSource,
};

/// Given a token, get proofs for the token price from the various supported sources
async fn collect_proof_from_sources(token: &Token) -> anyhow::Result<Vec<ProofTypes>> {
    // set the minimum number of proofs needed
    let min_proofs_required = 1;

    let (redstone_proof, pyth_proof) = tokio::join!(
        Pyth::get_proof(token.ticker.clone()),
        Redstone::get_proof(token.ticker.clone())
    );

    let all_proofs = vec![redstone_proof, pyth_proof];

    let valid_proofs: Vec<ProofTypes> = all_proofs
        .iter()
        .filter_map(|proof_res| proof_res.as_ref().ok().cloned())
        .collect();

    if valid_proofs.len() < min_proofs_required {
        anyhow::bail!("NOT ENOUGH PROOFS")
    }

    Ok(valid_proofs)
}

/// For a given currency pair fetch the proofs for the base token
/// and the quote token if it exists
pub async fn fetch_proofs(currency_pair: &mut CurrencyPair) -> anyhow::Result<()> {
    let base = currency_pair.base.clone();
    let quote = currency_pair.quote.clone();

    let base_proofs = collect_proof_from_sources(&base).await;
    // if theres an error with the proofs, then set the error flag to be true
    // otherwise sace the proofs
    if base_proofs.is_err() {
        currency_pair.error = base_proofs.map_err(|e| e.to_string()).err();
        return Ok(());
    } else {
        currency_pair.base.proofs = Some(base_proofs.unwrap());
    }

    // get the proofs for the quote if it exists
    if quote.is_some() {
        let quote = quote.unwrap();
        let quote_proofs = collect_proof_from_sources(&quote).await;
        // if theres an error with the proofs, then set the error flag to be true
        // otherwise save the proof
        if quote_proofs.is_err() {
            currency_pair.error = quote_proofs.map_err(|e| e.to_string()).err();
        } else {
            currency_pair.quote = Some(Token {
                ticker: quote.ticker,
                proofs: Some(quote_proofs?),
            })
        }
    }

    Ok(())
}

/// For a given price response potentially containig multiple currency pairs
/// go through all the currency pairs and get the proofs from various sources
pub async fn process_proofs(price_response: &mut Response) -> anyhow::Result<()> {
    for pair in &mut price_response.pairs {
        fetch_proofs(pair).await?; // Assuming fetch_data returns a Future
    }

    price_response.processed = true;
    Ok(())
}