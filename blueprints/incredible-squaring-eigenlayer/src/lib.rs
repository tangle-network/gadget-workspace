#![allow(dead_code)]

use alloy_sol_types::sol;
use gadget_macros::load_abi;
use serde::{Deserialize, Serialize};
use std::net::AddrParseError;
use thiserror::Error;

pub mod constants;
pub mod contexts;
pub mod jobs;
#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Job error: {0}")]
    Job(String),
    #[error("Chain error: {0}")]
    Chain(String),
    #[error("Context error: {0}")]
    Context(String),
    #[error("Event conversion error: {0}")]
    Conversion(String),
    #[error("Parse error: {0}")]
    Parse(#[from] AddrParseError),
    #[error("Event Listener Processor error: {0}")]
    Processor(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

type ProcessorError =
    gadget_event_listeners::core::Error<gadget_event_listeners::evm::error::Error>;

impl From<Error>
    for gadget_event_listeners::core::Error<gadget_event_listeners::evm::error::Error>
{
    fn from(value: Error) -> Self {
        gadget_event_listeners::core::Error::ProcessorError(
            gadget_event_listeners::evm::error::Error::Client(value.to_string()),
        )
    }
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    IncredibleSquaringTaskManager,
    "contracts/out/IncredibleSquaringTaskManager.sol/IncredibleSquaringTaskManager.json"
);

load_abi!(
    INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
    "contracts/out/IncredibleSquaringTaskManager.sol/IncredibleSquaringTaskManager.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    PauserRegistry,
    "./contracts/out/IPauserRegistry.sol/IPauserRegistry.json"
);
