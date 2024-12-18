// Pool builder
use crate::filter::{
    conditions::{ConditionBuilder, PoolCondition},
    field::{FieldWrapper, NumericFieldType, PoolField, StringFieldType},
};

// ===== Pool Builder =====
pub(crate) struct PoolBuilder {
    pub(crate) conditions: Vec<PoolCondition>,
}

#[allow(dead_code)]
impl PoolBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    // Transaction identification - Numeric fields
    pub fn nonce(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::Nonce),
            parent: self,
        }
    }

    // Gas & Value fields - Numeric
    pub fn value(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::Value),
            parent: self,
        }
    }

    pub fn gas_price(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::GasPrice),
            parent: self,
        }
    }

    pub fn max_fee_per_gas(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::MaxFeePerGas),
            parent: self,
        }
    }

    pub fn max_priority_fee(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::MaxPriorityFee),
            parent: self,
        }
    }

    pub fn first_seen(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::FirstSeen),
            parent: self,
        }
    }

    pub fn last_seen(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::LastSeen),
            parent: self,
        }
    }

    pub fn propagation_time(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::PropagationTime),
            parent: self,
        }
    }

    pub fn replacement_count(&mut self) -> FieldWrapper<'_, NumericFieldType<PoolField>, Self> {
        FieldWrapper {
            field: NumericFieldType(PoolField::ReplacementCount),
            parent: self,
        }
    }

    pub fn from(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::From),
            parent: self,
        }
    }

    pub fn to(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::To),
            parent: self,
        }
    }

    pub fn hash(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::Hash),
            parent: self,
        }
    }

    pub fn replaced_by(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::ReplacedBy),
            parent: self,
        }
    }
}

impl ConditionBuilder for PoolBuilder {
    type Condition = PoolCondition;

    fn push_condition(&mut self, condition: PoolCondition) {
        self.conditions.push(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{
        conditions::{NumericCondition, PoolCondition, StringCondition},
        NumericOps, StringOps,
    };

    const NONCE: u64 = 1;
    const VALUE: u64 = 1000;
    const GAS_PRICE: u64 = 2000;
    const MAX_FEE: u64 = 3000;
    const PRIORITY_FEE: u64 = 4000;
    const TIMESTAMP: u64 = 5000;
    const PROPAGATION: u64 = 100;
    const REPLACEMENTS: u64 = 3;

    const RANGE_START: u64 = 100;
    const RANGE_END: u64 = 200;

    const ADDRESS: &str = "0xABCD1234";
    const HASH: &str = "0x9876fedc";
    const PREFIX: &str = "0x";
    const SUFFIX: &str = "fedc";

    #[test]
    fn test_numeric_field_operations() {
        let mut builder = PoolBuilder::new();

        // Test various numeric operations
        builder.nonce().eq(NONCE);
        builder.value().gt(VALUE);
        builder.gas_price().gte(GAS_PRICE);
        builder.max_fee_per_gas().lt(MAX_FEE);
        builder.max_priority_fee().lte(PRIORITY_FEE);
        builder.propagation_time().between(RANGE_START, RANGE_END);

        let conditions = vec![
            PoolCondition::Nonce(NumericCondition::EqualTo(NONCE)),
            PoolCondition::Value(NumericCondition::GreaterThan(VALUE)),
            PoolCondition::GasPrice(NumericCondition::GreaterThanOrEqualTo(GAS_PRICE)),
            PoolCondition::MaxFeePerGas(NumericCondition::LessThan(MAX_FEE)),
            PoolCondition::MaxPriorityFee(NumericCondition::LessThanOrEqualTo(PRIORITY_FEE)),
            PoolCondition::PropagationTime(NumericCondition::Between(RANGE_START, RANGE_END)),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn test_string_field_operations() {
        let mut builder = PoolBuilder::new();

        // Test various string operations
        builder.from().eq(ADDRESS);
        builder.to().contains(HASH);
        builder.hash().starts_with(PREFIX);
        builder.replaced_by().ends_with(SUFFIX);

        let conditions = vec![
            PoolCondition::From(StringCondition::EqualTo(ADDRESS.to_string())),
            PoolCondition::To(StringCondition::Contains(HASH.to_string())),
            PoolCondition::Hash(StringCondition::StartsWith(PREFIX.to_string())),
            PoolCondition::ReplacedBy(StringCondition::EndsWith(SUFFIX.to_string())),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn test_timestamp_fields() {
        let mut builder = PoolBuilder::new();

        builder.first_seen().eq(TIMESTAMP);
        builder.last_seen().gt(TIMESTAMP);
        builder.propagation_time().lt(PROPAGATION);
        builder.replacement_count().eq(REPLACEMENTS);

        let conditions = vec![
            PoolCondition::FirstSeen(NumericCondition::EqualTo(TIMESTAMP)),
            PoolCondition::LastSeen(NumericCondition::GreaterThan(TIMESTAMP)),
            PoolCondition::PropagationTime(NumericCondition::LessThan(PROPAGATION)),
            PoolCondition::ReplacementCount(NumericCondition::EqualTo(REPLACEMENTS)),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn test_pool_fields() {
        let mut builder = PoolBuilder::new();

        // Mix different types of conditions
        builder.nonce().eq(NONCE);
        builder.from().contains(ADDRESS);
        builder.gas_price().gt(GAS_PRICE);
        builder.hash().starts_with(PREFIX);
        builder.propagation_time().between(RANGE_START, RANGE_END);

        let conditions = vec![
            PoolCondition::Nonce(NumericCondition::EqualTo(NONCE)),
            PoolCondition::From(StringCondition::Contains(ADDRESS.to_string())),
            PoolCondition::GasPrice(NumericCondition::GreaterThan(GAS_PRICE)),
            PoolCondition::Hash(StringCondition::StartsWith(PREFIX.to_string())),
            PoolCondition::PropagationTime(NumericCondition::Between(RANGE_START, RANGE_END)),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn builder_new() {
        let builder = PoolBuilder::new();
        assert!(builder.conditions.is_empty());
    }
}
