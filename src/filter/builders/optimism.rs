use crate::filter::{
    conditions::{FilterCondition, FilterNode, NodeBuilder},
    field::{DynField, DynValueFieldType, FieldWrapper},
};

use super::builder_ops::FilterBuilderOps;

// ===== OPTIMISIM FILTER BUILDER ============
pub struct OptimismFilterBuilder {
    pub(crate) nodes: Vec<FilterNode>,
}

impl FilterBuilderOps for OptimismFilterBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn take_nodes(&mut self) -> Vec<FilterNode> {
        std::mem::take(&mut self.nodes)
    }
}

impl NodeBuilder for OptimismFilterBuilder {
    type Condition = FilterCondition;

    fn append_node(&mut self, condition: FilterCondition) {
        let node = FilterNode {
            children: None,
            value: Some(condition),
        };
        self.nodes.push(node);
    }
}

#[allow(dead_code)]
impl OptimismFilterBuilder {
    pub fn field(&mut self, path: &str) -> FieldWrapper<'_, DynValueFieldType<DynField>, Self> {
        FieldWrapper {
            field: DynValueFieldType(DynField(path.to_string())),
            parent: self,
        }
    }
}
