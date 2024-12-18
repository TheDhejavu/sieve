// Block builder
use crate::filter::{
    conditions::{BlockCondition, ConditionBuilder},
    field::{BlockField, FieldWrapper, NumericFieldType, StringFieldType},
};

// ===== Block Builder =====
pub(crate) struct BlockBuilder {
    pub(crate) conditions: Vec<BlockCondition>,
}

impl ConditionBuilder for BlockBuilder {
    type Condition = BlockCondition;

    fn push_condition(&mut self, condition: BlockCondition) {
        self.conditions.push(condition)
    }
}
#[allow(dead_code)]
impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }
    pub fn number(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::Number),
            parent: self,
        }
    }

    pub fn timestamp(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::Timestamp),
            parent: self,
        }
    }

    pub fn size(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::Size),
            parent: self,
        }
    }

    pub fn gas_used(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::GasUsed),
            parent: self,
        }
    }

    pub fn gas_limit(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::GasLimit),
            parent: self,
        }
    }

    pub fn base_fee(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::BaseFee),
            parent: self,
        }
    }

    pub fn transaction_count(&mut self) -> FieldWrapper<'_, NumericFieldType<BlockField>, Self> {
        FieldWrapper {
            field: NumericFieldType(BlockField::TransactionCount),
            parent: self,
        }
    }

    pub fn hash(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::Hash),
            parent: self,
        }
    }

    pub fn parent_hash(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::ParentHash),
            parent: self,
        }
    }

    pub fn miner(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::Miner),
            parent: self,
        }
    }

    pub fn state_root(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::StateRoot),
            parent: self,
        }
    }

    pub fn receipts_root(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::ReceiptsRoot),
            parent: self,
        }
    }

    pub fn transactions_root(&mut self) -> FieldWrapper<'_, StringFieldType<BlockField>, Self> {
        FieldWrapper {
            field: StringFieldType(BlockField::TransactionsRoot),
            parent: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{
        conditions::{NumericCondition, StringCondition},
        NumericOps, StringOps,
    };

    const NUMBER: u64 = 1;
    const SIZE: u64 = 1000;
    const GAS_USED: u64 = 2000;
    const GAS_LIMIT: u64 = 2000;
    const BASE_FEE: u64 = 3000;
    const TRANSACTION_COUNT: u64 = 10;
    const TIMESTAMP: u64 = 5000;

    const HASH: &str = "0x123";
    const PREFIX: &str = "0x";
    const CONTENT: &str = "abc";
    const SUFFIX: &str = "def";

    #[test]
    fn test_numeric_field_operations() {
        let mut builder = BlockBuilder::new();

        // Test various numeric operations
        builder.number().eq(NUMBER);
        builder.size().gt(SIZE);
        builder.gas_used().gte(GAS_USED);
        builder.gas_limit().lt(GAS_LIMIT);
        builder.timestamp().lte(TIMESTAMP);
        builder.base_fee().eq(BASE_FEE);
        builder.transaction_count().eq(TRANSACTION_COUNT);

        let conditions = vec![
            BlockCondition::Number(NumericCondition::EqualTo(NUMBER)),
            BlockCondition::Size(NumericCondition::GreaterThan(SIZE)),
            BlockCondition::GasUsed(NumericCondition::GreaterThanOrEqualTo(GAS_USED)),
            BlockCondition::GasLimit(NumericCondition::LessThan(GAS_LIMIT)),
            BlockCondition::Timestamp(NumericCondition::LessThanOrEqualTo(TIMESTAMP)),
            BlockCondition::BaseFee(NumericCondition::EqualTo(BASE_FEE)),
            BlockCondition::TransactionCount(NumericCondition::EqualTo(TRANSACTION_COUNT)),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn test_string_field_operations() {
        let mut builder = BlockBuilder::new();

        // Test all string operations
        builder.hash().eq(HASH);
        builder.parent_hash().starts_with(PREFIX);
        builder.miner().contains(CONTENT);
        builder.state_root().ends_with(SUFFIX);
        builder.receipts_root().eq(HASH);
        builder.transactions_root().starts_with(PREFIX);

        let conditions = vec![
            BlockCondition::Hash(StringCondition::EqualTo(HASH.to_string())),
            BlockCondition::ParentHash(StringCondition::StartsWith(PREFIX.to_string())),
            BlockCondition::Miner(StringCondition::Contains(CONTENT.to_string())),
            BlockCondition::StateRoot(StringCondition::EndsWith(SUFFIX.to_string())),
            BlockCondition::ReceiptsRoot(StringCondition::EqualTo(HASH.to_string())),
            BlockCondition::TransactionsRoot(StringCondition::StartsWith(PREFIX.to_string())),
        ];

        assert_eq!(builder.conditions, conditions);
    }

    #[test]
    fn builder_new() {
        let builder = BlockBuilder::new();
        assert!(builder.conditions.is_empty());
    }
}
