use crate::utils::json::resolve_path;

use super::conditions::{
    ArrayCondition, DynFieldCondition, EventCondition, FilterCondition, NumericCondition,
    NumericType, StringCondition, TransactionCondition, ValueCondition,
};
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::U256;
use serde_json::Value;

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
            FilterCondition::DynField(_) => false,
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

// Evaluation for decoded data gotten from [`alloy_dyn_abi::DynSolValue`]
impl Evaluable<DynSolValue> for ValueCondition {
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
            Self::U64(condition) => {
                if let Some((value_uint, size)) = value.as_uint() {
                    // Check that we have a uint64 or smaller
                    if size <= 64 {
                        // If the value fits in u64, we can evaluate it
                        let limbs = value_uint.as_limbs();
                        if limbs[1] == 0 && limbs[2] == 0 && limbs[3] == 0 {
                            // Only lower 64 bits are used
                            let value_u64 = limbs[0];
                            return condition.evaluate(&value_u64);
                        }
                    }
                }
                false
            }
        }
    }
}

// Evaluation for blockchain data gotten from [`serde_json::Value`]
impl Evaluable<Value> for DynFieldCondition {
    fn evaluate(&self, value: &Value) -> bool {
        let path = &self.path;
        let condition = &self.condition;
        if let Some(field_value) = resolve_path(path, value) {
            match (field_value, condition) {
                (Value::String(s), ValueCondition::String(string_condition)) => {
                    string_condition.evaluate(&s.to_string())
                }
                (Value::Number(n), ValueCondition::U64(num_condition)) => {
                    if let Some(num) = n.as_u64() {
                        num_condition.evaluate(&num)
                    } else {
                        false
                    }
                }
                (Value::Number(n), ValueCondition::U128(num_condition)) => {
                    if let Some(num) = n.as_u128() {
                        num_condition.evaluate(&num)
                    } else {
                        false
                    }
                }
                (Value::String(hex_str), ValueCondition::U256(num_condition)) => {
                    if let Some(stripped) = hex_str.strip_prefix("0x") {
                        if let Ok(num) = U256::from_str_radix(stripped, 16) {
                            return num_condition.evaluate(&num);
                        }
                    }
                    false
                }
                (Value::String(hex_str), ValueCondition::U128(num_condition)) => {
                    if let Some(stripped) = hex_str.strip_prefix("0x") {
                        if let Ok(num) = u128::from_str_radix(stripped, 16) {
                            return num_condition.evaluate(&num);
                        }
                    }
                    false
                }
                (Value::String(hex_str), ValueCondition::U64(num_condition)) => {
                    if let Some(stripped) = hex_str.strip_prefix("0x") {
                        if let Ok(num) = u64::from_str_radix(stripped, 16) {
                            return num_condition.evaluate(&num);
                        }
                    }
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_u258_evaluation_condition() {
        let tx = json!({
            "transaction": {
                "value": "0x2386f26fc10000",  // 0.01 ETH
                "gasPrice": "0x4a817c800"      // 20 Gwei
            }
        });

        // Test if transaction value is > 0.005 ETH (half of 0.01)
        let condition = DynFieldCondition {
            path: "transaction.value".to_string(),
            condition: ValueCondition::U256(NumericCondition::GreaterThan(
                U256::from_str_radix("11C37937E08000", 16).unwrap(),
            )),
        };
        assert!(condition.evaluate(&tx));

        // Test if gas price is exactly 20 Gwei
        let condition = DynFieldCondition {
            path: "transaction.gasPrice".to_string(),
            condition: ValueCondition::U256(NumericCondition::EqualTo(
                U256::from_str_radix("4a817c800", 16).unwrap(),
            )),
        };
        assert!(condition.evaluate(&tx));
    }

    #[test]
    fn test_u64_hex_evaluation() {
        let tx = json!({
            "header": {
                "blockNumber": "0xf4240", // 1,000,000 in decimal
                "id": "0x1",              // Block 1
                "number": "0x1234"        // Block 4660
              }
        });

        let condition = DynFieldCondition {
            path: "header.blockNumber".to_string(),
            condition: ValueCondition::U64(NumericCondition::EqualTo(1_000_000)),
        };
        assert!(condition.evaluate(&tx));

        let condition = DynFieldCondition {
            path: "header.id".to_string(),
            condition: ValueCondition::U64(NumericCondition::EqualTo(1)),
        };
        assert!(condition.evaluate(&tx));

        let condition = DynFieldCondition {
            path: "header.number".to_string(),
            condition: ValueCondition::U64(NumericCondition::EqualTo(4660)),
        };
        assert!(condition.evaluate(&tx));
    }
}
