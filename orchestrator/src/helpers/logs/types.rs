use verity_dp_ic::verify::types::Request;


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
