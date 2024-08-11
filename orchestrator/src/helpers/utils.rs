use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns the absolute path of a folder name provided relative to the root directory of the project
pub fn get_root_path(folder_name: &str) -> PathBuf {
    let project_root_dir = env::current_dir().unwrap();
    project_root_dir.join(folder_name)
}

/// Get UTC timestamp
pub fn get_utc_timestamp() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    since_the_epoch.as_secs()
}
