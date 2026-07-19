use crate::node_execution_contract::{TaskContract, TaskInputDescriptor, TaskOutputDescriptor};
use crate::RuntimeError;
use bijux_dag_core::Graph;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScalarType {
    String,
    Integer,
    Float,
    Boolean,
    Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollectionType {
    List,
    Map,
    Set,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionedTypeRule {
    pub type_id: String,
    pub version: String,
    pub serialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaReference {
    pub schema_name: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeCoercionRule {
    pub from_type: String,
    pub to_type: String,
    pub compatibility: String,
    pub explicit_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NullabilityContract {
    pub nullable: bool,
    pub optional: bool,
    pub cardinality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretReference {
    pub secret_id: String,
    pub mount_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceReference {
    pub resource_id: String,
    pub resource_kind: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterCapabilityDeclaration {
    pub adapter_id: String,
    pub adapter_version: String,
    pub supports_types: Vec<String>,
    pub supports_replay_compatibility_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputEvolutionMarker {
    pub schema_reference: SchemaReference,
    pub backward_compatible: bool,
    pub forward_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskContractDiagnostic {
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskContractFingerprint {
    pub node_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartitionCollectionContract {
    pub partition_key: String,
    pub item_type: String,
    pub deterministic_partition_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolymorphicVariant {
    pub variant_id: String,
    pub input_type: String,
    pub output_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolymorphicTaskContract {
    pub node_id: String,
    pub variants: Vec<PolymorphicVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskTypeRegistry {
    pub scalar_types: Vec<ScalarType>,
    pub collection_types: Vec<CollectionType>,
    pub versioned_rules: Vec<VersionedTypeRule>,
    pub coercion_rules: Vec<TypeCoercionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompatibilityScore {
    pub node_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCompatibilityMatrixReport {
    pub relationships: Vec<TaskCompatibilityRelationship>,
    pub scores: Vec<CompatibilityScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskCompatibilityRelationship {
    pub producer_node_id: String,
    pub consumer_node_id: String,
    pub compatible: bool,
    pub reason: String,
}

pub fn default_task_type_registry() -> TaskTypeRegistry {
    TaskTypeRegistry {
        scalar_types: vec![
            ScalarType::String,
            ScalarType::Integer,
            ScalarType::Float,
            ScalarType::Boolean,
            ScalarType::Bytes,
        ],
        collection_types: vec![CollectionType::List, CollectionType::Map, CollectionType::Set],
        versioned_rules: vec![
            VersionedTypeRule {
                type_id: "string".to_string(),
                version: "v1".to_string(),
                serialization: "utf8".to_string(),
            },
            VersionedTypeRule {
                type_id: "integer".to_string(),
                version: "v1".to_string(),
                serialization: "i64".to_string(),
            },
        ],
        coercion_rules: vec![
            TypeCoercionRule {
                from_type: "integer".to_string(),
                to_type: "float".to_string(),
                compatibility: "lossless_range".to_string(),
                explicit_only: true,
            },
            TypeCoercionRule {
                from_type: "string".to_string(),
                to_type: "bytes".to_string(),
                compatibility: "utf8_only".to_string(),
                explicit_only: true,
            },
        ],
    }
}

pub fn validate_parameter_defaults(
    contract: &TaskContract,
    defaults: &BTreeMap<String, serde_json::Value>,
) -> Vec<TaskContractDiagnostic> {
    let mut diagnostics = Vec::new();
    for input in &contract.inputs {
        if input.required && !defaults.contains_key(&input.name) {
            diagnostics.push(TaskContractDiagnostic {
                code: "TC1001".to_string(),
                message: format!("missing default for required input '{}'", input.name),
                path: format!("/contracts/{}/inputs/{}", contract.node_id, input.name),
            });
        }
    }
    diagnostics
}

pub fn validate_cross_node_compatibility(
    producer: &TaskContract,
    consumer: &TaskContract,
) -> Vec<TaskContractDiagnostic> {
    let producer_outputs: BTreeMap<String, String> =
        producer.outputs.iter().map(|o| (o.name.clone(), o.schema_name.clone())).collect();
    let mut diagnostics = Vec::new();
    for input in &consumer.inputs {
        let compatible = producer_outputs
            .iter()
            .any(|(_, schema)| schema == &input.value_type || input.value_type == "artifact_ref");
        if !compatible {
            diagnostics.push(TaskContractDiagnostic {
                code: "TC2001".to_string(),
                message: format!(
                    "input '{}' in consumer '{}' is not satisfied by producer '{}'",
                    input.name, consumer.node_id, producer.node_id
                ),
                path: format!(
                    "/compatibility/{}/to/{}/{}",
                    producer.node_id, consumer.node_id, input.name
                ),
            });
        }
    }
    diagnostics
}

pub fn compute_task_contract_fingerprint(
    contract: &TaskContract,
) -> Result<TaskContractFingerprint, RuntimeError> {
    let payload = serde_json::to_vec(contract)?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(TaskContractFingerprint {
        node_id: contract.node_id.clone(),
        fingerprint: hex::encode(hasher.finalize()),
    })
}

pub fn check_replay_adapter_compatibility(
    declared: &AdapterCapabilityDeclaration,
    replay_adapter_version: &str,
) -> bool {
    declared.supports_replay_compatibility_check
        && declared.adapter_version == replay_adapter_version
}

pub fn compatibility_score_for_contract(
    contract: &TaskContract,
    diagnostics: &[TaskContractDiagnostic],
) -> CompatibilityScore {
    let penalty = diagnostics.len() as f64 * 0.15;
    let base = 1.0;
    CompatibilityScore { node_id: contract.node_id.clone(), score: (base - penalty).max(0.0) }
}

pub fn generate_task_contract_markdown(contract: &TaskContract) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Task contract {}", contract.node_id));
    lines.push(String::new());
    lines.push("## Inputs".to_string());
    for TaskInputDescriptor { name, value_type, required } in &contract.inputs {
        lines.push(format!("- {}: {} (required: {})", name, value_type, required));
    }
    lines.push(String::new());
    lines.push("## Outputs".to_string());
    for TaskOutputDescriptor { name, path, schema_name, schema_version } in &contract.outputs {
        lines.push(format!("- {}: {} (schema: {} {})", name, path, schema_name, schema_version));
    }
    lines.join("\n")
}

pub fn compatibility_matrix_report(
    graph: &Graph,
    contracts: &BTreeMap<String, TaskContract>,
) -> TaskCompatibilityMatrixReport {
    let mut relationships = Vec::new();
    let mut scores = Vec::new();
    for edge in &graph.edges {
        if let (Some(producer), Some(consumer)) =
            (contracts.get(&edge.from.node_id), contracts.get(&edge.to.node_id))
        {
            let diagnostics = validate_cross_node_compatibility(producer, consumer);
            relationships.push(TaskCompatibilityRelationship {
                producer_node_id: producer.node_id.clone(),
                consumer_node_id: consumer.node_id.clone(),
                compatible: diagnostics.is_empty(),
                reason: if diagnostics.is_empty() {
                    "producer output contract satisfies consumer input contract".to_string()
                } else {
                    diagnostics[0].message.clone()
                },
            });
            scores.push(compatibility_score_for_contract(consumer, &diagnostics));
        }
    }
    TaskCompatibilityMatrixReport { relationships, scores }
}
