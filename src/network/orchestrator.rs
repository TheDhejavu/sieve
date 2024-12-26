use alloy_rpc_types::{Header, Transaction as RpcTransaction};
use async_trait::async_trait;
use std::sync::{mpsc::Receiver, Arc};

use crate::filter::conditions::EventType;

#[derive(Debug, PartialEq, Clone)]
pub enum ChainData {
    // Ethereum chain data..
    Ethereum(EthereumData),
}

#[derive(Debug, PartialEq, Clone)]
pub enum EthereumData {
    BlockHeader(Arc<Header>),
    Transaction(Arc<RpcTransaction>),
    TransactionPool(Vec<RpcTransaction>),
}

impl ChainData {
    pub(crate) fn match_event_type(self, event_type: EventType) -> bool {
        match self {
            ChainData::Ethereum(eth_data) => match eth_data {
                EthereumData::BlockHeader(_) => matches!(event_type, EventType::BlockHeader),
                EthereumData::Transaction(_) => matches!(event_type, EventType::Transaction),
                EthereumData::TransactionPool(_) => matches!(event_type, EventType::Pool),
            },
        }
    }
}

/// [`ChainOrchestrator`] Orchestrates the lifecycle of chain data polling and retrieval.
///
/// Provides a unified interface for managing blockchain data streams.
#[async_trait]
pub trait ChainOrchestrator: Send + Sync {
    /// Starts the orchestrator and returns a receiver for chain data events.
    /// The receiver will stream Block or TransactionPool updates.
    async fn start(&mut self) -> Result<Receiver<ChainData>, Box<dyn std::error::Error>>;

    /// Gracefully stops the orchestrator and cleans up any resources.
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Returns the unique identifier/name of this chain orchestrator.
    fn name(&self) -> &str;
}
