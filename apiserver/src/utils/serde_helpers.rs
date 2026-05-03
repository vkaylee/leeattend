use serde::{Deserialize, Deserializer};

/// Deserializer to distinguish between missing, null, and provided values.
/// - Missing: `Option<Option<T>>` is `None` (requires `#[serde(default)]`)
/// - Null: `Option<Option<T>>` is `Some(None)`
/// - Value: `Option<Option<T>>` is `Some(Some(v))`
pub fn deserialize_option_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[cfg(test)]
#[allow(clippy::option_option)]
mod tests {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct TestStruct {
        #[serde(default, deserialize_with = "super::deserialize_option_option")]
        value: Option<Option<String>>,
    }

    #[test]
    fn test_deserialize_present_value() {
        let json = r#"{"value": "hello"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, Some(Some("hello".to_string())));
    }

    #[test]
    fn test_deserialize_null_value() {
        let json = r#"{"value": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, Some(None));
    }

    #[test]
    fn test_deserialize_missing_field() {
        let json = r#"{}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, None);
    }

    #[test]
    fn test_all_three_states_differ() {
        let missing: TestStruct = serde_json::from_str(r#"{}"#).unwrap();
        let null: TestStruct = serde_json::from_str(r#"{"value": null}"#).unwrap();
        let present: TestStruct = serde_json::from_str(r#"{"value": "x"}"#).unwrap();

        assert!(missing.value.is_none());
        assert_eq!(null.value, Some(None));
        assert!(present.value.unwrap().is_some());
    }

    #[derive(Deserialize, Debug)]
    struct TestNumeric {
        #[serde(default, deserialize_with = "super::deserialize_option_option")]
        count: Option<Option<i32>>,
    }

    #[test]
    fn test_deserialize_numeric_value() {
        let json = r#"{"count": 42}"#;
        let result: TestNumeric = serde_json::from_str(json).unwrap();
        assert_eq!(result.count, Some(Some(42)));
    }

    #[test]
    fn test_deserialize_numeric_null() {
        let json = r#"{"count": null}"#;
        let result: TestNumeric = serde_json::from_str(json).unwrap();
        assert_eq!(result.count, Some(None));
    }

    #[test]
    fn test_deserialize_numeric_missing() {
        let json = r#"{}"#;
        let result: TestNumeric = serde_json::from_str(json).unwrap();
        assert_eq!(result.count, None);
    }
}
