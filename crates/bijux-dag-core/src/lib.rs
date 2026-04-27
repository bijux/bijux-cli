#![allow(
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::pedantic,
    clippy::return_self_not_must_use,
    clippy::uninlined_format_args,
    clippy::unreadable_literal
)]

#[cfg(test)]
use criterion as _;
#[cfg(test)]
use serde_yaml as _;
#[cfg(test)]
use tempfile as _;

#[path = "build/builder.rs"]
pub mod builder;
#[path = "graph/canonical.rs"]
pub mod canonical;
#[path = "build/compile.rs"]
pub mod compile;
#[path = "build/contract.rs"]
pub mod contract;
#[path = "graph/edge.rs"]
pub mod edge;
#[path = "analysis/effects.rs"]
pub mod effects;
#[path = "contracts/error.rs"]
pub mod error;
#[path = "analysis/fingerprint.rs"]
pub mod fingerprint;
#[path = "graph/graph.rs"]
pub mod graph;
#[path = "graph/meta.rs"]
pub mod meta;
#[path = "graph/model.rs"]
pub mod model;
#[path = "graph/node.rs"]
pub mod node;
#[path = "pipeline/parse.rs"]
pub mod parse;
#[path = "planner/planner.rs"]
pub mod planner;
#[path = "pipeline/resolve.rs"]
pub mod resolve;
#[path = "graph/resources.rs"]
pub mod resources;
#[path = "analysis/semantics.rs"]
pub mod semantics;
#[path = "graph/topology.rs"]
pub mod topology;
#[path = "pipeline/validate.rs"]
pub mod validate;

pub use builder::{
    dry_run_preview, lint_graph, simulate_graph, DagBuilder, DagDryRunPreview, DagLintFinding,
    DagUnitHarness, NodeBuilder,
};
pub use compile::{
    compile_graph, compile_graph_contract, compile_graph_strict, compile_graph_with_defaults,
    negotiate_spec_version, CompatibilityDecision, DagCompilePlanHints, DagCompileResult,
};
pub use contract::{DagSnapshot, GraphContract, GraphExecutionPolicy};
pub use error::GraphError;
pub use model::{
    ContainerSpec, Edge, Effect, FileOutput, Graph, GraphFingerprintExplain, GraphId, GraphMeta,
    Node, NodeKind, NodeOutputRef, ParamValue, PortRef, RefSpec, ResolvedGraph, Resources,
    RetryPolicy, Severity, ValidationDiagnostic,
};
pub use node::{
    node_input_bindings, node_io_contract, NodeEnvBinding, NodeInputBinding, NodeInputSource,
    NodeIoContract, NodeOutputContract, NodeParamBinding, ParamBindingSource,
};
pub use parse::parse_graph_strict;
pub use planner::{
    can_runtime_execute_plan_without_raw_graph, graph_lowering_boundary_note,
    lower_graph_to_execution_plan, map_planner_error_to_graph_error, node_kind_supported,
    planner_alignment_required_doc, planner_alignment_required_schema,
    planner_alignment_required_test, planner_diagnostics_from_error, planner_identity_for_graph,
    ExecutionPlan, PlanOptions, PlannedEdge, PlannedNode, PlannerDiagnostic, PlannerError,
    PlannerSeverity, PLANNER_CONTRACT_VERSION,
};
pub const SPEC_VERSION: &str = "bijux-dag/v0.1";
pub const CANONICALIZATION_CONTRACT_VERSION: &str = "bijux-dag-canonical/v1";

pub mod stable {
    pub use crate::{
        canonical::{canonical_json, canonicalize_graph},
        compile::{
            compile_graph, compile_graph_contract, compile_graph_strict,
            compile_graph_with_defaults, negotiate_spec_version, CompatibilityDecision,
            DagCompilePlanHints, DagCompileResult,
        },
        contract::{DagSnapshot, GraphContract, GraphExecutionPolicy},
        lower_graph_to_execution_plan, parse_graph_strict, planner_identity_for_graph,
        validate::validate_graph, ExecutionPlan, Graph, GraphError, PlanOptions, PlannedEdge,
        PlannedNode, PlannerDiagnostic, PlannerError, PlannerSeverity, SPEC_VERSION,
    };
}

pub mod experimental {
    pub use crate::semantics::{
        classify_compatibility, complexity_score, enforce_late_binding_immutability,
        explain_graph, migration_patch, normalize_semantic_graph, semantic_diff, static_analysis,
        BranchDecisionNode, CompatibilityClassification, ConditionalExecution,
        DynamicEdgeExpansionRule, GraphCompositionContract, GraphComplexityScore,
        GraphExplainabilityModel, GraphMigrationPatch, GraphTemplate, JoinSemantics,
        LateBindingRule, MapFanOutSemantics, NormalizedSemanticGraph, ParameterBindingSemantics,
        PartitionSemantics, ReduceFanInSemantics, SemanticDiffClass, SemanticDiffReport,
        StaticAnalysisReport, SubgraphEmbedding, WindowingSemantics,
    };
}
