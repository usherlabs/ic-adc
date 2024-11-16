use anyhow::Context;
use anyhow::{Ok, Result};
use serde_json::Value;
use verity_dp_ic::verify::types::ProofTypes;

use crate::handlers::price::traits::PricingDataSource;
use crate::helpers::verity::get_verity_client;


#[derive(Debug)]
pub struct Pyth {}

impl Pyth {
    /// Given a ticker(e.g USDT) it should return the ID associated with it
    pub async fn get_ticker_id(ticker: String) -> Result<String> {
        let quote_currency_to_find = "USD";
        // TODO: we could cache this api call then refresh it on a daily basis using a cronjob
        let request_url = "https://hermes.pyth.network/v2/price_feeds".to_string();

        let mut ticker_id: Option<String> = None;
        let response = reqwest::get(request_url).await?.text().await?;
        let api_response: Vec<Value> = serde_json::from_str(&response)?;

        for item in api_response {
            if item["attributes"]["base"] == ticker
                && item["attributes"]["quote_currency"] == quote_currency_to_find
            {
                // Extract the string value associated with the "text" key
                let message_string = item["id"].as_str().unwrap_or("");
                ticker_id = Some(message_string.to_string())
            }
        }

        let ticker_id = match ticker_id {
            Some(id) => id.to_string(),
            None => anyhow::bail!("Invalid ticker id"),
        };
        Ok(ticker_id)
    }
}

impl PricingDataSource for Pyth {
    fn new() -> Self {
        Self {}
    }

    async fn get_url(ticker: String) -> Result<String> {
        let ticker_id = Self::get_ticker_id(ticker).await?;
        Ok(format!(
            "https://hermes.pyth.network/api/latest_price_feeds?ids[]={}",
            ticker_id
        ))
    }

    async fn get_proof(ticker: String) -> Result<ProofTypes> {
        // construct the request URL
        let request_url = Self::get_url(ticker).await?;
        let verity_client = get_verity_client();

        // get the proof using the verity client
        let response = verity_client.get(&request_url).send().await?;

        // check for a succesfull and valid response
        let http_response_string = response.subject.text().await?;
        Self::validate_response(http_response_string).await?;

        return Ok(ProofTypes::Pyth(response.proof));
    }

    /// Validate the response gotten before saving and sending the proof
    async fn validate_response(http_response_string: String) -> Result<()> {
        // Parse the JSON response
        let data: Value = serde_json::from_str(&http_response_string)?;
        let price = data[0]["price"]["price"]
            .as_str()
            .context("price.price field is missing")
            .and_then(|price| Ok(price))?;
        let exp = data[0]["price"]["expo"]
            .as_i64()
            .context("price.expo field is missing")
            .and_then(|exp| Ok(exp))?;

        // try parsing the price gotten to check for any errors
        let price: f64 = price.parse()?;
        let multiplier = 1.0 as f64 / (10 as f64).powf(exp.abs() as f64);
        let _: f64 = price * multiplier;

        Ok(())
    }
}
