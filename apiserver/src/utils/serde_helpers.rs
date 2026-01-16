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
