use k256::ecdsa::SigningKey;
use verity_client::client::{AnalysisConfig, VerityClient, VerityClientConfig};

use crate::config::Config;

pub const DEFAULT_PROVER_URL: &str = "http://127.0.0.1:8080";
pub const DEFAULT_PROVER_ZMQ_URL: &str = "tcp://127.0.0.1:8000";
pub const DEFAULT_ANALYSIS_URL: &str = "http://127.0.0.1:8000";

pub fn get_verity_client() -> VerityClient {
    let config = Config::env();
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::random(&mut rng);

    let verity_config = VerityClientConfig {
        prover_url: config.prover_url,
        prover_zmq: config.prover_zmq_url,
        analysis: if config.analysis_url.is_some() {
            Some(AnalysisConfig {
                analysis_url: String::from(config.analysis_url.unwrap()),
                signing_key,
            })
        } else {
            None
        },
    };

    VerityClient::new(verity_config)
}
