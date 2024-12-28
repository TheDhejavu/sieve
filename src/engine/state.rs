use alloy_primitives::map::HashMap;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

use crate::utils::decoder::DecodedLog;

#[derive(Hash, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub enum CacheKey {
    Event(String),        // [log:txhash] => event
    ContractCall(String), // [input:txhash] => contract call
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum DecodedData {
    ContractCall(DecodedContractCall),
    Event(DecodedLog),
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DecodedContractCall {
    /// Name of the called contract method
    pub(crate) method: String,
    /// Decoded parameters as key-value pairs
    pub(crate) parameters: HashMap<String, Value>,
}

/// Holds shared state for decoded data caching
#[derive(Clone)]
#[allow(dead_code)]
pub struct State {
    /// Thread-safe cache mapping CacheKeys to decoded data.
    /// Uses DashMap for concurrent access without explicit locking
    pub(crate) decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}
