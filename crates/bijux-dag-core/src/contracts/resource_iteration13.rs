use crate::Graph;
use serde::{Deserialize, Serialize};

/// Runtime resource requirement envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRequirementV1 {
    pub node_id: String,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub scratch_mb: u32,
    pub network_required: bool,
    pub walltime_ms: u64,
    pub accelerator: Option<String>,
}

/// Runtime resource availability envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAvailabilityV1 {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub scratch_mb: u32,
    pub network_available: bool,
    pub walltime_ms: u64,
    pub accelerators: Vec<String>,
}

/// Resource preflight refusal with deterministic code and message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePreflightRefusalV1 {
    pub node_id: String,
    pub code: String,
    pub message: String,
}

/// Resource preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePreflightReportV1 {
    pub requirements: Vec<ResourceRequirementV1>,
    pub refusals: Vec<ResourcePreflightRefusalV1>,
    pub admitted: bool,
}

/// Build resource requirements from graph nodes with deterministic defaults.
pub fn build_resource_requirements(graph: &Graph) -> Vec<ResourceRequirementV1> {
    graph
        .nodes
        .iter()
        .map(|node| {
            let resources = node.resources.as_ref();
            let mut disk_mb = 512;
            let mut scratch_mb = 1024;
            let mut walltime_ms = node.timeout_ms.unwrap_or(3_600_000);
            let mut network_required = false;
            let mut accelerator = None;
            for tag in &node.tags {
                if let Some(value) = tag.strip_prefix("disk_mb:") {
                    if let Ok(parsed) = value.parse::<u32>() {
                        disk_mb = parsed;
                    }
                } else if let Some(value) = tag.strip_prefix("scratch_mb:") {
                    if let Ok(parsed) = value.parse::<u32>() {
                        scratch_mb = parsed;
                    }
                } else if let Some(value) = tag.strip_prefix("walltime_ms:") {
                    if let Ok(parsed) = value.parse::<u64>() {
                        walltime_ms = parsed;
                    }
                } else if let Some(value) = tag.strip_prefix("accelerator:") {
                    if !value.trim().is_empty() {
                        accelerator = Some(value.trim().to_string());
                    }
                } else if tag == "network" {
                    network_required = true;
                }
            }
            ResourceRequirementV1 {
                node_id: node.id.clone(),
                cpu_cores: resources.map_or(1, |r| r.cpu),
                memory_mb: resources.map_or(256, |r| r.mem_mb),
                disk_mb,
                scratch_mb,
                network_required,
                walltime_ms,
                accelerator,
            }
        })
        .collect()
}

/// Validate resource requirements against runtime availability.
pub fn validate_resource_requirements(
    requirements: Vec<ResourceRequirementV1>,
    availability: &ResourceAvailabilityV1,
) -> ResourcePreflightReportV1 {
    let mut refusals = Vec::new();
    for requirement in &requirements {
        if requirement.cpu_cores == 0 || requirement.memory_mb == 0 {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_INVALID_RESOURCE_SHAPE".to_string(),
                message: "cpu_cores and memory_mb must be positive".to_string(),
            });
            continue;
        }
        if requirement.cpu_cores > availability.cpu_cores {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_CPU_UNAVAILABLE".to_string(),
                message: format!(
                    "requested cpu_cores {} exceeds available {}",
                    requirement.cpu_cores, availability.cpu_cores
                ),
            });
        }
        if requirement.memory_mb > availability.memory_mb {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_MEMORY_UNAVAILABLE".to_string(),
                message: format!(
                    "requested memory_mb {} exceeds available {}",
                    requirement.memory_mb, availability.memory_mb
                ),
            });
        }
        if requirement.disk_mb > availability.disk_mb {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_DISK_UNAVAILABLE".to_string(),
                message: format!(
                    "requested disk_mb {} exceeds available {}",
                    requirement.disk_mb, availability.disk_mb
                ),
            });
        }
        if requirement.scratch_mb > availability.scratch_mb {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_SCRATCH_UNAVAILABLE".to_string(),
                message: format!(
                    "requested scratch_mb {} exceeds available {}",
                    requirement.scratch_mb, availability.scratch_mb
                ),
            });
        }
        if requirement.network_required && !availability.network_available {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_NETWORK_UNAVAILABLE".to_string(),
                message: "node requires network but runtime has network disabled".to_string(),
            });
        }
        if requirement.walltime_ms > availability.walltime_ms {
            refusals.push(ResourcePreflightRefusalV1 {
                node_id: requirement.node_id.clone(),
                code: "R121_WALLTIME_UNAVAILABLE".to_string(),
                message: format!(
                    "requested walltime_ms {} exceeds available {}",
                    requirement.walltime_ms, availability.walltime_ms
                ),
            });
        }
        if let Some(accelerator) = &requirement.accelerator {
            if !availability.accelerators.iter().any(|value| value == accelerator) {
                refusals.push(ResourcePreflightRefusalV1 {
                    node_id: requirement.node_id.clone(),
                    code: "R121_ACCELERATOR_UNAVAILABLE".to_string(),
                    message: format!("requested accelerator '{}' is unavailable", accelerator),
                });
            }
        }
    }
    ResourcePreflightReportV1 { requirements, admitted: refusals.is_empty(), refusals }
}

#[cfg(test)]
mod tests {
    use super::{
        build_resource_requirements, validate_resource_requirements, ResourceAvailabilityV1,
    };
    use crate::{
        Edge, FileOutput, Graph, GraphMeta, Node, NodeKind, ParamValue, PortRef, Resources,
        RetryPolicy, SemanticNodeKind, TriggerRule,
    };

    fn sample_graph() -> Graph {
        Graph {
            spec: "bijux-dag/v0.1".to_string(),
            meta: Some(GraphMeta {
                name: "resource-preflight".to_string(),
                description: None,
                owners: Vec::new(),
                tags: Vec::new(),
            }),
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "align".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec!["reads".to_string()],
                outputs: vec![FileOutput {
                    name: "bam".to_string(),
                    path: "align.bam".to_string(),
                }],
                params: ParamValue::default(),
                container: None,
                timeout_ms: Some(6_000),
                resources: Some(Resources { cpu: 4, mem_mb: 4096 }),
                tags: vec![
                    "disk_mb:1024".to_string(),
                    "scratch_mb:2048".to_string(),
                    "accelerator:gpu".to_string(),
                    "network".to_string(),
                ],
                retry: RetryPolicy::default(),
                effects: Vec::new(),
                env_allowlist: Vec::new(),
                group: None,
                trigger_rule: TriggerRule::AllSuccess,
                branch: None,
            }],
            edges: vec![Edge {
                id: Some("e1".to_string()),
                kind: crate::EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "source".to_string(), port: "reads".to_string() },
                to: PortRef { node_id: "align".to_string(), port: "reads".to_string() },
            }],
        }
    }

    #[test]
    fn g121_resource_requirements_refuse_unavailable_capacity_before_run() {
        let requirements = build_resource_requirements(&sample_graph());
        let report = validate_resource_requirements(
            requirements,
            &ResourceAvailabilityV1 {
                cpu_cores: 2,
                memory_mb: 2048,
                disk_mb: 900,
                scratch_mb: 1024,
                network_available: false,
                walltime_ms: 5_000,
                accelerators: vec!["cpu-only".to_string()],
            },
        );
        assert!(!report.admitted);
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_CPU_UNAVAILABLE"));
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_MEMORY_UNAVAILABLE"));
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_DISK_UNAVAILABLE"));
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_SCRATCH_UNAVAILABLE"));
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_NETWORK_UNAVAILABLE"));
        assert!(report.refusals.iter().any(|refusal| refusal.code == "R121_WALLTIME_UNAVAILABLE"));
        assert!(report
            .refusals
            .iter()
            .any(|refusal| refusal.code == "R121_ACCELERATOR_UNAVAILABLE"));
    }
}
