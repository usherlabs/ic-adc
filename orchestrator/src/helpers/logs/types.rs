use types::{ProxyRequest, Request};


/// The type to represent DFX results.
pub type DfxResult<T = ()> = anyhow::Result<T>;

#[derive(Debug, Clone)]
pub struct EventLog {
    pub index: u64,
    pub timestamp: u64,
    pub logs: Request,
}

impl EventLog {
    pub fn new(index: u64, timestamp: u64, logs: Request) -> Self {
        Self {
            index,
            timestamp,
            logs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventUrlLog {
    pub index: u64,
    pub timestamp: u64,
    pub logs: ProxyRequest,
}

impl EventUrlLog {
    pub fn new(index: u64, timestamp: u64, logs:ProxyRequest) -> Self {
        Self {
            index,
            timestamp,
            logs,
        }
    }
}
