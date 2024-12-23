use std::marker::PhantomData;

use crate::filter::{
    conditions::{FilterCondition, FilterNode, LogicalOp, NodeBuilder},
    field::{DynField, DynValueFieldType, FieldWrapper},
};

use super::{builder_ops::FilterBuilderOps, logical_builder::LogicalFilterBuilder};

// ===== OPTIMISIM FILTER BUILDER ============
pub(crate) struct OptimismFilterBuilder {
    pub(crate) filters: Vec<FilterNode>,
}

impl FilterBuilderOps for OptimismFilterBuilder {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    fn take_filters(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.filters)
    }
}

impl NodeBuilder for OptimismFilterBuilder {
    type Condition = FilterCondition;

    fn append_node(&mut self, condition: FilterCondition) {
        let node = FilterNode {
            group: None,
            condition: Some(condition),
        };
        self.filters.push(node);
    }
}

#[allow(dead_code)]
impl OptimismFilterBuilder {
    /// Combines conditions with AND logic, requiring all conditions to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn and<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
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
    pub fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.and(f)
    }

    /// Applies a NOT operation to the given conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn not<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.not(f)
    }

    /// Alias for `not`.
    /// Provides a more readable way to express "except when" conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn unless<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.not(f)
    }

    /// Combines conditions with OR logic, requiring at least one condition to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    pub fn or<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
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
    pub fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder<OptimismFilterBuilder>
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let filter: LogicalFilterBuilder<'_, OptimismFilterBuilder> = LogicalFilterBuilder {
            filters: &mut self.filters,
            _builder: PhantomData,
        };
        filter.or(f)
    }

    pub fn field(&mut self, path: &str) -> FieldWrapper<'_, DynValueFieldType<DynField>, Self> {
        FieldWrapper {
            field: DynValueFieldType(DynField(path.to_string())),
            parent: self,
        }
    }
}

// ===== Main Filter Builder =====
#[allow(dead_code)]
pub(crate) struct MainOptimismFilterBuilder<'a> {
    pub(crate) filters: &'a mut Vec<FilterNode>,
}

#[allow(dead_code)]
impl MainOptimismFilterBuilder<'_> {
    pub fn optimisim<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut OptimismFilterBuilder),
    {
        let mut builder = OptimismFilterBuilder {
            filters: Vec::new(),
        };
        f(&mut builder);

        self.filters.extend(builder.filters);
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
