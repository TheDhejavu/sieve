#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    TransactionCondition(TransactionCondition),
    EventCondition(EventCondition),
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumericCondition {
    GreaterThan(u64),
    LessThan(u64),
    EqualTo(u64),
    Between(u64, u64),
}

#[derive(Debug, Clone)]
pub enum TransactionCondition {
    Value(NumericCondition),
    Gas(NumericCondition),
    GasPrice(NumericCondition),
    Nonce(NumericCondition),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringCondition {
    EqualTo(String),
    In(Vec<String>),
    StartsWith(String),
    EndsWith(String),
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
