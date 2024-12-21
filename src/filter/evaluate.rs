use std::sync::Arc;

use crate::engine::DecodedData;
use alloy_consensus::{BlockHeader, Transaction, Typed2718};
use alloy_dyn_abi::DynSolValue;
use alloy_rpc_types::{Block, Header, Log, Transaction as RpcTransaction};

use super::conditions::{
    ArrayCondition, BlockHeaderCondition, EventCondition, FilterCondition, NumericCondition,
    NumericType, ParameterCondition, PoolCondition, StringCondition, TransactionCondition,
};
pub(crate) trait Evaluable<T> {
    fn evaluate(&self, value: &T) -> bool;
}

pub(crate) trait EvaluableWithCondition<C> {
    // Check if we should proceed with full evaluation
    fn pre_evaluate(&self, _condition: &C) -> bool {
        // Default implementation always returns true
        // Individual implementations will override this
        true
    }

    fn evaluate(&self, condition: &C, decoded_data: Option<Arc<DecodedData>>) -> bool;
}

impl FilterCondition {
    pub(crate) fn needs_decoded_data(&self) -> bool {
        match self {
            FilterCondition::Transaction(transaction_condition) => {
                matches!(
                    transaction_condition,
                    TransactionCondition::Method(_)
                        | TransactionCondition::Parameter(_, _)
                        | TransactionCondition::Path(_, _)
                )
            }
            FilterCondition::Event(event_condition) => {
                matches!(event_condition, EventCondition::EventMatch { .. })
            }
            FilterCondition::Pool(_) => false,
            FilterCondition::BlockHeader(_) => false,
        }
    }
}

impl<T> Evaluable<T> for NumericCondition<T>
where
    T: NumericType,
{
    fn evaluate(&self, value: &T) -> bool {
        match self {
            Self::GreaterThan(threshold) => value > threshold,
            Self::GreaterThanOrEqualTo(threshold) => value >= threshold,
            Self::LessThan(threshold) => value < threshold,
            Self::LessThanOrEqualTo(threshold) => value <= threshold,
            Self::EqualTo(threshold) => value == threshold,
            Self::NotEqualTo(threshold) => value != threshold,
            Self::Between(min, max) => value >= min && value <= max,
            Self::Outside(min, max) => value < min || value > max,
        }
    }
}

impl Evaluable<String> for StringCondition {
    fn evaluate(&self, value: &String) -> bool {
        match self {
            Self::EqualTo(expected) => value == expected,
            Self::Contains(substring) => value.contains(substring),
            Self::StartsWith(prefix) => value.starts_with(prefix),
            Self::EndsWith(suffix) => value.ends_with(suffix),
            Self::Matches(pattern) => {
                // TODO: Use regex pattern matching here
                value == pattern
            }
        }
    }
}

impl<T> Evaluable<Vec<T>> for ArrayCondition<T>
where
    T: PartialEq,
{
    fn evaluate(&self, value: &Vec<T>) -> bool {
        match self {
            Self::Contains(item) => value.contains(item),
            Self::NotIn(items) => !items.iter().any(|item| value.contains(item)),
            Self::Empty => value.is_empty(),
            Self::NotEmpty => !value.is_empty(),
        }
    }
}

impl Evaluable<DynSolValue> for ParameterCondition {
    fn evaluate(&self, value: &DynSolValue) -> bool {
        match self {
            Self::U256(condition) => {
                if let Some((value_uint, size)) = value.as_uint() {
                    // Check that we have a uint256
                    if size == 256 {
                        return condition.evaluate(&value_uint);
                    }
                }
                false
            }

            Self::U128(condition) => {
                if let Some((value_uint, size)) = value.as_uint() {
                    // Check that we have a uint128 or smaller
                    if size <= 128 {
                        // If the value fits in u128, we can evaluate it
                        let limbs = value_uint.as_limbs();
                        if limbs[2] == 0 && limbs[3] == 0 {
                            // Only lower 128 bits are used
                            let value_u128 = (limbs[1] as u128) << 64 | (limbs[0] as u128);
                            return condition.evaluate(&value_u128);
                        }
                    }
                }
                false
            }

            Self::String(condition) => {
                if let Some(val_str) = value.as_str() {
                    return condition.evaluate(&val_str.to_string());
                }
                false
            }
        }
    }
}

impl EvaluableWithCondition<TransactionCondition> for RpcTransaction {
    fn evaluate(
        &self,
        condition: &TransactionCondition,
        decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        if !self.pre_evaluate(condition) {
            return false;
        }

        match condition {
            TransactionCondition::Value(condition) => condition.evaluate(&self.value()),
            TransactionCondition::GasPrice(condition) => {
                condition.evaluate(&self.gas_price().unwrap_or_default())
            }
            TransactionCondition::From(condition) => condition.evaluate(&self.from.to_string()),
            TransactionCondition::MaxFeePerGas(condition) => {
                condition.evaluate(&self.max_fee_per_gas())
            }
            TransactionCondition::MaxPriorityFee(condition) => {
                condition.evaluate(&self.max_priority_fee_per_gas().unwrap_or_default())
            }
            TransactionCondition::BlockNumber(condition) => {
                condition.evaluate(&self.block_number.unwrap_or_default())
            }
            TransactionCondition::BlockHash(condition) => {
                condition.evaluate(&self.block_hash.unwrap_or_default().to_string())
            }
            TransactionCondition::ChainId(condition) => {
                condition.evaluate(&self.chain_id().unwrap_or_default())
            }
            TransactionCondition::To(condition) => {
                condition.evaluate(&self.to().unwrap_or_default().to_string())
            }
            TransactionCondition::Nonce(condition) => condition.evaluate(&self.nonce()),
            TransactionCondition::Type(condition) => condition.evaluate(&self.ty()),
            TransactionCondition::TransactionIndex(condition) => {
                condition.evaluate(&self.transaction_index.unwrap_or_default())
            }
            TransactionCondition::Hash(condition) => {
                condition.evaluate(&self.inner.tx_hash().to_string())
            }
            TransactionCondition::Method(condition) => {
                if let Some(DecodedData::ContractCall(decoded)) =
                    decoded_data.as_ref().map(|arc| arc.as_ref())
                {
                    let method = decoded.get_method();
                    return condition.evaluate(&method.to_string());
                }
                false
            }
            TransactionCondition::Path(_path, _condition) => true,
            TransactionCondition::Parameter(param, condition) => {
                if let Some(DecodedData::ContractCall(decoded)) =
                    decoded_data.as_ref().map(|arc| arc.as_ref())
                {
                    let parameter_value = decoded.get_parameter(param);
                    if let Some(value) = parameter_value {
                        // return condition.evaluate(value);
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl EvaluableWithCondition<PoolCondition> for RpcTransaction {
    fn evaluate(&self, condition: &PoolCondition, _decoded_data: Option<Arc<DecodedData>>) -> bool {
        if !self.pre_evaluate(condition) {
            return false;
        }

        match condition {
            PoolCondition::Value(condition) => condition.evaluate(&self.value()),
            PoolCondition::GasPrice(condition) => {
                condition.evaluate(&self.gas_price().unwrap_or_default())
            }
            PoolCondition::From(condition) => condition.evaluate(&self.from.to_string()),
            PoolCondition::Nonce(condition) => condition.evaluate(&self.nonce()),
            PoolCondition::GasLimit(condition) => condition.evaluate(&self.gas_limit()),
            PoolCondition::Hash(condition) => condition.evaluate(&self.inner.tx_hash().to_string()),
            PoolCondition::To(condition) => {
                condition.evaluate(&self.to().unwrap_or_default().to_string())
            }
            _ => false,
        }
    }
}

impl EvaluableWithCondition<BlockHeaderCondition> for Header {
    fn evaluate(
        &self,
        condition: &BlockHeaderCondition,
        _decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        match condition {
            BlockHeaderCondition::BaseFee(condition) => {
                condition.evaluate(&self.base_fee_per_gas().unwrap_or_default())
            }
            BlockHeaderCondition::Number(condition) => condition.evaluate(&self.number),
            BlockHeaderCondition::Timestamp(condition) => condition.evaluate(&self.timestamp),
            BlockHeaderCondition::Size(condition) => {
                condition.evaluate(&self.size.unwrap_or_default())
            }
            BlockHeaderCondition::GasUsed(condition) => condition.evaluate(&self.gas_used),
            BlockHeaderCondition::GasLimit(condition) => condition.evaluate(&self.gas_limit),
            BlockHeaderCondition::Hash(condition) => condition.evaluate(&self.hash.to_string()),
            BlockHeaderCondition::ParentHash(condition) => {
                condition.evaluate(&self.parent_hash.to_string())
            }
            BlockHeaderCondition::StateRoot(condition) => {
                condition.evaluate(&self.state_root.to_string())
            }
            BlockHeaderCondition::ReceiptsRoot(condition) => {
                condition.evaluate(&self.receipts_root.to_string())
            }
            BlockHeaderCondition::TransactionsRoot(condition) => {
                condition.evaluate(&self.transactions_root.to_string())
            }
        }
    }
}

// For Log
impl EvaluableWithCondition<EventCondition> for Log {
    fn evaluate(
        &self,
        condition: &EventCondition,
        _decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        if !self.pre_evaluate(condition) {
            return false;
        }

        match condition {
            EventCondition::Contract(condition) => todo!(),
            EventCondition::BlockHash(condition) => todo!(),
            EventCondition::TxHash(condition) => todo!(),
            EventCondition::LogIndex(condition) => todo!(),
            EventCondition::BlockNumber(condition) => todo!(),
            EventCondition::TxIndex(condition) => todo!(),
            EventCondition::Topics(condition) => todo!(),
            EventCondition::Name(string_condition) => todo!(),
            EventCondition::EventMatch {
                signature,
                parameters,
            } => todo!(),
        }
    }

    fn pre_evaluate(&self, condition: &EventCondition) -> bool {
        match condition {
            EventCondition::Contract(_) => {
                // TODO: Check if address is in bloom filter
                true
            }
            _ => true,
        }
    }
}
