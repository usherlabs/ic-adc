use anyhow::Context;
use anyhow::{Ok, Result};
use serde_json::Value;

use crate::{handlers::price::traits::PricingDataSource, helpers::verity::get_verity_client};

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

    /// get latest price data for a currency from redstone api
    async fn get_price(ticker: String) -> Result<f64> {
        let request_url = Self::get_url(ticker).await?;
        // Send a GET request to the API using the verity client
        let verity_client = get_verity_client();
        let response = verity_client.get(&request_url).send().await?.text().await?;

        // Parse the JSON response
        let data: Value = serde_json::from_str(&response)?;

        // Access the 'price' property and return it
        data[0]["value"]
            .as_f64()
            .context("Price not available: JSON structure changed")
            .and_then(|exp| Ok(exp))
    }

    /// Get pair price i.e "BTC/USDT"
    async fn get_pair_price(currency_pair: String) -> Result<f64> {
        // Split the string into an iterator of substrings
        let parts: Vec<&str> = currency_pair.split('/').collect();

        // Assuming the first part is the quote and the second part is the base
        let base = match parts.get(0) {
            Some(base) => base.to_string(),
            None => anyhow::bail!("Missing base currency part"),
        };

        let quote = match parts.get(1) {
            Some(quote) => quote.to_string(),
            None => anyhow::bail!("Missing quote currency part"),
        };

        let base_price = Self::get_price(base).await?;
        let quote_price = Self::get_price(quote).await?;

        Ok(base_price / quote_price)
    }
}
