use alloy_primitives::U256;
use std::cmp::PartialOrd;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    NoneOf,
    Xor,
}

// Trait to represent any numeric type we want to support
pub trait NumericType: Clone + PartialEq + PartialOrd {
    fn from_string(value: String) -> Self;
}

// Implement for our supported numeric types
impl NumericType for u64 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl NumericType for u8 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl NumericType for u128 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl NumericType for U256 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

// Generic numeric condition that works with any numeric type
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumericCondition<T: NumericType> {
    GreaterThan(T),
    GreaterThanOrEqualTo(T),
    LessThan(T),
    LessThanOrEqualTo(T),
    EqualTo(T),
    NotEqualTo(T),
    Between(T, T),
    Outside(T, T),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayCondition<T> {
    Contains(T),
    NotIn(Vec<T>),
    Empty,
    NotEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringCondition {
    EqualTo(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Matches(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FilterCondition {
    Transaction(TransactionCondition),
    Event(EventCondition),
    Pool(PoolCondition),
    Block(BlockCondition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TransactionCondition {
    Gas(NumericCondition<u64>),
    Nonce(NumericCondition<u64>),
    Type(NumericCondition<u8>),
    ChainId(NumericCondition<u64>),
    BlockNumber(NumericCondition<u64>),
    TransactionIndex(NumericCondition<u64>),
    Value(NumericCondition<U256>),
    GasPrice(NumericCondition<u128>),
    MaxFeePerGas(NumericCondition<u128>),
    MaxPriorityFee(NumericCondition<u128>),
    TransferAmount(NumericCondition<U256>),
    From(StringCondition),
    To(StringCondition),
    Hash(StringCondition),
    BlockHash(StringCondition),
    AccessList(ArrayCondition<String>),
    TransferMethod(StringCondition),
    TransferTo(StringCondition),
    TransferFrom(StringCondition),
    TransferSpender(StringCondition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EventCondition {
    // String conditions
    Contract(StringCondition),
    BlockHash(StringCondition),
    TxHash(StringCondition),

    // Numeric conditions
    LogIndex(NumericCondition<u64>),
    BlockNumber(NumericCondition<u64>),
    TxIndex(NumericCondition<u64>),

    // Array condition
    Topics(ArrayCondition<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PoolCondition {
    Hash(StringCondition),
    To(StringCondition),
    From(StringCondition),
    Value(NumericCondition<U256>),
    Nonce(NumericCondition<u64>),
    GasPrice(NumericCondition<u128>),
    GasLimit(NumericCondition<u64>),
    Timestamp(NumericCondition<u64>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BlockCondition {
    BaseFee(NumericCondition<u128>),
    Number(NumericCondition<u64>),
    Timestamp(NumericCondition<u64>),
    Size(NumericCondition<u64>),
    GasUsed(NumericCondition<u64>),
    GasLimit(NumericCondition<u64>),
    TransactionCount(NumericCondition<u64>),

    Hash(StringCondition),
    ParentHash(StringCondition),
    Miner(StringCondition),
    StateRoot(StringCondition),
    ReceiptsRoot(StringCondition),
    TransactionsRoot(StringCondition),
}

pub(crate) trait ConditionBuilder {
    type Condition;
    fn push_condition(&mut self, condition: Self::Condition);
}

pub(crate) trait Evaluable<T> {
    fn evaluate(&self, value: &T) -> bool;
}

// [`FilterNode`] represents a hierarchical structure of logical filters used to evaluate
// specific conditions. Each node in the tree represents a logical operator
// (e.g., AND, OR) or a specific condition (e.g., Value > 100). The structure allows
// for a flexible combination of filters to evaluate complex criteria.
//
// Example:
//                [OR]
//            /          \
//       [AND]             [AND]
//      /     \           /     \
// [Value > 100] [Gas < 50] [Contract] [Nonce > 5]
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct FilterNode {
    pub(crate) group: Option<(LogicalOp, Vec<FilterNode>)>,
    pub(crate) condition: Option<FilterCondition>,
}

impl<T> Evaluable<T> for NumericCondition<T>
where
    T: NumericType,
{
    fn evaluate(&self, value: &T) -> bool {
        match self {
            Self::GreaterThan(threshold) => value > threshold,
            Self::GreaterThanOrEqualTo(threshold) => value >= threshold,
            Self::LessThan(threshold) => value < threshold,
            Self::LessThanOrEqualTo(threshold) => value <= threshold,
            Self::EqualTo(threshold) => value == threshold,
            Self::NotEqualTo(threshold) => value != threshold,
            Self::Between(min, max) => value >= min && value <= max,
            Self::Outside(min, max) => value < min || value > max,
        }
    }
}

impl Evaluable<String> for StringCondition {
    fn evaluate(&self, value: &String) -> bool {
        match self {
            Self::EqualTo(expected) => value == expected,
            Self::Contains(substring) => value.contains(substring),
            Self::StartsWith(prefix) => value.starts_with(prefix),
            Self::EndsWith(suffix) => value.ends_with(suffix),
            Self::Matches(pattern) => {
                // TODO: Use regex pattern matching here
                value == pattern
            }
        }
    }
}

impl<T> Evaluable<Vec<T>> for ArrayCondition<T>
where
    T: PartialEq,
{
    fn evaluate(&self, value: &Vec<T>) -> bool {
        match self {
            Self::Contains(item) => value.contains(item),
            Self::NotIn(items) => !items.iter().any(|item| value.contains(item)),
            Self::Empty => value.is_empty(),
            Self::NotEmpty => !value.is_empty(),
        }
    }
}
