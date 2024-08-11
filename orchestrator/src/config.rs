use anyhow::Result;
use candid::Principal;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::helpers::logs::ic::MAMANGEMENT_CANISTER_ID;
use crate::helpers::utils::get_root_path;

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
}

impl Default for Config {
    fn default() -> Self {
        let icp_url = DEFAULT_SHARED_LOCAL_BIND.to_string();
        let canister_principal = MAMANGEMENT_CANISTER_ID.to_string();
        let canister_principal =
            Principal::from_str(&canister_principal).expect("invalid CANISTER_PRINCIPAL");
        let keyfile_path = DEFAULT_IDENTITY_PATH.to_string();

        Self {
            url: icp_url,
            canister: canister_principal,
            keyfile_path: keyfile_path,
        }
    }
}

impl Config {
    /// Get an agent associated with this config file
    pub async fn get_agent(&self) -> Result<Agent> {
        let agent = create_agent(&self).await?;
        Ok(agent)
    }

    /// Gets the configuration either read from specified location or from
    /// the default location. If there is no file at the default location,
    /// it creates the file with default values.
    pub fn get_and_persist(config: &Option<PathBuf>) -> Result<Config> {
        let config_location = if let Some(config) = config {
            fs::canonicalize(config).unwrap()
        } else {
            let default_location = default_config_location("orchestrator");

            if !Path::exists(&default_location) {
                let default_config = Config::default();
                write_config_file(&default_config, &default_location)?;
            }

            default_location
        };

        read_config_file(&config_location)
    }
}

/// Gets the default location the config file should reside in
pub fn default_config_location(node: &str) -> PathBuf {
    get_root_path(".config").join(format!("{node}.yaml"))
}

/// Read a yaml configuration file into a struct
pub fn read_config_file(location: &PathBuf) -> Result<Config> {
    let file = std::fs::File::open(location)?;
    let config: Config = serde_yaml::from_reader(file)?;

    Ok(config)
}

/// Write a yaml configuration struct into a file
pub fn write_config_file(config: &Config, location: &PathBuf) -> Result<()> {
    let prefix = location.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let value = serde_yaml::to_string(config).unwrap();
    Ok(fs::write(location, &value)?)
}
