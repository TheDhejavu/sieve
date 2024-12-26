use lru::LruCache;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::network::orchestrator::{ChainData, EthereumData};

use super::Chain;

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
    pub fn new(chain: Chain) -> Self {
        let (sender, _) = broadcast::channel(100);

        Self {
            chain,
            sender,
            block_header_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(1_0000).unwrap(),
            ))),
            tx_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(1_0000).unwrap(),
            ))),
        }
    }

    pub(crate) async fn process_data(
        &self,
        data: ChainData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match data {
            ChainData::Ethereum(eth_data) => {
                match eth_data {
                    EthereumData::Block(block) => {
                        let block_id = format!("{:?}-{:?}", block.header.number, block.header.hash);

                        let mut cache = self.block_header_cache.write().await;
                        if cache.put(block_id, ()).is_none() {
                            let _ = self
                                .sender
                                .send(ChainData::Ethereum(EthereumData::Block(block)));
                        }
                    }
                    EthereumData::TransactionPool(txs) => {
                        let mut unique_txs = Vec::new();
                        let mut cache = self.tx_cache.write().await;

                        for tx in txs {
                            let tx_id = format!("{:?}-{:?}", tx.transaction_index, tx.block_hash);
                            if cache.put(tx_id, ()).is_none() {
                                unique_txs.push(tx);
                            }
                        }

                        if !unique_txs.is_empty() {
                            let _ = self.sender.send(ChainData::Ethereum(
                                EthereumData::TransactionPool(unique_txs),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChainData> {
        self.sender.subscribe()
    }

    pub async fn has_seen_block(&self, block_id: &str) -> bool {
        self.block_header_cache.read().await.contains(block_id)
    }

    pub async fn has_seen_tx(&self, tx_id: &str) -> bool {
        self.tx_cache.read().await.contains(tx_id)
    }
}
