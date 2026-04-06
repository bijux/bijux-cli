#![forbid(unsafe_code)]
//! JSON serialization adapter primitives.

/// Serialize a value to JSON string.
pub fn encode<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserialize a value from JSON string.
pub fn decode<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(text)
}
