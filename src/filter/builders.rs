// transaction builder

use super::{
    conditions::{EventCondition, FilterCondition, FilterNode, LogicalOp, TransactionCondition},
    field::{EventField, FieldWrapper, NumericFieldType, StringFieldType, TxField},
};

pub trait ConditionBuilder {
    type Condition;

    fn push_condition(&mut self, condition: Self::Condition);
}

// ===== Transaction Builder =====
pub struct TxBuilder {
    conditions: Vec<TransactionCondition>,
}

impl ConditionBuilder for TxBuilder {
    type Condition = TransactionCondition;

    fn push_condition(&mut self, condition: TransactionCondition) {
        self.conditions.push(condition)
    }
}

impl TxBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn value(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::Value),
            parent: self,
        }
    }

    pub fn gas_price(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::GasPrice),
            parent: self,
        }
    }

    pub fn gas(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::Gas),
            parent: self,
        }
    }

    pub fn nonce(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::Nonce),
            parent: self,
        }
    }
}

// ===== Event Builder ========
pub struct EventBuilder {
    conditions: Vec<EventCondition>,
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

// ===== Filter Builder =====
pub struct FilterBuilder {
    filters: Vec<FilterNode>,
}

// transaction builder
impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
    pub fn tx<F>(&mut self, f: F) -> BlockFilterBuilder
    where
        F: FnOnce(&mut TxBuilder),
    {
        let filter = BlockFilterBuilder {
            filters: &mut self.filters,
        };
        filter.tx(f)
    }

    // Event builder
    pub fn event<F>(&mut self, f: F) -> BlockFilterBuilder
    where
        F: FnOnce(&mut EventBuilder),
    {
        let filter = BlockFilterBuilder {
            filters: &mut self.filters,
        };
        filter.event(f)
    }

    // Logical Operations.
    pub fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.or(f)
    }

    pub fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.and(f)
    }

    pub fn and<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.and(f)
    }

    pub fn or<F>(&mut self, f: F) -> LogicalFilterBuilder
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let filter = LogicalFilterBuilder {
            filters: &mut self.filters,
        };
        filter.or(f)
    }
}

// ===== Block Builder =====
pub struct BlockFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

impl<'a> BlockFilterBuilder<'a> {
    pub fn tx<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut TxBuilder),
    {
        let mut builder = TxBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::TransactionCondition(condition)),
            };
            self.filters.push(node);
        }
        self
    }

    pub fn event<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EventBuilder),
    {
        let mut builder = EventBuilder::new();
        f(&mut builder);

        for condition in builder.conditions {
            let node = FilterNode {
                group: None,
                condition: Some(FilterCondition::EventCondition(condition)),
            };
            self.filters.push(node);
        }
        self
    }

    pub fn build(&self) -> FilterNode {
        FilterNode {
            group: Some((LogicalOp::And, self.filters.clone())),
            condition: None,
        }
    }
}

// ===== Logical Builder =====
pub struct LogicalFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

impl<'a> LogicalFilterBuilder<'a> {
    pub fn and<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::And, f)
    }

    pub fn or<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        self.build_logical_operation(LogicalOp::Or, f)
    }

    fn build_logical_operation<F>(self, op: LogicalOp, f: F) -> Self
    where
        F: FnOnce(&mut FilterBuilder),
    {
        let mut builder = FilterBuilder::new();
        f(&mut builder);

        // Only create OR group if we have multiple filters
        match builder.filters.len() {
            0 => self,
            1 => {
                self.filters.extend(builder.filters);
                self
            }
            _ => {
                let node = FilterNode {
                    group: Some((op, builder.filters)),
                    condition: None,
                };
                self.filters.push(node);
                self
            }
        }
    }
    pub fn build(&self) -> FilterNode {
        FilterNode {
            group: Some((LogicalOp::And, self.filters.clone())),
            condition: None,
        }
    }
}
