use super::{
    conditions::{
        ArrayCondition, BlockHeaderCondition, ContractCondition,
        DynFieldCondition, EventCondition, FilterCondition, NodeBuilder, NumericCondition,
        PoolCondition, StringCondition, TransactionCondition, ValueCondition,
    },
    operations::{ArrayOps, NumericOps, StringOps},
};
use alloy_primitives::U256;

pub struct U8FieldType<T>(pub T);
pub struct U64FieldType<T>(pub T);
pub struct U128FieldType<T>(pub T);
pub struct U256FieldType<T>(pub T);
pub struct StringFieldType<T>(pub T);
pub struct ArrayFieldType<T>(pub T);

pub struct DynValueFieldType<T>(pub T);

pub struct U8FieldCondition<T>(pub T, pub NumericCondition<u8>);
pub struct U64FieldCondition<T>(pub T, pub NumericCondition<u64>);
pub struct U128FieldCondition<T>(pub T, pub NumericCondition<u128>);
pub struct U256FieldCondition<T>(pub T, pub NumericCondition<U256>);
pub struct StringFieldCondition<T>(pub T, pub StringCondition);
pub struct ArrayFieldCondition<T, V>(pub T, pub ArrayCondition<V>);

// === Transaction Fields ======
// Contract-specific fields
#[derive(Debug, Clone)]
pub enum ContractField {
    Method,            // Function name/selector
    Parameter(String), // Named parameter: "amountIn", "to", "amount" etc
    Path(String),      // For nested params like "path.tokenIn"
}

pub struct DynField(pub String);

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
    Contract(ContractField),
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
    Name,
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
    StateRoot,        // State root hash
    ReceiptsRoot,     // Receipts root hash
    TransactionsRoot, // Transactions root hash
}

// ==== Pool-specific fields (mempool) ====
#[derive(Debug, Clone)]
pub enum PoolField {
    Hash,      // Transaction hash
    To,        // Recipient address
    From,      // Sender address
    Value,     // ETH value
    Nonce,     // Transaction nonce
    GasPrice,  // Gas price
    GasLimit,  // Gas limit
    Timestamp, // When tx added to pool
}

pub(crate) struct FieldWrapper<'a, T, P> {
    pub(crate) field: T,
    pub(crate) parent: &'a mut P,
}

impl From<U64FieldCondition<TxField>> for TransactionCondition {
    fn from(fc: U64FieldCondition<TxField>) -> TransactionCondition {
        let U64FieldCondition(field, value) = fc;
        match field {
            TxField::Nonce => TransactionCondition::Nonce(value),
            TxField::Gas => TransactionCondition::Gas(value),
            TxField::ChainId => TransactionCondition::ChainId(value),
            TxField::BlockNumber => TransactionCondition::BlockNumber(value),
            TxField::TransactionIndex => TransactionCondition::TransactionIndex(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl From<U256FieldCondition<TxField>> for TransactionCondition {
    fn from(fc: U256FieldCondition<TxField>) -> TransactionCondition {
        let U256FieldCondition(field, value) = fc;
        match field {
            TxField::Value => TransactionCondition::Value(value),
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

impl From<U8FieldCondition<TxField>> for TransactionCondition {
    fn from(fc: U8FieldCondition<TxField>) -> TransactionCondition {
        let U8FieldCondition(field, value) = fc;
        match field {
            TxField::Type => TransactionCondition::Type(value),
            _ => panic!("Field does not support U8 numeric conditions"),
        }
    }
}

impl From<U128FieldCondition<TxField>> for TransactionCondition {
    fn from(fc: U128FieldCondition<TxField>) -> TransactionCondition {
        let U128FieldCondition(field, value) = fc;
        match field {
            TxField::GasPrice => TransactionCondition::GasPrice(value),
            TxField::MaxFeePerGas => TransactionCondition::MaxFeePerGas(value),
            TxField::MaxPriorityFee => TransactionCondition::MaxPriorityFee(value),
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl From<U64FieldCondition<EventField>> for EventCondition {
    fn from(fc: U64FieldCondition<EventField>) -> EventCondition {
        let U64FieldCondition(field, value) = fc;
        match field {
            EventField::LogIndex => EventCondition::LogIndex(value),
            EventField::BlockNumber => EventCondition::BlockNumber(value),
            EventField::TxIndex => EventCondition::TxIndex(value),
            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl From<U64FieldCondition<BlockField>> for BlockHeaderCondition {
    fn from(fc: U64FieldCondition<BlockField>) -> BlockHeaderCondition {
        let U64FieldCondition(field, value) = fc;
        match field {
            BlockField::Number => BlockHeaderCondition::Number(value),
            BlockField::Timestamp => BlockHeaderCondition::Timestamp(value),
            BlockField::GasUsed => BlockHeaderCondition::GasUsed(value),
            BlockField::GasLimit => BlockHeaderCondition::GasLimit(value),
            BlockField::BaseFee => BlockHeaderCondition::BaseFee(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl From<U256FieldCondition<BlockField>> for BlockHeaderCondition {
    fn from(fc: U256FieldCondition<BlockField>) -> BlockHeaderCondition {
        let U256FieldCondition(field, value) = fc;
        match field {
            BlockField::Size => BlockHeaderCondition::Size(value),
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

impl From<U128FieldCondition<ContractField>> for ContractCondition {
    fn from(fc: U128FieldCondition<ContractField>) -> ContractCondition {
        let U128FieldCondition(field, value) = fc;
        match field {
            ContractField::Parameter(path) => {
                ContractCondition::Parameter(path.to_string(), ValueCondition::U128(value))
            }
            ContractField::Path(path) => {
                ContractCondition::Path(path.to_string(), ValueCondition::U128(value))
            }
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl From<U256FieldCondition<ContractField>> for ContractCondition {
    fn from(fc: U256FieldCondition<ContractField>) -> ContractCondition {
        let U256FieldCondition(field, value) = fc;
        match field {
            ContractField::Parameter(path) => {
                ContractCondition::Parameter(path.to_string(), ValueCondition::U256(value))
            }
            ContractField::Path(path) => {
                ContractCondition::Path(path.to_string(), ValueCondition::U256(value))
            }
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

impl From<StringFieldCondition<ContractField>> for ContractCondition {
    fn from(fc: StringFieldCondition<ContractField>) -> ContractCondition {
        let StringFieldCondition(field, value) = fc;
        match field {
            ContractField::Parameter(path) => {
                ContractCondition::Parameter(path.to_string(), ValueCondition::String(value))
            }
            ContractField::Path(path) => {
                ContractCondition::Path(path.to_string(), ValueCondition::String(value))
            }
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl From<U64FieldCondition<PoolField>> for PoolCondition {
    fn from(fc: U64FieldCondition<PoolField>) -> PoolCondition {
        let U64FieldCondition(field, value) = fc;
        match field {
            PoolField::Nonce => PoolCondition::Nonce(value),
            PoolField::GasLimit => PoolCondition::GasLimit(value),
            _ => panic!("Field does not support u64 numeric conditions"),
        }
    }
}

impl From<U128FieldCondition<PoolField>> for PoolCondition {
    fn from(fc: U128FieldCondition<PoolField>) -> PoolCondition {
        let U128FieldCondition(field, value) = fc;
        match field {
            PoolField::GasPrice => PoolCondition::GasPrice(value),
            _ => panic!("Field does not support U128 numeric conditions"),
        }
    }
}

impl From<U256FieldCondition<PoolField>> for PoolCondition {
    fn from(fc: U256FieldCondition<PoolField>) -> PoolCondition {
        let U256FieldCondition(field, value) = fc;
        match field {
            PoolField::Value => PoolCondition::Value(value),
            _ => panic!("Field does not support U256 numeric conditions"),
        }
    }
}

impl From<U64FieldCondition<DynField>> for BlockHeaderCondition {
    fn from(fc: U64FieldCondition<DynField>) -> BlockHeaderCondition {
        let U64FieldCondition(field, value) = fc;
        BlockHeaderCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U64(value),
        })
    }
}

impl From<U128FieldCondition<DynField>> for BlockHeaderCondition {
    fn from(fc: U128FieldCondition<DynField>) -> BlockHeaderCondition {
        let U128FieldCondition(field, value) = fc;
        BlockHeaderCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U128(value),
        })
    }
}

impl From<U256FieldCondition<DynField>> for BlockHeaderCondition {
    fn from(fc: U256FieldCondition<DynField>) -> BlockHeaderCondition {
        let U256FieldCondition(field, value) = fc;
        BlockHeaderCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U256(value),
        })
    }
}

impl From<U64FieldCondition<DynField>> for FilterCondition {
    fn from(fc: U64FieldCondition<DynField>) -> FilterCondition {
        let U64FieldCondition(field, value) = fc;
        FilterCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U64(value),
        })
    }
}

impl From<U128FieldCondition<DynField>> for FilterCondition {
    fn from(fc: U128FieldCondition<DynField>) -> FilterCondition {
        let U128FieldCondition(field, value) = fc;
        FilterCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U128(value),
        })
    }
}

impl From<U256FieldCondition<DynField>> for FilterCondition {
    fn from(fc: U256FieldCondition<DynField>) -> FilterCondition {
        let U256FieldCondition(field, value) = fc;
        FilterCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::U256(value),
        })
    }
}

impl From<StringFieldCondition<DynField>> for FilterCondition {
    fn from(fc: StringFieldCondition<DynField>) -> FilterCondition {
        let StringFieldCondition(field, value) = fc;
        FilterCondition::DynField(DynFieldCondition {
            path: field.0,
            condition: ValueCondition::String(value),
        })
    }
}

macro_rules! impl_numeric_ops {
    ($type:ty, $field_type:ident, $condition_type:ident) => {
        impl<T, P, C> NumericOps<$type> for FieldWrapper<'_, $field_type<T>, P>
        where
            $condition_type<T>: Into<C>,
            P: NodeBuilder<Condition = C>,
        {
            fn gt(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::GreaterThan(value)).into();
                self.parent.append_node(condition);
            }

            fn lt(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::LessThan(value)).into();
                self.parent.append_node(condition);
            }

            fn eq(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::EqualTo(value)).into();
                self.parent.append_node(condition);
            }

            fn lte(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::LessThanOrEqualTo(value))
                        .into();
                self.parent.append_node(condition);
            }

            fn gte(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::GreaterThanOrEqualTo(value))
                        .into();
                self.parent.append_node(condition);
            }

            fn neq(self, value: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::NotEqualTo(value)).into();
                self.parent.append_node(condition);
            }

            fn between(self, min: $type, max: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::Between(min, max)).into();
                self.parent.append_node(condition);
            }

            fn outside(self, min: $type, max: $type) {
                let condition =
                    $condition_type(self.field.0, NumericCondition::Outside(min, max)).into();
                self.parent.append_node(condition);
            }
        }
    };
}

// NumericOps for each Types
impl_numeric_ops!(u8, U8FieldType, U8FieldCondition);
impl_numeric_ops!(u64, U64FieldType, U64FieldCondition);
impl_numeric_ops!(u128, U128FieldType, U128FieldCondition);
impl_numeric_ops!(U256, U256FieldType, U256FieldCondition);

impl_numeric_ops!(u64, DynValueFieldType, U64FieldCondition);
impl_numeric_ops!(u128, DynValueFieldType, U128FieldCondition);
impl_numeric_ops!(U256, DynValueFieldType, U256FieldCondition);

impl From<StringFieldCondition<TxField>> for TransactionCondition {
    fn from(fc: StringFieldCondition<TxField>) -> TransactionCondition {
        let StringFieldCondition(field, value) = fc;
        match field {
            TxField::From => TransactionCondition::From(value),
            TxField::To => TransactionCondition::To(value),
            TxField::Hash => TransactionCondition::Hash(value),
            TxField::BlockHash => TransactionCondition::BlockHash(value),
            // Non-string fields should panic
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl From<StringFieldCondition<EventField>> for EventCondition {
    fn from(fc: StringFieldCondition<EventField>) -> EventCondition {
        let StringFieldCondition(field, value) = fc;
        match field {
            EventField::Contract => EventCondition::Contract(value),
            EventField::BlockHash => EventCondition::BlockHash(value),
            EventField::TxHash => EventCondition::TxHash(value),
            // EventField::Parameter(param) => EventCondition::Parameter(param.to_string(), value),
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl From<StringFieldCondition<PoolField>> for PoolCondition {
    fn from(fc: StringFieldCondition<PoolField>) -> PoolCondition {
        let StringFieldCondition(field, value) = fc;
        match field {
            PoolField::Hash => PoolCondition::Hash(value),
            PoolField::From => PoolCondition::From(value),
            PoolField::To => PoolCondition::To(value),
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl From<StringFieldCondition<BlockField>> for BlockHeaderCondition {
    fn from(fc: StringFieldCondition<BlockField>) -> BlockHeaderCondition {
        let StringFieldCondition(field, value) = fc;
        match field {
            BlockField::Hash => BlockHeaderCondition::Hash(value),
            BlockField::ParentHash => BlockHeaderCondition::ParentHash(value),
            BlockField::StateRoot => BlockHeaderCondition::StateRoot(value),
            BlockField::ReceiptsRoot => BlockHeaderCondition::ReceiptsRoot(value),
            BlockField::TransactionsRoot => BlockHeaderCondition::TransactionsRoot(value),
            _ => panic!("Field does not support string conditions"),
        }
    }
}

// === StringOps =====
macro_rules! impl_string_ops {
    ($field_type:ident, $condition_type:ident) => {
        impl<T, P, C> StringOps for FieldWrapper<'_, $field_type<T>, P>
        where
            $condition_type<T>: Into<C>,
            P: NodeBuilder<Condition = C>,
        {
            fn starts_with(self, prefix: &str) {
                let condition = $condition_type(
                    self.field.0,
                    StringCondition::StartsWith(prefix.to_string()),
                )
                .into();
                self.parent.append_node(condition);
            }

            fn ends_with(self, suffix: &str) {
                let condition =
                    $condition_type(self.field.0, StringCondition::EndsWith(suffix.to_string()))
                        .into();
                self.parent.append_node(condition);
            }

            fn contains(self, substring: &str) {
                let condition = $condition_type(
                    self.field.0,
                    StringCondition::Contains(substring.to_string()),
                )
                .into();
                self.parent.append_node(condition);
            }

            fn matches(self, regex_pattern: &str) {
                let condition = $condition_type(
                    self.field.0,
                    StringCondition::Matches(regex_pattern.to_string()),
                )
                .into();
                self.parent.append_node(condition);
            }

            fn exact(self, value: &str) {
                let condition =
                    $condition_type(self.field.0, StringCondition::EqualTo(value.to_string()))
                        .into();
                self.parent.append_node(condition);
            }
        }
    };
}

impl_string_ops!(DynValueFieldType, StringFieldCondition);
impl_string_ops!(StringFieldType, StringFieldCondition);

impl From<ArrayFieldCondition<TxField, String>> for TransactionCondition {
    fn from(fc: ArrayFieldCondition<TxField, String>) -> TransactionCondition {
        let ArrayFieldCondition(field, value) = fc;
        match field {
            TxField::AccessList => TransactionCondition::AccessList(value),
            _ => panic!("Field does not support string array conditions"),
        }
    }
}

impl From<ArrayFieldCondition<EventField, String>> for EventCondition {
    fn from(fc: ArrayFieldCondition<EventField, String>) -> EventCondition {
        let ArrayFieldCondition(field, value) = fc;
        match field {
            EventField::Topics => EventCondition::Topics(value),
            _ => panic!("Field does not support array conditions"),
        }
    }
}

macro_rules! impl_array_ops {
    ($value_type:ty) => {
        impl<F, B, C> ArrayOps<$value_type> for FieldWrapper<'_, ArrayFieldType<F>, B>
        where
            ArrayFieldCondition<F, $value_type>: Into<C>,
            B: NodeBuilder<Condition = C>,
        {
            fn contains(self, value: $value_type) {
                let condition =
                    ArrayFieldCondition(self.field.0, ArrayCondition::Contains(value)).into();
                self.parent.append_node(condition);
            }

            fn not_in(self, values: Vec<$value_type>) {
                let condition =
                    ArrayFieldCondition(self.field.0, ArrayCondition::NotIn(values)).into();
                self.parent.append_node(condition);
            }

            fn empty(self) {
                let condition = ArrayFieldCondition(self.field.0, ArrayCondition::Empty).into();
                self.parent.append_node(condition);
            }

            fn not_empty(self) {
                let condition = ArrayFieldCondition(self.field.0, ArrayCondition::NotEmpty).into();
                self.parent.append_node(condition);
            }
        }
    };
}

impl_array_ops!(String);
// impl_array_ops!(DynField);
