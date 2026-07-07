use crate::{resources, Graph};
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

/// Data locality advisory row per node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataLocalityAdvisoryRowV1 {
    pub node_id: String,
    pub estimated_input_transfer_mb: u64,
    pub estimated_output_transfer_mb: u64,
    pub preferred_execution_site: String,
    pub advisory_only: bool,
    pub reversible: bool,
}

/// Data locality advisory report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataLocalityAdvisoryReportV1 {
    pub rows: Vec<DataLocalityAdvisoryRowV1>,
}

/// Advisory cost row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostPlanningAdvisoryRowV1 {
    pub node_id: String,
    pub cost_score: u64,
    pub drivers: Vec<String>,
    pub duration_claimed: bool,
}

/// Advisory cost report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostPlanningAdvisoryReportV1 {
    pub rows: Vec<CostPlanningAdvisoryRowV1>,
    pub expensive_nodes: Vec<String>,
}

/// Inputs for advisory capacity what-if modeling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityWhatIfInputV1 {
    pub queued_runs: u32,
    pub average_node_runtime_ms: u64,
    pub storage_free_mb: u64,
    pub evidence_snapshot_id: String,
}

/// Advisory capacity what-if report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityWhatIfReportV1 {
    pub estimated_queue_pressure: String,
    pub estimated_storage_footprint_mb: u64,
    pub estimated_execution_class: String,
    pub advisory_only: bool,
    pub tied_to_evidence_snapshot_id: String,
}

/// Confidence label for planner estimates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannerConfidenceLabelV1 {
    Measured,
    Static,
    Configured,
    Heuristic,
}

/// Confidence row describing estimate provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerConfidenceEntryV1 {
    pub node_id: String,
    pub estimate: String,
    pub confidence: PlannerConfidenceLabelV1,
    pub rationale: String,
}

/// Planner confidence report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerConfidenceReportV1 {
    pub entries: Vec<PlannerConfidenceEntryV1>,
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
                accelerator: resources::node_accelerator(node),
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
    if resources::node_gpu_devices(node) > 0
        || resources::node_accelerator(node).as_deref() == Some("gpu")
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
        crate::NodeKind::Python => ExecutionPoolV1::Shell,
        crate::NodeKind::Shell => ExecutionPoolV1::Shell,
        crate::NodeKind::Http => ExecutionPoolV1::Local,
        crate::NodeKind::FileTransform => ExecutionPoolV1::Local,
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
            adapter_kind: "python".to_string(),
            input_contract: "json-call".to_string(),
            output_contract: "declared-json-artifact".to_string(),
            effects: vec!["filesystem".to_string(), "env".to_string()],
            cacheable: true,
            sandbox_profile: "process".to_string(),
            side_effect_class: "writes_run".to_string(),
        },
        AdapterCapabilityDescriptorV1 {
            adapter_kind: "http".to_string(),
            input_contract: "http-request".to_string(),
            output_contract: "http-response-artifact".to_string(),
            effects: vec!["filesystem".to_string(), "network".to_string()],
            cacheable: true,
            sandbox_profile: "runtime".to_string(),
            side_effect_class: "writes_run".to_string(),
        },
        AdapterCapabilityDescriptorV1 {
            adapter_kind: "file_transform".to_string(),
            input_contract: "relative-input-paths".to_string(),
            output_contract: "declared-file-artifact".to_string(),
            effects: vec!["filesystem".to_string()],
            cacheable: true,
            sandbox_profile: "runtime".to_string(),
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

/// Build data-locality advisory without mutating graph semantics.
pub fn build_data_locality_advisory_report(graph: &Graph) -> DataLocalityAdvisoryReportV1 {
    let rows = graph
        .nodes
        .iter()
        .map(|node| {
            let mut estimated_output_transfer_mb = 100_u64;
            let mut preferred_execution_site = "local".to_string();
            for tag in &node.tags {
                if let Some(value) = tag.strip_prefix("artifact_mb:") {
                    if let Ok(parsed) = value.parse::<u64>() {
                        estimated_output_transfer_mb = parsed;
                    }
                } else if let Some(value) = tag.strip_prefix("site:") {
                    if !value.trim().is_empty() {
                        preferred_execution_site = value.trim().to_string();
                    }
                }
            }
            let estimated_input_transfer_mb = (node.inputs.len() as u64) * 64;
            DataLocalityAdvisoryRowV1 {
                node_id: node.id.clone(),
                estimated_input_transfer_mb,
                estimated_output_transfer_mb,
                preferred_execution_site,
                advisory_only: true,
                reversible: true,
            }
        })
        .collect();
    DataLocalityAdvisoryReportV1 { rows }
}

/// Build advisory cost report based on static graph signals.
pub fn build_cost_planning_advisory_report(graph: &Graph) -> CostPlanningAdvisoryReportV1 {
    let mut rows = Vec::new();
    let mut expensive_nodes = Vec::new();
    for node in &graph.nodes {
        let mut score = (node.inputs.len() as u64) * 5 + (node.outputs.len() as u64) * 5;
        let mut drivers = vec!["io_shape".to_string()];
        if let Some(resources) = node.resources.as_ref() {
            score += (resources.cpu as u64) * 2;
            score += (resources.mem_mb as u64) / 512;
            drivers.push("resource_hints".to_string());
        }
        for tag in &node.tags {
            if let Some(value) = tag.strip_prefix("artifact_mb:") {
                if let Ok(parsed) = value.parse::<u64>() {
                    score += parsed / 128;
                    drivers.push("artifact_volume".to_string());
                }
            }
            if tag == "expansion:matrix" {
                score += 50;
                drivers.push("expansion".to_string());
            }
        }
        if score >= 50 {
            expensive_nodes.push(node.id.clone());
        }
        rows.push(CostPlanningAdvisoryRowV1 {
            node_id: node.id.clone(),
            cost_score: score,
            drivers,
            duration_claimed: false,
        });
    }
    CostPlanningAdvisoryReportV1 { rows, expensive_nodes }
}

/// Build advisory capacity what-if report tied to evidence snapshot context.
pub fn build_capacity_what_if_report(
    graph: &Graph,
    input: &CapacityWhatIfInputV1,
) -> CapacityWhatIfReportV1 {
    let node_count = graph.nodes.len() as u64;
    let estimated_storage_footprint_mb = node_count
        .saturating_mul(128)
        .saturating_add((input.queued_runs as u64).saturating_mul(64));
    let estimated_queue_pressure = if input.queued_runs >= 100 {
        "high".to_string()
    } else if input.queued_runs >= 25 {
        "moderate".to_string()
    } else {
        "low".to_string()
    };
    let estimated_execution_class = if input.average_node_runtime_ms > 120_000 {
        "long-running".to_string()
    } else if graph.nodes.iter().any(|node| resources::node_gpu_devices(node) > 0) {
        "accelerated".to_string()
    } else {
        "standard".to_string()
    };
    CapacityWhatIfReportV1 {
        estimated_queue_pressure,
        estimated_storage_footprint_mb,
        estimated_execution_class,
        advisory_only: true,
        tied_to_evidence_snapshot_id: input.evidence_snapshot_id.clone(),
    }
}

/// Build confidence labels for planner estimates.
pub fn build_planner_confidence_report(graph: &Graph) -> PlannerConfidenceReportV1 {
    let entries = graph
        .nodes
        .iter()
        .map(|node| {
            let (confidence, rationale) = if node
                .tags
                .iter()
                .any(|tag| tag == "confidence:measured")
            {
                (PlannerConfidenceLabelV1::Measured, "evidence-backed measurement".to_string())
            } else if node.tags.iter().any(|tag| tag == "confidence:configured") {
                (PlannerConfidenceLabelV1::Configured, "operator-configured estimate".to_string())
            } else if node.resources.is_some() {
                (PlannerConfidenceLabelV1::Static, "declared static resource hints".to_string())
            } else {
                (PlannerConfidenceLabelV1::Heuristic, "default heuristic estimate".to_string())
            };
            PlannerConfidenceEntryV1 {
                node_id: node.id.clone(),
                estimate: "resource_and_cost".to_string(),
                confidence,
                rationale,
            }
        })
        .collect();
    PlannerConfidenceReportV1 { entries }
}

#[cfg(test)]
mod tests {
    use super::{
        adapter_capability_for_kind, build_capacity_what_if_report,
        build_cost_planning_advisory_report, build_data_locality_advisory_report,
        build_planner_confidence_report, build_resource_requirements, plan_pool_placement,
        planner_runnable_from_capabilities, validate_resource_requirements, CapacityWhatIfInputV1,
        ExecutionPoolV1, PlannerConfidenceLabelV1, ResourceAvailabilityV1,
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "align".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec!["reads".to_string()],
                outputs: vec![FileOutput::new("bam".to_string(), "align.bam".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: Some(6_000),
                resources: Some(Resources {
                    cpu: 4,
                    mem_mb: 4096,
                    gpu_devices: 0,
                    named_resources: std::collections::BTreeMap::new(),
                }),
                tags: vec![
                    "disk_mb:1024".to_string(),
                    "scratch_mb:2048".to_string(),
                    "accelerator:gpu".to_string(),
                    "network".to_string(),
                ],
                retry: RetryPolicy::default(),
                cache: Default::default(),
                effects: Vec::new(),
                env_allowlist: Vec::new(),
                group: None,
                trigger_rule: TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
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
        let python = adapter_capability_for_kind("python").expect("python capability");
        let http = adapter_capability_for_kind("http").expect("http capability");
        let file_transform =
            adapter_capability_for_kind("file_transform").expect("file_transform capability");
        assert_eq!(shell.sandbox_profile, "process");
        assert_eq!(python.sandbox_profile, "process");
        assert_eq!(http.sandbox_profile, "runtime");
        assert_eq!(file_transform.sandbox_profile, "runtime");
        assert!(planner_runnable_from_capabilities("shell", false));
        assert!(planner_runnable_from_capabilities("python", false));
        assert!(planner_runnable_from_capabilities("http", false));
        assert!(planner_runnable_from_capabilities("file_transform", false));
        assert!(planner_runnable_from_capabilities("http", true));
        assert!(!planner_runnable_from_capabilities("file_transform", true));
        assert!(!planner_runnable_from_capabilities("python", true));
        assert!(!planner_runnable_from_capabilities("shell", true));
        assert!(planner_runnable_from_capabilities("container", true));
        assert!(!planner_runnable_from_capabilities("external-unregistered", false));
    }

    #[test]
    fn g126_data_locality_advisory_is_visible_and_reversible() {
        let mut graph = sample_graph();
        graph.nodes[0].tags.push("artifact_mb:900".to_string());
        graph.nodes[0].tags.push("site:gpu-cluster".to_string());
        let report = build_data_locality_advisory_report(&graph);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].estimated_output_transfer_mb, 900);
        assert_eq!(report.rows[0].preferred_execution_site, "gpu-cluster");
        assert!(report.rows[0].advisory_only);
        assert!(report.rows[0].reversible);
    }

    #[test]
    fn g127_cost_planning_is_advisory_and_does_not_claim_runtime_duration() {
        let mut graph = sample_graph();
        graph.nodes[0].tags.push("artifact_mb:4096".to_string());
        graph.nodes[0].tags.push("expansion:matrix".to_string());
        let report = build_cost_planning_advisory_report(&graph);
        assert_eq!(report.rows.len(), 1);
        assert!(report.expensive_nodes.iter().any(|node_id| node_id == "align"));
        assert!(!report.rows[0].duration_claimed);
        assert!(report.rows[0].drivers.iter().any(|driver| driver == "artifact_volume"));
    }

    #[test]
    fn g128_capacity_what_if_is_advisory_and_evidence_tied() {
        let graph = sample_graph();
        let report = build_capacity_what_if_report(
            &graph,
            &CapacityWhatIfInputV1 {
                queued_runs: 40,
                average_node_runtime_ms: 10_000,
                storage_free_mb: 32_000,
                evidence_snapshot_id: "evidence-2026-05-01T06:00:00Z".to_string(),
            },
        );
        assert_eq!(report.estimated_queue_pressure, "moderate");
        assert!(report.estimated_storage_footprint_mb > 0);
        assert_eq!(report.tied_to_evidence_snapshot_id, "evidence-2026-05-01T06:00:00Z");
        assert!(report.advisory_only);
    }

    #[test]
    fn g129_planner_confidence_labels_are_explicit() {
        let mut graph = sample_graph();
        graph.nodes[0].tags.push("confidence:configured".to_string());
        let report = build_planner_confidence_report(&graph);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].confidence, PlannerConfidenceLabelV1::Configured);
        assert!(report.entries[0].rationale.contains("operator-configured"));
    }
}
