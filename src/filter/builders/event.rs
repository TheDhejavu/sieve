use crate::filter::{
    conditions::{ConditionBuilder, EventCondition},
    field::{ArrayFieldType, EventField, FieldWrapper, NumericFieldType, StringFieldType},
};

// ===== Event Builder ========
pub(crate) struct EventBuilder {
    pub(crate) conditions: Vec<EventCondition>,
}

impl ConditionBuilder for EventBuilder {
    type Condition = EventCondition;

    fn push_condition(&mut self, condition: EventCondition) {
        self.conditions.push(condition)
    }
}

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

    pub fn log_index(&mut self) -> FieldWrapper<'_, NumericFieldType<EventField>, Self> {
        FieldWrapper {
            field: NumericFieldType(EventField::LogIndex),
            parent: self,
        }
    }

    pub fn block_number(&mut self) -> FieldWrapper<'_, NumericFieldType<EventField>, Self> {
        FieldWrapper {
            field: NumericFieldType(EventField::BlockNumber),
            parent: self,
        }
    }

    pub fn tx_index(&mut self) -> FieldWrapper<'_, NumericFieldType<EventField>, Self> {
        FieldWrapper {
            field: NumericFieldType(EventField::TxIndex),
            parent: self,
        }
    }

    // Array type field
    pub fn topics(&mut self) -> FieldWrapper<'_, ArrayFieldType<EventField>, Self> {
        FieldWrapper {
            field: ArrayFieldType(EventField::Topics),
            parent: self,
        }
    }
}
