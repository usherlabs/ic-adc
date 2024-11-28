use anyhow::Result;
use types::ProofTypes;


pub trait PricingDataSource {
    fn new() -> Self;
    fn get_url(ticker: String) -> impl std::future::Future<Output = Result<String, anyhow::Error>>;
    fn get_proof(ticker: String) -> impl std::future::Future<Output = Result<ProofTypes, anyhow::Error>>;
    fn validate_response(http_response_string: String) -> impl std::future::Future<Output = Result<(), anyhow::Error>>;
}