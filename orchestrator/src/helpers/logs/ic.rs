use anyhow::Context;
use candid::utils::ArgumentDecoder;
use candid::{CandidType, Principal};
use ic_agent::identity::Secp256k1Identity;
use ic_agent::Agent;
use ic_utils::call::SyncCall;
use ic_utils::interfaces::management_canister::{FetchCanisterLogsResponse, MgmtMethod};
use ic_utils::interfaces::ManagementCanister;
use time::OffsetDateTime;
use types::{ProxyRequest, Request};

use super::types::EventLog;
use super::types::{DfxResult, EventUrlLog};
use crate::config::Config;

pub const MAMANGEMENT_CANISTER_ID: &str = "aaaaa-aa";
pub const DEFAULT_SHARED_LOCAL_BIND: &str = "http://127.0.0.1:4943";
pub const DEFAULT_IC_GATEWAY: &str = "https://icp0.io";
pub const DEFAULT_IC_GATEWAY_TRAILING_SLASH: &str = "https://icp0.io/";
pub const DEFAULT_IDENTITY_PATH: &str = "./identity.pem";
pub const DEFAULT_JOB_SCHEDULE: &str = "1/60 * * * * *";

fn format_bytes(bytes: &[u8]) -> String {
    format!("(bytes) 0x{}", hex::encode(bytes))
}

/// Create an IC agent which has been authenticated to query the logs of a canister specified
pub async fn create_agent(config: &Config) -> anyhow::Result<Agent> {
    let identity = Secp256k1Identity::from_pem_file(&config.keyfile_path)?;
    let agent = Agent::builder()
        .with_transport(ic_agent::agent::http_transport::ReqwestTransport::create(
            config.url.clone(),
        )?)
        .with_boxed_identity(Box::new(identity))
        .with_verify_query_signatures(true)
        .build()?;

    if config.is_dev {
        agent.fetch_root_key().await?;
    }

    Ok(agent)
}

/// Get the raw logs from a canister
pub async fn get_canister_logs(
    config: &Config,
    start_timestamp: Option<u64>,
) -> anyhow::Result<(Vec<EventLog>, Vec<EventUrlLog>)> {
    let canister_id = config.canister;
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    let agent = config.get_agent().await.unwrap();

    let (out,): (FetchCanisterLogsResponse,) = do_management_query_call(
        canister_id,
        MgmtMethod::FetchCanisterLogs.as_ref(),
        In { canister_id },
        &agent,
    )
    .await?;

    let (formatted_logs, formatted_url_logs) = format_canister_logs(out);

    if let Some(timestamp) = start_timestamp {
        // filter the logs by timestamp

        Ok((
            formatted_logs
                .clone()
                .iter()
                .filter(|event| event.timestamp > timestamp)
                .cloned()
                .collect(),
            formatted_url_logs
                .clone()
                .iter()
                .filter(|event| event.timestamp > timestamp)
                .cloned()
                .collect(),
        ))
    } else {
        Ok((formatted_logs.clone(), formatted_url_logs.clone()))
    }
}

/// Parse the valid event logs into a well formatted `EventLog`
fn format_canister_logs(logs: FetchCanisterLogsResponse) -> (Vec<EventLog>, Vec<EventUrlLog>) {
    let mut valid_logs = (vec![], vec![]);
    logs.canister_log_records.into_iter().for_each(|r| {
        let time = OffsetDateTime::from_unix_timestamp_nanos(r.timestamp_nanos as i128)
            .expect("Invalid canister log record timestamp");

        let message = if let Ok(s) = String::from_utf8(r.content.clone()) {
            if format!("{s:?}").contains("\\u{") {
                format_bytes(&r.content)
            } else {
                s
            }
        } else {
            format_bytes(&r.content)
        };

        let parsed_message_result: Result<Request, serde_json::Error> =
            serde_json::from_str(&message);
        if parsed_message_result.is_ok() {
            valid_logs.0.push(EventLog::new(
                r.idx,
                time.unix_timestamp() as u64,
                parsed_message_result.unwrap(),
            ))
        }

        let parsed_message_result: Result<ProxyRequest, serde_json::Error> =
            serde_json::from_str(&message);
        if parsed_message_result.is_ok() {
            valid_logs.1.push(EventUrlLog::new(
                r.idx,
                time.unix_timestamp() as u64,
                parsed_message_result.unwrap(),
            ))
        }
    });

    valid_logs
}

async fn do_management_query_call<A, O>(
    destination_canister: Principal,
    method: &str,
    arg: A,
    agent: &Agent,
) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let mgr = ManagementCanister::create(agent);

    let out = mgr
        .query(method)
        .with_arg(arg)
        .with_effective_canister_id(destination_canister)
        .build()
        .call()
        .await
        .context("Query call failed.")?;

    Ok(out)
}
