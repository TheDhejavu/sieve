use crate::filter::{
    conditions::{ContractCondition, EventCondition, FilterCondition, FilterNode, NodeBuilder},
    field::{
        ArrayFieldType, ContractField, DynValueFieldType, EventField, FieldWrapper,
        StringFieldType, U64FieldType,
    },
};

use super::builder_ops::FilterBuilderOps;

// ===== Event Builder ========
pub struct EventBuilder {
    pub(crate) nodes: Vec<FilterNode>,
}

impl NodeBuilder for EventBuilder {
    type Condition = EventCondition;

    fn append_node(&mut self, condition: EventCondition) {
        self.nodes.push(FilterNode {
            children: None,
            value: Some(FilterCondition::Event(condition)),
        })
    }
}

#[allow(dead_code)]
impl Default for EventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn contract(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::Contract),
            parent: self,
        }
    }

    pub fn name(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::Name),
            parent: self,
        }
    }

    pub fn block_hash(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::BlockHash),
            parent: self,
        }
    }

    pub fn tx_hash(&mut self) -> FieldWrapper<'_, StringFieldType<EventField>, Self> {
        FieldWrapper {
            field: StringFieldType(EventField::TxHash),
            parent: self,
        }
    }

    pub fn log_index(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::LogIndex),
            parent: self,
        }
    }

    pub fn block_number(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::BlockNumber),
            parent: self,
        }
    }

    pub fn tx_index(&mut self) -> FieldWrapper<'_, U64FieldType<EventField>, Self> {
        FieldWrapper {
            field: U64FieldType(EventField::TxIndex),
            parent: self,
        }
    }

    pub fn topics(&mut self) -> FieldWrapper<'_, ArrayFieldType<EventField>, Self> {
        FieldWrapper {
            field: ArrayFieldType(EventField::Topics),
            parent: self,
        }
    }

    pub fn signature(&mut self, signature: &str) -> SignatureEventBuilder<Self> {
        SignatureEventBuilder::new(self, signature.to_string())
    }
}

impl FilterBuilderOps for EventBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn take_nodes(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.nodes)
    }
}

#[allow(dead_code)]
pub struct SignatureEventBuilder<'a, B> {
    parent: &'a mut B,
    signature: String,
    parameter_current_index: Option<usize>,
}

impl NodeBuilder for SignatureEventBuilder<'_, EventBuilder> {
    type Condition = ContractCondition;

    fn append_node(&mut self, condition: ContractCondition) {
        match condition {
            ContractCondition::Parameter(param, parameter_condition) => {
                if let Some(idx) = self.parameter_current_index {
                    if let Some(node) = self.parent.nodes.get_mut(idx) {
                        if let Some(FilterCondition::Event(EventCondition::EventData {
                            parameters,
                            ..
                        })) = node.value.as_mut()
                        {
                            parameters.push((param, parameter_condition));
                        }
                    }
                } else {
                    self.parent.append_node(EventCondition::EventData {
                        signature: self.signature.clone(),
                        parameters: vec![(param, parameter_condition)],
                    });
                    self.parameter_current_index = Some(self.parent.nodes.len() - 1);
                }
            }
            ContractCondition::Path(_, _) => (),
        };
    }
}

impl<'a> SignatureEventBuilder<'a, EventBuilder> {
    pub fn new(parent: &'a mut EventBuilder, signature: String) -> Self {
        Self {
            parent,
            signature,
            parameter_current_index: None,
        }
    }

    pub fn params(
        &mut self,
        name: &str,
    ) -> FieldWrapper<'_, DynValueFieldType<ContractField>, Self> {
        FieldWrapper {
            field: DynValueFieldType(ContractField::Parameter(name.to_string())),
            parent: self,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{
        conditions::{ArrayCondition, FilterCondition, NumericCondition, StringCondition},
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
    const CONTENT: &str = "0x456";
    const PREFIX: &str = "0x";
    const SUFFIX: &str = "789";

    const TOPIC: &str = "topic1";

    #[test]
    fn test_event_numeric_field_operations() {
        let mut builder = EventBuilder::new();

        builder.block_number().eq(VALUES[0]);
        builder.block_number().gt(VALUES[1]);
        builder.block_number().gte(VALUES[2]);
        builder.block_number().lt(VALUES[3]);
        builder.block_number().lte(VALUES[4]);
        builder
            .block_number()
            .between(VALUES[5], VALUES[5] + BASE_VALUE);

        let expected_nodes = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::EqualTo(VALUES[0]),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::GreaterThan(VALUES[1]),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::GreaterThanOrEqualTo(VALUES[2]),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::LessThan(VALUES[3]),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::LessThanOrEqualTo(VALUES[4]),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::BlockNumber(
                    NumericCondition::Between(VALUES[5], VALUES[5] + BASE_VALUE),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_nodes);
    }

    #[test]
    fn test_event_string_field_operations() {
        let mut builder = EventBuilder::new();

        builder.contract().exact(ADDRESS);
        builder.contract().contains(CONTENT);
        builder.contract().starts_with(PREFIX);
        builder.contract().ends_with(SUFFIX);

        let expected_nodes = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Contract(
                    StringCondition::EqualTo(ADDRESS.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Contract(
                    StringCondition::Contains(CONTENT.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Contract(
                    StringCondition::StartsWith(PREFIX.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Contract(
                    StringCondition::EndsWith(SUFFIX.to_string()),
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_nodes);
    }

    #[test]
    fn test_event_array_field_operations() {
        let mut builder = EventBuilder::new();
        builder.topics().contains(TOPIC.to_string());
        builder.topics().not_empty();

        let expected_nodes = vec![
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Topics(
                    ArrayCondition::Contains(TOPIC.to_string()),
                ))),
            },
            FilterNode {
                children: None,
                value: Some(FilterCondition::Event(EventCondition::Topics(
                    ArrayCondition::NotEmpty,
                ))),
            },
        ];

        assert_eq!(builder.nodes, expected_nodes);
    }
}
