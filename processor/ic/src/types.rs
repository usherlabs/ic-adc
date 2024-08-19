use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// a struct which would be used to
/// communicate data requested by the ADC
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PriceRequest {
    /// the id of this request
    pub id: String,
    /// the principal of the canister which originated this request
    pub owner: Principal,
    /// a vector of strings representing the currency pair e.b ["BTC", "BTC/USDT"]
    pub pairs: Vec<String>,
    // add other proprties about the price here
}

impl PriceRequest {
    pub fn new(id: String, owner: Principal, string_currency_pair: String) -> Self {
        let pairs_list: Vec<String> = string_currency_pair
            .split(',')
            .map(|s| s.trim().to_ascii_uppercase().to_string()) // Remove leading/trailing whitespace from each substring
            .collect();

        Self {
            id,
            owner,
            pairs: pairs_list,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct PriceResponse {
    /// the id of this request
    pub id: String,
    /// the principal of the canister which originated this request
    pub owner: Principal,
    /// the pairs to be processed, currently these are currency pairs but they will eventually be proofs
    pub pairs: Vec<CurrencyPair>,
    /// when we `convert` a request to a response, the price/proof information is not fetched yet
    /// this property indicates if the metadata information about this request has been succesfully fetched
    /// and is ready to be sent to the canister
    processed: bool,
}

/// a struct representing a currency pair
#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct CurrencyPair {
    /// the base currency
    base: String,
    /// the quote currency
    quote: String,
    /// the price aggregated from several sources
    price: Option<f64>,
    /// each price represents a different source
    sources: Option<Vec<f64>>,
    /// a string representation of the price pair "USDT/BTC"
    repr: String,
}


// src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request_pait() {
        let new_currency_pair = " ETh/BtC , soL ";
        let new_request = PriceRequest::new(
            String::from("id"),
            Principal::anonymous(),
            new_currency_pair.to_string(),
        );
        // print!("{:?}", new_request);
        assert_eq!(new_request.pairs.len(), 2);
        assert_eq!(new_request.pairs.get(0).unwrap(), "ETH/BTC");
        assert_eq!(new_request.pairs.get(0).unwrap(), "SOL");
    }
}
