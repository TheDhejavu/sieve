use alloy_primitives::map::HashMap;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub(crate) enum CacheKey {
    Event(String),        // [log:txhash] => event
    ContractCall(String), // [input:txhash] => contract call
}

pub(crate) enum DecodedData {
    ContractCall(DecodedContractCall),
    Event(DecodedEvent),
}

pub(crate) struct DecodedContractCall {
    pub(crate) method: String,
    pub(crate) parameters: HashMap<String, Value>,
}

pub(crate) struct DecodedEvent {
    pub(crate) event: String, // Name of the event (e.g., "Transfer")
    pub(crate) parameters: HashMap<String, Value>, // Decoded event parameters
}

impl DecodedContractCall {
    pub(crate) fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    pub(crate) fn get_method(&self) -> &str {
        &self.method
    }
}

impl DecodedEvent {
    pub(crate) fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    pub(crate) fn get_event(&self) -> &str {
        &self.event
    }
}

#[allow(dead_code)]
pub(crate) struct State {
    pub(crate) decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}
