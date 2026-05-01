use crate::Graph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

/// Planner pool class.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionPoolV1 {
    Local,
    Shell,
    Container,
    Batch,
    HighMemory,
    Gpu,
    Offline,
}

/// Pool placement decision per node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoolPlacementDecisionV1 {
    pub node_id: String,
    pub requested_pool: ExecutionPoolV1,
    pub assigned_pool: Option<ExecutionPoolV1>,
    pub diagnostic: Option<String>,
}

/// Pool placement planning report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoolPlacementReportV1 {
    pub placements: Vec<PoolPlacementDecisionV1>,
    pub diagnostics: Vec<String>,
}

/// Machine-readable adapter capability descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCapabilityDescriptorV1 {
    pub adapter_kind: String,
    pub input_contract: String,
    pub output_contract: String,
    pub effects: Vec<String>,
    pub cacheable: bool,
    pub sandbox_profile: String,
    pub side_effect_class: String,
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

/// Plan node placement to execution pools with deterministic unavailable-pool diagnostics.
pub fn plan_pool_placement(
    graph: &Graph,
    pool_availability: &BTreeMap<ExecutionPoolV1, bool>,
) -> PoolPlacementReportV1 {
    let mut placements = Vec::new();
    let mut diagnostics = Vec::new();
    for node in &graph.nodes {
        let requested_pool = infer_requested_pool(node);
        let available = pool_availability.get(&requested_pool).copied().unwrap_or(false);
        let (assigned_pool, diagnostic) = if available {
            (Some(requested_pool.clone()), None)
        } else {
            let message = format!(
                "node '{}' requested unavailable pool '{}'",
                node.id,
                execution_pool_label(&requested_pool)
            );
            diagnostics.push(message.clone());
            (None, Some(message))
        };
        placements.push(PoolPlacementDecisionV1 {
            node_id: node.id.clone(),
            requested_pool,
            assigned_pool,
            diagnostic,
        });
    }
    PoolPlacementReportV1 { placements, diagnostics }
}

fn infer_requested_pool(node: &crate::Node) -> ExecutionPoolV1 {
    if node.tags.iter().any(|tag| tag == "offline") {
        return ExecutionPoolV1::Offline;
    }
    if node
        .tags
        .iter()
        .any(|tag| tag == "gpu" || tag.strip_prefix("accelerator:").is_some_and(|value| value == "gpu"))
    {
        return ExecutionPoolV1::Gpu;
    }
    if node.resources.as_ref().is_some_and(|resource| resource.mem_mb >= 65_536) {
        return ExecutionPoolV1::HighMemory;
    }
    if node.tags.iter().any(|tag| tag == "batch") {
        return ExecutionPoolV1::Batch;
    }
    match node.kind {
        crate::NodeKind::Container => ExecutionPoolV1::Container,
        crate::NodeKind::Shell => ExecutionPoolV1::Shell,
        _ => ExecutionPoolV1::Local,
    }
}

fn execution_pool_label(pool: &ExecutionPoolV1) -> &'static str {
    match pool {
        ExecutionPoolV1::Local => "local",
        ExecutionPoolV1::Shell => "shell",
        ExecutionPoolV1::Container => "container",
        ExecutionPoolV1::Batch => "batch",
        ExecutionPoolV1::HighMemory => "high-memory",
        ExecutionPoolV1::Gpu => "gpu",
        ExecutionPoolV1::Offline => "offline",
    }
}

/// Built-in adapter capability registry used by planning admission.
pub fn builtin_adapter_capability_registry() -> Vec<AdapterCapabilityDescriptorV1> {
    vec![
        AdapterCapabilityDescriptorV1 {
            adapter_kind: "const".to_string(),
            input_contract: "literal-or-ref".to_string(),
            output_contract: "declared-artifact".to_string(),
            effects: vec!["filesystem".to_string()],
            cacheable: true,
            sandbox_profile: "restricted".to_string(),
            side_effect_class: "read_only".to_string(),
        },
        AdapterCapabilityDescriptorV1 {
            adapter_kind: "shell".to_string(),
            input_contract: "argv-only".to_string(),
            output_contract: "declared-artifact".to_string(),
            effects: vec!["filesystem".to_string(), "env".to_string()],
            cacheable: true,
            sandbox_profile: "process".to_string(),
            side_effect_class: "writes_run".to_string(),
        },
        AdapterCapabilityDescriptorV1 {
            adapter_kind: "container".to_string(),
            input_contract: "container-contract".to_string(),
            output_contract: "declared-artifact".to_string(),
            effects: vec!["filesystem".to_string(), "env".to_string(), "network".to_string()],
            cacheable: true,
            sandbox_profile: "container".to_string(),
            side_effect_class: "executes_adapter".to_string(),
        },
    ]
}

/// Return capability descriptor for a node kind.
pub fn adapter_capability_for_kind(kind: &str) -> Option<AdapterCapabilityDescriptorV1> {
    builtin_adapter_capability_registry().into_iter().find(|entry| entry.adapter_kind == kind)
}

/// Decide if planner can mark a node runnable from capability declarations.
pub fn planner_runnable_from_capabilities(kind: &str, requires_network: bool) -> bool {
    let Some(capability) = adapter_capability_for_kind(kind) else {
        return false;
    };
    if requires_network {
        capability.effects.iter().any(|effect| effect == "network")
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        adapter_capability_for_kind, build_resource_requirements, plan_pool_placement,
        planner_runnable_from_capabilities, validate_resource_requirements, ExecutionPoolV1,
        ResourceAvailabilityV1,
    };
    use crate::{
        Edge, FileOutput, Graph, GraphMeta, Node, NodeKind, ParamValue, PortRef, Resources,
        RetryPolicy, SemanticNodeKind, TriggerRule,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn g122_pool_placement_reports_unavailable_pools_deterministically() {
        let graph = sample_graph();
        let report = plan_pool_placement(
            &graph,
            &BTreeMap::from([
                (ExecutionPoolV1::Local, true),
                (ExecutionPoolV1::Shell, true),
                (ExecutionPoolV1::Container, true),
                (ExecutionPoolV1::Batch, true),
                (ExecutionPoolV1::HighMemory, true),
                (ExecutionPoolV1::Gpu, false),
                (ExecutionPoolV1::Offline, false),
            ]),
        );
        assert_eq!(report.placements.len(), 1);
        assert_eq!(report.placements[0].requested_pool, ExecutionPoolV1::Gpu);
        assert!(report.placements[0].assigned_pool.is_none());
        assert!(report.diagnostics[0].contains("requested unavailable pool 'gpu'"));
    }

    #[test]
    fn g123_adapter_capabilities_are_machine_readable_for_planner_runnability() {
        let shell = adapter_capability_for_kind("shell").expect("shell capability");
        assert_eq!(shell.sandbox_profile, "process");
        assert!(planner_runnable_from_capabilities("shell", false));
        assert!(!planner_runnable_from_capabilities("shell", true));
        assert!(planner_runnable_from_capabilities("container", true));
        assert!(!planner_runnable_from_capabilities("external-unregistered", false));
    }
}
