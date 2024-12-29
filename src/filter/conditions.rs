use alloy_primitives::{Selector, U256};
use std::{
    cmp::PartialOrd,
    hash::Hash,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use crate::config::Chain;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    Xor,
}

#[allow(dead_code)]
pub trait NumericType: Clone + PartialEq + PartialOrd {
    fn from_string(value: String) -> Self;
}

impl NumericType for u64 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap_or_default()
    }
}

impl NumericType for u8 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap_or_default()
    }
}

impl NumericType for u128 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap_or_default()
    }
}

impl NumericType for U256 {
    fn from_string(value: String) -> Self {
        value.parse().unwrap_or_default()
    }
}

// Generic numeric condition that works with any numeric type
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArrayCondition<T> {
    Contains(T),
    NotIn(Vec<T>),
    Empty,
    NotEmpty,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StringCondition {
    EqualTo(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Matches(String),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(dead_code)]
pub enum FilterCondition {
    Transaction(TransactionCondition),
    Event(EventCondition),
    Pool(PoolCondition),
    BlockHeader(BlockHeaderCondition),
    DynField(DynFieldCondition),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DynFieldCondition {
    pub(crate) path: String,
    pub(crate) condition: ValueCondition,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ValueCondition {
    U64(NumericCondition<u64>),
    U128(NumericCondition<u128>),
    U256(NumericCondition<U256>),
    String(StringCondition),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
    From(StringCondition),
    To(StringCondition),
    Hash(StringCondition),
    BlockHash(StringCondition),
    AccessList(ArrayCondition<String>),

    CallData {
        paths: Vec<DynFieldCondition>,
        method_selector: Selector,
        parameters: Vec<DynFieldCondition>,
    },

    DynField(DynFieldCondition),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

    EventData {
        signature: String,
        parameters: Vec<(String, ValueCondition)>,
    },

    // Array condition
    Topics(ArrayCondition<String>),

    DynField(DynFieldCondition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ContractCondition {
    Parameter(String, ValueCondition),
    Path(String, ValueCondition),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PoolCondition {
    Hash(StringCondition),
    To(StringCondition),
    From(StringCondition),
    Value(NumericCondition<U256>),
    Nonce(NumericCondition<u64>),
    GasPrice(NumericCondition<u128>),
    GasLimit(NumericCondition<u64>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BlockHeaderCondition {
    BaseFee(NumericCondition<u64>),
    Number(NumericCondition<u64>),
    Timestamp(NumericCondition<u64>),
    GasUsed(NumericCondition<u64>),
    GasLimit(NumericCondition<u64>),

    ParentHash(StringCondition),
    StateRoot(StringCondition),
    ReceiptsRoot(StringCondition),
    TransactionsRoot(StringCondition),

    DynField(DynFieldCondition),
}

pub(crate) trait NodeBuilder {
    type Condition;
    fn append_node(&mut self, condition: Self::Condition);
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub(crate) enum EventType {
    Transaction = 0,
    LogEvent = 1,
    BlockHeader = 2,
    Pool = 3,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Filter {
    pub(crate) id: u64,
    pub(crate) chain: Chain,
    pub(crate) event_type: Option<EventType>,
    pub(crate) filter_node: Arc<FilterNode>,
}

static FILTER_SEQ: AtomicU64 = AtomicU64::new(0);
impl Filter {
    pub(crate) fn new(
        chain: Chain,
        filter_node: Arc<FilterNode>,
        event_type: Option<EventType>,
    ) -> Self {
        let id = FILTER_SEQ.fetch_add(1, Ordering::SeqCst);
        Self {
            id,
            chain,
            event_type,
            filter_node,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn chain(&self) -> &Chain {
        &self.chain
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn event_type(&self) -> Option<EventType> {
        self.event_type.clone()
    }

    pub fn filter_node(&self) -> Arc<FilterNode> {
        self.filter_node.clone()
    }
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct FilterNode {
    pub children: Option<(LogicalOp, Vec<FilterNode>)>,
    pub value: Option<FilterCondition>,
}

impl FilterNode {
    pub(crate) fn optimize(self) -> FilterNode {
        // TODO:
        // 1. Re-order conditions based on priority (basic to complex)
        // 2. Re-order Logical operations to enable short-circuit
        // 3. Flatten nested logical operations if possible to reduce unnecessary recursive calls during evaluation.
        // 4. Benchmark!!!!!!!!

        if let Some((op, nodes)) = self.children {
            let filtered_nodes: Vec<_> = nodes
                .into_iter()
                .map(|node| node.optimize()) // Optimize owned node
                .filter(|node| node.value.is_some() || node.children.is_some())
                .collect();

            match filtered_nodes.len() {
                0 => FilterNode {
                    children: None,
                    value: None,
                },
                1 => filtered_nodes.into_iter().next().unwrap_or(FilterNode {
                    children: None,
                    value: None,
                }),
                _ => FilterNode {
                    children: Some((op, filtered_nodes)),
                    value: None,
                },
            }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::conditions::{StringCondition, TransactionCondition};

    #[test]
    fn test_optimize_single_condition() {
        let condition = FilterCondition::Transaction(TransactionCondition::From(
            StringCondition::EqualTo("0x123".to_string()),
        ));
        let node = FilterNode {
            children: None,
            value: Some(condition.clone()),
        };
        let optimized = node.optimize();
        assert_eq!(optimized.value, Some(condition));
    }

    #[test]
    fn test_optimize_single_node_children() {
        // ensure that children with single node should be flattened
        let condition = FilterCondition::Transaction(TransactionCondition::From(
            StringCondition::EqualTo("0x123".to_string()),
        ));
        let inner_node = FilterNode {
            children: None,
            value: Some(condition.clone()),
        };
        let node = FilterNode {
            children: Some((LogicalOp::And, vec![inner_node])),
            value: None,
        };

        let optimized = node.optimize();
        assert_eq!(optimized.value, Some(condition));
        assert!(optimized.children.is_none());
    }

    #[test]
    fn test_optimize_multi_node_children() {
        // children with multiple nodes should stay as children
        let condition1 = FilterCondition::Transaction(TransactionCondition::From(
            StringCondition::EqualTo("0x123".to_string()),
        ));
        let condition2 = FilterCondition::Transaction(TransactionCondition::To(
            StringCondition::EqualTo("0x456".to_string()),
        ));

        let node = FilterNode {
            children: Some((
                LogicalOp::And,
                vec![
                    FilterNode {
                        children: None,
                        value: Some(condition1),
                    },
                    FilterNode {
                        children: None,
                        value: Some(condition2),
                    },
                ],
            )),
            value: None,
        };

        let optimized = node.optimize();
        assert!(optimized.children.is_some());
        assert!(optimized.value.is_none());
    }

    #[test]
    fn test_optimize_empty_children() {
        // children with only empty nodes should become empty
        let node = FilterNode {
            children: Some((
                LogicalOp::And,
                vec![
                    FilterNode {
                        children: None,
                        value: None,
                    },
                    FilterNode {
                        children: None,
                        value: None,
                    },
                ],
            )),
            value: None,
        };

        let optimized = node.optimize();
        assert!(optimized.children.is_none() && optimized.value.is_none());
    }
}
