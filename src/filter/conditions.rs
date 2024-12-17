#[derive(Debug, Clone)]
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
pub enum FilterCondition {
    TransactionCondition(TransactionCondition),
    EventCondition(EventCondition),
}

#[derive(Debug, Clone)]
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
pub enum EventCondition {
    Contract(StringCondition),
    Topic(StringCondition),
}

/*
Filter tree represents tree structure of filters:
                [OR]
            /          \
    [AND]              [AND]
    /    \            /     \
[Value>100] [Gas<50] [Contract] [Nonce>5]
*/
#[derive(Clone)]
pub struct FilterNode {
    pub group: Option<(LogicalOp, Vec<FilterNode>)>,
    pub condition: Option<FilterCondition>,
}

pub trait ConditionBuilder {
    type Condition;

    fn push_condition(&mut self, condition: Self::Condition);
}
