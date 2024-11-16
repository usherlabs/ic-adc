use anyhow::Context;
use anyhow::{Ok, Result};
use serde_json::Value;
use verity_dp_ic::verify::types::ProofTypes;

use crate::handlers::price::traits::PricingDataSource;
use crate::helpers::verity::get_verity_client;


#[derive(Debug)]
pub struct Redstone {}

impl PricingDataSource for Redstone {
    fn new() -> Self {
        Self {}
    }

    async fn get_url(ticker: String) -> Result<String> {
        Ok(format!(
            "https://api.redstone.finance/prices?symbol={ticker}&provider=redstone&limit=1"
        ))
    }

    async fn get_proof(ticker: String) -> Result<ProofTypes> {
        // construct the request URL
        let request_url = Self::get_url(ticker).await?;
        let verity_client = get_verity_client();

        // get the proof using the verity client
        let response = verity_client.get(&request_url).send().await?;

        // check for a succesfull and valid response
        let http_response_string= response.subject.text().await?;  
        Self::validate_response(http_response_string).await?;

        return Ok(ProofTypes::Redstone(response.proof));
    }

    /// Validate the response gotten before saving and sending the proof
    async fn validate_response(http_response_string: String) -> Result<()> {
        // Parse the JSON response
        let data: Value = serde_json::from_str(&http_response_string)?;

        // Access the 'price' property and return it
        data[0]["value"]
            .as_f64()
            .context("Price not available: JSON structure changed")
            .and_then(|_| Ok(()))
    }
}
