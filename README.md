# sieve
A real-time data streaming & filtering engine for Ethereum & the superchain.

## Overview
Sieve provides a simple and expressive way to filter blockchain data streams:
- Transactions (confirmed and pending)
- Events (logs)
- Blocks

## Streaming Layer 
The system ingests blockchain data through both RPC and Gossipsub protocols, each chain configuration specifying its RPC endpoints, WebSocket connections, Gossipsub address, and bootstrap peers.

It is composed of **three main components** that work together to provide a reliable block & transaction stream. 

- Node Manager layer
- Connection Orchestrator
- Ingestion Pipeline

### v1.0 (watcher):
**RPC Calls (*busy-polling*):**

- Pending Transactions:
    - `txpool_content`
    - `eth_newPendingTransactionFilter`
- Block & Transactions:
    - `eth_getBlockByNumber`
    - `eth_getBlockByHash`
    - `eth_getLogs`
    - `eth_getTransactionReceipt`

**Gossipsub (*reactive*):**

- Block gossip:
    - `BeaconBlock`
    - `ExecutionPayload`
    - `ExecutionPayloadHeader`
- Transaction gossip:
    - `TransactionAnnounce`
    - `TransactionPropagation`

**WebSocket (*reactive*):**

- `eth_subscribe`:
    - `newHeads`
    - `newPendingTransactions`
    - `logs`

## Emitted Events
- Transaction
- Event (log)
- Block Header 

### Proposed Usage (*stream*):
```rust
use sieve::{runtime::Runtime, config::Chain, FilterBuilder, NumericOps, StringOps};

#[tokio::main]
async fn main() {
    // 1. Chain Configuration
    let chains = vec![
        Chain::builder()
            .rpc("https://mainnet.optimism.io")
            .ws("wss://...")
            .gossipsub("/ip4/0.0.0.0/tcp/9000")
            .bootstrap_peers(vec![...])
            .name(OPTIMISIM),
        Chain::builder()
            .rpc("https://base-mainnet...")
            .name(BASE),
        Chain::builder()
            .rpc("https://eth-mainnet...")
            .name(ETHEREUM)
    ];

    // 2. Create Runtime with configuration
    let runtime = Runtime::builder()
        .chains(chains)
        .worker_threads(4)
        .build()?;

    // 3. Create Filter
    let value_filter = FilterBuilder::new()
        .tx(|t| {
            t.value().gt(1000);          // Value > 1000
            t.gas_price().lt(50);        // Gas price < 50
        })
        .build();

    // 4. Subscribe to stream
    let mut stream = runtime.subscribe(transfer_filter.clone());
    while let Some(event) = stream.next().await {
        println!("Transfer: {:?}", event);
    }

    // 5. Or Scheduled task
    let scheduled = runtime.submit_after(
        transfer_filter,
        |event| println!("Scheduled: {:?}", event),
        Duration::from_secs(10)
    );
}
```

## Filter Engine
The filter engine uses a tree structure to represent complex logical combinations of conditions that can match against blockchain data (transactions, blocks, events). 

### Usage

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    // Filter for high-value transactions
    let value_filter = FilterBuilder::new()
        .tx(|t| {
            t.value().gt(1000);          // Value > 1000
            t.gas_price().lt(50);        // Gas price < 50
        })
        .build();

    // Filter for Uniswap events
    let event_filter = FilterBuilder::new()
        .event(|e| {
            e.contract().eq("UniswapV2Factory");
        })
        .build();
}
```

## Status
🚧 Experimental - Not ready for production use 