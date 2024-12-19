use crate::filter::{
    conditions::{ConditionBuilder, EventCondition},
    field::{ArrayFieldType, EventField, FieldWrapper, StringFieldType, U64FieldType},
};

// ===== Event Builder ========
pub(crate) struct EventBuilder {
    pub(crate) conditions: Vec<EventCondition>,
}

#[allow(dead_code)]
impl EventBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn contract(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::Contract),
            parent: self,
        }
    }

    pub fn block_hash(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::BlockHash),
            parent: self,
        }
    }

    pub fn tx_hash(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::TxHash),
            parent: self,
        }
    }

    pub fn log_index(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::LogIndex),
            parent: self,
        }
    }

    pub fn block_number(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::BlockNumber),
            parent: self,
        }
    }

    pub fn tx_index(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::TxIndex),
            parent: self,
        }
    }

    pub fn topics(&mut self) -> FieldWrapper<'_, ArrayFieldType<EventField>, Self> {
        FieldWrapper {
            field: ArrayFieldType(EventField::Topics),
            parent: self,
        }
    }

    pub fn param(&mut self, name: &str) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::DataParameter(name.to_string())),
            parent: self,
        }
    }
}

impl ConditionBuilder for EventBuilder {
    type Condition = EventCondition;

    fn push_condition(&mut self, condition: EventCondition) {
        self.conditions.push(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{
        conditions::{ArrayCondition, NumericCondition, StringCondition},
        ArrayOps, NumericOps, StringOps,
    };

    const BASE_VALUE: u64 = 100;
    const VALUES: [u64; 6] = [
        BASE_VALUE,     // eq
        BASE_VALUE * 2, // gt
        BASE_VALUE * 3, // gte
        BASE_VALUE * 4, // lt
        BASE_VALUE * 5, // lte
        BASE_VALUE * 6, // between start
    ];

    const ADDRESS: &str = "0x123";
    const CONTENT: &str = "0x456";
    const PREFIX: &str = "0x";
    const SUFFIX: &str = "789";

    const TOPIC: &str = "topic1";

    #[test]
    fn test_event_numeric_field_operations() {
        let mut builder = EventBuilder::new();

        // Test all numeric operations for block_number
        builder.block_number().eq(VALUES[0]);
        builder.block_number().gt(VALUES[1]);
        builder.block_number().gte(VALUES[2]);
        builder.block_number().lt(VALUES[3]);
        builder.block_number().lte(VALUES[4]);
        builder
            .block_number()
            .between(VALUES[5], VALUES[5] + BASE_VALUE);

        let expected_conditions = vec![
            EventCondition::BlockNumber(NumericCondition::EqualTo(VALUES[0])),
            EventCondition::BlockNumber(NumericCondition::GreaterThan(VALUES[1])),
            EventCondition::BlockNumber(NumericCondition::GreaterThanOrEqualTo(VALUES[2])),
            EventCondition::BlockNumber(NumericCondition::LessThan(VALUES[3])),
            EventCondition::BlockNumber(NumericCondition::LessThanOrEqualTo(VALUES[4])),
            EventCondition::BlockNumber(NumericCondition::Between(
                VALUES[5],
                VALUES[5] + BASE_VALUE,
            )),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_event_string_field_operations() {
        let mut builder = EventBuilder::new();

        // Test all string operations for contract address
        builder.contract().eq(ADDRESS);
        builder.contract().contains(CONTENT);
        builder.contract().starts_with(PREFIX);
        builder.contract().ends_with(SUFFIX);

        let expected_conditions = vec![
            EventCondition::Contract(StringCondition::EqualTo(ADDRESS.to_string())),
            EventCondition::Contract(StringCondition::Contains(CONTENT.to_string())),
            EventCondition::Contract(StringCondition::StartsWith(PREFIX.to_string())),
            EventCondition::Contract(StringCondition::EndsWith(SUFFIX.to_string())),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_event_array_field_operations() {
        let mut builder = EventBuilder::new();

        // Test array operations for topics
        builder.topics().contains(TOPIC.to_string());
        builder.topics().not_empty();

        let expected_conditions = vec![
            EventCondition::Topics(ArrayCondition::Contains(TOPIC.to_string())),
            EventCondition::Topics(ArrayCondition::NotEmpty),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }
}