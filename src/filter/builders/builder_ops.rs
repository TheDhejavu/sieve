use crate::filter::conditions::FilterNode;

pub trait FilterBuilderOps {
    fn new() -> Self;
    fn take_nodes(&mut self) -> Vec<FilterNode>;
}
