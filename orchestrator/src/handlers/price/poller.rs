use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::fs;
use std::path::{Path, PathBuf};

use crate::helpers::utils::{get_root_path, get_utc_timestamp};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LogPollerState {
    pub start_timestamp: u64,
}

impl Default for LogPollerState {
    fn default() -> Self {
        Self {
            start_timestamp: get_utc_timestamp(),
        }
    }
}

impl LogPollerState {
    pub fn get_struct_name() -> &'static str {
        type_name::<Self>().split(":").last().unwrap()
    }

    pub fn new(start_timestamp: u64) -> Self {
        Self { start_timestamp }
    }

    /// save this struct to a particular point in state
    pub fn save_state(&self) -> Result<()> {
        let storage_path: PathBuf = Self::get_storage_path();

        // Serialize the struct to a JSON string
        let json_string = serde_json::to_string(&self.clone())?;

        // create directory if it does not exist
        if !Path::exists(&storage_path) {
            let prefix = storage_path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
        }

        // Write the JSON string to a file
        Ok(fs::write(storage_path, json_string)?)
    }

    /// restore this struct from a aparticular point in state
    pub fn load_state() -> Result<Self> {
        let storage_path: PathBuf = Self::get_storage_path();

        // create directory if it does not exist
        if !Path::exists(&storage_path) {
            let new_state = Self::default();
            new_state.save_state()?;

            return Ok(new_state);
        }

        // Read the file into a String
        let contents = fs::read_to_string(storage_path)?;

        // Parse the JSON string into your Rust structs
        let loaded_state: Self = serde_json::from_str(&contents)?;

        Ok(loaded_state)
    }

    /// get the default path for the storage which should be a .cache folder
    pub fn get_storage_path() -> PathBuf {
        let struct_name = Self::get_struct_name();
        let storage_path = get_root_path(".cache").join(format!("{struct_name}.json"));

        storage_path
    }
}
