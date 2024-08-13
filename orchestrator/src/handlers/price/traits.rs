use anyhow::Result;

pub trait PricingDataSource {
    fn new() -> Self;
    fn get_url(ticker: String) -> impl std::future::Future<Output = Result<String, anyhow::Error>>;
    fn get_price(ticker: String) -> impl std::future::Future<Output = Result<f64, anyhow::Error>>;
    fn get_pair_price(currency_pair: String) -> impl std::future::Future<Output = Result<f64, anyhow::Error>>;
}