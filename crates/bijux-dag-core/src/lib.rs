//! Deterministic graph authoring, validation, and planning for Bijux DAG.
//!
//! Prefer the crate root for focused imports, [`stable`] for an explicit
//! compatibility surface, and [`prelude`] for parse/validate/plan workflows.
//! The `experimental-public-api` feature enables research and compatibility
//! contracts that are intentionally excluded from the default docs surface.
//!
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

#[cfg(feature = "experimental-public-api")]
#[path = "contracts/authoring_iteration03.rs"]
mod authoring_iteration03;
#[doc(hidden)]
#[path = "build/builder.rs"]
pub mod builder;
#[doc(hidden)]
#[path = "graph/canonical.rs"]
pub mod canonical;
#[doc(hidden)]
#[path = "build/compile.rs"]
pub mod compile;
#[doc(hidden)]
#[path = "build/contract.rs"]
pub mod contract;
#[doc(hidden)]
#[path = "graph/edge.rs"]
pub mod edge;
#[doc(hidden)]
#[path = "analysis/effects.rs"]
pub mod effects;
#[doc(hidden)]
#[path = "contracts/error.rs"]
pub mod error;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/execution_iteration12.rs"]
mod execution_iteration12;
#[doc(hidden)]
#[path = "analysis/fingerprint.rs"]
pub mod fingerprint;
#[doc(hidden)]
#[path = "graph/graph.rs"]
pub mod graph;
#[doc(hidden)]
#[path = "graph/input.rs"]
pub mod input;
#[doc(hidden)]
#[path = "graph/meta.rs"]
pub mod meta;
#[doc(hidden)]
#[path = "graph/model.rs"]
pub mod model;
#[doc(hidden)]
#[path = "graph/node.rs"]
pub mod node;
#[doc(hidden)]
#[path = "pipeline/parse.rs"]
pub mod parse;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/performance_iteration19_contracts.rs"]
mod performance_iteration19_contracts;
#[doc(hidden)]
#[path = "planner/planner.rs"]
pub mod planner;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/planner_iteration05.rs"]
mod planner_iteration05;
#[doc(hidden)]
#[path = "pipeline/resolve.rs"]
pub mod resolve;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/resource_iteration13.rs"]
mod resource_iteration13;
#[doc(hidden)]
#[path = "graph/resources.rs"]
pub mod resources;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/scientific_integration_contracts.rs"]
mod scientific_integration_contracts;
#[doc(hidden)]
#[path = "analysis/semantics.rs"]
pub mod semantics;
#[cfg(feature = "experimental-public-api")]
#[path = "contracts/semantics_iteration04.rs"]
mod semantics_iteration04;
#[doc(hidden)]
#[path = "graph/topology.rs"]
pub mod topology;
#[doc(hidden)]
#[path = "pipeline/validate.rs"]
pub mod validate;

pub use builder::{
    dry_run_preview, lint_graph, simulate_graph, DagBuilder, DagDryRunPreview, DagLintFinding,
    DagUnitHarness, NodeBuilder,
};
pub use canonical::{canonical_json, canonicalize_graph};
pub use compile::{
    compile_graph, compile_graph_contract, compile_graph_strict, compile_graph_with_defaults,
    negotiate_spec_version, CompatibilityDecision, DagCompilePlanHints, DagCompileResult,
};
pub use contract::{DagSnapshot, GraphContract, GraphExecutionPolicy};
pub use edge::{EdgeDependencyKind, TypedEdge};
pub use error::GraphError;
pub use input::{
    materialize_graph_input_value, validate_graph_input_value, GraphInputKind, GraphInputSpec,
    GraphInputViolation,
};
pub use model::{
    cache_behavior_enabled, cache_behavior_is_default, edge_kind_is_default,
    semantic_kind_is_default, trigger_rule_is_default, BranchSpec, CacheBehavior, ContainerSpec,
    Edge, EdgeKind, Effect, FileOutput, Graph, GraphFingerprintExplain, GraphId, GraphMeta, Node,
    NodeKind, NodeOutputRef, ParamValue, PortRef, RefSpec, ResolvedGraph, Resources, RetryPolicy,
    SemanticNodeKind, Severity, TriggerRule, ValidationDiagnostic,
};
pub use node::{
    derive_interface, node_input_bindings, node_io_contract, NodeEnvBinding, NodeInputBinding,
    NodeInputSource, NodeIoContract, NodeOutputContract, NodeParamBinding, ParamBindingSource,
};
pub use parse::parse_graph_strict;
pub use planner::{
    can_runtime_execute_plan_without_raw_graph, graph_lowering_boundary_note,
    lower_graph_to_execution_plan, map_planner_error_to_graph_error, node_kind_supported,
    planner_alignment_required_doc, planner_alignment_required_schema,
    planner_alignment_required_test, planner_diagnostics_from_error, planner_identity_for_graph,
    BranchPathAnalysis, ExecutionPlan, PlanOptions, PlannedBranchContract, PlannedEdge,
    PlannedNode, PlannerDiagnostic, PlannerError, PlannerSeverity, PLANNER_CONTRACT_VERSION,
};
pub use resolve::resolve_graph;
pub use resources::GraphDefaults;
pub use topology::deterministic_topology_order;
pub use validate::{
    validate_graph, validate_schema, validate_semantics, validate_topology,
    validation_rule_registry, ValidationDomain,
};
pub const SPEC_VERSION: &str = "bijux-dag/v0.1";
pub const CANONICALIZATION_CONTRACT_VERSION: &str = "bijux-dag-canonical/v1";

/// Explicit long-lived graph authoring, validation, and planning surface.
pub mod stable {
    pub use crate::{
        canonical_json, canonicalize_graph, compile_graph, compile_graph_contract,
        compile_graph_strict, compile_graph_with_defaults, lower_graph_to_execution_plan,
        negotiate_spec_version, parse_graph_strict, planner_identity_for_graph, validate_graph,
        CompatibilityDecision, DagCompilePlanHints, DagCompileResult, DagSnapshot, ExecutionPlan,
        Graph, GraphContract, GraphError, GraphExecutionPolicy, PlanOptions, PlannedEdge,
        PlannedNode, PlannerDiagnostic, PlannerError, PlannerSeverity, SPEC_VERSION,
    };
}

/// Common imports for parse, validate, canonicalize, and plan workflows.
pub mod prelude {
    pub use crate::stable::{
        canonical_json, canonicalize_graph, compile_graph, compile_graph_contract,
        compile_graph_strict, compile_graph_with_defaults, negotiate_spec_version, validate_graph,
        CompatibilityDecision, DagCompilePlanHints, DagCompileResult, DagSnapshot, ExecutionPlan,
        Graph, GraphContract, GraphError, GraphExecutionPolicy, PlanOptions, PlannedEdge,
        PlannedNode, PlannerDiagnostic, PlannerError, PlannerSeverity, SPEC_VERSION,
    };
    pub use crate::{
        lower_graph_to_execution_plan, parse_graph_strict, planner_identity_for_graph,
    };
}

/// Opt-in research and compatibility contracts that are outside the stable lane.
#[cfg(feature = "experimental-public-api")]
pub mod experimental {
    pub mod authoring_contracts {
        pub use crate::authoring_iteration03::*;
    }
    pub mod execution_contracts {
        pub use crate::execution_iteration12::*;
    }
    pub mod planner_contracts {
        pub use crate::performance_iteration19_contracts::*;
        pub use crate::planner_iteration05::*;
    }
    pub mod resource_capabilities {
        pub use crate::resource_iteration13::*;
    }
    pub mod scientific_integration {
        pub use crate::scientific_integration_contracts::*;
    }
    pub mod semantic_contracts {
        pub use crate::semantics_iteration04::*;
    }
    pub use crate::semantics::{
        classify_compatibility, complexity_score, enforce_late_binding_immutability, explain_graph,
        migration_patch, normalize_semantic_graph, semantic_diff, static_analysis,
        BranchDecisionNode, CompatibilityClassification, ConditionalExecution,
        DynamicEdgeExpansionRule, GraphComplexityScore, GraphCompositionContract,
        GraphExplainabilityModel, GraphMigrationPatch, GraphTemplate, JoinSemantics,
        LateBindingRule, MapFanOutSemantics, NormalizedSemanticGraph, ParameterBindingSemantics,
        PartitionSemantics, ReduceFanInSemantics, SemanticDiffClass, SemanticDiffReport,
        StaticAnalysisReport, SubgraphEmbedding, WindowingSemantics,
    };
}
