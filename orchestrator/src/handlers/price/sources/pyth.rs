use crate::{handlers::price::traits::PricingDataSource, helpers::verity::get_verity_client};
use anyhow::{Ok, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct Pyth {}

impl Pyth {
    /// Given a ticker(e.g USDT) it should return the ID associated with it
    pub async fn get_ticker_id(ticker: String) -> Result<String> {
        let quote_currency_to_find = "USD"; // Quote currency you're looking for
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

        let ticker_id = ticker_id.unwrap();
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

    /// get latest price data for a currency from pyth api
    async fn get_price(ticker: String) -> Result<f64> {
        let request_url = Self::get_url(ticker).await?;
        // Send a GET request to the API
        let verity_client = get_verity_client();
        let response = verity_client
            .get(&request_url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        // let response = reqwest::get(&request_url).await?.text().await?;

        // // Parse the JSON response
        let data: Value = serde_json::from_str(&response)?;
        let price = data[0]["price"]["price"].as_str().unwrap();
        let exp = data[0]["price"]["expo"].as_i64().unwrap();

        let price: f64 = price.parse()?;
        let multiplier = 1.0 as f64 / (10 as f64).powf(exp.abs() as f64);
        let price: f64 = price * multiplier;

        Ok(price)
    }

    /// Get pair price i.e "BTC/USDT"
    async fn get_pair_price(currency_pair: String) -> Result<f64> {
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
