// Block Header builder
use crate::filter::{
    conditions::{BlockHeaderCondition, FilterCondition, FilterNode, NodeBuilder},
    field::{
        BlockField, DynField, DynValueFieldType, FieldWrapper, StringFieldType, U256FieldType,
        U64FieldType,
    },
};

use super::builder_ops::FilterBuilderOps;

// ===== BlockHeader Builder =====
pub struct BlockHeaderBuilder {
    pub(crate) nodes: Vec<FilterNode>,
}

impl NodeBuilder for BlockHeaderBuilder {
    type Condition = BlockHeaderCondition;

    fn append_node(&mut self, condition: BlockHeaderCondition) {
        self.nodes.push(FilterNode {
            group: None,
            condition: Some(FilterCondition::BlockHeader(condition)),
        })
    }
}

#[allow(dead_code)]
impl BlockHeaderBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    pub fn number(&mut self) -> FieldWrapper<'_, U64FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U64FieldType(BlockField::Number),
            parent: self,
        }
    }

    pub fn timestamp(&mut self) -> FieldWrapper<'_, U64FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U64FieldType(BlockField::Timestamp),
            parent: self,
        }
    }

    pub fn size(&mut self) -> FieldWrapper<'_, U256FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U256FieldType(BlockField::Size),
            parent: self,
        }
    }

    pub fn gas_used(&mut self) -> FieldWrapper<'_, U64FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U64FieldType(BlockField::GasUsed),
            parent: self,
        }
    }

    pub fn gas_limit(&mut self) -> FieldWrapper<'_, U64FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U64FieldType(BlockField::GasLimit),
            parent: self,
        }
    }

    pub fn base_fee(&mut self) -> FieldWrapper<'_, U64FieldType<BlockField>, Self> {
        FieldWrapper {
            field: U64FieldType(BlockField::BaseFee),
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

    pub fn field(&mut self, path: &str) -> FieldWrapper<'_, DynValueFieldType<DynField>, Self> {
        FieldWrapper {
            field: DynValueFieldType(DynField(path.to_string())),
            parent: self,
        }
    }
}

impl FilterBuilderOps for BlockHeaderBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn take_nodes(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.nodes)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;
    use crate::filter::{
        conditions::{FilterCondition, NumericCondition, StringCondition},
        NumericOps, StringOps,
    };

    const NUMBER: u64 = 1;
    const SIZE: u64 = 1000;
    const GAS_USED: u64 = 2000;
    const GAS_LIMIT: u64 = 2000;
    const TIMESTAMP: u64 = 5000;

    const HASH: &str = "0x123";
    const PREFIX: &str = "0x";
    const SUFFIX: &str = "def";

    #[test]
    fn test_numeric_field_operations() {
        let mut builder = BlockHeaderBuilder::new();

        // Test various numeric operations
        builder.number().eq(NUMBER);
        builder.size().gt(U256::from(SIZE));
        builder.gas_used().gte(GAS_USED);
        builder.gas_limit().lt(GAS_LIMIT);
        builder.timestamp().lte(TIMESTAMP);
        builder.base_fee().eq(100);

        let expected_nodes = vec![
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(BlockHeaderCondition::Number(
                    NumericCondition::EqualTo(NUMBER),
                ))),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(BlockHeaderCondition::Size(
                    NumericCondition::GreaterThan(U256::from(SIZE)),
                ))),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(BlockHeaderCondition::GasUsed(
                    NumericCondition::GreaterThanOrEqualTo(GAS_USED),
                ))),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::GasLimit(NumericCondition::LessThan(GAS_LIMIT)),
                )),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::Timestamp(NumericCondition::LessThanOrEqualTo(TIMESTAMP)),
                )),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(BlockHeaderCondition::BaseFee(
                    NumericCondition::EqualTo(100),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_nodes);
    }

    #[test]
    fn test_string_field_operations() {
        let mut builder = BlockHeaderBuilder::new();

        // Test all string operations
        builder.hash().exact(HASH);
        builder.parent_hash().starts_with(PREFIX);
        builder.state_root().ends_with(SUFFIX);
        builder.receipts_root().exact(HASH);
        builder.transactions_root().starts_with(PREFIX);

        let expected_nodes = vec![
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(BlockHeaderCondition::Hash(
                    StringCondition::EqualTo(HASH.to_string()),
                ))),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::ParentHash(StringCondition::StartsWith(
                        PREFIX.to_string(),
                    )),
                )),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::StateRoot(StringCondition::EndsWith(SUFFIX.to_string())),
                )),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::ReceiptsRoot(StringCondition::EqualTo(HASH.to_string())),
                )),
            },
            FilterNode {
                group: None,
                condition: Some(FilterCondition::BlockHeader(
                    BlockHeaderCondition::TransactionsRoot(StringCondition::StartsWith(
                        PREFIX.to_string(),
                    )),
                )),
            },
        ];

        assert_eq!(builder.nodes, expected_nodes);
    }

    #[test]
    fn builder_new() {
        let builder = BlockHeaderBuilder::new();
        assert!(builder.nodes.is_empty());
    }
}
