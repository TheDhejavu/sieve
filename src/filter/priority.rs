use super::conditions::{
    BlockHeaderCondition, EventCondition, FilterCondition, PoolCondition, TransactionCondition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Basic = 0,   // Simple numeric comparisons
    Hash = 1,    // Hash and address comparisons
    Array = 2,   // Array operations
    Complex = 3, // Operations requiring decoding
}

pub trait Prioritized {
    fn priority(&self) -> Priority;
}

macro_rules! impl_condition_priority {
    (
        $type:ty,
        basic: [$($basic:pat),* $(,)?],
        hash: [$($hash:pat),* $(,)?],
        array: [$($array:pat),* $(,)?],
        complex: [$($complex:pat),* $(,)?]
    ) => {
        impl Prioritized for $type {
            fn priority(&self) -> Priority {
                match self {
                    $($basic => Priority::Basic,)*
                    $($hash => Priority::Hash,)*
                    $($array => Priority::Array,)*
                    $($complex => Priority::Complex,)*
                }
            }
        }
    };
}

impl_condition_priority!(
    TransactionCondition,
    basic: [
        Self::Gas(_),
        Self::Nonce(_),
        Self::Type(_),
        Self::ChainId(_),
        Self::BlockNumber(_),
        Self::TransactionIndex(_),
        Self::GasPrice(_),
        Self::MaxFeePerGas(_),
        Self::MaxPriorityFee(_),
        Self::DynField(_),
    ],
    hash: [
        Self::From(_),
        Self::To(_),
        Self::Hash(_),
        Self::BlockHash(_)
    ],
    array: [
        Self::AccessList(_)
    ],
    complex: [
        Self::Value(_),
        Self::CallData{..}
    ]
);

impl_condition_priority!(
    EventCondition,
    basic: [
        Self::LogIndex(_),
        Self::BlockNumber(_),
        Self::TxIndex(_),
        Self::DynField(_),
    ],
    hash: [
        Self::Contract(_),
        Self::BlockHash(_),
        Self::TxHash(_)
    ],
    array: [
        Self::Topics(_)
    ],
    complex: [
        Self::EventData{..}
    ]
);

impl_condition_priority!(
    PoolCondition,
    basic: [
        Self::Nonce(_),
        Self::GasPrice(_),
        Self::GasLimit(_),
    ],
    hash: [
        Self::Hash(_),
        Self::To(_),
        Self::From(_)
    ],
    array: [],
    complex: [
        Self::Value(_)
    ]
);

impl_condition_priority!(
    BlockHeaderCondition,
    basic: [
        Self::BaseFee(_),
        Self::Number(_),
        Self::Timestamp(_),
        Self::GasUsed(_),
        Self::GasLimit(_),
        Self::DynField(_),
    ],
    hash: [
        Self::ParentHash(_),
        Self::StateRoot(_),
        Self::ReceiptsRoot(_),
        Self::TransactionsRoot(_)
    ],
    array: [],
    complex: []
);

impl Prioritized for FilterCondition {
    fn priority(&self) -> Priority {
        match self {
            Self::Transaction(cond) => cond.priority(),
            Self::Event(cond) => cond.priority(),
            Self::Pool(cond) => cond.priority(),
            Self::BlockHeader(cond) => cond.priority(),
            Self::DynField(_cond) => Priority::Complex,
        }
    }
}
