// transaction builder
use crate::filter::{
    conditions::{ConditionBuilder, TransactionCondition},
    field::{
        ArrayFieldType, ContractField, FieldWrapper, ParamFieldType, StringFieldType, TxField,
        U128FieldType, U256FieldType, U64FieldType, U8FieldType,
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

    pub fn value(&mut self) -> FieldWrapper<'_, U256FieldType<TxField>, Self> {
        FieldWrapper {
            field: U256FieldType(TxField::Value),
            parent: self,
        }
    }

    pub fn gas_price(&mut self) -> FieldWrapper<'_, U128FieldType<TxField>, Self> {
        FieldWrapper {
            field: U128FieldType(TxField::GasPrice),
            parent: self,
        }
    }

    pub fn gas(&mut self) -> FieldWrapper<'_, U64FieldType<TxField>, Self> {
        FieldWrapper {
            field: U64FieldType(TxField::Gas),
            parent: self,
        }
    }

    pub fn nonce(&mut self) -> FieldWrapper<'_, U64FieldType<TxField>, Self> {
        FieldWrapper {
            field: U64FieldType(TxField::Nonce),
            parent: self,
        }
    }

    pub fn max_fee_per_gas(&mut self) -> FieldWrapper<'_, U128FieldType<TxField>, Self> {
        FieldWrapper {
            field: U128FieldType(TxField::MaxFeePerGas),
            parent: self,
        }
    }

    pub fn max_priority_fee(&mut self) -> FieldWrapper<'_, U128FieldType<TxField>, Self> {
        FieldWrapper {
            field: U128FieldType(TxField::MaxPriorityFee),
            parent: self,
        }
    }

    pub fn chain_id(&mut self) -> FieldWrapper<'_, U64FieldType<TxField>, Self> {
        FieldWrapper {
            field: U64FieldType(TxField::ChainId),
            parent: self,
        }
    }

    pub fn block_number(&mut self) -> FieldWrapper<'_, U64FieldType<TxField>, Self> {
        FieldWrapper {
            field: U64FieldType(TxField::BlockNumber),
            parent: self,
        }
    }

    pub fn index(&mut self) -> FieldWrapper<'_, U64FieldType<TxField>, Self> {
        FieldWrapper {
            field: U64FieldType(TxField::TransactionIndex),
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

    pub fn tx_type(&mut self) -> FieldWrapper<'_, U8FieldType<TxField>, Self> {
        FieldWrapper {
            field: U8FieldType(TxField::Type),
            parent: self,
        }
    }

    pub fn access_list(&mut self) -> FieldWrapper<'_, ArrayFieldType<TxField>, Self> {
        FieldWrapper {
            field: ArrayFieldType(TxField::AccessList),
            parent: self,
        }
    }

    pub fn contract(&mut self) -> ContractBuilder<Self> {
        ContractBuilder::new(self)
    }
}

#[allow(dead_code)]
pub struct ContractBuilder<'a, B> {
    parent: &'a mut B,
}

impl ConditionBuilder for ContractBuilder<'_, TxBuilder> {
    type Condition = TransactionCondition;

    fn push_condition(&mut self, condition: TransactionCondition) {
        self.parent.push_condition(condition)
    }
}

impl<'a> ContractBuilder<'a, TxBuilder> {
    pub fn new(parent: &'a mut TxBuilder) -> Self {
        Self { parent }
    }

    pub fn method(&mut self) -> FieldWrapper<'_, StringFieldType<ContractField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(ContractField::Method),
            parent: self.parent,
        }
    }

    pub fn params(
        &mut self,
        name: &str,
    ) -> FieldWrapper<'_, ParamFieldType<ContractField>, TxBuilder> {
        FieldWrapper {
            field: ParamFieldType(ContractField::Parameter(name.to_string())),
            parent: self.parent,
        }
    }
    
    pub fn path(
        &mut self,
        path: &str,
    ) -> FieldWrapper<'_, StringFieldType<ContractField>, TxBuilder> {
        FieldWrapper {
            field: StringFieldType(ContractField::Path(path.to_string())),
            parent: self.parent,
        }
    }
}
#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;
    use crate::filter::{
        conditions::{
            ArrayCondition, NumericCondition, ParameterCondition, StringCondition,
            TransactionCondition,
        },
        ArrayOps, NumericOps, StringOps,
    };

    const ADDRESS: &str = "0x123";
    const PREFIX: &str = "0x";
    const CONTENT: &str = "abc";
    const METHOD: &str = "transfer";

    #[test]
    fn test_tx_numeric_field_operations() {
        let mut builder = TxBuilder::new();
        builder.value().eq(U256::from(100));
        builder.gas_price().gt(100);
        builder.gas().lt(100);
        builder.nonce().between(100, 200);
        builder.max_fee_per_gas().lte(100_u128);
        builder.max_priority_fee().gte(100_u128);

        let expected_conditions = vec![
            TransactionCondition::Value(NumericCondition::EqualTo(U256::from(100))),
            TransactionCondition::GasPrice(NumericCondition::GreaterThan(100)),
            TransactionCondition::Gas(NumericCondition::LessThan(100)),
            TransactionCondition::Nonce(NumericCondition::Between(100, 200)),
            TransactionCondition::MaxFeePerGas(NumericCondition::LessThanOrEqualTo(100)),
            TransactionCondition::MaxPriorityFee(NumericCondition::GreaterThanOrEqualTo(100)),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_string_field_operations() {
        let mut builder = TxBuilder::new();

        builder.from().exact(ADDRESS);
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
    fn test_tx_contract_operations() {
        let mut builder = TxBuilder::new();
        let mut transfer = builder.contract();

        transfer.method().exact(METHOD);
        transfer
            .path("tokenIn")
            .exact("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");

        transfer.params("amountIn").gt(100_u128);

        let expected_conditions = vec![
            TransactionCondition::Method(StringCondition::EqualTo(METHOD.to_string())),
            TransactionCondition::Path(
                "tokenIn".to_string(),
                StringCondition::EqualTo("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string()),
            ),
            TransactionCondition::Parameter(
                "amountIn".to_string(),
                ParameterCondition::U128(NumericCondition::GreaterThan(100_u128)),
            ),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }

    #[test]
    fn test_tx_chain_specific_fields() {
        let mut builder = TxBuilder::new();

        builder.chain_id().eq(100);
        builder.block_number().gt(100);
        builder.index().lt(100);
        builder.tx_type().eq(1);

        let expected_conditions = vec![
            TransactionCondition::ChainId(NumericCondition::EqualTo(100)),
            TransactionCondition::BlockNumber(NumericCondition::GreaterThan(100)),
            TransactionCondition::TransactionIndex(NumericCondition::LessThan(100)),
            TransactionCondition::Type(NumericCondition::EqualTo(1)),
        ];

        assert_eq!(builder.conditions, expected_conditions);
    }
}
