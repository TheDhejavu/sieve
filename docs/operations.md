## Logical Operations
Logical operations in sieve is about combining different conditions to filter stream. In their simplest form, they allow sieve to combine basic checks using AND, OR, and NOT operators. Think of them as building blocks that let you say things like *"trigger events for all blocks with transactions above 10 ETH"* or *"show me events that are either deposits or withdrawals."*

**Operations:**

- `AND` - All conditions must be true
- `OR` - At least one condition must be true
- `NOT` - Negate the condition
- `ANY_OF` - Like OR but for grouped conditions
- `ALL_OF` - Like AND but for grouped conditions
- `XOR` - Exactly one condition must be true
- `UNLESS` - Alias for NOT

## Core Logical Operations

### OR Operations

The `or` and `any_of` operations trigger when at least one condition is true. 

```rust
// Simple OR filter for transaction monitoring
let filter = FilterBuilder::new().transaction(|f| {
    f.or(|tx| {
        tx.value().gt(U256::from(1000));     // Value > 1000
        tx.gas_price().lt(50000);            // OR Gas price < 50
        tx.nonce().eq(5);                    // OR Nonce = 5
    });
});

// ANY_OF example for protocol monitoring
let filter = FilterBuilder::new().event(|f| {
    f.any_of(|e| {
        e.contract().exact("0xdAC17F958D2ee523a2206206994597C13D831ec7"); // USDT
        e.contract().exact("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"); 
        e.contract().exact("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"); // USDC
    });
});
```

### AND Operations

The `and` and `all_of` operations require all conditions to be true. They're useful for precise filtering scenarios where multiple criteria must be met.

```rust
// Using AND to ensure multiple conditions
let filter = FilterBuilder::new().transaction(|f| {
    f.all_of(|f| {
        f.gas_price().between(50, 150);      // Gas price must be in range
        f.value().gt(U256::from(100));       // AND value must be high
    });
});
```

### XOR Operation

The `xor` operation ensures exactly one condition is true, useful for mutually exclusive scenarios.

```rust
// XOR for mutually exclusive conditions
let filter = FilterBuilder::new().xor(|f| {
    f.block(|b| b.number().gt(1000000));     // Either high block number
    f.block(|b| b.gas_used().lt(100000));    // OR low gas usage, but not both
});
```

### Negation Operations

The `not` and `unless` operations exclude matching conditions from the results.

```rust
// Excluding specific transactions
let filter = FilterBuilder::new().not(|f| {
    f.tx(|t| t.from().eq("0xdAC17F958D2ee523a2206206994597C13D831ec7"));       // Exclude from specific address
});

// UNLESS as an alternative syntax
let filter = FilterBuilder::new().unless(|f| {
    f.tx(|t| t.value().eq(0));               // Skip zero-value transfers
});
```

## Advanced Usage Examples

### Multi-Protocol Monitoring

Combine multiple operations to monitor various protocols simultaneously:

```rust
let filter = FilterBuilder::new().event(|f| {
    // Monitor multiple tokens & DEX
    f.any_of(|e| {
        e.contract().exact("0xc00e94Cb662C3520282E6f5717214004A7f26888");
        e.contract().exact("0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B");
        e.contract().exact("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
    });

    // Monitor lending protocols
    f.any_of(|e| {
        e.contract().exact("0xc00e94Cb662C3520282E6f5717214004A7f26888"); // COMP
        e.contract().exact("0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B"); // Compound 
        e.topics().contains("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
    });
});
```

### Transaction Pattern Recognition

Create complex patterns to identify specific transaction types:

```rust
let filter = FilterBuilder::new().transaction(|f| {
    f.value().gt(U256::from(100));           // Base value requirement

    f.all_of(|f| {
        f.gas_price().between(50, 150);      // Gas price in range
    });

    f.or(|t| {
        t.gas().gt(500000);                  // Either high gas
        t.value().eq(U256::from(100));       // OR specific value
    });
});
```

### L2 Chain Filtering

Special filters for Layer 2 chains like Optimism:

```rust
let filter = FilterBuilder::new().optimism(|op| {
    op.all_of(|f| {
        f.field("sequenceNumber").gt(500u64);
        f.field("batch.index").gt(100u128);
    });

    op.any_of(|f| {
        f.field("l1BlockNumber").gt(1000000000000000000u128);
        f.field("l1TxOrigin").starts_with("0x");
    });
});
```