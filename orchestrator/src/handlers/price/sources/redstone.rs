use anyhow::{Ok, Result};
use serde_json::Value;

use crate::handlers::price::traits::PricingDataSource;

#[derive(Debug)]
pub struct Redstone {
}

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
        // Send a GET request to the API
        //TODO: use the verity prover to get some data
        let response = reqwest::get(&request_url).await?.text().await?;

        // Parse the JSON response
        let data: Value = serde_json::from_str(&response)?;

        // Access the 'price' property and return it
        if let Some(price) = data[0]["value"].as_f64() {
            Ok(price)
        } else {
            anyhow::bail!("Price not available: JSON structure changed:{}", response)
        }
    }

    /// Get pair price i.e "BTC/USDT"
    async fn get_pair_price(currency_pair: String) -> Result<f64>{
        // Split the string into an iterator of substrings
        let parts: Vec<&str> = currency_pair.split('/').collect();

        // Assuming the first part is the quote and the second part is the base
        let base = parts.get(0).unwrap().to_string(); // Default to "Unknown" if the split results in less than two parts
        let quote = parts.get(1).unwrap().to_string(); // Default to "Unknown" if the split results in less than two parts

        let base_price = Self::get_price(base).await?;
        let quote_price = Self::get_price(quote).await?;

        Ok(base_price / quote_price)
    }
}
