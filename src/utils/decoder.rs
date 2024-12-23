use alloy_dyn_abi::{DynSolCall, DynSolEvent, DynSolReturns, DynSolType, DynSolValue};
use alloy_primitives::{keccak256, LogData, B256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DecodedLog {
    pub(crate) name: String,
    pub(crate) params: HashMap<String, DecodedParam>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DecodedParam {
    pub value: DynSolValue,
    pub indexed: bool,
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidFormat,
    InvalidParameter,
    UnableToDecode,
    UnsupportedType,
}

#[derive(Debug, Clone)]
pub struct ParsedParameter {
    pub name: String,
    pub type_info: DynSolType,
    pub is_indexed: bool,
}

pub(crate) struct EventDefinition {
    pub name: String,
    pub event: DynSolEvent,
    pub definitions: Vec<(String, String, bool)>, // (name, type, indexed)
}

/// Parses a Solidity signature into its name and parameters; does not support nested signatures.
/// Why not ? nested signatures are quite complicated and prove to be less useful because:
/// 1. parameter names are not known at runtime without abi
/// 2. nested structures are just positional , making it painfuly hard to come up with the best filter approach
///    without high cognitive load just to write filters for filtering contract data.
///
/// # Parameters
/// - `sig`: The Solidity signature as a string (e.g., `transfer(address indexed to, uint256 value)`).
pub(crate) fn parse_signature(sig: &str) -> Result<(String, Vec<ParsedParameter>), DecodeError> {
    let (name, params) = match sig.split_once('(') {
        Some((n, p)) => (n.trim(), p.trim_end_matches(')')),
        None => return Err(DecodeError::InvalidFormat),
    };
    let mut parameters = Vec::new();

    for param in params.split(',').filter(|s| !s.is_empty()) {
        let parts = param.split_whitespace().collect::<Vec<&str>>();

        let (ty, is_indexed, name) = match parts.len() {
            2 => (parts[0], false, parts[1]),
            3 if parts[1] == "indexed" => (parts[0], true, parts[2]),
            _ => return Err(DecodeError::InvalidParameter),
        };

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

        parameters.push(ParsedParameter {
            name: name.to_string(),
            type_info: sol_type,
            is_indexed,
        });
    }

    Ok((name.to_string(), parameters))
}

impl DecodedLog {
    pub(crate) fn get_parameter(&self, name: &str) -> Option<&DynSolValue> {
        if let Some(param) = self.params.get(name) {
            return Some(&param.value);
        }
        None
    }
}

impl EventDefinition {
    pub fn from_signature(sig: &str) -> Result<Self, DecodeError> {
        let (name, params) = parse_signature(sig)?;

        let mut indexed = Vec::new();
        let mut body = Vec::new();
        let mut definitions = Vec::new();

        for param in params {
            definitions.push((
                param.name,
                format!("{:?}", param.type_info),
                param.is_indexed,
            ));

            if param.is_indexed {
                indexed.push(param.type_info);
            } else {
                body.push(param.type_info);
            }
        }

        let keccak2_fixed_bytes = keccak256(sig.as_bytes());
        let src = keccak2_fixed_bytes.as_slice();
        let sig_hash = B256::from_slice(src);

        Ok(EventDefinition {
            name,
            event: DynSolEvent::new_unchecked(Some(sig_hash), indexed, DynSolType::Tuple(body)),
            definitions,
        })
    }
    pub(crate) fn decode_log(&self, log: &LogData) -> Result<DecodedLog, DecodeError> {
        let decoded = self
            .event
            .decode_log_data(log, true)
            .map_err(|_| DecodeError::UnableToDecode)?;

        // Parameters are stored in a HashMap for O(1) lookups by name, improving performance when
        // accessing parameters like "from", "to", or "value". This eliminates the need for linear
        // scans through a vec of [`DynSolValue`], making the code more efficient when handling larger event logs.
        let mut params = HashMap::new();
        let indexed = decoded
            .indexed
            .into_iter()
            .zip(self.definitions.iter().filter(|(_, _, indexed)| *indexed));

        for (value, (name, _, _)) in indexed {
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
            .zip(self.definitions.iter().filter(|(_, _, indexed)| !*indexed));

        for (value, (name, _, _)) in decoded_data {
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DecodedCall {
    pub(crate) name: String,
    pub(crate) parameters: Vec<DynSolValue>,
}

#[allow(dead_code)]
pub(crate) struct CallDefinition {
    pub name: String,
    pub call: DynSolCall,
}

#[allow(dead_code)]
impl CallDefinition {
    pub fn from_signature(sig: &str) -> Result<Self, DecodeError> {
        let (name, params) = parse_signature(sig)?;

        let mut parameters = Vec::with_capacity(params.len());
        for param in params {
            parameters.push(param.type_info);
        }

        // Create the 4-byte selector
        let selector = keccak256(sig.as_bytes())[..4]
            .try_into()
            .map_err(|_| DecodeError::UnableToDecode)?;

        // TODO: handle returns later...
        let returns = DynSolReturns::new(Vec::new());

        Ok(CallDefinition {
            name: name.clone(),
            call: DynSolCall::new(selector, parameters, Some(name), returns),
        })
    }

    pub(crate) fn decode_call_data(&self, data: &[u8]) -> Result<DecodedCall, DecodeError> {
        let decoded: Vec<DynSolValue> = self
            .call
            .abi_decode_input(data, true)
            .map_err(|_| DecodeError::UnableToDecode)?;

        Ok(DecodedCall {
            name: self.name.clone(),
            parameters: decoded,
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
        let event_def = EventDefinition::from_signature(sig).unwrap();

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
