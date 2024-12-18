use alloy_primitives::U256;

use super::conditions::{ArrayCondition, NumericCondition, NumericType, StringCondition};

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
    fn eq(self, value: &str);
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

/// Converts a field to a numeric condition.
///
/// This trait is implemented by field types that can be converted into numeric conditions.
pub trait U8FieldToCondition<C> {
    fn to_u8_condition(&self, value: NumericCondition<u8>) -> C;
}

pub trait U64FieldToCondition<C> {
    fn to_u64_condition(&self, value: NumericCondition<u64>) -> C;
}

pub trait U256FieldToCondition<C> {
    fn to_u256_condition(&self, value: NumericCondition<U256>) -> C;
}

pub trait U128FieldToCondition<C> {
    fn to_u128_condition(&self, value: NumericCondition<u128>) -> C;
}
/// Converts a field to a string condition.
///
/// This trait is implemented by field types that can be converted into string conditions.
pub trait StringFieldToCondition<C> {
    fn to_condition(&self, value: StringCondition) -> C;
}

/// Converts a field to an array condition.
pub trait ArrayFieldToCondition<C, T> {
    fn to_condition(&self, value: ArrayCondition<T>) -> C;
}
