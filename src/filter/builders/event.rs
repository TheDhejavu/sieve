use crate::filter::{
    conditions::{ConditionBuilder, EventCondition},
    field::{EventField, FieldWrapper, StringFieldType},
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
}
