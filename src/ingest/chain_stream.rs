use lru::LruCache;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::{
    lru_cache,
    network::orchestrator::{ChainData, EthereumData},
};

use super::Chain;

#[derive(Clone)]
pub(crate) struct ChainStream {
    chain: Chain,
    sender: broadcast::Sender<ChainData>,
    block_header_cache: Arc<RwLock<LruCache<String, ()>>>,
    tx_cache: Arc<RwLock<LruCache<String, ()>>>,
}

impl ChainStream {
    pub fn new(chain: Chain) -> Self {
        let (sender, _) = broadcast::channel(100);

        Self {
            chain,
            sender,
            block_header_cache: lru_cache!(1_000),
            tx_cache: lru_cache!(10_000),
        }
    }

    pub(crate) async fn process_data(
        &self,
        data: ChainData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match data {
            ChainData::Ethereum(eth_data) => {
                match eth_data {
                    EthereumData::BlockHeader(header) => {
                        let block_header_id = format!("{:?}-{:?}", header.number, header.hash);

                        let mut cache = self.block_header_cache.write().await;
                        if cache.put(block_header_id, ()).is_none() {
                            let _ = self
                                .sender
                                .send(ChainData::Ethereum(EthereumData::BlockHeader(header)));
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
                    EthereumData::Transaction(transaction) => todo!(),
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
