# sieve
<!-- [![CICD](https://github.com/thedhejavu/sieve/actions/workflows/CI.yml/badge.svg)](https://github.com/TheDhejavu/sieve/actions/workflows/CI.yml) -->
![minimum rustc 1.81](https://img.shields.io/badge/rustc-1.81.0+-red.svg)


![Banner](docs/images/banner.png)

A real-time data streaming engine with an expressive DSL for Ethereum & superchains

## Overview
Sieve offers a simple and expressive way for filtering blockchain data streams and emitting events when specified conditions are met. We try to make sieve as humanly expressive as possible (comparable to "magic"). It's also an experiment - if it fails, we throw it away and rebuild from scratch. The major pain point is, we want you to be able to create listeners (streams from filters) dynamically (millions if possible) that emit events based on this. Let's imagine something: your user sends 100ETH on base chain and immediately you set up a listener on the fly to listen to this event on the base network and react accordingly. The listeners stay active till seen / timeouts. We also try to do alot of things like decoding data when we come accross fields with conditions that needs decoded data for evaluation, it's recommended to be explicit in this case by including correlated conditions to help Sieve understand exactly what to look for. However, without specific explicit instructions, Sieve falls back to heuristic approaches which, while functional, may impact performance.

### Supported Emitted Events
- Transactions: Both confirmed and pending.
- Events (Logs): Filtered logs from smart contract interactions.
- Block Headers: Key details from block headers.

## Streaming Layer 
The system ingests blockchain data through both RPC and Gossipsub protocols, each chain configuration specifying its RPC endpoints, WebSocket connections, Gossipsub address, and bootstrap peers.

It is composed of **three main components** that work together to provide a reliable block & transaction stream. 

- Network layer
- Connection Orchestrator
- Ingestion Pipeline

## L1 (Ethereum)
We prioritize Ethereum data expressiveness by hardcoding commonly used fields, since these fields are relatively stable across the Ethereum ecosystem and often share relationships with L2s. 

### Filter (*v1.0*)
```rust
// Simple OR filter for transaction monitoring
let filter = FilterBuilder::new().transaction(|f| {
    f.or(|tx| {
        // Value > 1000
        tx.value().gt(U256::from(1000));

        // OR Gas price < 50
        tx.gas_price().lt(50000); 
        
        // OR Nonce = 5          
        tx.nonce().eq(5);                  
    });
});

// Pattern matching using AND/OR combinations
let filter = FilterBuilder::new().transaction(|f| {
    // Base value requirement
    f.value().gt(U256::from(100));           

    f.all_of(|f| {
        // Gas price must be in range
        f.gas_price().between(50, 150);     
    });

    f.or(|t| {
        // Either high gas
        t.gas().gt(500000);     
        // OR specific value            
        t.value().eq(U256::from(100));       
    });
});
```

### ingest / watcher (*v1.0*):
**RPC Calls (*busy-polling*):**

- Pending Transactions:
    - `eth_newPendingTransactionFilter`
- Block & Transactions:
    - `eth_getBlockByNumber`

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
use sieve::FilterBuilder;

fn main() {
    // Create a filter for Optimism-related events
    let op_filter = FilterBuilder::new()
        .optimism(|op| {
            // Filter for events where the `l1BlockNumber` is greater than 10^18
            op.field("l1BlockNumber").gt(1000000000000000000u128);

            // Filter for events where `l1TxOrigin` starts with "0x" 
            op.field("l1TxOrigin").starts_with("0x");

            // Filter for events where `queueIndex` is less than 100
            op.field("queueIndex").lt(100u64);
        });

    // Create a filter for Base-related events
    let base_filter = FilterBuilder::new()
        .base(|base| {
            // Filter for events where the `l1BlockNumber` is greater than 10^18
            base.field("l1BlockNumber").gt(1000000000000000000u128);
        });
}

```

### Proposed Usage (*stream*):
```rust
use sieve::{Sieve, config::{ChainConfig, Chain}, FilterBuilder, NumericOps, StringOps};

#[tokio::main]
use sieve::{FilterBuilder, NumericOps, StringOps};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Chain Configuration
    let chains = vec![
        // Optimisim chain....
        ChainConfigBuilder::builder()
            .rpc("https://mainnet.optimism.io/...")    
            .ws("wss://ws-mainnet.optimism.io/...")     
            .gossipsub("/ip4/0.0.0.0/tcp/9000")    
            .bootstrap_peers(vec!["/ip4/127.0.0.1/tcp/8000"])
            .chain(Chain::Optimisim)
            .build(),    

        // Base chain.....
        ChainConfigBuilder::builder()
            .rpc("https://mainnet.base.org")        
            .chain(Chain::Base)
            .build(),         

        // Ethereum chain....
        ChainConfigBuilder::builder()
            .rpc("https://mainnet.infura.io/v3/...") 
            .chain(Chain::Ethereum)   
            .build(),                        
    ];

    // 2. Connect to chains via `Sieve`
    let sieve = Sieve::connect(chains)?;                   

    // 3. Create Filter
   let pool_filter = FilterBuilder::new().pool(|f| {
        f.any_of(|p| {
            // High value pending transaction
            p.value().gt(U256::from(1000000000000000000u64));
            // Specific sender/receiver
            p.from().starts_with("0xdead");
            p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
        });
    });

    // 4. Subscribe to stream
    let mut stream = sieve.subscribe(pool_filter.clone());
    while let Some(event) = stream.next().await {
        println!("Pool: {:?}", event);
    }

    Ok(())
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
        .transaction(|f| {
            f.or(|tx| {
                tx.value().gt(U256::from(1000));
                tx.gas_price().lt(50000);
            });
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
    // Subscribe to events matching the Ethereum filter
    let mut stream = sieve.subscribe(eth_filter);

    // Process incoming events from the subscription
    while let Some(event) = stream.next().await {
        println!("{:?} new event", event);
    }
}

```

**Subscribe All:**
The `subscribe_all` context allows you to subscribe to independent filters

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    // Subscribe to all events that match the provided filters
    let mut stream = sieve.subscribe_all([eth_filter, op_filter]);

    // Process incoming events from the stream
    while let Some(event) = stream.next().await {
        println!("{:?} new event", event);
    }
}

```

**Watch Within:**
The `watch_within` context allows for time-bounded cross-chain correlation

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    // Create an event stream monitored within a 30-minute time window
    let mut stream = sieve.watch_within(
        Duration::from_secs(1800), // Define a 30-minute time window
        eth_filter,                // Filter for Ethereum-related events
        op_filter                  // Filter for operation-related events
    );

    // Process incoming events from the stream
    while let Some(event) = stream.next().await {
        match event {
            // Handle matched events within the time window
            Event::Match { l1_event, l2_event } => {
                println!("Matched events within time window");
            }
            // Handle events that timed out without a match
            Event::Timeout(_) => {
                println!("Event timed out");
            }
        }
    }
}

```

## Status
🚧 Experimental - Not ready for production use 