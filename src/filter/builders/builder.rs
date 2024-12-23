use std::marker::PhantomData;

use crate::filter::conditions::FilterNode;

use super::{
    block_header::BlockHeaderBuilder,
    ethereum::{EthereumFilterBuilder, MainFilterBuilder, PoolFilterBuilder},
    event::EventBuilder,
    logical_builder::LogicalFilterBuilder,
    optimism::{MainOptimismFilterBuilder, OptimismFilterBuilder},
    pool::PoolBuilder,
    transaction::TxBuilder,
};

/// FilterBuilder allows constructing complex filter conditions using a builder pattern.
pub struct FilterBuilder {
    filters: Vec<FilterNode>,
}

#[allow(dead_code)]
impl FilterBuilder {
    /// Creates a new empty [`FilterBuilder`].
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

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

    // ====== Layer 2 ========
    /// Adds Optimism L2-specific conditions to the filter.
    ///
    /// Returns a [`MainFilterBuilder`] for further configuration.
    pub fn optimism<F>(&mut self, f: F) -> MainOptimismFilterBuilder
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter = MainOptimismFilterBuilder {
            filters: &mut self.filters,
        };

        filter.optimisim(f)
    }

    // ====== Logical Operations for L1 ========

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
