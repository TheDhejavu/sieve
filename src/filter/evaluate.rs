use alloy_consensus::{BlockHeader, Transaction, Typed2718};
use alloy_primitives::U256;
use alloy_rpc_types::{Block, Log, Transaction as RpcTransaction};

use crate::engine::DecodedData;
use serde_json::Value;

use super::conditions::{
    ArrayCondition, BlockCondition, EventCondition, NumericCondition, NumericType,
    ParameterCondition, PoolCondition, StringCondition, TransactionCondition,
};

pub(crate) trait Evaluable<T> {
    fn evaluate(&self, value: &T) -> bool;
}

pub(crate) trait EvaluableWithDecodedData<T> {
    // Check if we should proceed with full evaluation
    fn pre_evaluate(&self, value: &T) -> bool {
        // Default implementation always returns true
        // Individual implementations will override this
        true
    }

    fn evaluate(&self, value: &T, context: Option<&DecodedData>) -> bool;
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

impl Evaluable<Value> for ParameterCondition {
    fn evaluate(&self, value: &Value) -> bool {
        match self {
            Self::U256(condition) => {
                if let Some(val_str) = value.as_str() {
                    // Convert string value to U256
                    let val = U256::from_string(val_str.to_string());
                    return condition.evaluate(&val);
                }
                false
            }

            Self::U128(condition) => {
                if let Some(val_str) = value.as_str() {
                    // Convert string value to u128
                    let val = u128::from_string(val_str.to_string());
                    return condition.evaluate(&val);
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

impl EvaluableWithDecodedData<RpcTransaction> for TransactionCondition {
    fn evaluate(&self, tx: &RpcTransaction, decoded_data: Option<&DecodedData>) -> bool {
        match self {
            TransactionCondition::Value(condition) => condition.evaluate(&tx.value()),
            TransactionCondition::GasPrice(condition) => {
                condition.evaluate(&tx.gas_price().unwrap_or_default())
            }
            TransactionCondition::From(condition) => condition.evaluate(&tx.from.to_string()),
            TransactionCondition::MaxFeePerGas(condition) => {
                condition.evaluate(&tx.max_fee_per_gas())
            }
            TransactionCondition::MaxPriorityFee(condition) => {
                condition.evaluate(&tx.max_priority_fee_per_gas().unwrap_or_default())
            }
            TransactionCondition::BlockNumber(condition) => {
                condition.evaluate(&tx.block_number.unwrap_or_default())
            }
            TransactionCondition::BlockHash(condition) => {
                condition.evaluate(&tx.block_hash.unwrap_or_default().to_string())
            }
            TransactionCondition::ChainId(condition) => {
                condition.evaluate(&tx.chain_id().unwrap_or_default())
            }
            TransactionCondition::To(condition) => {
                condition.evaluate(&tx.to().unwrap_or_default().to_string())
            }
            TransactionCondition::Nonce(condition) => condition.evaluate(&tx.nonce()),
            TransactionCondition::Type(condition) => condition.evaluate(&tx.ty()),
            TransactionCondition::TransactionIndex(condition) => {
                condition.evaluate(&tx.transaction_index.unwrap_or_default())
            }
            TransactionCondition::Hash(condition) => {
                condition.evaluate(&tx.inner.tx_hash().to_string())
            }
            // Contract call specific conditions
            TransactionCondition::Method(condition) => {
                if let Some(DecodedData::ContractCall(decoded)) = decoded_data {
                    let method = decoded.get_method();
                    return condition.evaluate(&method.to_string());
                }
                false
            }
            TransactionCondition::Path(_path, _condition) => true,
            TransactionCondition::Parameter(param, condition) => {
                if let Some(DecodedData::ContractCall(decoded)) = decoded_data {
                    let parameter_value = decoded.get_parameter(param);
                    if let Some(value) = parameter_value {
                        return condition.evaluate(value);
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl EvaluableWithDecodedData<RpcTransaction> for PoolCondition {
    fn evaluate(&self, tx_pool: &RpcTransaction, _decoded_data: Option<&DecodedData>) -> bool {
        if !self.pre_evaluate(tx_pool) {
            return false;
        }

        match self {
            PoolCondition::Value(condition) => condition.evaluate(&tx_pool.value()),
            PoolCondition::GasPrice(condition) => {
                condition.evaluate(&tx_pool.gas_price().unwrap_or_default())
            }
            PoolCondition::From(condition) => condition.evaluate(&tx_pool.from.to_string()),
            PoolCondition::Nonce(condition) => condition.evaluate(&tx_pool.nonce()),
            PoolCondition::GasLimit(condition) => condition.evaluate(&tx_pool.gas_limit()),
            PoolCondition::Hash(condition) => {
                condition.evaluate(&tx_pool.inner.tx_hash().to_string())
            }
            PoolCondition::To(condition) => {
                condition.evaluate(&tx_pool.to().unwrap_or_default().to_string())
            }
            _ => false,
        }
    }
}

impl Evaluable<Block> for BlockCondition {
    fn evaluate(&self, block: &Block) -> bool {
        match self {
            BlockCondition::BaseFee(condition) => {
                condition.evaluate(&block.header.base_fee_per_gas().unwrap_or_default())
            }
            BlockCondition::Number(condition) => condition.evaluate(&block.header.number),
            BlockCondition::Timestamp(condition) => condition.evaluate(&block.header.timestamp),
            BlockCondition::Size(condition) => {
                condition.evaluate(&block.header.size.unwrap_or_default())
            }
            BlockCondition::GasUsed(condition) => condition.evaluate(&block.header.gas_used),
            BlockCondition::GasLimit(condition) => condition.evaluate(&block.header.gas_limit),
            BlockCondition::TransactionCount(condition) => {
                condition.evaluate(&(block.transactions.len() as u64))
            }
            BlockCondition::Hash(condition) => condition.evaluate(&block.header.hash.to_string()),
            BlockCondition::ParentHash(condition) => {
                condition.evaluate(&block.header.parent_hash.to_string())
            }
            BlockCondition::StateRoot(condition) => {
                condition.evaluate(&block.header.state_root.to_string())
            }
            BlockCondition::ReceiptsRoot(condition) => {
                condition.evaluate(&block.header.receipts_root.to_string())
            }
            BlockCondition::TransactionsRoot(condition) => {
                condition.evaluate(&block.header.transactions_root.to_string())
            }
        }
    }
}

impl EvaluableWithDecodedData<Log> for EventCondition {
    fn evaluate(&self, log: &Log, _decoded_data: Option<&DecodedData>) -> bool {
        if !self.pre_evaluate(log) {
            return false;
        }

        match self {
            EventCondition::Contract(condition) => todo!(),
            EventCondition::BlockHash(condition) => todo!(),
            EventCondition::TxHash(condition) => todo!(),
            EventCondition::Parameter(_, condition) => todo!(),
            EventCondition::LogIndex(condition) => todo!(),
            EventCondition::BlockNumber(condition) => todo!(),
            EventCondition::TxIndex(condition) => todo!(),
            EventCondition::Topics(condition) => todo!(),
            EventCondition::Name(string_condition) => todo!(),
        }
    }

    fn pre_evaluate(&self, log: &Log) -> bool {
        match self {
            Self::Contract(condition) => {
                // TODO: Check if address is in bloom filter
                true
            }
            Self::Parameter(name, _) => {
                // TODO: Check if topic is in bloom filter
                true
            }
            _ => true,
        }
    }
}
