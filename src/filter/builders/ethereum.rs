use std::marker::PhantomData;

use crate::filter::conditions::{FilterNode, LogicalOp};

use super::{
    block_header::BlockHeaderBuilder, builder_ops::FilterBuilderOps, event::EventBuilder,
    logical_builder::LogicalFilterBuilder, pool::PoolBuilder, transaction::TxBuilder,
};

// ===== ETHEREUM FILTER BUILDER ============
pub(crate) struct EthereumFilterBuilder {
    pub(crate) filters: Vec<FilterNode>,
}

impl FilterBuilderOps for EthereumFilterBuilder {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    fn take_filters(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.filters)
    }
}

impl EthereumFilterBuilder {
    /// Adds transaction conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn tx<F>(&mut self, f: F) -> MainFilterBuilder
    where
        F: FnOnce(&mut TxBuilder),
    {
        let filter = MainFilterBuilder {
            filters: &mut self.filters,
        };
        filter.tx(f)
    }

    /// Adds event(logs) conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn event<F>(&mut self, f: F) -> MainFilterBuilder
    where
        F: FnOnce(&mut EventBuilder),
    {
        let filter = MainFilterBuilder {
            filters: &mut self.filters,
        };
        filter.event(f)
    }

    /// Adds pool conditions to the filter.
    ///
    /// Returns a [`PoolFilterBuilder`] for further configuration.
    pub fn pool<F>(&mut self, f: F) -> PoolFilterBuilder
    where
        F: FnOnce(&mut PoolBuilder),
    {
        let filter = PoolFilterBuilder {
            filters: &mut self.filters,
        };
        filter.pool(f)
    }

    /// Adds block header conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn block_header<F>(&mut self, f: F) -> MainFilterBuilder
    where
        F: FnOnce(&mut BlockHeaderBuilder),
    {
        let filter = MainFilterBuilder {
            filters: &mut self.filters,
        };
        filter.block_header(f)
    }

    /// Combines conditions with AND logic, requiring all conditions to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn and<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.and(f)
    }

    /// Alias for `and`. Combines conditions requiring all to be true.
    /// Provides a more readable alternative when combining multiple conditions
    /// that must all be satisfied.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.and(f)
    }

    /// Applies a NOT operation to the given conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn not<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.not(f)
    }

    /// Alias for `not`.
    /// Provides a more readable way to express "except when" conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn unless<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.not(f)
    }

    /// Combines conditions with OR logic, requiring at least one condition to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn or<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.or(f)
    }

    /// Alias for `or`.
    /// Provides a more readable alternative for specifying that any one
    /// of multiple conditions should match.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder<EthereumFilterBuilder>
    where
        F: FnOnce(&mut EthereumFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, EthereumFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.or(f)
    }
}

// ===== Main Filter Builder =====
#[allow(dead_code)]
pub struct MainFilterBuilder<'a> {
    pub(crate) filters: &'a mut Vec<FilterNode>,
}

#[allow(dead_code)]
impl MainFilterBuilder<'_> {
    pub fn tx<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut TxBuilder),
    {
        let mut builder = TxBuilder::new();
        f(&mut builder);

        self.filters.extend(builder.nodes);
        self
    }

    pub fn event<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EventBuilder),
    {
        let mut builder = EventBuilder::new();
        f(&mut builder);

        self.filters.extend(builder.nodes);
        self
    }

    pub fn block_header<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut BlockHeaderBuilder),
    {
        let mut builder = BlockHeaderBuilder::new();
        f(&mut builder);

        self.filters.extend(builder.nodes);
        self
    }

    pub fn build(&self) -> FilterNode {
        FilterNode {
            group: Some((LogicalOp::And, self.filters.clone())),
            condition: None,
        }
        .optimize()
    }
}

// ===== Pool Filter Builder =====
#[allow(dead_code)]
pub(crate) struct PoolFilterBuilder<'a> {
    pub(crate) filters: &'a mut Vec<FilterNode>,
}

impl PoolFilterBuilder<'_> {
    pub fn pool<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut PoolBuilder),
    {
        let mut builder = PoolBuilder::new();
        f(&mut builder);
        self.filters.extend(builder.nodes);
        self
    }
}
