use std::marker::PhantomData;

use crate::filter::{
    conditions::{ConditionBuilder, FilterCondition, FilterNode, LogicalOp},
    field::{DynField, DynValueFieldType, FieldWrapper},
};

use super::{
    block_header::BlockHeaderBuilder, event::EventBuilder, pool::PoolBuilder,
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
    pub fn optimism(&mut self) -> DynamicFilterBuilder {
        DynamicFilterBuilder {
            conditions: Vec::new(),
        }
    }

    // ====== Logical Operations ========

    /// Combines conditions with AND logic, requiring all conditions to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn and<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.and(f)
    }

    /// Alias for `and`. Combines conditions requiring all to be true.
    /// Provides a more readable alternative when combining multiple conditions
    /// that must all be satisfied.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.and(f)
    }

    /// Applies a NOT operation to the given conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn not<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.not(f)
    }

    /// Alias for `not`.
    /// Provides a more readable way to express "except when" conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn unless<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.not(f)
    }

    /// Combines conditions with OR logic, requiring at least one condition to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn or<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.or(f)
    }

    /// Alias for `or`.
    /// Provides a more readable alternative for specifying that any one
    /// of multiple conditions should match.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.or(f)
    }
}

// / ===== Optimisim Filter Builder =====
#[allow(dead_code)]
pub struct DynamicFilterBuilder {
    conditions: Vec<FilterCondition>,
}

impl ConditionBuilder for DynamicFilterBuilder {
    type Condition = FilterCondition;

    fn push_condition(&mut self, condition: Self::Condition) {
        self.conditions.push(condition)
    }
}

#[allow(dead_code)]
impl DynamicFilterBuilder {
    pub fn field(&mut self, path: &str) -> FieldWrapper<'_, DynValueFieldType<DynField>, Self> {
        FieldWrapper {
            field: DynValueFieldType(DynField(path.to_string())),
            parent: self,
        }
    }
}

// ===== Main Filter Builder =====
#[allow(dead_code)]
pub struct MainFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

#[allow(dead_code)]
impl MainFilterBuilder<'_> {
    pub fn tx<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut TxBuilder),
    {
        let mut builder = TxBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::Transaction(condition)),
            };
            self.filters.push(node);
        }
        self
    }

    pub fn event<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EventBuilder),
    {
        let mut builder = EventBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::Event(condition)),
            };
            self.filters.push(node);
        }

        self
    }

    pub fn block_header<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut BlockHeaderBuilder),
    {
        let mut builder = BlockHeaderBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(condition)),
            };
            self.filters.push(node);
        }
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
pub struct PoolFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

impl PoolFilterBuilder<'_> {
    pub fn pool<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut PoolBuilder),
    {
        let mut builder = PoolBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::Pool(condition)),
            };
            self.filters.push(node);
        }
        self
    }
}

// ===== Logical Builder =====
#[allow(dead_code)]
pub struct LogicalFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

#[allow(dead_code)]
impl LogicalFilterBuilder<'_> {
    pub fn and<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::And, f)
    }

    pub fn not<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::Not, f)
    }
    pub fn unless<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::Not, f)
    }

    pub fn xor<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::Xor, f)
    }

    pub fn or<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::Or, f)
    }

    fn build_logical_operation<F>(self, op: LogicalOp, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let mut builder = FilterBuilder::new();
        f(&mut builder);

        // Only create OR group if we have multiple filters
        match builder.filters.len() {
            0 => self,
            1 => {
                self.filters.extend(builder.filters);
                self
            }
            _ => {
                let node = FilterNode {
                    group: Some((op, builder.filters)),
                    condition: None,
                };
                self.filters.push(node);
                self
            }
        }
    }
    pub fn build(&self) -> FilterNode {
        FilterNode {
            group: Some((LogicalOp::And, self.filters.clone())),
            condition: None,
        }
    }
}
