use anyhow::{Ok, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct Redstone {
    pub price: Option<f64>,
}

impl Redstone {
    pub fn new() -> Self {
        Self { price: None }
    }

    fn get_url(ticker: String) -> Result<String> {
        Ok(format!(
            "https://api.redstone.finance/prices?symbol={ticker}&provider=redstone&limit=1"
        ))
    }

    /// set and get latest price data from redstone api
    pub async fn get_price(ticker: String) -> Result<f64> {
        let request_url = Self::get_url(ticker)?;
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

    // get pair price "BTC/USDT"
    pub async fn get_pair_price(currency_pair: String) -> Result<f64>{
        // Split the string into an iterator of substrings
        let parts: Vec<&str> = currency_pair.split('/').collect();

        // Assuming the first part is the base and the second part is the quote
        let quote = parts.get(0).unwrap().to_string(); // Default to "Unknown" if the split results in less than two parts
        let base = parts.get(1).unwrap().to_string(); // Default to "Unknown" if the split results in less than two parts

        let quote_price = Self::get_price(quote).await?;
        let base_price = Self::get_price(base).await?;

        Ok(quote_price / base_price)
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[tokio::test]
    async fn it_works() {
        let ticker = "FTM";
        let result = Redstone::get_price(ticker.to_string()).await.unwrap();

        println!("{}", result);
        // assert_eq!(result, 4); // Assert that the result is equal to 4
    }
}
