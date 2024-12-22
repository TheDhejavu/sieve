use super::conditions::{
    ArrayCondition, EventCondition, FilterCondition, NumericCondition, NumericType,
    ParameterCondition, StringCondition, TransactionCondition,
};
use alloy_dyn_abi::DynSolValue;
pub(crate) trait Evaluable<T> {
    fn evaluate(&self, value: &T) -> bool;
}

impl FilterCondition {
    pub(crate) fn needs_decoded_data(&self) -> bool {
        match self {
            FilterCondition::Transaction(transaction_condition) => {
                matches!(transaction_condition, TransactionCondition::CallData { .. })
            }
            FilterCondition::Event(event_condition) => {
                matches!(event_condition, EventCondition::EventData { .. })
            }
            FilterCondition::Pool(_) => false,
            FilterCondition::BlockHeader(_) => false,
        }
    }
}

impl<T> Evaluable<T> for NumericCondition<T>
where
    T: NumericType,
{
    fn evaluate(&self, value: &T) -> bool {
        match self {
            Self::GreaterThan(threshold) => value > threshold,
            Self::GreaterThanOrEqualTo(threshold) => value >= threshold,
            Self::LessThan(threshold) => value < threshold,
            Self::LessThanOrEqualTo(threshold) => value <= threshold,
            Self::EqualTo(threshold) => value == threshold,
            Self::NotEqualTo(threshold) => value != threshold,
            Self::Between(min, max) => value >= min && value <= max,
            Self::Outside(min, max) => value < min || value > max,
        }
    }
}

impl Evaluable<String> for StringCondition {
    fn evaluate(&self, value: &String) -> bool {
        match self {
            Self::EqualTo(expected) => value == expected,
            Self::Contains(substring) => value.contains(substring),
            Self::StartsWith(prefix) => value.starts_with(prefix),
            Self::EndsWith(suffix) => value.ends_with(suffix),
            Self::Matches(pattern) => {
                // TODO: Use regex pattern matching here
                value == pattern
            }
        }
    }
}

impl<T> Evaluable<Vec<T>> for ArrayCondition<T>
where
    T: PartialEq,
{
    fn evaluate(&self, value: &Vec<T>) -> bool {
        match self {
            Self::Contains(item) => value.contains(item),
            Self::NotIn(items) => !items.iter().any(|item| value.contains(item)),
            Self::Empty => value.is_empty(),
            Self::NotEmpty => !value.is_empty(),
        }
    }
}

impl Evaluable<DynSolValue> for ParameterCondition {
    fn evaluate(&self, value: &DynSolValue) -> bool {
        match self {
            Self::U256(condition) => {
                if let Some((value_uint, size)) = value.as_uint() {
                    // Check that we have a uint256
                    if size == 256 {
                        return condition.evaluate(&value_uint);
                    }
                }
                false
            }
            Self::U128(condition) => {
                if let Some((value_uint, size)) = value.as_uint() {
                    // Check that we have a uint128 or smaller
                    if size <= 128 {
                        // If the value fits in u128, we can evaluate it
                        let limbs = value_uint.as_limbs();
                        if limbs[2] == 0 && limbs[3] == 0 {
                            // Only lower 128 bits are used
                            let value_u128 = (limbs[1] as u128) << 64 | (limbs[0] as u128);
                            return condition.evaluate(&value_u128);
                        }
                    }
                }
                false
            }
            Self::String(condition) => {
                if let Some(val_str) = value.as_str() {
                    return condition.evaluate(&val_str.to_string());
                }
                false
            }
        }
    }
}
