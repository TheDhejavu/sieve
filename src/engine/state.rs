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
    pub(crate) method: String,
    pub(crate) parameters: HashMap<String, Value>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct State {
    pub(crate) decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}
