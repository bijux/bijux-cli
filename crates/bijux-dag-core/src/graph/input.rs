use serde::de::Error as DeError;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct GraphInputSpec {
    pub kind: GraphInputKind,
    pub required: bool,
    pub default: Option<Value>,
}

impl GraphInputSpec {
    pub fn from_default_value(default: Value) -> Result<Self, String> {
        if default.is_null() {
            return Err(
                "graph input shorthand must provide a non-null default or an explicit schema"
                    .to_string(),
            );
        }
        Ok(Self {
            kind: GraphInputKind::infer_shorthand(&default),
            required: false,
            default: Some(default),
        })
    }

    pub fn effective_value(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    pub fn with_effective_value(&self, value: Value) -> Self {
        Self { kind: self.kind.clone(), required: self.required, default: Some(value) }
    }

    pub fn schema_json(&self) -> Value {
        let mut payload = serde_json::Map::new();
        payload.insert("type".to_string(), Value::String(self.kind.kind_name().to_string()));
        self.kind.append_schema_fields(&mut payload);
        if self.required {
            payload.insert("required".to_string(), Value::Bool(true));
        }
        if let Some(default) = &self.default {
            payload.insert("default".to_string(), default.clone());
        }
        Value::Object(payload)
    }

    fn serialize_as_shorthand(&self) -> bool {
        !self.required
            && self.default.is_some()
            && self.kind.matches_shorthand_value(self.default.as_ref().expect("checked above"))
    }
}

impl Serialize for GraphInputSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.serialize_as_shorthand() {
            return self.default.as_ref().expect("checked above").serialize(serializer);
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", self.kind.kind_name())?;
        self.kind.serialize_schema_fields(&mut map)?;
        if self.required {
            map.serialize_entry("required", &self.required)?;
        }
        if let Some(default) = &self.default {
            map.serialize_entry("default", default)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for GraphInputSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        graph_input_spec_from_value(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphInputKind {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Enum { values: Vec<String> },
    Array { items: Option<Box<GraphInputKind>> },
    Object { properties: Option<BTreeMap<String, GraphInputSpec>> },
}

impl GraphInputKind {
    fn infer_shorthand(value: &Value) -> Self {
        match value {
            Value::String(_) => Self::String,
            Value::Bool(_) => Self::Boolean,
            Value::Number(number) if number.is_i64() || number.is_u64() => Self::Integer,
            Value::Number(_) => Self::Float,
            Value::Array(items) => {
                let inferred = infer_common_array_item_kind(items);
                Self::Array { items: inferred.map(Box::new) }
            }
            Value::Object(_) => Self::Object { properties: None },
            Value::Null => Self::String,
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Path => "path",
            Self::Enum { .. } => "enum",
            Self::Array { .. } => "array",
            Self::Object { .. } => "object",
        }
    }

    fn append_schema_fields(&self, payload: &mut serde_json::Map<String, Value>) {
        match self {
            Self::Enum { values } => {
                payload.insert(
                    "values".to_string(),
                    Value::Array(values.iter().cloned().map(Value::String).collect()),
                );
            }
            Self::Array { items } => {
                if let Some(items) = items.as_deref() {
                    payload.insert(
                        "items".to_string(),
                        GraphInputSpec { kind: items.clone(), required: false, default: None }
                            .schema_json(),
                    );
                }
            }
            Self::Object { properties } => {
                if let Some(properties) = properties {
                    let mut property_map = serde_json::Map::new();
                    for (property_name, property_spec) in properties {
                        property_map.insert(property_name.clone(), property_spec.schema_json());
                    }
                    payload.insert("properties".to_string(), Value::Object(property_map));
                }
            }
            Self::String | Self::Integer | Self::Float | Self::Boolean | Self::Path => {}
        }
    }

    fn matches_shorthand_value(&self, value: &Value) -> bool {
        match self {
            Self::String => matches!(value, Value::String(_)),
            Self::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
            Self::Float => value.is_number(),
            Self::Boolean => matches!(value, Value::Bool(_)),
            Self::Path => false,
            Self::Enum { .. } => false,
            Self::Array { items } => match value {
                Value::Array(entries) => {
                    if let Some(item_kind) = items.as_deref() {
                        entries.iter().all(|entry| item_kind.accepts_value(entry))
                    } else {
                        true
                    }
                }
                _ => false,
            },
            Self::Object { properties } => matches!((value, properties), (Value::Object(_), None)),
        }
    }

    fn accepts_value(&self, value: &Value) -> bool {
        match self {
            Self::String | Self::Path => matches!(value, Value::String(_)),
            Self::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
            Self::Float => value.is_number(),
            Self::Boolean => matches!(value, Value::Bool(_)),
            Self::Enum { values } => value
                .as_str()
                .is_some_and(|candidate| values.iter().any(|allowed| allowed == candidate)),
            Self::Array { items } => match value {
                Value::Array(entries) => items.as_deref().is_none_or(|item_kind| {
                    entries.iter().all(|entry| item_kind.accepts_value(entry))
                }),
                _ => false,
            },
            Self::Object { properties } => match value {
                Value::Object(map) => properties.as_ref().is_none_or(|declared| {
                    map.iter().all(|(key, candidate)| {
                        declared
                            .get(key)
                            .is_some_and(|property| property.kind.accepts_value(candidate))
                    })
                }),
                _ => false,
            },
        }
    }

    fn serialize_schema_fields<S>(&self, map: &mut S) -> Result<(), S::Error>
    where
        S: SerializeMap,
    {
        match self {
            Self::Enum { values } => map.serialize_entry("values", values),
            Self::Array { items } => {
                if let Some(items) = items {
                    map.serialize_entry("items", items)?;
                }
                Ok(())
            }
            Self::Object { properties } => {
                if let Some(properties) = properties {
                    map.serialize_entry("properties", properties)?;
                }
                Ok(())
            }
            Self::String | Self::Integer | Self::Float | Self::Boolean | Self::Path => Ok(()),
        }
    }
}

impl Serialize for GraphInputKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.kind_name())
    }
}

impl<'de> Deserialize<'de> for GraphInputKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "string" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            "path" => Ok(Self::Path),
            "enum" => Ok(Self::Enum { values: Vec::new() }),
            "array" => Ok(Self::Array { items: None }),
            "object" => Ok(Self::Object { properties: None }),
            _ => Err(D::Error::custom(format!("unsupported graph input type: {value}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphInputViolation {
    pub path: String,
    pub message: String,
}

pub fn validate_graph_input_value(
    spec: &GraphInputSpec,
    value: &Value,
    path: &str,
) -> Result<(), GraphInputViolation> {
    materialize_graph_input_value(spec, value, path).map(|_| ())
}

pub fn materialize_graph_input_value(
    spec: &GraphInputSpec,
    value: &Value,
    path: &str,
) -> Result<Value, GraphInputViolation> {
    materialize_value_against_kind(&spec.kind, value, path)
}

fn materialize_value_against_kind(
    kind: &GraphInputKind,
    value: &Value,
    path: &str,
) -> Result<Value, GraphInputViolation> {
    match kind {
        GraphInputKind::String => {
            expect_value_type(value, path, "string", Value::is_string)?;
            Ok(value.clone())
        }
        GraphInputKind::Integer => expect_value_type(value, path, "integer", |candidate| {
            candidate.as_i64().is_some() || candidate.as_u64().is_some()
        })
        .map(|_| value.clone()),
        GraphInputKind::Float => {
            expect_value_type(value, path, "float", Value::is_number)?;
            Ok(value.clone())
        }
        GraphInputKind::Boolean => {
            expect_value_type(value, path, "boolean", Value::is_boolean)?;
            Ok(value.clone())
        }
        GraphInputKind::Path => {
            expect_value_type(value, path, "path string", Value::is_string)?;
            Ok(value.clone())
        }
        GraphInputKind::Enum { values } => {
            let Some(candidate) = value.as_str() else {
                return Err(GraphInputViolation {
                    path: path.to_string(),
                    message: "expected enum string".to_string(),
                });
            };
            if values.iter().any(|allowed| allowed == candidate) {
                Ok(value.clone())
            } else {
                Err(GraphInputViolation {
                    path: path.to_string(),
                    message: format!(
                        "expected one of [{}], got {:?}",
                        values.join(", "),
                        candidate
                    ),
                })
            }
        }
        GraphInputKind::Array { items } => {
            let Value::Array(entries) = value else {
                return Err(GraphInputViolation {
                    path: path.to_string(),
                    message: "expected array".to_string(),
                });
            };
            let mut normalized = Vec::with_capacity(entries.len());
            if let Some(item_kind) = items {
                for (index, entry) in entries.iter().enumerate() {
                    normalized.push(materialize_value_against_kind(
                        item_kind,
                        entry,
                        &format!("{path}/{index}"),
                    )?);
                }
            } else {
                normalized.extend(entries.iter().cloned());
            }
            Ok(Value::Array(normalized))
        }
        GraphInputKind::Object { properties } => {
            let Value::Object(entries) = value else {
                return Err(GraphInputViolation {
                    path: path.to_string(),
                    message: "expected object".to_string(),
                });
            };
            if let Some(properties) = properties {
                let mut normalized = serde_json::Map::new();
                for (key, property_spec) in properties {
                    if entries.contains_key(key)
                        || !property_spec.required
                        || property_spec.default.is_some()
                    {
                        continue;
                    }
                    return Err(GraphInputViolation {
                        path: format!("{path}/{key}"),
                        message: "missing required object property".to_string(),
                    });
                }
                for (key, property_spec) in properties {
                    let property_path = format!("{path}/{key}");
                    match entries.get(key) {
                        Some(candidate) => {
                            normalized.insert(
                                key.clone(),
                                materialize_value_against_kind(
                                    &property_spec.kind,
                                    candidate,
                                    &property_path,
                                )?,
                            );
                        }
                        None => {
                            if let Some(default) = &property_spec.default {
                                normalized.insert(
                                    key.clone(),
                                    materialize_graph_input_value(
                                        property_spec,
                                        default,
                                        &format!("{property_path}/default"),
                                    )?,
                                );
                            }
                        }
                    }
                }
                for key in entries.keys() {
                    if !properties.contains_key(key) {
                        return Err(GraphInputViolation {
                            path: format!("{path}/{key}"),
                            message: "undeclared object property".to_string(),
                        });
                    }
                }
                Ok(Value::Object(normalized))
            } else {
                Ok(value.clone())
            }
        }
    }
}

fn expect_value_type<F>(
    value: &Value,
    path: &str,
    expected: &str,
    predicate: F,
) -> Result<(), GraphInputViolation>
where
    F: Fn(&Value) -> bool,
{
    if predicate(value) {
        return Ok(());
    }
    Err(GraphInputViolation { path: path.to_string(), message: format!("expected {expected}") })
}

fn graph_input_spec_from_value(value: Value) -> Result<GraphInputSpec, String> {
    match value {
        Value::Object(mut map) if map.contains_key("type") => {
            parse_explicit_graph_input_spec(&mut map)
        }
        other => GraphInputSpec::from_default_value(other),
    }
}

fn parse_explicit_graph_input_spec(
    map: &mut serde_json::Map<String, Value>,
) -> Result<GraphInputSpec, String> {
    let type_name = take_required_string(map, "type")?;
    let required = take_optional_bool(map, "required")?.unwrap_or(false);
    let default = map.remove("default");
    let kind = match type_name.as_str() {
        "string" => {
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::String
        }
        "integer" => {
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Integer
        }
        "float" => {
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Float
        }
        "boolean" => {
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Boolean
        }
        "path" => {
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Path
        }
        "enum" => {
            let values = take_required_string_array(map, "values")?;
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Enum { values }
        }
        "array" => {
            let items = map.remove("items").ok_or_else(|| {
                "graph input type \"array\" requires an \"items\" schema".to_string()
            })?;
            ensure_no_extra_keys(map, &[])?;
            let item_kind = parse_graph_input_kind(items)?;
            GraphInputKind::Array { items: Some(Box::new(item_kind)) }
        }
        "object" => {
            let properties =
                map.remove("properties").map(parse_graph_input_properties).transpose()?;
            ensure_no_extra_keys(map, &[])?;
            GraphInputKind::Object { properties }
        }
        _ => return Err(format!("unsupported graph input type: {type_name}")),
    };
    Ok(GraphInputSpec { kind, required, default })
}

fn parse_graph_input_kind(value: Value) -> Result<GraphInputKind, String> {
    match graph_input_spec_from_value(value)? {
        GraphInputSpec { kind, required: false, default: None } => Ok(kind),
        GraphInputSpec { .. } => {
            Err("array item schemas must declare only the item type surface".to_string())
        }
    }
}

fn parse_graph_input_properties(value: Value) -> Result<BTreeMap<String, GraphInputSpec>, String> {
    let Value::Object(entries) = value else {
        return Err("graph input object properties must be a JSON object".to_string());
    };
    let mut properties = BTreeMap::new();
    for (key, property_value) in entries {
        properties.insert(key, graph_input_spec_from_value(property_value)?);
    }
    Ok(properties)
}

fn infer_common_array_item_kind(items: &[Value]) -> Option<GraphInputKind> {
    let mut inferred =
        items.iter().filter(|value| !value.is_null()).map(GraphInputKind::infer_shorthand);
    let first = inferred.next()?;
    if inferred.all(|candidate| candidate == first) {
        Some(first)
    } else {
        None
    }
}

fn take_required_string(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, String> {
    let value = map.remove(key).ok_or_else(|| format!("missing required field: {key}"))?;
    value.as_str().map(str::to_string).ok_or_else(|| format!("field {key} must be a string"))
}

fn take_optional_bool(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    map.remove(key)
        .map(|value| value.as_bool().ok_or_else(|| format!("field {key} must be a boolean")))
        .transpose()
}

fn take_required_string_array(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>, String> {
    let value = map.remove(key).ok_or_else(|| format!("missing required field: {key}"))?;
    let Value::Array(items) = value else {
        return Err(format!("field {key} must be an array"));
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let Some(item) = item.as_str() else {
            return Err(format!("field {key} must contain only strings"));
        };
        values.push(item.to_string());
    }
    Ok(values)
}

fn ensure_no_extra_keys(
    map: &serde_json::Map<String, Value>,
    allowed: &[&str],
) -> Result<(), String> {
    let extras = map
        .keys()
        .filter(|key| !allowed.iter().any(|allowed_key| allowed_key == &key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if extras.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown graph input schema fields: {}", extras.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_graph_input_value, GraphInputKind, GraphInputSpec};
    use serde_json::json;

    #[test]
    fn scalar_defaults_roundtrip_as_shorthand() {
        let spec = GraphInputSpec::from_default_value(json!("eu-west-1")).expect("spec");
        let encoded = serde_json::to_value(&spec).expect("json");
        assert_eq!(encoded, json!("eu-west-1"));
    }

    #[test]
    fn explicit_object_schema_validates_nested_paths() {
        let spec: GraphInputSpec = serde_json::from_value(json!({
            "type":"object",
            "properties":{
                "tenant":{"type":"string","required":true},
                "attempts":{"type":"integer"}
            }
        }))
        .expect("spec");

        let error = validate_graph_input_value(&spec, &json!({"attempts":"x"}), "/inputs/payload")
            .expect_err("invalid payload");
        assert_eq!(error.path, "/inputs/payload/tenant");
        assert_eq!(error.message, "missing required object property");
    }

    #[test]
    fn array_item_schema_requires_bare_type_surface() {
        let error = serde_json::from_value::<GraphInputSpec>(json!({
            "type":"array",
            "items":{"type":"string","default":"x"}
        }))
        .expect_err("invalid schema");
        assert!(error.to_string().contains("array item schemas"));
    }

    #[test]
    fn shorthand_null_is_rejected() {
        let error = serde_json::from_value::<GraphInputSpec>(json!(null)).expect_err("null");
        assert!(error.to_string().contains("non-null default"));
    }

    #[test]
    fn homogeneous_array_defaults_infer_item_schema() {
        let spec = GraphInputSpec::from_default_value(json!([1, 2, 3])).expect("spec");
        assert_eq!(
            spec.kind,
            GraphInputKind::Array { items: Some(Box::new(GraphInputKind::Integer)) }
        );
    }
}
