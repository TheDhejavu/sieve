# sieve
A real-time data streaming & filtering engine for Ethereum & the superchain.


## Overview
Sieve offers a simple and expressive way for filtering blockchain data streams and emitting events when specified conditions are met. We try to make sieve as humanly expressive as possible (comparable to "magic"). It's also an experiment - if it fails, we throw it away and rebuild from scratch. The major pain point is, we want you to be able to create listeners (streams from filters) dynamically (millions if possible) that emit events based on this. Let's imagine something: your user sends 100ETH on base chain and immediately you set up a listener on the fly to listen to this event on the base network and react accordingly. The listeners stay active till seen / timeouts. We also try to do alot of things like decoding data when we come accross fields with conditions that needs decoded data for evaluation, it's recommended to be explicit in this case by including correlated conditions to help Sieve understand exactly what to look for. However, without specific explicit instructions, Sieve falls back to heuristic approaches which, while functional, may impact performance.

### Supported Emitted Events
- Transactions: Both confirmed and pending.
- Events (Logs): Filtered logs from smart contract interactions.
- Block Headers: Key details from block headers.

## Streaming Layer 
The system ingests blockchain data through both RPC and Gossipsub protocols, each chain configuration specifying its RPC endpoints, WebSocket connections, Gossipsub address, and bootstrap peers.

It is composed of **three main components** that work together to provide a reliable block & transaction stream. 

- Node Manager layer
- Connection Orchestrator
- Ingestion Pipeline

## L1 (Ethereum)
We prioritize Ethereum data expressiveness by hardcoding commonly used fields, since these fields are relatively stable across the Ethereum ecosystem and often share relationships with L2s. 

### Filter (*v1.0*)
```rust
let value_filter = FilterBuilder::new()
    .tx(|t| {
        t.value().gt(U256::from(1000000));
        t.gas_price().lt(50_000_000_000);
        t.gas().between(21000, 100000);
        t.nonce().eq(5);

        t.access_list()
            .contains("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
    });
    .build();
```

### ingest / watcher (*v1.0*):
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

## L2 (Superchain)
Supporting L2s through chain context and dynamic fields. Rather than hardcoding chain-specific logic, developers can specify chain context and use flexible field conditions, while still maintaining harcoded cross-chain specific name

### Idea ?
**Basic filter**:

```rust
let op_filter = FilterBuilder::new()
    .optimism(|op| {
        op.field("l1BlockNumber").gt(1000000000000000000u128);

        op.field("l1TxOrigin").starts_with("0x");
        op.field("queueIndex").lt(100u64);

    })
    .build();

let base_filter = FilterBuilder::new()
    .base(|base| {
        base.field("l1BlockNumber").gt(1000000000000000000u128);

    })
    .build();
```

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
    // Single chain (L1)
    let eth_filter = FilterBuilder::new()
        .tx(|t| {
            t.value().gt(1000);          // Value > 1000
            t.gas_price().lt(50);        // Gas price < 50
        })
        .build();

    // Single chain (L2 - Optimisim)
    let op_filter = FilterBuilder::new()
        .optimism(|op| op.field("l1BlockNumber").gt(2000))
        .build();
}
```

## Stream Listeners
**Subsribe:**
```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    let mut stream = runtime.subsribe(eth_filter);

    while let Some(event) = stream.next().await {
        match event {
           println!("{:?} new event", event);
        }
    }
}
```

**Subscribe All:**
The `subscribe_all` context allows you to subscribe to independent filters

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    let mut stream = runtime.subscribe_all([eth_filter, op_filter]);

    while let Some(event) = stream.next().await {
        match event {
           println!("{:?} new event", event);
        }
    }
}
```

**Watch Within:**
The `watch_within` context allows for time-bounded cross-chain correlation

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    let mut stream = runtime.watch_within(
        Duration::from_secs(1800),  // 30 min window
        eth_filter,
        op_filter
    );

    while let Some(event) = stream.next().await {
        match event {
            Event::Match { event_1, event_2 } => {
                println!("Matched events within time window");
            }
            Event::Timeout(event) => {
                println!("Event timed out");
            }
        }
    }
}
```

## Status
🚧 Experimental - Not ready for production use 