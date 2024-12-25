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
pub enum IngestError {
    ChainNotFound(Chain),
    OrchestrationError(String),
}

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

struct ChainState {
    chain_stream: Arc<ChainStream>,
    orchestrator: Box<dyn ChainOrchestrator>,
    handle: JoinHandle<()>,
}

struct Ingest {
    chain_states: HashMap<Chain, ChainState>,
}

impl Ingest {
    //
    pub(crate) async fn new(configs: Vec<ChainConfig>) -> Self {
        let mut chain_states = HashMap::new();

        for config in configs {
            let chain = config.chain();

            match config.chain() {
                Chain::Ethereum => {
                    // Start RPC orchestrator if configured...
                    if !config.rpc_url().is_empty() {
                        let orchestrator = EthereumRpcOrchestrator::new(
                            format!("{:?}", config.chain()),
                            config.rpc_url().to_string(),
                            DEFAULT_POLL_INTERVAL,
                        )
                        .unwrap();

                        let chain_stream = Arc::new(ChainStream::new(chain.clone()));
                        let mut orchestrator = Box::new(orchestrator);
                        let receiver = orchestrator.start().await.unwrap();

                        let stream_clone = chain_stream.clone();
                        let handle = tokio::spawn(async move {
                            while let Ok(data) = receiver.recv() {
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
    ///
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
}
