use anyhow::Context;
use serde_json::Value;

use super::PricingDataSource;

#[derive(Debug)]
pub struct Redstone {}

impl PricingDataSource for Redstone {
    /// Extract the redstone price from a string representation of the body of the http response
    fn get_price(http_response_string: String) -> anyhow::Result<f64> {
        // Parse the JSON response
        let data: Value = serde_json::from_str(&http_response_string)?;

        // Access the 'price' property and return it
        let price = data[0]["value"]
            .as_f64()
            .context("Price not available: JSON structure changed")
            .and_then(|val| Ok(val))?;
        return Ok(price);
    }
}
