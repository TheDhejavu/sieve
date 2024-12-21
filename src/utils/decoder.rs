use alloy_dyn_abi::{DynSolEvent, DynSolType, DynSolValue};
use alloy_primitives::{keccak256, LogData, B256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct DecodedLog {
    pub(crate) name: String,
    pub(crate) params: HashMap<String, DecodedParam>,
}

#[derive(Debug, Clone)]
pub(crate) struct DecodedParam {
    pub value: DynSolValue,
    pub indexed: bool,
}

#[derive(Debug)]
pub(crate) enum DecodeError {
    InvalidFormat,
    InvalidParameter,
    UnableToDecode,
    UnsupportedType,
}

pub(crate) struct EventDefinition {
    pub name: String,
    pub event: DynSolEvent,
    pub param_names: Vec<(String, bool)>,
}

pub(crate) fn parse_event_signature(sig: &str) -> Result<EventDefinition, DecodeError> {
    let (name, params) = match sig.split_once('(') {
        Some((n, p)) => (n.trim(), p.trim_end_matches(')')),
        None => return Err(DecodeError::InvalidFormat),
    };

    let mut indexed = Vec::new();
    let mut body = Vec::new();
    let mut param_names = Vec::new();

    for param in params.split(',').filter(|s| !s.is_empty()) {
        let parts = param.split_whitespace().collect::<Vec<&str>>();

        let (ty, is_indexed, name) = match parts.len() {
            2 => (parts[0], false, parts[1]),
            3 if parts[1] == "indexed" => (parts[0], true, parts[2]),
            _ => return Err(DecodeError::InvalidParameter),
        };

        param_names.push((name.to_string(), is_indexed));

        let sol_type = match ty {
            // Address
            "address" => DynSolType::Address,

            // Integers
            "uint256" => DynSolType::Uint(256),
            "uint128" => DynSolType::Uint(128),
            "uint64" => DynSolType::Uint(64),
            "uint32" => DynSolType::Uint(32),
            "uint16" => DynSolType::Uint(16),
            "uint8" => DynSolType::Uint(8),

            "int256" => DynSolType::Int(256),
            "int128" => DynSolType::Int(128),
            "int64" => DynSolType::Int(64),
            "int32" => DynSolType::Int(32),
            "int16" => DynSolType::Int(16),
            "int8" => DynSolType::Int(8),

            // Boolean
            "bool" => DynSolType::Bool,

            // Strings and Bytes
            "string" => DynSolType::String,
            "bytes" => DynSolType::Bytes,

            // Fixed bytes
            "bytes1" => DynSolType::FixedBytes(1),
            "bytes2" => DynSolType::FixedBytes(2),
            "bytes3" => DynSolType::FixedBytes(3),
            "bytes4" => DynSolType::FixedBytes(4),
            "bytes8" => DynSolType::FixedBytes(8),
            "bytes16" => DynSolType::FixedBytes(16),
            "bytes32" => DynSolType::FixedBytes(32),
            _ => return Err(DecodeError::UnsupportedType),
        };

        if is_indexed {
            indexed.push(sol_type);
        } else {
            body.push(sol_type);
        }
    }

    let keccak2_fixed_bytes = keccak256(sig.as_bytes());
    let src = keccak2_fixed_bytes.as_slice();
    let sig_hash = B256::from_slice(src);

    Ok(EventDefinition {
        name: name.to_string(),
        event: DynSolEvent::new_unchecked(Some(sig_hash), indexed, DynSolType::Tuple(body)),
        param_names,
    })
}

impl DecodedLog {
    pub(crate) fn get_parameter(&self, name: &str) -> Option<&DynSolValue> {
        if let Some(param) = self.params.get(name) {
            return Some(&param.value);
        }
        None
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

impl EventDefinition {
    pub(crate) fn decode_log(&self, log: &LogData) -> Result<DecodedLog, DecodeError> {
        let decoded = self
            .event
            .decode_log_data(log, true)
            .map_err(|_| DecodeError::UnableToDecode)?;

        let mut params = HashMap::new();
        let indexed = decoded
            .indexed
            .into_iter()
            .zip(self.param_names.iter().filter(|(_, indexed)| *indexed));

        for (value, (name, _)) in indexed {
            params.insert(
                name.clone(),
                DecodedParam {
                    value,
                    indexed: true,
                },
            );
        }

        let decoded_data = decoded
            .body
            .into_iter()
            .zip(self.param_names.iter().filter(|(_, indexed)| !*indexed));

        for (value, (name, _)) in decoded_data {
            params.insert(
                name.clone(),
                DecodedParam {
                    value,
                    indexed: false,
                },
            );
        }

        Ok(DecodedLog {
            name: self.name.clone(),
            params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, Bytes};
    use alloy_primitives::{Address, U256};

    #[test]
    fn test_transfer_event_decode() {
        // Parse the Transfer event signature
        let sig = "Transfer(address indexed from,address indexed to,uint256 value)";
        let event_def = parse_event_signature(sig).unwrap();

        let from_addr = Address::from_slice(&hex!("1234567890123456789012345678901234567890"));
        let to_addr = Address::from_slice(&hex!("9876543210987654321098765432109876543210"));
        let value = U256::from(1000000000000000000u128); // 1 ETH

        let topics = vec![
            event_def.event.topic_0().unwrap(),
            from_addr.into_word(),
            to_addr.into_word(),
        ];

        let data = Bytes::from(value.to_be_bytes_vec());

        let log = LogData::new_unchecked(topics, data);

        // Decode the log
        let decoded_log = event_def.decode_log(&log).unwrap();

        // Verify the decoding
        assert_eq!(decoded_log.name, "Transfer");
        assert_eq!(decoded_log.params.len(), 3);

        // Verify from address - now using HashMap lookup
        let from_param = decoded_log.params.get("from").unwrap();
        assert!(from_param.indexed);
        assert!(matches!(from_param.value, DynSolValue::Address(addr) if addr == from_addr));

        // Verify to address
        let to_param = decoded_log.params.get("to").unwrap();
        assert!(to_param.indexed);
        assert!(matches!(to_param.value, DynSolValue::Address(addr) if addr == to_addr));

        // Verify value
        let value_param = decoded_log.params.get("value").unwrap();
        assert!(!value_param.indexed);
        assert!(matches!(value_param.value, DynSolValue::Uint(val, 256) if val == value));
    }
}
