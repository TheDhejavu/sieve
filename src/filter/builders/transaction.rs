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

#[allow(dead_code)]
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
    use super::*;
    use crate::filter::{
        conditions::{ArrayCondition, NumericCondition, StringCondition, TransactionCondition},
        ArrayOps, NumericOps, StringOps,
    };

    const BASE_VALUE: u64 = 100;
    const VALUES: [u64; 6] = [
        BASE_VALUE,     // eq
        BASE_VALUE * 2, // gt
        BASE_VALUE * 3, // gte
        BASE_VALUE * 4, // lt
        BASE_VALUE * 5, // lte
        BASE_VALUE * 6, // between start
    ];

    const ADDRESS: &str = "0x123";
    const PREFIX: &str = "0x";
    const CONTENT: &str = "abc";
    const METHOD: &str = "transfer";

    #[test]
    fn test_tx_numeric_field_operations() {
        let mut builder = TxBuilder::new();

        builder.value().eq(VALUES[0]);
        builder.gas_price().gt(VALUES[1]);
        builder.gas().lt(VALUES[2]);
        builder.nonce().between(VALUES[3], VALUES[4]);
        builder.max_fee_per_gas().lte(VALUES[5]);
        builder.max_priority_fee().gte(VALUES[0]);

        let expected_conditions = vec![
            TransactionCondition::Value(NumericCondition::EqualTo(VALUES[0])),
            TransactionCondition::GasPrice(NumericCondition::GreaterThan(VALUES[1])),
            TransactionCondition::Gas(NumericCondition::LessThan(VALUES[2])),
            TransactionCondition::Nonce(NumericCondition::Between(VALUES[3], VALUES[4])),
            TransactionCondition::MaxFeePerGas(NumericCondition::LessThanOrEqualTo(VALUES[5])),
            TransactionCondition::MaxPriorityFee(NumericCondition::GreaterThanOrEqualTo(VALUES[0])),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_string_field_operations() {
        let mut builder = TxBuilder::new();

        builder.from().eq(ADDRESS);
        builder.to().starts_with(PREFIX);
        builder.block_hash().contains(CONTENT);
        builder.hash().ends_with(CONTENT);

        let expected_conditions = vec![
            TransactionCondition::From(StringCondition::EqualTo(ADDRESS.to_string())),
            TransactionCondition::To(StringCondition::StartsWith(PREFIX.to_string())),
            TransactionCondition::BlockHash(StringCondition::Contains(CONTENT.to_string())),
            TransactionCondition::Hash(StringCondition::EndsWith(CONTENT.to_string())),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_array_field_operations() {
        let mut builder = TxBuilder::new();

        builder.access_list().empty();
        builder.access_list().not_empty();

        let expected_conditions = vec![
            TransactionCondition::AccessList(ArrayCondition::Empty),
            TransactionCondition::AccessList(ArrayCondition::NotEmpty),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_transfer_operations() {
        let mut builder = TxBuilder::new();
        let mut transfer = builder.transfer();

        transfer.amount().eq(VALUES[0]);
        transfer.method().eq(METHOD);
        transfer.to().eq(ADDRESS);
        transfer.from().starts_with(PREFIX);
        transfer.spender().contains(CONTENT);

        let expected_conditions = vec![
            TransactionCondition::TransferAmount(NumericCondition::EqualTo(VALUES[0])),
            TransactionCondition::TransferMethod(StringCondition::EqualTo(METHOD.to_string())),
            TransactionCondition::TransferTo(StringCondition::EqualTo(ADDRESS.to_string())),
            TransactionCondition::TransferFrom(StringCondition::StartsWith(PREFIX.to_string())),
            TransactionCondition::TransferSpender(StringCondition::Contains(CONTENT.to_string())),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_chain_specific_fields() {
        let mut builder = TxBuilder::new();

        builder.chain_id().eq(VALUES[0]);
        builder.block_number().gt(VALUES[1]);
        builder.index().lt(VALUES[2]);
        builder.tx_type().eq(VALUES[3]);

        let expected_conditions = vec![
            TransactionCondition::ChainId(NumericCondition::EqualTo(VALUES[0])),
            TransactionCondition::BlockNumber(NumericCondition::GreaterThan(VALUES[1])),
            TransactionCondition::TransactionIndex(NumericCondition::LessThan(VALUES[2])),
            TransactionCondition::Type(NumericCondition::EqualTo(VALUES[3])),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }
}
