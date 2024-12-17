use super::conditions::{NumericCondition, StringCondition};

pub trait NumericOps {
    fn gt(self, value: u64);
    fn lt(self, value: u64);
    fn eq(self, value: u64);
    fn between(self, min: u64, max: u64);
}

pub trait StringOps {
    fn starts_with(self, prefix: &str);
    fn ends_with(self, suffix: &str);
    fn contains(self, substring: &str);
    fn eq(self, value: &str);
}

pub trait NumericFieldToCondition<C> {
    fn to_condition(&self, value: NumericCondition) -> C;
}

pub trait StringFieldToCondition<C> {
    fn to_condition(&self, value: StringCondition) -> C;
}
