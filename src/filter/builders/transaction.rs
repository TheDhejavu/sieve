// transaction builder
use crate::filter::{
    conditions::{ConditionBuilder, TransactionCondition},
    field::{
        ArrayFieldType, FieldWrapper, NumericFieldType, StringFieldType, TransferField, TxField,
    },
};

// ===== Transaction Builder =====
pub(crate) struct TxBuilder {
    pub(crate) conditions: Vec<TransactionCondition>,
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

    pub fn max_fee_per_gas(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::MaxFeePerGas),
            parent: self,
        }
    }

    pub fn max_priority_fee(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::MaxPriorityFee),
            parent: self,
        }
    }

    pub fn chain_id(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::ChainId),
            parent: self,
        }
    }

    pub fn block_number(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::BlockNumber),
            parent: self,
        }
    }

    pub fn index(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::TransactionIndex),
            parent: self,
        }
    }

    pub fn from(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, Self> {
        FieldWrapper {
            field: StringFieldType(TxField::From),
            parent: self,
        }
    }

    pub fn to(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, Self> {
        FieldWrapper {
            field: StringFieldType(TxField::To),
            parent: self,
        }
    }

    pub fn hash(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, Self> {
        FieldWrapper {
            field: StringFieldType(TxField::Hash),
            parent: self,
        }
    }

    pub fn block_hash(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, Self> {
        FieldWrapper {
            field: StringFieldType(TxField::BlockHash),
            parent: self,
        }
    }

    pub fn tx_type(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, Self> {
        FieldWrapper {
            field: NumericFieldType(TxField::Type),
            parent: self,
        }
    }

    pub fn access_list(&mut self) -> FieldWrapper<'_, ArrayFieldType<TxField>, Self> {
        FieldWrapper {
            field: ArrayFieldType(TxField::AccessList),
            parent: self,
        }
    }

    pub fn transfer(&mut self) -> TxTransferBuilder<Self> {
        TxTransferBuilder::new(self)
    }
}

pub struct TxTransferBuilder<'a, B> {
    parent: &'a mut B,
}

impl ConditionBuilder for TxTransferBuilder<'_, TxBuilder> {
    type Condition = TransactionCondition;

    fn push_condition(&mut self, condition: TransactionCondition) {
        self.parent.push_condition(condition)
    }
}

impl<'a> TxTransferBuilder<'a, TxBuilder> {
    pub fn new(parent: &'a mut TxBuilder) -> Self {
        Self { parent }
    }

    pub fn amount(&mut self) -> FieldWrapper<'_, NumericFieldType<TxField>, TxBuilder> {
        FieldWrapper {
            field: NumericFieldType(TxField::Transfer(TransferField::Amount)),
            parent: self.parent,
        }
    }

    pub fn method(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(TxField::Transfer(TransferField::Method)),
            parent: self.parent,
        }
    }

    pub fn to(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(TxField::Transfer(TransferField::To)),
            parent: self.parent,
        }
    }

    pub fn from(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(TxField::Transfer(TransferField::From)),
            parent: self.parent,
        }
    }

    pub fn spender(&mut self) -> FieldWrapper<'_, StringFieldType<TxField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(TxField::Transfer(TransferField::Spender)),
            parent: self.parent,
        }
    }
}
