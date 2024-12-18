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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use crate::filter::{
        conditions::{NumericCondition, StringCondition},
        ArrayOps, NumericOps, StringOps,
    };

    use super::*;

    #[test]
    fn test_basic_transaction_fields() {
        let mut builder = TxBuilder::new();

        // Test numeric conditions
        builder.value().eq(1000);
        builder.gas_price().gt(50);
        builder.gas().lt(100000);
        builder.nonce().between(5, 10);
        builder.max_fee_per_gas().lte(200);
        builder.max_priority_fee().gte(10);
        builder.chain_id().eq(1);
        builder.block_number().gt(1000000);
        builder.index().lt(100);
        builder.tx_type().eq(2);

        // Test string conditions
        builder.from().eq("0x123");
        builder.to().starts_with("0x");
        builder.block_hash().contains("abc");

        // Test array conditions
        builder.access_list().empty();

        // Verify conditions were added
        assert_eq!(builder.conditions.len(), 14);

        // Verify specific conditions
        match &builder.conditions[0] {
            TransactionCondition::Value(value) => {
                assert!(matches!(value, NumericCondition::EqualTo(1000)));
            }
            _ => panic!("Expected Value condition"),
        }
    }

    #[test]
    fn test_transfer_builder() {
        let mut builder = TxBuilder::new();

        // Test all transfer-specific fields
        let mut transfer = builder.transfer();
        transfer.amount().eq(100);
        transfer.method().eq("transfer");
        transfer.to().eq("0x.....");
        transfer.from().eq("0x.....");
        transfer.spender().eq("0x....");
        assert_eq!(builder.conditions.len(), 5);

        // Verify specific transfer conditions
        match &builder.conditions[0] {
            TransactionCondition::TransferAmount(amount) => {
                assert!(matches!(amount, NumericCondition::EqualTo(100)));
            }
            _ => panic!("Expected TransferAmount condition"),
        }

        match &builder.conditions[1] {
            TransactionCondition::TransferMethod(method) => {
                assert!(matches!(method, StringCondition::EqualTo(m) if m == "transfer"));
            }
            _ => panic!("Expected TransferMethod condition"),
        }
    }
}
