use alloy_primitives::map::HashMap;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

use crate::utils::decoder::DecodedLog;

#[derive(Hash, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub(crate) enum CacheKey {
    Event(String),        // [log:txhash] => event
    ContractCall(String), // [input:txhash] => contract call
}

#[derive(Clone)]
pub(crate) enum DecodedData {
    ContractCall(DecodedContractCall),
    Event(DecodedLog),
}

#[derive(Clone)]
pub(crate) struct DecodedContractCall {
    pub(crate) method: String,
    pub(crate) parameters: HashMap<String, Value>,
}

impl DecodedContractCall {
    pub(crate) fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    pub(crate) fn get_method(&self) -> &str {
        &self.method
    }
}

#[allow(dead_code)]
pub(crate) struct State {
    pub(crate) decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}
