use super::{
    conditions::{
        ArrayCondition, ConditionBuilder, EventCondition, NumericCondition, StringCondition,
        TransactionCondition,
    },
    operations::{
        ArrayFieldToCondition, ArrayOps, NumericFieldToCondition, NumericOps,
        StringFieldToCondition, StringOps,
    },
};

pub struct NumericFieldType<T>(pub T);
pub struct StringFieldType<T>(pub T);
pub struct ArrayFieldType<T>(pub T);

// === Transaction Fields ======

// Transfer-specific fields
#[derive(Debug, Clone)]
pub enum TransferField {
    Method,  // The transfer method (transfer, transferFrom, approve)
    To,      // Recipient address
    From,    // Source address (for transferFrom)
    Amount,  // Transfer amount
    Spender, // Spender address (for approvals)
}

// Transaction specific field
#[derive(Debug, Clone)]
pub enum TxField {
    // Basic transaction fields
    Nonce,          // Transaction sequence number
    Value,          // Amount of ETH being transferred
    Gas,            // Gas limit
    GasPrice,       // Gas price (legacy transactions)
    MaxFeePerGas,   // Maximum total fee per gas (EIP-1559)
    MaxPriorityFee, // Maximum priority fee per gas (EIP-1559)

    // Address fields
    From, // Sender address
    To,   // Recipient address (None for contract creation)

    // Transaction type
    Type, // Transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)

    // Chain specific
    ChainId, // Chain identifier

    // Access list (EIP-2930)
    AccessList, // List of addresses and storage keys

    // Computed fields
    Hash,             // Transaction hash
    BlockNumber,      // Block number where tx was included
    BlockHash,        // Hash of the block where tx was included
    TransactionIndex, // Index of tx in the block

    // Contract related
    Transfer(TransferField),
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
            // Basic transaction fields
            TxField::Value => TransactionCondition::Value(value),
            TxField::Nonce => TransactionCondition::Nonce(value),
            TxField::Gas => TransactionCondition::Gas(value),
            TxField::GasPrice => TransactionCondition::GasPrice(value),
            TxField::MaxFeePerGas => TransactionCondition::MaxFeePerGas(value),
            TxField::MaxPriorityFee => TransactionCondition::MaxPriorityFee(value),

            // Chain specific
            TxField::ChainId => TransactionCondition::ChainId(value),

            // Block related
            TxField::BlockNumber => TransactionCondition::BlockNumber(value),
            TxField::TransactionIndex => TransactionCondition::TransactionIndex(value),

            // Transfer specific
            TxField::Transfer(TransferField::Amount) => TransactionCondition::TransferAmount(value),

            // Non-numeric fields should panic or return a sensible default
            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl StringFieldToCondition<TransactionCondition> for TxField {
    fn to_condition(&self, value: StringCondition) -> TransactionCondition {
        match self {
            // Basic address fields
            TxField::From => TransactionCondition::From(value),
            TxField::To => TransactionCondition::To(value),

            // Hash fields
            TxField::Hash => TransactionCondition::Hash(value),
            TxField::BlockHash => TransactionCondition::BlockHash(value),

            // Transfer specific fields
            TxField::Transfer(TransferField::Method) => TransactionCondition::TransferMethod(value),
            TxField::Transfer(TransferField::To) => TransactionCondition::TransferTo(value),
            TxField::Transfer(TransferField::From) => TransactionCondition::TransferFrom(value),
            TxField::Transfer(TransferField::Spender) => {
                TransactionCondition::TransferSpender(value)
            }

            // Non-string fields should panic
            _ => panic!("Field does not support string conditions"),
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

impl ArrayFieldToCondition<TransactionCondition, String> for TxField {
    fn to_condition(&self, value: ArrayCondition<String>) -> TransactionCondition {
        match self {
            TxField::AccessList => TransactionCondition::AccessList(value),
            _ => panic!("Field does not support string array conditions"),
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

    fn lte(self, value: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::LessThanOrEqualTo(value));
        self.parent.push_condition(condition);
    }

    fn gte(self, value: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::GreaterThanOrEqualTo(value));
        self.parent.push_condition(condition);
    }

    fn neq(self, value: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::NotEqualTo(value));
        self.parent.push_condition(condition);
    }

    fn outside(self, min: u64, max: u64) {
        let condition = self
            .field
            .0
            .to_condition(NumericCondition::Outside(min, max));
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
            .to_condition(StringCondition::Contains(substring.to_string()));
        self.parent.push_condition(condition);
    }

    fn matches(self, regex_pattern: &str) {
        let condition = self
            .field
            .0
            .to_condition(StringCondition::Matches(regex_pattern.to_string()));
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

impl<F, B, C, T> ArrayOps<T> for FieldWrapper<'_, ArrayFieldType<F>, B>
where
    F: ArrayFieldToCondition<C, T>,
    B: ConditionBuilder<Condition = C>,
    T: Clone,
{
    fn contains(self, value: T) {
        let condition = self.field.0.to_condition(ArrayCondition::Contains(value));
        self.parent.push_condition(condition);
    }

    fn not_in(self, values: Vec<T>) {
        let condition = self.field.0.to_condition(ArrayCondition::NotIn(values));
        self.parent.push_condition(condition);
    }

    fn is_empty(self) {
        let condition = self.field.0.to_condition(ArrayCondition::Empty);
        self.parent.push_condition(condition);
    }

    fn is_not_empty(self) {
        let condition = self.field.0.to_condition(ArrayCondition::NotEmpty);
        self.parent.push_condition(condition);
    }
}
