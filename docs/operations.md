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

### Here are practical examples for each logical operator in the filter engine:

**or** - Match if one condition is true

```rust
FilterBuilder::new()
    .or(|f| {
        f.tx(|t| t.value().gt(100));            // Value > 100
        f.tx(|t| t.gas_price().lt(50));         // OR low gas price
    })
    .build();
```
**any_of** - Match if any condition is true (alias for `.or`)

```rust
FilterBuilder::new()
    .any_of(|f| {
        f.tx(|t| t.value().gt(1000));              
        f.event(|e| e.address().eq("0x.....")); 
    })
    .build();
```

**xor** - Exactly one condition must match 

```rust
FilterBuilder::new()
    .xor(|f| {
        f.block(|b| b.number().gt(1000000));    // Either high block
        f.block(|b| b.gas_used().lt(100000));   // OR low gas, not both
    })
    .build();
```

**not** - match everything EXCEPT the condition

```rust
FilterBuilder::new()
    .not(|f| {
        f.tx(|t| t.from().eq("0x......"));   // Not from blacklisted address
    })
    .build();

```

**unless** - Alias for `.not`

```rust
FilterBuilder::new()
    .unless(|f| {
        f.tx(|t| t.value().eq(0))  // Skip zero value transfers
    })
    .build();
```

**and** - All conditions must match 

```rust
FilterBuilder::new()
    .and(|f| {
        f.tx(|t| t.value().gt(1000));           // High value AND
        f.block(|b| b.gas_used().gt(800000));   // High gas usage
    })
    .build();
```

**all_of** - alias of `.and`

```rust
FilterBuilder::new()
    .all_of(|f| {
        f.tx(|t| t.value().gt(1000));           // High value AND
        f.block(|b| b.gas_used().gt(800000));   // High gas usage
    })
    .build();
```