use crate::filter::conditions::FilterNode;

pub trait FilterBuilderOps {
    fn new() -> Self;
    fn take_filters(&mut self) -> Vec<FilterNode>;
}
