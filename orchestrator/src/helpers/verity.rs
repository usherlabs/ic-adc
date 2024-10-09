use k256::SecretKey;
use rand::rngs::OsRng;
use crate::config::Config;
use verity_client::client::{AnalysisConfig, VerityClient, VerityClientConfig};

pub const DEFAULT_PROVER_URL: &str = "http://127.0.0.1:8080";
pub const DEFAULT_PROVER_ZMQ_URL: &str = "tcp://127.0.0.1:8000";

pub fn get_verity_client() -> VerityClient {
    let config = Config::env();
    let secret_key = SecretKey::random(&mut OsRng);

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
