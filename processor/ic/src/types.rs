use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

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
    /// Contains information about which information should be fetched about the pairs in this request
    pub opts: RequestOpts,
    // add other proprties about the price here
}

impl Request {
    pub fn new(id: String, owner: Principal, string_currency_pair: String, opts: RequestOpts) -> Self {
        let pairs_list: Vec<String> = string_currency_pair
            .split(',')
            .map(|s| s.trim().to_ascii_uppercase().to_string()) // Remove leading/trailing whitespace from each substring
            .collect();

        Self {
            id,
            owner,
            pairs: pairs_list,
            opts
        }
    }
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
    processed: bool,
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

#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct RequestOpts {
    price: bool,
}

impl Default for RequestOpts {
    fn default() -> Self {
        Self { price: true }
    }
}

// src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request_pair() {
        let new_currency_pair = " ETh/BtC , soL ";
        let new_request = Request::new(
            String::from("id"),
            Principal::anonymous(),
            new_currency_pair.to_string(),
            RequestOpts::default()
        );
        // print!("{:?}", new_request);
        assert_eq!(new_request.pairs.len(), 2);
        assert_eq!(new_request.pairs.get(0).unwrap(), "ETH/BTC");
        assert_eq!(new_request.pairs.get(1).unwrap(), "SOL");
    }
}
