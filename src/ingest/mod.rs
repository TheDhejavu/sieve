use std::{collections::HashMap, sync::Arc, time::Duration};

mod chain_stream;

use chain_stream::ChainStream;
use futures::Stream;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use crate::{
    config::{Chain, ChainConfig},
    network::{
        ethereum::rpc::EthereumRpcOrchestrator,
        orchestrator::{ChainData, ChainOrchestrator},
    },
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum IngestError {
    ChainNotFound(Chain),
    OrchestrationError(String),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::ChainNotFound(_) => write!(f, "Chain not found"),
            IngestError::OrchestrationError(msg) => write!(f, "Orchestration error: {msg}"),
        }
    }
}

impl std::error::Error for IngestError {}

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[allow(dead_code)]
struct ChainState {
    /// Stream handler for processing and broadcasting chain data
    chain_stream: Arc<ChainStream>,
    /// Chain-specific orchestrator implementation
    orchestrator: Box<dyn ChainOrchestrator>,
    /// Handle to the async task processing incoming chain data
    handle: JoinHandle<()>,
}

/// Main coordinator for ingesting and managing chain (l2, ethereum) data streams.
/// Handles multiple chains simultaneously, providing unified access to their data streams.
pub struct Ingest {
    chain_states: HashMap<Chain, ChainState>,
}

#[allow(dead_code)]
impl Ingest {
    /// Creates a new [`Ingest`] instance with the specified chain configurations.
    pub(crate) async fn new(configs: Vec<ChainConfig>) -> Self {
        let mut chain_states = HashMap::new();

        for config in configs {
            let chain = config.chain();

            match config.chain() {
                Chain::Ethereum => {
                    let chain_stream = Arc::new(ChainStream::new(chain.clone()));

                    // Start RPC orchestrator if configured...
                    if !config.rpc_url().is_empty() {
                        let orchestrator = EthereumRpcOrchestrator::new(
                            format!("{:?}", config.chain()),
                            config.rpc_url().to_string(),
                            DEFAULT_POLL_INTERVAL,
                        )
                        .unwrap();

                        let mut orchestrator = Box::new(orchestrator);
                        let mut receiver = orchestrator.start().await.unwrap();

                        let stream_clone = chain_stream.clone();
                        let handle = tokio::spawn(async move {
                            while let Some(data) = receiver.recv().await {
                                let _ = stream_clone.process_data(data).await;
                            }
                        });

                        chain_states.insert(
                            chain,
                            ChainState {
                                chain_stream,
                                orchestrator,
                                handle,
                            },
                        );
                    }
                }
                Chain::Optimism => println!("<> implement optimism"),
                Chain::Base => println!("<> implement base"),
            }
        }

        Self { chain_states }
    }
    /// Subscribe to chain data as a Stream, allowing for ergonomic async operations
    /// and stream combinators.
    pub async fn subscribe_stream(
        &self,
        chain: Chain,
    ) -> Result<impl Stream<Item = Result<ChainData, BroadcastStreamRecvError>>, IngestError> {
        let receiver = self.subscribe(chain)?;
        Ok(BroadcastStream::new(receiver))
    }

    /// Subscribe to a specific chain's processed and deduplicated data stream
    pub fn subscribe(&self, chain: Chain) -> Result<broadcast::Receiver<ChainData>, IngestError> {
        self.chain_states
            .get(&chain)
            .map(|state| state.chain_stream.subscribe())
            .ok_or_else(|| IngestError::ChainNotFound(chain.clone()))
    }

    /// Check if a chain is currently being orchestrated
    pub fn is_active(&self, chain: &Arc<Chain>) -> bool {
        self.chain_states.contains_key(chain)
    }

    /// Stop a specific chain's orchestration
    pub async fn stop_chain(&mut self, chain: Arc<Chain>) -> Result<(), IngestError> {
        if let Some(state) = self.chain_states.remove(&chain) {
            state
                .orchestrator
                .stop()
                .await
                .map_err(|e| IngestError::OrchestrationError(e.to_string()))?;
            state.handle.abort();
        }
        Ok(())
    }

    /// Stop all chain orchestration
    pub async fn stop_all(&mut self) -> Result<(), IngestError> {
        for (_, state) in self.chain_states.drain() {
            state
                .orchestrator
                .stop()
                .await
                .map_err(|e| IngestError::OrchestrationError(e.to_string()))?;
            state.handle.abort();
        }
        Ok(())
    }

    /// Returns a list of currently active chains.
    pub fn active_chains(&self) -> Vec<Chain> {
        self.chain_states.keys().cloned().collect::<Vec<Chain>>()
    }
}
