#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    NoneOf,
    Xor,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumericCondition {
    GreaterThan(u64),
    GreaterThanOrEqualTo(u64),

    LessThan(u64),
    LessThanOrEqualTo(u64),

    EqualTo(u64),
    NotEqualTo(u64),

    Between(u64, u64),
    Outside(u64, u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayCondition<T> {
    Contains(T),
    NotIn(Vec<T>),
    Empty,
    NotEmpty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringCondition {
    EqualTo(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Matches(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FilterCondition {
    Transaction(TransactionCondition),
    Event(EventCondition),
    Pool(PoolCondition),
    Block(BlockCondition),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TransactionCondition {
    // Amount fields - Numeric
    Value(NumericCondition),
    Gas(NumericCondition),
    GasPrice(NumericCondition),
    MaxFeePerGas(NumericCondition),
    MaxPriorityFee(NumericCondition),

    // Counter fields - Numeric
    Nonce(NumericCondition),
    Type(NumericCondition),
    ChainId(NumericCondition),
    BlockNumber(NumericCondition),
    TransactionIndex(NumericCondition),

    // Address fields - String
    From(StringCondition),
    To(StringCondition),

    // Hash fields - String
    Hash(StringCondition),
    BlockHash(StringCondition),

    // Access list - Array
    AccessList(ArrayCondition<String>),

    // Transfer conditions - Decoded Input
    TransferMethod(StringCondition),
    TransferTo(StringCondition),
    TransferFrom(StringCondition),
    TransferAmount(NumericCondition),
    TransferSpender(StringCondition),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EventCondition {
    // String conditions
    Contract(StringCondition),
    BlockHash(StringCondition),
    TxHash(StringCondition),

    // Numeric conditions
    LogIndex(NumericCondition),
    BlockNumber(NumericCondition),
    TxIndex(NumericCondition),

    // Array condition
    Topics(ArrayCondition<String>),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PoolCondition {
    // Transaction identification - String
    Hash(StringCondition),
    From(StringCondition),
    To(StringCondition),
    ReplacedBy(StringCondition),

    // Gas & Value - Numeric
    Value(NumericCondition),
    GasPrice(NumericCondition),
    MaxFeePerGas(NumericCondition),
    MaxPriorityFee(NumericCondition),
    Gas(NumericCondition),

    // Counter fields - Numeric
    Nonce(NumericCondition),
    ReplacementCount(NumericCondition),
    PropagationTime(NumericCondition),

    // Temporal fields - Numeric (timestamps)
    FirstSeen(NumericCondition),
    LastSeen(NumericCondition),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BlockCondition {
    // Core block info - Numeric
    Number(NumericCondition),
    Timestamp(NumericCondition),

    // Block metadata - Numeric
    Size(NumericCondition),
    GasUsed(NumericCondition),
    GasLimit(NumericCondition),
    BaseFee(NumericCondition),
    TransactionCount(NumericCondition),

    // Hash fields - String
    Hash(StringCondition),
    ParentHash(StringCondition),

    // Mining info - String
    Miner(StringCondition),

    // Root hashes - String
    StateRoot(StringCondition),
    ReceiptsRoot(StringCondition),
    TransactionsRoot(StringCondition),
}

// FilterNode represents a hierarchical structure of logical filters used to evaluate
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
pub struct FilterNode {
    pub group: Option<(LogicalOp, Vec<FilterNode>)>,
    pub condition: Option<FilterCondition>,
}

pub trait ConditionBuilder {
    type Condition;

    fn push_condition(&mut self, condition: Self::Condition);
}
