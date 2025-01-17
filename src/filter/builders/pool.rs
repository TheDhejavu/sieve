// Pool builder
use crate::filter::{
    conditions::{FilterCondition, FilterNode, NodeBuilder, PoolCondition},
    field::{FieldWrapper, PoolField, StringFieldType, U128FieldType, U256FieldType, U64FieldType},
};

use super::builder_ops::FilterBuilderOps;

// ===== Pool Builder =====
pub struct PoolBuilder {
    pub(crate) nodes: Vec<FilterNode>,
}

#[allow(dead_code)]
impl Default for PoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn nonce(&mut self) -> FieldWrapper<'_, U64FieldType<PoolField>, Self> {
        FieldWrapper {
            field: U64FieldType(PoolField::Nonce),
            parent: self,
        }
    }

    pub fn value(&mut self) -> FieldWrapper<'_, U256FieldType<PoolField>, Self> {
        FieldWrapper {
            field: U256FieldType(PoolField::Value),
            parent: self,
        }
    }

    pub fn gas_price(&mut self) -> FieldWrapper<'_, U128FieldType<PoolField>, Self> {
        FieldWrapper {
            field: U128FieldType(PoolField::GasPrice),
            parent: self,
        }
    }

    pub fn gas_limit(&mut self) -> FieldWrapper<'_, U64FieldType<PoolField>, Self> {
        FieldWrapper {
            field: U64FieldType(PoolField::GasLimit),
            parent: self,
        }
    }

    pub fn from(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::From),
            parent: self,
        }
    }

    pub fn to(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::To),
            parent: self,
        }
    }

    pub fn hash(&mut self) -> FieldWrapper<'_, StringFieldType<PoolField>, Self> {
        FieldWrapper {
            field: StringFieldType(PoolField::Hash),
            parent: self,
        }
    }

    pub fn timestamp(&mut self) -> FieldWrapper<'_, U64FieldType<PoolField>, Self> {
        FieldWrapper {
            field: U64FieldType(PoolField::Timestamp),
            parent: self,
        }
    }
}

impl NodeBuilder for PoolBuilder {
    type Condition = PoolCondition;

    fn append_node(&mut self, condition: PoolCondition) {
        // root node is a condition without a
        self.nodes.push(FilterNode {
            children: None,
            value: Some(FilterCondition::Pool(condition)),
        })
    }
}

impl FilterBuilderOps for PoolBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn take_nodes(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.nodes)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::ruint::aliases::U256;

    use super::*;
    use crate::filter::{
        conditions::{NumericCondition, PoolCondition, StringCondition},
        NumericOps, StringOps,
    };

    const NONCE: u64 = 1;
    const ADDRESS: &str = "0xABCD1234";
    const HASH: &str = "0x9876fedc";
    const PREFIX: &str = "0x";

    #[test]
    fn test_numeric_field_operations() {
        let mut builder = PoolBuilder::new();

        // Test various numeric operations
        builder.nonce().eq(NONCE);
        builder.value().gt(U256::from(100));
        builder.gas_price().gte(100);

        let expected_conditions = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::Nonce(
                    NumericCondition::EqualTo(NONCE),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::Value(
                    NumericCondition::GreaterThan(U256::from(100)),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::GasPrice(
                    NumericCondition::GreaterThanOrEqualTo(100),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_conditions);
    }

    #[test]
    fn test_string_field_operations() {
        let mut builder = PoolBuilder::new();

        // Test various string operations
        builder.from().exact(ADDRESS);
        builder.to().contains(HASH);
        builder.hash().starts_with(PREFIX);

        let expected_conditions = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::From(
                    StringCondition::EqualTo(ADDRESS.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::To(
                    StringCondition::Contains(HASH.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::Hash(
                    StringCondition::StartsWith(PREFIX.to_string()),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_conditions);
    }

    #[test]
    fn test_pool_fields() {
        let mut builder = PoolBuilder::new();

        builder.nonce().eq(NONCE);
        builder.from().contains(ADDRESS);
        builder.gas_price().gt(100);
        builder.hash().starts_with(PREFIX);

        let expected_conditions = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::Nonce(
                    NumericCondition::EqualTo(NONCE),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::From(
                    StringCondition::Contains(ADDRESS.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::GasPrice(
                    NumericCondition::GreaterThan(100),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Pool(PoolCondition::Hash(
                    StringCondition::StartsWith(PREFIX.to_string()),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_conditions);
    }

    #[test]
    fn builder_new() {
        let builder = PoolBuilder::new();
        assert!(builder.nodes.is_empty());
    }
}
