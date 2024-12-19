use dashmap::DashMap;
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub(crate) enum CacheKey {
    TransferData(String),
    ContractCall(String),
}

#[allow(dead_code)]
pub(crate) enum DecodedData {
    Transfer {
        method: String,
        to: String,
        amount: String,
    },
    Call {
        method: String,
        params: Vec<String>,
    },
}

#[allow(dead_code)]
pub(crate) struct State {
    pub(crate) decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}
