use serde_json::Value;

pub(crate) fn resolve_path<'a>(path: &str, value: &'a Value) -> Option<&'a Value> {
    // 1. Split the path by dots to get each level
    let parts = path.split('.').collect::<Vec<&str>>();
    let mut current = value;

    // 2. Traverse through each part of the path
    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_path() {
        let data = json!({
            "transactions": {
                "from": "0x123",
                "details": {
                    "gas": 5000
                }
            },
            "logs": ["log1", "log2"]
        });

        assert_eq!(
            resolve_path("transactions.from", &data),
            Some(&json!("0x123"))
        );

        assert_eq!(
            resolve_path("transactions.details.gas", &data),
            Some(&json!(5000))
        );

        assert_eq!(resolve_path("logs", &data), Some(&json!(["log1", "log2"])));
        assert_eq!(resolve_path("transactions.nothing_to_see_here", &data), None);
    }
}
