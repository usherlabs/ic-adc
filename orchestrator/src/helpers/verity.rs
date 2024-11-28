use verity_client::client::{VerityClient, VerityClientConfig};

use crate::config::Config;

pub const DEFAULT_PROVER_URL: &str = "http://127.0.0.1:8080";

pub fn get_verity_client() -> VerityClient {
    let config = Config::env();

    let verity_config = VerityClientConfig {
        prover_url: config.prover_url,
    };

    VerityClient::new(verity_config)
}
