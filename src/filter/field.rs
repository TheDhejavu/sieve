use super::{
    builders::ConditionBuilder,
    conditions::{EventCondition, NumericCondition, StringCondition, TransactionCondition},
    operations::{NumericFieldToCondition, NumericOps, StringFieldToCondition, StringOps},
};

pub struct NumericFieldType<T>(pub T);
pub struct StringFieldType<T>(pub T);
pub struct ArrayFieldType<T>(pub T);

// === Transaction Fields ======
#[derive(Debug, Clone)]
pub enum TxField {
    Value,
    Nonce,
    Gas,
    GasPrice,
}

// === Event Fields ======
#[derive(Debug, Clone)]
pub enum EventField {
    Contract,
    Topic,
}

pub struct FieldWrapper<'a, T, P> {
    pub field: T,
    pub parent: &'a mut P,
}

impl NumericFieldToCondition<TransactionCondition> for TxField {
    fn to_condition(&self, value: NumericCondition) -> TransactionCondition {
        match self {
            TxField::Value => TransactionCondition::Value(value),
            TxField::Nonce => TransactionCondition::Nonce(value),
            TxField::Gas => TransactionCondition::Gas(value),
            TxField::GasPrice => TransactionCondition::GasPrice(value),
        }
    }
}

impl StringFieldToCondition<EventCondition> for EventField {
    fn to_condition(&self, value: StringCondition) -> EventCondition {
        match self {
            EventField::Contract => EventCondition::Contract(value),
            EventField::Topic => EventCondition::Topic(value),
        }
    }
}

impl<F, B, C> NumericOps for FieldWrapper<'_, NumericFieldType<F>, B>
where
    F: NumericFieldToCondition<C>,
    B: ConditionBuilder<Condition = C>,
{
    fn gt(self, value: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::GreaterThan(value));
        self.parent.push_condition(condition);
    }

    fn lt(self, value: u64) {
        let condition = self.field.0.to_condition(NumericCondition::LessThan(value));
        self.parent.push_condition(condition);
    }

    fn eq(self, value: u64) {
        let condition = self.field.0.to_condition(NumericCondition::EqualTo(value));
        self.parent.push_condition(condition);
    }

    fn between(self, min: u64, max: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::Between(min, max));
        self.parent.push_condition(condition);
    }
}

impl<F, B, C> StringOps for FieldWrapper<'_, StringFieldType<F>, B>
where
    F: StringFieldToCondition<C>,
    B: ConditionBuilder<Condition = C>,
{
    fn starts_with(self, prefix: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::StartsWith(prefix.to_string()));
        self.parent.push_condition(condition);
    }

    fn ends_with(self, suffix: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::EndsWith(suffix.to_string()));
        self.parent.push_condition(condition);
    }

    fn contains(self, substring: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::StartsWith(substring.to_string()));
        self.parent.push_condition(condition);
    }

    fn eq(self, value: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::EqualTo(value.to_string()));
        self.parent.push_condition(condition);
    }
}
