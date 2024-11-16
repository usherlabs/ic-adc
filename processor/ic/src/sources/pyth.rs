use serde_json::Value;

use super::PricingDataSource;
use anyhow::Context;

#[derive(Debug)]
pub struct Pyth {}

impl PricingDataSource for Pyth {
    /// Extract the pyth price from a string representation of the body of the http response
    fn get_price(http_response_string: String) -> anyhow::Result<f64> {
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
        let asset_price: f64 = price * multiplier;

        return Ok(asset_price);
    }
}
