use super::{
    conditions::{
        ArrayCondition, BlockCondition, ConditionBuilder, EventCondition, NumericCondition,
        PoolCondition, StringCondition, TransactionCondition,
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
#[allow(dead_code)]
pub enum TransferField {
    Method,  // The transfer method (transfer, transferFrom, approve)
    To,      // Recipient address
    From,    // Source address (for transferFrom)
    Amount,  // Transfer amount
    Spender, // Spender address (for approvals)
}

// Transaction specific field
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    ChainId,

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

// Event-specific fields (logs)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EventField {
    Contract,    // Contract address that generated the event
    Topics,      // Array of 0 to 4 32-byte topics (first is event signature)
    LogIndex,    // Integer of the log index position in the block
    BlockNumber, // Block number where this log was
    BlockHash,   // Hash of the block where this log was
    TxHash,      // Hash of the transaction that created this log
    TxIndex,     // Integer of the transaction's index position
}

// ===  Block-specific fields ===
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BlockField {
    // Core block info
    Number,     // Block number/height
    Hash,       // Block hash
    ParentHash, // Previous block hash
    Timestamp,  // Block timestamp

    // Block metadata
    Size,     // Block size in bytes
    GasUsed,  // Gas used in this block
    GasLimit, // Block gas limit
    BaseFee,  // Base fee per gas (EIP-1559)

    // Mining info
    Miner,

    // Block content info
    TransactionCount, // Number of transactions
    StateRoot,        // State root hash
    ReceiptsRoot,     // Receipts root hash
    TransactionsRoot, // Transactions root hash
}

// ==== Pool-specific fields (mempool) ====
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PoolField {
    // Transaction identification
    Hash,  // Transaction hash
    From,  // Sender address
    To,    // Recipient address
    Nonce, // Transaction nonce

    // Gas & Value
    Value,          // ETH value
    GasPrice,       // Legacy gas price
    MaxFeePerGas,   // EIP-1559 max fee
    MaxPriorityFee, // EIP-1559 priority fee

    // Temporal
    FirstSeen, // Timestamp first seen
    LastSeen,  // Timestamp last seen

    // Mempool specific
    ReplacedBy,       // Hash of replacing transaction
    ReplacementCount, // Number of times replaced
    PropagationTime,  // Time to propagate to network
}

pub struct FieldWrapper<'a, T, P> {
    pub field: T,
    pub parent: &'a mut P,
}

// === TxField =====
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

impl ArrayFieldToCondition<TransactionCondition, String> for TxField {
    fn to_condition(&self, value: ArrayCondition<String>) -> TransactionCondition {
        match self {
            TxField::AccessList => TransactionCondition::AccessList(value),
            _ => panic!("Field does not support string array conditions"),
        }
    }
}

// === EventField =====
impl NumericFieldToCondition<EventCondition> for EventField {
    fn to_condition(&self, value: NumericCondition) -> EventCondition {
        match self {
            EventField::LogIndex => EventCondition::LogIndex(value),
            EventField::BlockNumber => EventCondition::BlockNumber(value),
            EventField::TxIndex => EventCondition::TxIndex(value),
            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl StringFieldToCondition<EventCondition> for EventField {
    fn to_condition(&self, value: StringCondition) -> EventCondition {
        match self {
            EventField::Contract => EventCondition::Contract(value),
            EventField::BlockHash => EventCondition::BlockHash(value),
            EventField::TxHash => EventCondition::TxHash(value),
            _ => panic!("Field does not support string conditions"),
        }
    }
}

impl ArrayFieldToCondition<EventCondition, String> for EventField {
    fn to_condition(&self, value: ArrayCondition<String>) -> EventCondition {
        match self {
            EventField::Topics => EventCondition::Topics(value),
            _ => panic!("Field does not support array conditions"),
        }
    }
}

// === PoolField =====
impl NumericFieldToCondition<PoolCondition> for PoolField {
    fn to_condition(&self, value: NumericCondition) -> PoolCondition {
        match self {
            // Value & Gas fields
            PoolField::Value => PoolCondition::Value(value),
            PoolField::GasPrice => PoolCondition::GasPrice(value),
            PoolField::MaxFeePerGas => PoolCondition::MaxFeePerGas(value),
            PoolField::MaxPriorityFee => PoolCondition::MaxPriorityFee(value),

            // Counter fields
            PoolField::Nonce => PoolCondition::Nonce(value),
            PoolField::ReplacementCount => PoolCondition::ReplacementCount(value),
            PoolField::PropagationTime => PoolCondition::PropagationTime(value),

            // Time fields
            PoolField::FirstSeen => PoolCondition::FirstSeen(value),
            PoolField::LastSeen => PoolCondition::LastSeen(value),

            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl StringFieldToCondition<PoolCondition> for PoolField {
    fn to_condition(&self, value: StringCondition) -> PoolCondition {
        match self {
            // Transaction identification
            PoolField::Hash => PoolCondition::Hash(value),
            PoolField::From => PoolCondition::From(value),
            PoolField::To => PoolCondition::To(value),
            PoolField::ReplacedBy => PoolCondition::ReplacedBy(value),

            _ => panic!("Field does not support string conditions"),
        }
    }
}

// === BlockField =====
impl NumericFieldToCondition<BlockCondition> for BlockField {
    fn to_condition(&self, value: NumericCondition) -> BlockCondition {
        match self {
            // Core block info
            BlockField::Number => BlockCondition::Number(value),
            BlockField::Timestamp => BlockCondition::Timestamp(value),

            // Block metadata
            BlockField::Size => BlockCondition::Size(value),
            BlockField::GasUsed => BlockCondition::GasUsed(value),
            BlockField::GasLimit => BlockCondition::GasLimit(value),
            BlockField::BaseFee => BlockCondition::BaseFee(value),
            BlockField::TransactionCount => BlockCondition::TransactionCount(value),

            _ => panic!("Field does not support numeric conditions"),
        }
    }
}

impl StringFieldToCondition<BlockCondition> for BlockField {
    fn to_condition(&self, value: StringCondition) -> BlockCondition {
        match self {
            // Hash fields
            BlockField::Hash => BlockCondition::Hash(value),
            BlockField::ParentHash => BlockCondition::ParentHash(value),

            // Mining info
            BlockField::Miner => BlockCondition::Miner(value),

            // Root hashes
            BlockField::StateRoot => BlockCondition::StateRoot(value),
            BlockField::ReceiptsRoot => BlockCondition::ReceiptsRoot(value),
            BlockField::TransactionsRoot => BlockCondition::TransactionsRoot(value),

            _ => panic!("Field does not support string conditions"),
        }
    }
}

// === NumericOps =====
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

// === StringOps =====
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
