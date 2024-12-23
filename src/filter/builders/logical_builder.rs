use super::builder_ops::FilterBuilderOps;
use crate::filter::conditions::{FilterNode, LogicalOp};
use std::marker::PhantomData;

/// ===== Logical Builder =====
#[allow(dead_code)]
pub(crate) struct LogicalFilterBuilder<'a, B: FilterBuilderOps> {
    pub(crate) filters: &'a mut Vec<FilterNode>,
    pub(crate) _builder: PhantomData<B>,
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
        let builder_filters = builder.take_filters();

        // Only create OR group if we have multiple filters
        match builder_filters.len() {
            0 => self,
            1 => {
                self.filters.extend(builder_filters);
                self
            }
            _ => {
                let node = FilterNode {
                    group: Some((op, builder_filters)),
                    condition: None,
                };
                self.filters.push(node);
                self
            }
        }
    }
    pub fn build(&self) -> FilterNode {
        if self.filters.len() == 1 {
            self.filters[0].clone()
        } else {
            FilterNode {
                group: Some((LogicalOp::And, self.filters.clone())),
                condition: None,
            }
        }
    }
}
