// ! Filter the ORM-Like definitions of filters for sieve

// Logical Operations
#[derive(Debug, Clone)]
pub enum LogicalOp{
    And,
    Or,
    Not,
}

// Conditions
#[derive(Debug, Clone)]
pub enum FilterCondition {
    TransactionCondition(TransactionCondition),
    EventCondition(EventCondition)
}

// First define the possible transaction fields
#[derive(Debug, Clone)]
pub enum TxField {
    Value,
    Nonce,
    Gas,
    GasPrice
}

// Condition types remain the same
#[derive(Debug, PartialEq, Clone)]
pub enum ValueCondition {
    GreaterThan(u64),
    LessThan(u64),
    EqualTo(u64),
    Between(u64, u64),
}

// Transaction conditions
#[derive(Debug, Clone)]
pub enum TransactionCondition {
    Value(ValueCondition),
    Gas(ValueCondition),
    GasPrice(ValueCondition),
    Nonce(ValueCondition),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextCondition {
    EqualTo(String),
    In(Vec<String>),
    StartsWith(String),
    EndsWith(String),
}

// Event conditions
#[derive(Debug, Clone)]
pub enum EventCondition {
    Contract(TextCondition),
}

enum TxCondition {
    Nonce,
    Value
}

/*
Filter tree represents tree structure of filters:
                [OR]
            /          \
    [AND]              [AND]
    /    \            /     \
[Value>100] [Gas<50] [Contract] [Nonce>5]
*/

#[derive(Clone)]
pub struct FilterNode {
    group: Option<(LogicalOp, Vec<FilterNode>)>,
    condition: Option<FilterCondition>,
}

// condition builder
pub struct ConditionBuilder<'a>  {
    field: TxField,
    parent: &'a mut TxBuilder,
}

impl<'a> ConditionBuilder<'a> {
    pub fn new( field: TxField, parent: &'a mut  TxBuilder) -> Self {
        Self{ field, parent }
    }

    pub fn gt(mut self, value: u64) -> &'a mut TxBuilder  {
        let condition = ValueCondition::GreaterThan(value);
        let tx_condition = match self.field {
            TxField::Value => TransactionCondition::Value(condition),
            TxField::Nonce => TransactionCondition::Nonce(condition),
            TxField::Gas => TransactionCondition::Gas(condition),
            TxField::GasPrice => TransactionCondition::GasPrice(condition),
        };
        self.parent.conditions.push(tx_condition);
        self.parent
    }

    pub fn lt(self, value: u64) {

    }
}


// transaction builder

pub struct TxBuilder {
    conditions: Vec<TransactionCondition>,
}

impl TxBuilder {
    pub fn new() -> Self {
        Self{conditions: Vec::new()}
    }

    pub fn value(&mut self) -> ConditionBuilder {
        ConditionBuilder::new(TxField::Value, self)
    }

    pub fn gas_price(&mut self) -> ConditionBuilder {
        ConditionBuilder::new(TxField::Value, self)
    }
}

// event builder 
pub struct EventBuilder {
    conditions: Vec<EventCondition>,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self{conditions: Vec::new()}
    }
}


pub struct FilterBuilder {
    filters: Vec<FilterNode>, 
}

// transaction builder
impl FilterBuilder {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }
    pub fn tx<F>(&mut self, f: F) -> BlockFilterBuilder
    where 
        F: FnOnce(&mut TxBuilder)
    {
        let filter = BlockFilterBuilder {
            filters: &mut self.filters
        };
        filter.tx(f) 
    }

    // Event builder
    pub fn event<F>(&mut self, f: F) -> BlockFilterBuilder
    where
        F: FnOnce(&mut EventBuilder),
    {
        let filter = BlockFilterBuilder {
            filters: &mut self.filters
        };
        filter.event(f) 
    }

    // Logical Operations.
    pub fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder
        where 
            F: FnOnce(&mut FilterBuilder) {

        let filter = LogicalFilterBuilder {
            filters: &mut self.filters
        };
        filter.any_of(f) 
    }

    pub fn and<F>(&mut self, f: F) -> LogicalFilterBuilder
        where 
            F: FnOnce(&mut FilterBuilder) {

        let filter = LogicalFilterBuilder {
            filters: &mut self.filters
        };
        filter.and(f) 
    }
}

// For block
pub struct BlockFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>,
}

impl<'a> BlockFilterBuilder<'a> {
    pub fn tx<F>(self, f: F) -> Self
    where 
        F: FnOnce(&mut TxBuilder)
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
        F: FnOnce(&mut EventBuilder)
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


pub struct LogicalFilterBuilder<'a> {
    filters: &'a mut Vec<FilterNode>, 
}

impl<'a> LogicalFilterBuilder<'a> {
    pub fn and<F>(self, f: F) -> Self
    where 
        F: FnOnce(&mut FilterBuilder)
    {
        let mut builder = FilterBuilder::new();
        f(&mut builder);

        let node = FilterNode {
            group: Some((LogicalOp::And, builder.filters)),
            condition: None
        };
        self.filters.push(node);
        self
    }
    pub fn any_of<F>(self, f: F) -> Self
    where 
        F: FnOnce(&mut FilterBuilder)
    {
        let mut builder = FilterBuilder::new();
        f(&mut builder);

        let node = FilterNode {
            group: Some((LogicalOp::Or, builder.filters)),
            condition: None
        };
        self.filters.push(node);
        self
    }
    pub fn build(&self) -> FilterNode {
        FilterNode {
            group: Some((LogicalOp::And, self.filters.clone())),
            condition: None,
        }
    }
}
