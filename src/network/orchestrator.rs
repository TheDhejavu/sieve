use alloy_network::{AnyRpcBlock, AnyRpcTransaction};
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, PartialEq, Clone)]
pub enum ChainData {
    AnyRPCNetwork(AnyRPCNetwork),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnyRPCNetwork {
    Block(AnyRpcBlock),
    TransactionPool(AnyRpcTransaction),
}

/// [`ChainOrchestrator`] Orchestrates the lifecycle of chain data polling and retrieval.
///
/// Provides a unified interface for managing blockchain data streams.
#[allow(dead_code)]
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
