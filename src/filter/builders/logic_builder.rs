use super::builder_ops::FilterBuilderOps;
use crate::filter::conditions::{FilterNode, LogicalOp};
use std::marker::PhantomData;

/// ===== Logical Builder =====
#[allow(dead_code)]
pub struct LogicalFilterBuilder<'a, B: FilterBuilderOps> {
    pub(crate) nodes: &'a mut Vec<FilterNode>,
    pub(crate) _marker: PhantomData<B>,
}

#[allow(dead_code)]
impl<B: FilterBuilderOps> LogicalFilterBuilder<'_, B> {
    pub fn and<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        self.build_logical_operation(LogicalOp::And, f)
    }

    pub fn not<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        self.build_logical_operation(LogicalOp::Not, f)
    }
    pub fn unless<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        self.build_logical_operation(LogicalOp::Not, f)
    }

    pub fn xor<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        self.build_logical_operation(LogicalOp::Xor, f)
    }

    pub fn or<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        self.build_logical_operation(LogicalOp::Or, f)
    }

    fn build_logical_operation<F>(self, op: LogicalOp, f: F) -> Self
    where
        F: FnOnce(&mut B),
    {
        let mut builder = B::new();
        f(&mut builder);
        let builder_nodes = builder.take_nodes();

        // Only create OR group if we have multiple filters
        match builder_nodes.len() {
            0 => self,
            1 => {
                self.nodes.extend(builder_nodes);
                self
            }
            _ => {
                let node = FilterNode {
                    group: Some((op, builder_nodes)),
                    condition: None,
                };
                self.nodes.push(node);
                self
            }
        }
    }
}
