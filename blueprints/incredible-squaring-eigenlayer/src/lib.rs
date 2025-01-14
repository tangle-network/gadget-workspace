#![allow(dead_code)]
use alloy_sol_types::sol;
use gadget_macros::load_abi;
use serde::{Deserialize, Serialize};
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
    #[error("Event conversion error: {0}")]
    Conversion(String),
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
