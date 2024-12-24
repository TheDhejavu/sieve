use super::{
    block_header::BlockHeaderBuilder, builder_ops::FilterBuilderOps, event::EventBuilder,
    optimism::OptimismFilterBuilder, pool::PoolBuilder, transaction::TxBuilder,
};
use crate::filter::{
    conditions::{FilterNode, LogicalOp},
    LogicalOps,
};
use std::marker::PhantomData;

/// ===== LOGICAL OPERATION BUILDER =====
#[allow(dead_code)]
pub struct LogicalFilterBuilder<'a, B: FilterBuilderOps> {
    pub(crate) nodes: &'a mut Vec<FilterNode>,
    pub(crate) _marker: PhantomData<B>,
}

#[allow(dead_code)]
impl<B: FilterBuilderOps> LogicalFilterBuilder<'_, B> {
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

impl<T> LogicalOps<T> for T
where
    T: AsMut<Vec<FilterNode>> + FilterBuilderOps,
{
    fn and<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::And, f)
    }

    fn all_of<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::And, f)
    }

    fn not<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::Not, f)
    }

    fn unless<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::Not, f)
    }

    fn or<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::Or, f)
    }

    fn any_of<F>(&mut self, f: F) -> LogicalFilterBuilder<T>
    where
        F: FnOnce(&mut T),
    {
        LogicalFilterBuilder {
            nodes: self.as_mut(),
            _marker: PhantomData,
        }
        .build_logical_operation(LogicalOp::Or, f)
    }
}

impl AsMut<Vec<FilterNode>> for TxBuilder {
    fn as_mut(&mut self) -> &mut Vec<FilterNode> {
        &mut self.nodes
    }
}

impl AsMut<Vec<FilterNode>> for BlockHeaderBuilder {
    fn as_mut(&mut self) -> &mut Vec<FilterNode> {
        &mut self.nodes
    }
}

impl AsMut<Vec<FilterNode>> for EventBuilder {
    fn as_mut(&mut self) -> &mut Vec<FilterNode> {
        &mut self.nodes
    }
}

impl AsMut<Vec<FilterNode>> for PoolBuilder {
    fn as_mut(&mut self) -> &mut Vec<FilterNode> {
        &mut self.nodes
    }
}

impl AsMut<Vec<FilterNode>> for OptimismFilterBuilder {
    fn as_mut(&mut self) -> &mut Vec<FilterNode> {
        &mut self.nodes
    }
}
