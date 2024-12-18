// Pool builder
use crate::filter::{
    conditions::{ConditionBuilder, PoolCondition},
    field::{FieldWrapper, NumericFieldType, PoolField, StringFieldType},
};

// ===== Pool Builder =====
pub(crate) struct PoolBuilder {
    pub(crate) conditions: Vec<PoolCondition>,
}

impl ConditionBuilder for PoolBuilder {
    type Condition = PoolCondition;

    fn push_condition(&mut self, condition: PoolCondition) {
        self.conditions.push(condition)
    }
}

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
