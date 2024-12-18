use super::conditions::{ArrayCondition, NumericCondition, StringCondition};

/// Operations available for numeric fields that allow comparison and range checks.
///
#[allow(dead_code)]
pub trait NumericOps {
    /// Creates a "greater than" condition with the specified value.
    fn gt(self, value: u64);

    /// Creates a "greater than or equal to" condition with the specified value.
    fn gte(self, value: u64);

    /// Creates a "less than" condition with the specified value.
    fn lt(self, value: u64);

    /// Creates a "less than or equal to" condition with the specified value.
    fn lte(self, value: u64);

    /// Creates an "equal to" condition with the specified value.
    fn eq(self, value: u64);

    /// Creates a "not equal to" condition with the specified value.
    fn neq(self, value: u64);

    /// Creates a "between" condition with the specified minimum and maximum values (inclusive).
    fn between(self, min: u64, max: u64);

    /// Creates an "outside" condition checking if value is outside the specified range (exclusive).
    fn outside(self, min: u64, max: u64);
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
    fn eq(self, value: &str);
}

/// Operations available for array fields that allow various array matching operations.
#[allow(dead_code)]
pub trait ArrayOps<T> {
    /// Creates a condition that checks if array contains the specified value
    fn contains(self, value: T);

    /// Creates a condition that checks if array is not in the values.
    fn not_in(self, values: Vec<T>);

    /// Creates a condition that checks if array is empty
    fn is_empty(self);

    /// Creates a condition that checks if array is not empty
    fn is_not_empty(self);
}

/// Converts a field to a numeric condition of type `C`.
///
/// This trait is implemented by field types that can be converted into numeric conditions.
pub trait NumericFieldToCondition<C> {
    fn to_condition(&self, value: NumericCondition) -> C;
}

/// Converts a field to a string condition of type `C`.
///
/// This trait is implemented by field types that can be converted into string conditions.
pub trait StringFieldToCondition<C> {
    fn to_condition(&self, value: StringCondition) -> C;
}

/// Converts a field to an array condition of type `C`.
pub trait ArrayFieldToCondition<C, T> {
    fn to_condition(&self, value: ArrayCondition<T>) -> C;
}
