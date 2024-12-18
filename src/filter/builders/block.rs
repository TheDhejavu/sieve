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

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    // Core block info - Numeric
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

    // Block metadata - Numeric
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

    // Hash fields - String
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
