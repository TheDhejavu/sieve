use super::{
    block_header::BlockHeaderBuilder, builder_ops::FilterBuilderOps, event::EventBuilder,
    optimism::OptimismFilterBuilder, pool::PoolBuilder, transaction::TxBuilder,
};
use crate::filter::conditions::{FilterNode, LogicalOp};

/// FilterBuilder allows constructing complex filter conditions using a builder pattern.
pub struct FilterBuilder;

#[allow(dead_code)]
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
            group: Some((LogicalOp::And, builder.nodes)),
            condition: None,
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
            group: Some((LogicalOp::And, builder.nodes)),
            condition: None,
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
            group: Some((LogicalOp::And, builder.nodes)),
            condition: None,
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
            group: Some((LogicalOp::And, builder.nodes)),
            condition: None,
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
            group: Some((LogicalOp::And, builder.nodes)),
            condition: None,
        }
        .optimize()
    }
}
