use anyhow::{Ok, Result};
use candid::Principal;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::helpers::logs::ic::{
    DEFAULT_IC_GATEWAY, DEFAULT_IC_GATEWAY_TRAILING_SLASH, DEFAULT_JOB_SCHEDULE,
    MAMANGEMENT_CANISTER_ID,
};
use crate::helpers::utils::get_env_or_default;
use crate::helpers::verity::DEFAULT_PROVER_URL;

use super::helpers::logs::ic::{create_agent, DEFAULT_IDENTITY_PATH, DEFAULT_SHARED_LOCAL_BIND};

/// The configuration for the orchestrator
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The URL of the ICP server to connect to
    pub url: String,
    /// The Canister's principal
    pub canister: Principal,
    /// The path to the pem keyfile to generate an identity from
    pub keyfile_path: String,
    /// The schedule of the job to poll the canister logs
    pub job_schedule: String,
    /// HTTP URL of the prover
    pub prover_url: String,
    /// is this dev or prod env
    pub is_dev: bool,
}

impl Config {
    /// Get an agent associated with this config file
    pub async fn get_agent(&self) -> Result<Agent> {
        let agent = create_agent(&self).await?;
        Ok(agent)
    }

    /// Get the information of the connected notary
    pub async fn get_connected_notary(&self) -> Result<NotaryInformation> {
        let notary_info_url = format!("{}/notaryinfo", self.prover_url.clone());
        let notary_information = reqwest::get(notary_info_url)
            .await?
            .json::<NotaryInformation>()
            .await?;

        Ok(notary_information)
    }

    /// Derive this struct by reading and populating the right environment variables
    pub fn env() -> Self {
        let icp_url = get_env_or_default("ICP_URL", DEFAULT_SHARED_LOCAL_BIND);
        let canister_principal = get_env_or_default("ADC_CANISTER", MAMANGEMENT_CANISTER_ID);
        let canister_principal =
            Principal::from_str(&canister_principal[..]).expect("invalid CANISTER_PRINCIPAL");
        let keyfile_path = get_env_or_default("ICP_IDENTITY_FILEPATH", DEFAULT_IDENTITY_PATH);
        let job_schedule = get_env_or_default("JOB_SCHEDULE", DEFAULT_JOB_SCHEDULE);
        let prover_url = get_env_or_default("PROVER_URL", DEFAULT_PROVER_URL);
        let is_mainnet = matches!(
            &icp_url[..],
            DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
        );

        Self {
            url: icp_url,
            canister: canister_principal,
            keyfile_path: keyfile_path,
            job_schedule: job_schedule,
            prover_url: prover_url,
            is_dev: !is_mainnet,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotaryInformation {
    pub version: String,
    pub public_key: String,
    pub git_commit_hash: String,
    pub git_commit_timestamp: String,
}
