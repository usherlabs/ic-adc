use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// ARC Caller canister ID
    #[arg(long, value_name = "CANISTER", env)]
    pub canister: String,

    /// Assets to request prices for
    #[arg(long, value_name = "ASSETS", env)]
    pub assets: String,

    /// Schedule in CRON notation
    #[arg(long, value_name = "CRON", env)]
    pub cron: String,

    /// Development mode - use local IC replica
    #[arg(long)]
    pub dev: bool,
}