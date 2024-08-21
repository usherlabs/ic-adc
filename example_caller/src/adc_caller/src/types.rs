use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Clone, Debug, CandidType)]
pub struct ADCResponse {
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
    price: Option<InformationDetails>,
    /// a string representation of the price pair "USDT/BTC"
    repr: String,
    // TODO: add in other properties
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
