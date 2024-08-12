use serde::{Deserialize, Serialize};

/// a struct which would be used to
/// communicate data requested by the ADC 
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PriceRequest {
    pub pairs: Vec<String>,
    // add other proprties about the price here
}