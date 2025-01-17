use std::sync::Arc;

use super::{
    block_header::BlockHeaderBuilder, event::EventBuilder, pool::PoolBuilder,
    transaction::TxBuilder,
};
use crate::{
    config::Chain,
    filter::conditions::{EventType, Filter, FilterNode, LogicalOp},
};

/// FilterBuilder allows constructing complex filter conditions using a builder pattern.
pub struct FilterBuilder {
    chain: Chain,
}

#[allow(dead_code)]
impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterBuilder {
    /// Creates a new empty [`FilterBuilder`].
    pub fn new() -> Self {
        Self {
            // default chain is `Ethereum`
            chain: Chain::Ethereum,
        }
    }
    /// Adds transaction conditions to the filter.
    ///
    /// Returns a [`Filter`] after  configuration is completed.
    pub fn transaction<F>(&mut self, f: F) -> Filter
    where
        F: FnOnce(&mut TxBuilder),
    {
        let mut builder = TxBuilder::new();
        f(&mut builder);

        let filter_node = FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize();

        Filter::new(
            self.chain.clone(),
            Arc::new(filter_node),
            Some(EventType::Transaction),
        )
    }

    /// Adds event(logs) conditions to the filter.
    ///
    /// Returns a [`Filter`] after  configuration is completed.
    pub fn event<F>(&mut self, f: F) -> Filter
    where
        F: FnOnce(&mut EventBuilder),
    {
        let mut builder = EventBuilder::new();
        f(&mut builder);

        let filter_node = FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize();

        Filter::new(
            self.chain.clone(),
            Arc::new(filter_node),
            Some(EventType::Transaction),
        )
    }

    /// Adds pool conditions to the filter.
    ///
    /// Returns a [`Filter`] after  configuration is completed.
    pub fn pool<F>(&mut self, f: F) -> Filter
    where
        F: FnOnce(&mut PoolBuilder),
    {
        let mut builder = PoolBuilder::new();
        f(&mut builder);

        let filter_node = FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize();

        Filter::new(
            self.chain.clone(),
            Arc::new(filter_node),
            Some(EventType::Pool),
        )
    }

    /// Adds block header conditions to the filter.
    ///
    /// Returns a [`Filter`] after  configuration is completed.
    pub fn block_header<F>(&mut self, f: F) -> Filter
    where
        F: FnOnce(&mut BlockHeaderBuilder),
    {
        let mut builder = BlockHeaderBuilder::new();
        f(&mut builder);

        let filter_node = FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize();

        Filter::new(
            self.chain.clone(),
            Arc::new(filter_node),
            Some(EventType::BlockHeader),
        )
    }

    /// ====== Layer 1 & Layer 2 context ========
    /// Sets the chain context for this filter
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = chain;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{
        conditions::{
            BlockHeaderCondition, EventCondition, FilterCondition, NumericCondition, PoolCondition,
            StringCondition, TransactionCondition,
        },
        NumericOps, StringOps,
    };
    use alloy_primitives::U256;

    const ADDRESS: &str = "0x123456789";
    const BLOCK_HASH: &str = "0xabcdef123456";
    const BASE_VALUE: u64 = 100;

    #[test]
    fn test_transaction_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder
            .transaction(|tx| {
                tx.from().exact(ADDRESS);
                tx.value().gt(U256::from(BASE_VALUE));
            })
            .filter_node();

        match &node.children {
            Some((op, nodes)) => {
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::Transaction(TransactionCondition::From(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
                    }
                    _ => panic!("Expected Transaction From condition"),
                }

                match &nodes[1].value {
                    Some(FilterCondition::Transaction(TransactionCondition::Value(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(U256::from(BASE_VALUE)));
                    }
                    _ => panic!("Expected Transaction Value condition"),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_event_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder
            .event(|ev| {
                ev.contract().exact(ADDRESS);
                ev.block_number().gt(BASE_VALUE);
            })
            .filter_node();

        match &node.children {
            Some((op, nodes)) => {
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::Event(EventCondition::Contract(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
                    }
                    _ => unreachable!(),
                }

                match &nodes[1].value {
                    Some(FilterCondition::Event(EventCondition::BlockNumber(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => unreachable!(),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_pool_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder
            .pool(|pool| {
                pool.from().exact(ADDRESS);
                pool.gas_limit().gt(BASE_VALUE);
            })
            .filter_node();

        match &node.children {
            Some((op, nodes)) => {
                // ---------------------------------------|
                //             [AND]
                //            /    \
                // [from:ADDRESS]   [gas_limit:BASE_VALUE]
                //----------------------------------------|
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::Pool(PoolCondition::From(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
                    }
                    _ => unreachable!(),
                }

                match &nodes[1].value {
                    Some(FilterCondition::Pool(PoolCondition::GasLimit(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => unreachable!(),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_block_header_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder
            .block_header(|bh| {
                bh.number().gt(BASE_VALUE);
                bh.parent_hash().exact(BLOCK_HASH);
            })
            .filter_node();

        match &node.children {
            Some((op, nodes)) => {
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::BlockHeader(BlockHeaderCondition::Number(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => unreachable!(),
                }

                match &nodes[1].value {
                    Some(FilterCondition::BlockHeader(BlockHeaderCondition::ParentHash(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(BLOCK_HASH.to_string()));
                    }
                    _ => unreachable!(),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_optimism_filter() {
        let builder = FilterBuilder::new();
        let field_name = "l1BlockNumber";
        let field_value = "12345";

        let filter = builder.chain(Chain::Optimism).transaction(|tx| {
            tx.field(field_name).exact(field_value);
            tx.field(field_name).exact(field_value);
            tx.field(field_name).exact(field_value);
        });

        assert_eq!(filter.chain, Chain::Optimism);
        match &filter.filter_node().children {
            Some((op, nodes)) => {
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(nodes.len(), 3);
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_empty_filter() {
        let mut builder = FilterBuilder::new();
        let node = builder.transaction(|_| {}).filter_node();

        assert!(
            node.children.is_none(),
            "Empty filter should not have group"
        );
        assert!(
            node.value.is_none(),
            "Empty filter should not have condition"
        );
    }
}
