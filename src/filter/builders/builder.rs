use super::{
    block_header::BlockHeaderBuilder, builder_ops::FilterBuilderOps, event::EventBuilder,
    optimism::OptimismFilterBuilder, pool::PoolBuilder, transaction::TxBuilder,
};
use crate::filter::conditions::{FilterNode, LogicalOp};

/// FilterBuilder allows constructing complex filter conditions using a builder pattern.
pub struct FilterBuilder;

#[allow(dead_code)]
impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterBuilder {
    /// Creates a new empty [`FilterBuilder`].
    pub fn new() -> Self {
        Self {}
    }
    /// Adds transaction conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn transaction<F>(&mut self, f: F) -> FilterNode
    where
        F: FnOnce(&mut TxBuilder),
    {
        let mut builder = TxBuilder::new();
        f(&mut builder);

        FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize()
    }

    /// Adds event(logs) conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn event<F>(&mut self, f: F) -> FilterNode
    where
        F: FnOnce(&mut EventBuilder),
    {
        let mut builder = EventBuilder::new();
        f(&mut builder);

        FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize()
    }

    /// Adds pool conditions to the filter.
    ///
    /// Returns a [`PoolFilterBuilder`] for further configuration.
    pub fn pool<F>(&mut self, f: F) -> FilterNode
    where
        F: FnOnce(&mut PoolBuilder),
    {
        let mut builder = PoolBuilder::new();
        f(&mut builder);

        FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize()
    }

    /// Adds block header conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn block_header<F>(&mut self, f: F) -> FilterNode
    where
        F: FnOnce(&mut BlockHeaderBuilder),
    {
        let mut builder = BlockHeaderBuilder::new();
        f(&mut builder);

        FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize()
    }

    // ====== Layer 2 ========
    /// Adds Optimism L2-specific conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn optimism<F>(&mut self, f: F) -> FilterNode
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let mut builder = OptimismFilterBuilder::new();
        f(&mut builder);

        FilterNode {
            children: Some((LogicalOp::And, builder.nodes)),
            value: None,
        }
        .optimize()
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

        let node = builder.transaction(|tx| {
            tx.from().exact(ADDRESS);
            tx.value().gt(U256::from(BASE_VALUE));
        });

        let (op, nodes) = node.children.expect("Expected group in node");
        assert_eq!(op, LogicalOp::And);
        assert_eq!(nodes.len(), 2);

        match &nodes[0].value {
            Some(FilterCondition::Transaction(TransactionCondition::From(cond))) => {
                assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
            }
            other => assert!(false, "Expected Transaction From condition, got {:?}", other),
        }

        match &nodes[1].value {
            Some(FilterCondition::Transaction(TransactionCondition::Value(cond))) => {
                assert_eq!(*cond, NumericCondition::GreaterThan(U256::from(BASE_VALUE)));
            }
            other => assert!(false, "Expected Transaction Value condition, got {:?}", other),
        }
    }

    #[test]
    fn test_event_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder.event(|ev| {
            ev.contract().exact(ADDRESS);
            ev.block_number().gt(BASE_VALUE);
        });

        match node.children {
            Some((op, nodes)) => {
                assert_eq!(op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::Event(EventCondition::Contract(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
                    }
                    _ => panic!("Expected Event Contract condition"),
                }

                match &nodes[1].value {
                    Some(FilterCondition::Event(EventCondition::BlockNumber(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => panic!("Expected Event BlockNumber condition"),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_pool_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder.pool(|pool| {
            pool.from().exact(ADDRESS);
            pool.gas_limit().gt(BASE_VALUE);
        });

        match node.children {
            Some((op, nodes)) => {
                //             [AND]
                //            /    \
                // [from:ADDRESS]   [gas_limit:BASE_VALUE]
                assert_eq!(op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::Pool(PoolCondition::From(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(ADDRESS.to_string()));
                    }
                    _ => panic!("Expected Pool From condition"),
                }

                match &nodes[1].value {
                    Some(FilterCondition::Pool(PoolCondition::GasLimit(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => panic!("Expected Pool GasLimit condition"),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_block_header_filter() {
        let mut builder = FilterBuilder::new();

        let node = builder.block_header(|bh| {
            bh.number().gt(BASE_VALUE);
            bh.parent_hash().exact(BLOCK_HASH);
        });

        match node.children {
            Some((op, nodes)) => {
                assert_eq!(op, LogicalOp::And);
                assert_eq!(nodes.len(), 2);

                match &nodes[0].value {
                    Some(FilterCondition::BlockHeader(BlockHeaderCondition::Number(cond))) => {
                        assert_eq!(*cond, NumericCondition::GreaterThan(BASE_VALUE));
                    }
                    _ => panic!("Expected BlockHeader Number condition"),
                }

                match &nodes[1].value {
                    Some(FilterCondition::BlockHeader(BlockHeaderCondition::ParentHash(cond))) => {
                        assert_eq!(*cond, StringCondition::EqualTo(BLOCK_HASH.to_string()));
                    }
                    _ => panic!("Expected BlockHeader ParentHash condition"),
                }
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_optimism_filter() {
        let mut builder = FilterBuilder::new();
        let field_name = "l1BlockNumber";
        let field_value = "12345";

        let node = builder.optimism(|opt| {
            opt.field(field_name).exact(field_value);
            opt.field(field_name).exact(field_value);
            opt.field(field_name).exact(field_value);
        });

        match node.children {
            Some((op, nodes)) => {
                assert_eq!(op, LogicalOp::And);
                assert_eq!(nodes.len(), 3);
            }
            None => panic!("Expected group in node"),
        }
    }

    #[test]
    fn test_empty_filter() {
        let mut builder = FilterBuilder::new();
        let node = builder.transaction(|_| {});

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
