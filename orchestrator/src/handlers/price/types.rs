use std::fmt::Display;

use anyhow::Result;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::handlers::price::DEFAULT_BASE_CURRENCY;

use super::{
    sources::{pyth::Pyth, redstone::Redstone},
    traits::PricingDataSource,
};

/// a struct which would be used to
/// communicate data requested by the ADC
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Request {
    /// the id of this request
    pub id: String,
    /// the principal of the canister which originated this request
    pub owner: Principal,
    /// a vector of strings representing the currency pair e.b ["BTC", "BTC/USDT"]
    pub pairs: Vec<String>,
    // add other proprties about the price here
    pub opts: RequestOpts,
}

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct Response {
    /// the id of this request
    pub id: String,
    /// the principal of the canister which originated this request
    pub owner: Principal,
    /// the pairs to be processed, currently these are currency pairs but they will eventually be proofs
    pub pairs: Vec<CurrencyPair>,
    /// when we `convert` a request to a response, the price/proof information is not fetched yet
    /// this property indicates if the metadata information about this request has been succesfully fetched
    /// and is ready to be sent to the canister
    pub processed: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct ErrorResponse {
    /// the id of this request
    pub id: String,
    /// the principal of the canister which originated this request
    pub owner: Principal,
    /// A text describing the error message
    pub message: String,
}

impl ErrorResponse {
    pub fn new(id: String, owner: Principal, message: String) -> Self {
        Self { id, owner, message }
    }
}

impl From<Request> for Response {
    fn from(request: Request) -> Self {
        let pairs: Vec<CurrencyPair> = request
            .pairs
            .iter()
            .filter_map(|pair_string| CurrencyPair::try_from(pair_string.to_owned()).ok())
            .collect();

        Self {
            id: request.id,
            owner: request.owner,
            pairs,
            processed: false,
        }
    }
}

impl Response {
    pub async fn process_prices(&mut self) -> Result<()> {
        for pair in &mut self.pairs {
            pair.fetch_prices().await?; // Assuming fetch_data returns a Future
        }

        self.processed = true;
        Ok(())
    }
}

/// a struct representing a currency pair
#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct CurrencyPair {
    /// the base currency
    base: String,
    /// the quote currency
    quote: String,
    /// the price aggregated from several sources
    price: Option<InformationDetails>,
    // TODO: add in other properties
    /// a string representation of the price pair "USDT/BTC"
    repr: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct InformationDetails {
    value: f64,
    sources: Vec<f64>,
}

impl InformationDetails {
    fn new(value: f64, sources: Vec<f64>) -> Self {
        Self { value, sources }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct RequestOpts {
    pub price: bool,
}

impl CurrencyPair {
    async fn fetch_prices(&mut self) -> Result<()> {
        // TODO: if a data source is down, do we throw an error for the whole process or proceed without it
        // fetch price from redstone
        let redstone_price = Redstone::get_pair_price(self.to_string()).await?;
        // fetch price from pyth
        let pyth_price = Pyth::get_pair_price(self.to_string()).await?;

        // update the struct if both are available
        let sources = vec![redstone_price, pyth_price];
        let consensus_price = Self::resolve_prices(&sources);

        self.price = Some(InformationDetails::new(consensus_price, sources));

        Ok(())
    }

    /// The strategy which we would use to determine one price among our sources
    fn resolve_prices(prices: &Vec<f64>) -> f64 {
        let sum = prices.iter().sum::<f64>();
        sum / prices.len() as f64
    }
}

impl TryFrom<String> for CurrencyPair {
    type Error = anyhow::Error;

    fn try_from(currency_pair: String) -> Result<Self, Self::Error> {
        //TODO: Do we need to have a whitelist of currencies we support and fail if base not included
        let currency_pair = currency_pair.to_uppercase();
        let base_and_quote: Vec<&str> = currency_pair.split('/').collect();

        if base_and_quote.len() > 2 {
            anyhow::bail!("invalid currency_pair")
        }

        let base = base_and_quote[0];
        let quote = if base_and_quote.len() == 2 {
            base_and_quote[1]
        } else {
            DEFAULT_BASE_CURRENCY
        };

        Ok(Self {
            base: base.to_string(),
            quote: quote.to_string(),
            repr: currency_pair,
            price: None,
        })
    }
}

impl Display for CurrencyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_currency_pair_with_base() {
        let pair_string = "BTC/ETH";
        let pair = CurrencyPair::try_from(String::from(pair_string)).unwrap();

        assert_eq!(pair.base, "BTC");
        assert_eq!(pair.quote, "ETH");
        assert_eq!(pair.repr, pair_string);
        assert_eq!(pair.to_string(), pair_string);
    }

    #[test]
    fn test_currency_pair_without_base() {
        let pair_string = "BTC";
        let pair = CurrencyPair::try_from(String::from(pair_string)).unwrap();

        assert_eq!(pair.base, "BTC");
        assert_eq!(pair.quote, DEFAULT_BASE_CURRENCY);
        assert_eq!(pair.repr, pair_string);
        assert_eq!(pair.to_string(), format!("BTC/{}", DEFAULT_BASE_CURRENCY));
    }

    #[test]
    fn test_resolve_prices() {
        let prices_sources = vec![1.0, 2.0, 3.0];
        let resolved_price = CurrencyPair::resolve_prices(&prices_sources);

        assert_eq!(resolved_price, 2.0);
    }
}
