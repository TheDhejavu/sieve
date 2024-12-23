use super::{
    builders::{builder_ops::FilterBuilderOps, logical_builder::LogicalFilterBuilder},
    conditions::NumericType,
};
/// Operations available for numeric fields that allow comparison and range checks.
///
#[allow(dead_code)]
pub trait NumericOps<T: NumericType> {
    /// Creates a "greater than" condition with the specified value.
    fn gt(self, value: T);

    /// Creates a "greater than or equal to" condition with the specified value.
    fn gte(self, value: T);

    /// Creates a "less than" condition with the specified value.
    fn lt(self, value: T);

    /// Creates a "less than or equal to" condition with the specified value.
    fn lte(self, value: T);

    /// Creates an "equal to" condition with the specified value.
    fn eq(self, value: T);

    /// Creates a "not equal to" condition with the specified value.
    fn neq(self, value: T);

    /// Creates a "between" condition with the specified minimum and maximum values (inclusive).
    fn between(self, min: T, max: T);

    /// Creates an "outside" condition checking if value is outside the specified range (exclusive).
    fn outside(self, min: T, max: T);
}

/// Operations available for string fields that allow various string matching operations.
///
#[allow(dead_code)]
pub trait StringOps {
    /// Creates a condition that matches strings starting with the specified prefix.
    fn starts_with(self, prefix: &str);

    /// Creates a condition that matches strings ending with the specified suffix.
    fn ends_with(self, suffix: &str);

    /// Creates a condition that matches strings containing the specified substring.
    fn contains(self, substring: &str);

    /// Creates a condition that matches strings containing the specified substring using regex pattern.
    fn matches(self, substring: &str);

    /// Creates a condition that matches strings exactly equal to the specified value.
    fn exact(self, value: &str);
}

/// Operations available for array fields that allow various array matching operations.
#[allow(dead_code)]
pub trait ArrayOps<T> {
    /// Creates a condition that checks if array is empty
    fn empty(self);

    /// Creates a condition that checks if array is not empty
    fn not_empty(self);

    /// Creates a condition that checks if array contains the specified value
    fn contains(self, value: T);

    /// Creates a condition that checks if array is not in the values.
    fn not_in(self, values: Vec<T>);
}

#[allow(dead_code)]
pub trait LogicalOps<B: FilterBuilderOps> {
    /// Combines conditions with AND logic, requiring all conditions to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn and<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);

    /// Alias for `and`. Combines conditions requiring all to be true.
    /// Provides a more readable alternative when combining multiple conditions
    /// that must all be satisfied.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);

    /// Applies a NOT operation to the given conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn not<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);

    /// Alias for `not`.
    /// Provides a more readable way to express "except when" conditions.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn unless<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);

    /// Combines conditions with OR logic, requiring at least one condition to be true.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn or<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);

    /// Alias for `or`.
    /// Provides a more readable alternative for specifying that any one
    /// of multiple conditions should match.
    ///
    /// Returns a [`LogicalFilterBuilder`] for further configuration.
    fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder<B>
    where
        F: FnOnce(&mut B);
}
