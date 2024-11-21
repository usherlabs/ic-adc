mod cli;

use std::str::FromStr;

use anyhow::{bail, Result};
use candid::{encode_args, Principal};
use clap::Parser;
use cli::Cli;
use ic_agent::Agent;
use rand::seq::SliceRandom;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let scheduler = JobScheduler::new().await?;

    let dev = cli.dev;
    let assets = cli.assets.clone();
    let canister_id = Principal::from_str(&cli.canister)?;

    let job = Job::new_async(&cli.cron, move |_uuid, _lock| {
        let assets = assets.clone();
        Box::pin(async move {
            let assets = assets.clone();
            if let Err(e) = call_adc(dev, &canister_id, &assets).await {
                error!("{}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    signal::ctrl_c().await?;

    Ok(())
}

async fn create_agent(dev: bool) -> Result<Agent> {
    let url = if dev {
        "http://127.0.0.1:4943"
    } else {
        "https://icp0.io"
    };

    let agent = Agent::builder().with_url(url).build()?;

    if dev {
        agent.fetch_root_key().await?;
    }

    Ok(agent)
}

async fn call_adc(dev: bool, canister_id: &Principal, assets: &str) -> Result<()> {
    let agent = create_agent(dev).await?;
    let asset = choose_asset(&assets)?;

    info!("Submitting ADC request for {}", asset);
    agent
        .update(canister_id, "submit_adc_request")
        .with_arg(encode_args((asset, ()))?)
        .await?;

    Ok(())
}

fn choose_asset(assets: &str) -> Result<String> {
    let assets: Vec<String> = assets
        .split(",")
        .map(|s| s.trim().to_ascii_uppercase())
        .collect();
    if assets.len() == 0 {
        bail!("Assets list is empty");
    }

    let asset = assets.choose(&mut rand::thread_rng()).unwrap();
    Ok(asset.to_string())
}
