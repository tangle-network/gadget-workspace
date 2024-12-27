use gadget_std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub(crate) protocols: Vec<ProtocolToml>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolToml {
    pub package: String,
    pub repository: Option<HashMap<String, String>>,
    pub bin_hashes: Option<HashMap<String, String>>,
}
