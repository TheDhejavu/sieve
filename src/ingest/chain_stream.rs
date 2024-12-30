use lru::LruCache;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::{broadcast, RwLock};

use crate::network::orchestrator::{AnyRPCNetwork, ChainData};

use super::Chain;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainStreamError {
    #[error("Broadcast channel error: {0}")]
    BroadcastError(#[from] broadcast::error::SendError<ChainData>),
}

/// [`ChainStream`] manages the processing and broadcasting of chain data,
/// implementing caching mechanisms and provieds functionality to deduplicate
/// and broadcast chain events to multiple subscribers
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ChainStream {
    chain: Chain,
    sender: broadcast::Sender<ChainData>,
    block_header_cache: Arc<RwLock<LruCache<String, ()>>>,
    tx_cache: Arc<RwLock<LruCache<String, ()>>>,
}

#[allow(dead_code)]
impl ChainStream {
    /// Creates a new [`ChainStream`] instance with specified chain.
    pub fn new(chain: Chain) -> Self {
        let (sender, _) = broadcast::channel(100);

        Self {
            chain,
            sender,
            block_header_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(1_0000).unwrap(),
            ))),
            tx_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(1_0000).unwrap(),
            ))),
        }
    }
    /// Processes incoming chain data, caching and broadcasting new blocks and transactions.
    /// Implements deduplication using LRU cache to prevent broadcasting duplicate events.
    pub(crate) async fn process_data(&self, data: ChainData) -> Result<(), ChainStreamError> {
        match data {
            ChainData::AnyRPCNetwork(eth_data) => match eth_data {
                AnyRPCNetwork::Block(block) => {
                    let block_id = format!("{:?}-{:?}", block.header.number, block.header.hash);

                    let mut cache = self.block_header_cache.write().await;
                    if cache.put(block_id, ()).is_none() {
                        self.sender
                            .send(ChainData::AnyRPCNetwork(AnyRPCNetwork::Block(block)))
                            .map_err(ChainStreamError::BroadcastError)?;
                    }
                }
                AnyRPCNetwork::TransactionPool(tx) => {
                    let mut cache = self.tx_cache.write().await;

                    let tx_id = format!("{:?}-{:?}", tx.transaction_index, tx.block_hash);
                    if cache.put(tx_id, ()).is_none() {
                        self.sender
                            .send(ChainData::AnyRPCNetwork(AnyRPCNetwork::TransactionPool(tx)))
                            .map_err(ChainStreamError::BroadcastError)?;
                    }
                }
            },
        }
        Ok(())
    }

    /// Creates a new subscription to the chain data broadcast channel.
    pub fn subscribe(&self) -> broadcast::Receiver<ChainData> {
        self.sender.subscribe()
    }

    /// Checks if a block with the given ID has been processed.
    pub async fn has_seen_block(&self, block_id: &str) -> bool {
        self.block_header_cache.read().await.contains(block_id)
    }

    /// Checks if a transaction with the given ID has been processed.
    pub async fn has_seen_tx(&self, tx_id: &str) -> bool {
        self.tx_cache.read().await.contains(tx_id)
    }
}
