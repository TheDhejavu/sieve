use alloy_consensus::{transaction::PooledTransaction, Transaction, Typed2718};
use alloy_primitives::U256;
use alloy_rpc_types::Transaction as RpcTransaction;

use super::{
    conditions::{
        ArrayCondition, BlockCondition, ConditionBuilder, Evaluable, EventCondition,
        NumericCondition, PoolCondition, StringCondition, TransactionCondition,
    },
    operations::{
        ArrayFieldToCondition, ArrayOps, NumericOps, StringFieldToCondition, StringOps,
        U128FieldToCondition, U256FieldToCondition, U64FieldToCondition, U8FieldToCondition,
    },
};

pub struct U8FieldType<T>(pub T);
pub struct U64FieldType<T>(pub T);
pub struct U128FieldType<T>(pub T);
pub struct U256FieldType<T>(pub T);

pub struct StringFieldType<T>(pub T);
pub struct ArrayFieldType<T>(pub T);

// === Transaction Fields ======

// Transfer-specific fields
#[derive(Debug, Clone)]
pub enum TransferField {
    Method,  // The transfer method (transfer, transferFrom, approve)
    To,      // Recipient address
    From,    // Source address (for transferFrom)
    Amount,  // Transfer amount
    Spender, // Spender address (for approvals)
}

// Transaction specific field
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TxField {
    Nonce,          // Transaction sequence number
    Value,          // Amount of ETH being transferred
    Gas,            // Gas limit
    GasPrice,       // Gas price (legacy transactions)
    MaxFeePerGas,   // Maximum total fee per gas (EIP-1559)
    MaxPriorityFee, // Maximum priority fee per gas (EIP-1559)
    From,           // Sender address
    To,             // Recipient address (None for contract creation)
    Type,           // Transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)
    ChainId,
    AccessList,       // List of addresses and storage keys
    Hash,             // Transaction hash
    BlockNumber,      // Block number where tx was included
    BlockHash,        // Hash of the block where tx was included
    TransactionIndex, // Index of tx in the block
    Transfer(TransferField),
}

// Event-specific fields (logs)
#[derive(Debug, Clone)]
pub enum EventField {
    Contract,    // Contract address that generated the event
    Topics,      // Array of 0 to 4 32-byte topics (first is event signature)
    LogIndex,    // Integer of the log index position in the block
    BlockNumber, // Block number where this log was
    BlockHash,   // Hash of the block where this log was
    TxHash,      // Hash of the transaction that created this log
    TxIndex,     // Integer of the transaction's index position
}

// ===  Block-specific fields ===
#[derive(Debug, Clone)]
pub enum BlockField {
    Number,     // Block number/height
    Hash,       // Block hash
    ParentHash, // Previous block hash
    Timestamp,  // Block timestamp
    Size,       // Block size in bytes
    GasUsed,    // Gas used in this block
    GasLimit,   // Block gas limit
    BaseFee,    // Base fee per gas (EIP-1559)
    Miner,
    TransactionCount, // Number of transactions
    StateRoot,        // State root hash
    ReceiptsRoot,     // Receipts root hash
    TransactionsRoot, // Transactions root hash
}

// ==== Pool-specific fields (mempool) ====
#[derive(Debug, Clone)]
pub enum PoolField {
    Hash,             // Transaction hash
    From,             // Sender address
    To,               // Recipient address
    Nonce,            // Transaction nonce
    Value,            // ETH value
    GasPrice,         // Legacy gas price
    MaxFeePerGas,     // EIP-1559 max fee
    MaxPriorityFee,   // EIP-1559 priority fee
    FirstSeen,        // Timestamp first seen
    LastSeen,         // Timestamp last seen
    ReplacedBy,       // Hash of replacing transaction
    ReplacementCount, // Number of times replaced
    PropagationTime,  // Time to propagate to network
}

pub(crate) struct FieldWrapper<'a, T, P> {
    pub(crate) field: T,
    pub(crate) parent: &'a mut P,
}

impl U64FieldToCondition<TransactionCondition> for TxField {
    fn to_u64_condition(&self, value: NumericCondition<u64>) -> TransactionCondition {
        match self {
            TxField::Nonce => TransactionCondition::Nonce(value),
            TxField::Gas => TransactionCondition::Gas(value),
            TxField::ChainId => TransactionCondition::ChainId(value),
            TxField::BlockNumber => TransactionCondition::BlockNumber(value),
            TxField::TransactionIndex => TransactionCondition::TransactionIndex(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl U256FieldToCondition<TransactionCondition> for TxField {
    fn to_u256_condition(&self, value: NumericCondition<U256>) -> TransactionCondition {
        match self {
            TxField::Value => TransactionCondition::Value(value),
            TxField::Transfer(TransferField::Amount) => TransactionCondition::TransferAmount(value),
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

impl U8FieldToCondition<TransactionCondition> for TxField {
    fn to_u8_condition(&self, value: NumericCondition<u8>) -> TransactionCondition {
        match self {
            TxField::Type => TransactionCondition::Type(value),
            _ => panic!("Field does not support U8 numeric conditions"),
        }
    }
}

impl U128FieldToCondition<TransactionCondition> for TxField {
    fn to_u128_condition(&self, value: NumericCondition<u128>) -> TransactionCondition {
        match self {
            TxField::GasPrice => TransactionCondition::GasPrice(value),
            TxField::MaxFeePerGas => TransactionCondition::MaxFeePerGas(value),
            TxField::MaxPriorityFee => TransactionCondition::MaxPriorityFee(value),
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl U64FieldToCondition<EventCondition> for EventField {
    fn to_u64_condition(&self, value: NumericCondition<u64>) -> EventCondition {
        match self {
            EventField::LogIndex => EventCondition::LogIndex(value),
            EventField::BlockNumber => EventCondition::BlockNumber(value),
            EventField::TxIndex => EventCondition::TxIndex(value),
            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl U64FieldToCondition<BlockCondition> for BlockField {
    fn to_u64_condition(&self, value: NumericCondition<u64>) -> BlockCondition {
        match self {
            BlockField::Number => BlockCondition::Number(value),
            BlockField::Timestamp => BlockCondition::Timestamp(value),
            BlockField::Size => BlockCondition::Size(value),
            BlockField::GasUsed => BlockCondition::GasUsed(value),
            BlockField::GasLimit => BlockCondition::GasLimit(value),
            BlockField::TransactionCount => BlockCondition::TransactionCount(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl U128FieldToCondition<BlockCondition> for BlockField {
    fn to_u128_condition(&self, value: NumericCondition<u128>) -> BlockCondition {
        match self {
            BlockField::BaseFee => BlockCondition::BaseFee(value),
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl U64FieldToCondition<PoolCondition> for PoolField {
    fn to_u64_condition(&self, value: NumericCondition<u64>) -> PoolCondition {
        match self {
            PoolField::Nonce => PoolCondition::Nonce(value),
            PoolField::ReplacementCount => PoolCondition::ReplacementCount(value),
            PoolField::PropagationTime => PoolCondition::PropagationTime(value),
            PoolField::FirstSeen => PoolCondition::FirstSeen(value),
            PoolField::LastSeen => PoolCondition::LastSeen(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl U128FieldToCondition<PoolCondition> for PoolField {
    fn to_u128_condition(&self, value: NumericCondition<u128>) -> PoolCondition {
        match self {
            PoolField::GasPrice => PoolCondition::GasPrice(value),
            PoolField::MaxFeePerGas => PoolCondition::MaxFeePerGas(value),
            PoolField::MaxPriorityFee => PoolCondition::MaxPriorityFee(value),
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl U256FieldToCondition<PoolCondition> for PoolField {
    fn to_u256_condition(&self, value: NumericCondition<U256>) -> PoolCondition {
        match self {
            PoolField::Value => PoolCondition::Value(value),
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

macro_rules! impl_numeric_ops {
    ($type:ty, $field_type:ident, $field_trait:ident, $condition_method:ident) => {
        impl<T, P, C> NumericOps<$type> for FieldWrapper<'_, $field_type<T>, P>
        where
            T: $field_trait<C>,
            P: ConditionBuilder<Condition = C>,
        {
            fn gt(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::GreaterThan(value));
                self.parent.push_condition(condition);
            }

            fn lt(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::LessThan(value));
                self.parent.push_condition(condition);
            }

            fn eq(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::EqualTo(value));
                self.parent.push_condition(condition);
            }

            fn lte(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::LessThanOrEqualTo(value));
                self.parent.push_condition(condition);
            }

            fn gte(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::GreaterThanOrEqualTo(value));
                self.parent.push_condition(condition);
            }

            fn neq(self, value: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::NotEqualTo(value));
                self.parent.push_condition(condition);
            }

            fn between(self, min: $type, max: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::Between(min, max));
                self.parent.push_condition(condition);
            }

            fn outside(self, min: $type, max: $type) {
                let condition = self
                    .field
                    .0
                    .$condition_method(NumericCondition::Outside(min, max));
                self.parent.push_condition(condition);
            }
        }
    };
}

// NumericOps for each numeric type
impl_numeric_ops!(u8, U8FieldType, U8FieldToCondition, to_u8_condition);
impl_numeric_ops!(u64, U64FieldType, U64FieldToCondition, to_u64_condition);
impl_numeric_ops!(u128, U128FieldType, U128FieldToCondition, to_u128_condition);
impl_numeric_ops!(U256, U256FieldType, U256FieldToCondition, to_u256_condition);

impl StringFieldToCondition<TransactionCondition> for TxField {
    fn to_condition(&self, value: StringCondition) -> TransactionCondition {
        match self {
            TxField::From => TransactionCondition::From(value),
            TxField::To => TransactionCondition::To(value),
            TxField::Hash => TransactionCondition::Hash(value),
            TxField::BlockHash => TransactionCondition::BlockHash(value),
            TxField::Transfer(TransferField::Method) => TransactionCondition::TransferMethod(value),
            TxField::Transfer(TransferField::To) => TransactionCondition::TransferTo(value),
            TxField::Transfer(TransferField::From) => TransactionCondition::TransferFrom(value),
            TxField::Transfer(TransferField::Spender) => {
                TransactionCondition::TransferSpender(value)
            }
            // Non-string fields should panic
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl StringFieldToCondition<EventCondition> for EventField {
    fn to_condition(&self, value: StringCondition) -> EventCondition {
        match self {
            EventField::Contract => EventCondition::Contract(value),
            EventField::BlockHash => EventCondition::BlockHash(value),
            EventField::TxHash => EventCondition::TxHash(value),
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl StringFieldToCondition<PoolCondition> for PoolField {
    fn to_condition(&self, value: StringCondition) -> PoolCondition {
        match self {
            PoolField::Hash => PoolCondition::Hash(value),
            PoolField::From => PoolCondition::From(value),
            PoolField::To => PoolCondition::To(value),
            PoolField::ReplacedBy => PoolCondition::ReplacedBy(value),

            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl StringFieldToCondition<BlockCondition> for BlockField {
    fn to_condition(&self, value: StringCondition) -> BlockCondition {
        match self {
            BlockField::Hash => BlockCondition::Hash(value),
            BlockField::ParentHash => BlockCondition::ParentHash(value),
            BlockField::Miner => BlockCondition::Miner(value),
            BlockField::StateRoot => BlockCondition::StateRoot(value),
            BlockField::ReceiptsRoot => BlockCondition::ReceiptsRoot(value),
            BlockField::TransactionsRoot => BlockCondition::TransactionsRoot(value),

            _ => panic!("Field does not support string conditions"),
        }
    }
}

// === StringOps =====
impl<T, P, C> StringOps for FieldWrapper<'_, StringFieldType<T>, P>
where
    T: StringFieldToCondition<C>,
    P: ConditionBuilder<Condition = C>,
{
    fn starts_with(self, prefix: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::StartsWith(prefix.to_string()));
        self.parent.push_condition(condition);
    }

    fn ends_with(self, suffix: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::EndsWith(suffix.to_string()));
        self.parent.push_condition(condition);
    }

    fn contains(self, substring: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::Contains(substring.to_string()));
        self.parent.push_condition(condition);
    }

    fn matches(self, regex_pattern: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::Matches(regex_pattern.to_string()));
        self.parent.push_condition(condition);
    }

    fn eq(self, value: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::EqualTo(value.to_string()));
        self.parent.push_condition(condition);
    }
}

impl ArrayFieldToCondition<TransactionCondition, String> for TxField {
    fn to_condition(&self, value: ArrayCondition<String>) -> TransactionCondition {
        match self {
            TxField::AccessList => TransactionCondition::AccessList(value),
            _ => panic!("Field does not support string array conditions"),
        }
    }
}

impl ArrayFieldToCondition<EventCondition, String> for EventField {
    fn to_condition(&self, value: ArrayCondition<String>) -> EventCondition {
        match self {
            EventField::Topics => EventCondition::Topics(value),
            _ => panic!("Field does not support array conditions"),
        }
    }
}

impl<F, B, C, T> ArrayOps<T> for FieldWrapper<'_, ArrayFieldType<F>, B>
where
    F: ArrayFieldToCondition<C, T>,
    B: ConditionBuilder<Condition = C>,
    T: Clone,
{
    fn contains(self, value: T) {
        let condition = self.field.0.to_condition(ArrayCondition::Contains(value));
        self.parent.push_condition(condition);
    }

    fn not_in(self, values: Vec<T>) {
        let condition = self.field.0.to_condition(ArrayCondition::NotIn(values));
        self.parent.push_condition(condition);
    }

    fn empty(self) {
        let condition = self.field.0.to_condition(ArrayCondition::Empty);
        self.parent.push_condition(condition);
    }

    fn not_empty(self) {
        let condition = self.field.0.to_condition(ArrayCondition::NotEmpty);
        self.parent.push_condition(condition);
    }
}

impl Evaluable<RpcTransaction> for TransactionCondition {
    fn evaluate(&self, tx: &RpcTransaction) -> bool {
        match self {
            TransactionCondition::Value(condition) => condition.evaluate(&tx.value()),
            TransactionCondition::GasPrice(condition) => {
                condition.evaluate(&tx.gas_price().unwrap_or_default())
            }
            TransactionCondition::From(condition) => condition.evaluate(&tx.from.to_string()),
            TransactionCondition::MaxFeePerGas(condition) => {
                condition.evaluate(&tx.max_fee_per_gas())
            }
            TransactionCondition::MaxPriorityFee(condition) => {
                condition.evaluate(&tx.max_priority_fee_per_gas().unwrap_or_default())
            }
            TransactionCondition::BlockNumber(condition) => {
                condition.evaluate(&tx.block_number.unwrap_or_default())
            }
            TransactionCondition::BlockHash(condition) => {
                condition.evaluate(&tx.block_hash.unwrap_or_default().to_string())
            }
            TransactionCondition::ChainId(condition) => {
                condition.evaluate(&tx.chain_id().unwrap_or_default())
            }
            TransactionCondition::To(condition) => {
                condition.evaluate(&tx.to().unwrap_or_default().to_string())
            }
            TransactionCondition::Nonce(condition) => condition.evaluate(&tx.nonce()),
            TransactionCondition::Type(condition) => condition.evaluate(&tx.ty()),
            TransactionCondition::TransactionIndex(condition) => {
                condition.evaluate(&tx.transaction_index.unwrap_or_default())
            }
            TransactionCondition::Hash(condition) => {
                condition.evaluate(&tx.inner.tx_hash().to_string())
            }
            _ => false,
        }
    }
}

impl Evaluable<PooledTransaction> for PoolCondition {
    fn evaluate(&self, pool_tx: &PooledTransaction) -> bool {
        match self {
            PoolCondition::Hash(condition) => condition.evaluate(&pool_tx.hash().to_string()),
            PoolCondition::From(condition) => todo!(),
            PoolCondition::To(condition) => todo!(),
            PoolCondition::ReplacedBy(condition) => todo!(),
            PoolCondition::Value(condition) => todo!(),
            PoolCondition::GasPrice(condition) => todo!(),
            PoolCondition::MaxFeePerGas(condition) => todo!(),
            PoolCondition::MaxPriorityFee(condition) => todo!(),
            PoolCondition::Gas(condition) => todo!(),
            PoolCondition::Nonce(condition) => todo!(),
            PoolCondition::ReplacementCount(condition) => todo!(),
            PoolCondition::PropagationTime(condition) => todo!(),
            PoolCondition::FirstSeen(condition) => todo!(),
            PoolCondition::LastSeen(condition) => todo!(),
        }
    }
}
