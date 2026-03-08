#![allow(dead_code)]

#[path = "adapters/adapter.rs"]
mod adapter;
#[path = "adapters/api.rs"]
pub mod adapter_api;
#[path = "adapters/conformance.rs"]
pub mod adapter_conformance;
#[cfg(test)]
#[path = "internal/testing/adapter_contract_tests.rs"]
mod adapter_contract_tests;
#[path = "adapters/sdk.rs"]
mod adapter_sdk;
pub mod adapters;
#[path = "internal/analysis/adaptive_scheduler.rs"]
mod adaptive_scheduler;
#[path = "internal/workflow/ai_operator_assist.rs"]
mod ai_operator_assist;
#[path = "internal/control/api.rs"]
pub mod api;
mod artifacts;
#[path = "adapters/async_adapter.rs"]
mod async_adapter;
#[path = "internal/identity/auth_identity.rs"]
mod auth_identity;
#[path = "internal/identity/authz_policy.rs"]
mod authz_policy;
mod backend;
#[path = "backend/runtime/backend_cluster.rs"]
mod backend_cluster;
#[path = "backend/runtime/batch_execution.rs"]
mod batch_execution;
pub mod builtins;
pub mod cache;
#[path = "internal/control/clock.rs"]
mod clock;
#[path = "internal/control/config.rs"]
pub mod config;
#[path = "backend/runtime/container_execution.rs"]
mod container_execution;
#[path = "diagnostics/runtime/control_plane.rs"]
mod control_plane;
#[path = "diagnostics/runtime/control_plane_api.rs"]
mod control_plane_api;
#[path = "backend/distributed/coordination.rs"]
mod coordination;
#[path = "internal/analysis/cost_optimization.rs"]
mod cost_optimization;
#[path = "internal/analysis/dataset_semantics.rs"]
mod dataset_semantics;
mod diagnostics;
#[path = "backend/distributed/distributed.rs"]
mod distributed;
#[path = "backend/distributed/distribution_readiness.rs"]
mod distribution_readiness;
#[path = "runtime_core/execution/engine.rs"]
mod engine;
mod error;
#[path = "runtime_core/execution/flow.rs"]
pub mod execution;
#[path = "backend/runtime/execution_backend.rs"]
mod execution_backend;
#[path = "runtime_core/execution/context.rs"]
pub mod execution_context;
#[path = "runtime_core/planning/execution_plan.rs"]
mod execution_plan;
#[path = "internal/ext/extension_catalog.rs"]
mod extension_catalog;
#[path = "adapters/external.rs"]
mod external_adapter;
#[path = "backend/distributed/federated_scheduling.rs"]
mod federated_scheduling;
#[path = "internal/ext/formal_verification.rs"]
mod formal_verification;
#[path = "backend/distributed/geo_federation.rs"]
mod geo_federation;
#[path = "backend/distributed/ha_scheduler.rs"]
mod ha_scheduler;
#[path = "backend/distributed/infrastructure.rs"]
mod infrastructure;
mod internal;
#[path = "runtime_core/governance/invariants.rs"]
pub mod invariants;
#[cfg(test)]
#[path = "internal/testing/invariants_tests.rs"]
mod invariants_tests;
#[path = "internal/control/io.rs"]
mod io;
#[path = "backend/runtime/local_executor.rs"]
mod local_executor;
#[path = "runtime_core/execution/node_result.rs"]
pub mod node_result;
#[path = "diagnostics/runtime/observability.rs"]
mod observability;
#[path = "diagnostics/runtime/observability_deep.rs"]
mod observability_deep;
#[path = "diagnostics/runtime/operations_governance.rs"]
mod operations_governance;
#[path = "artifacts/storage/path_authorization.rs"]
mod path_authorization;
#[path = "internal/perf/performance_capacity.rs"]
mod performance_capacity;
#[path = "runtime_core/planning/planner.rs"]
mod planner;
#[path = "runtime_core/planning/planner_analysis.rs"]
mod planner_analysis;
pub mod policy;
#[path = "artifacts/storage/recovery.rs"]
mod recovery;
#[path = "adapters/runtime_registry.rs"]
mod registry;
#[path = "backend/runtime/remote_execution_model.rs"]
mod remote_execution_model;
#[path = "backend/runtime/remote_executor.rs"]
mod remote_executor;
mod replay;
#[path = "runtime_core/execution/run_context.rs"]
pub mod run_context;
#[path = "runtime_core/execution/run_state.rs"]
mod run_state;
#[path = "internal/control/runtime.rs"]
mod runtime;
#[cfg(test)]
#[path = "internal/testing/runtime_boundary_tests.rs"]
mod runtime_boundary_tests;
pub mod runtime_core;
#[cfg(test)]
#[path = "internal/testing/runtime_policy_trace_tests.rs"]
mod runtime_policy_trace_tests;
#[path = "runtime_core/governance/semantics.rs"]
mod runtime_semantics;
#[path = "runtime_core/governance/sacred_execution.rs"]
mod sacred_execution;
#[path = "runtime_core/execution/scheduler.rs"]
mod scheduler;
#[path = "runtime_core/execution/scheduler_workload.rs"]
mod scheduler_workload;
#[path = "internal/identity/secrets_security.rs"]
mod secrets_security;
#[path = "internal/identity/security_env.rs"]
mod security_env;
#[path = "internal/control/selectors.rs"]
pub mod selectors;
#[path = "artifacts/storage/semantic_lineage.rs"]
mod semantic_lineage;
#[path = "internal/control/services.rs"]
pub mod services;
pub mod simulated_platform;
#[path = "runtime_core/execution/state_machine.rs"]
pub mod state_machine;
#[cfg(test)]
#[path = "internal/testing/state_machine_tests.rs"]
mod state_machine_tests;
#[path = "artifacts/storage/store.rs"]
mod store;
#[path = "backend/runtime/subprocess.rs"]
pub mod subprocess;
#[path = "internal/identity/supply_chain_trust.rs"]
mod supply_chain_trust;
#[path = "internal/control/task_contract.rs"]
mod task_contract;
#[path = "internal/control/task_types.rs"]
mod task_types;
#[path = "internal/identity/tenancy.rs"]
mod tenancy;
#[cfg(test)]
#[path = "internal/testing/test_support.rs"]
mod test_support;
#[path = "artifacts/storage/trace.rs"]
pub mod trace;
#[path = "artifacts/storage/upgrade_compatibility.rs"]
mod upgrade_compatibility;
#[path = "internal/workflow/workflow_product.rs"]
mod workflow_product;
use adapter::{Adapter, AdapterId, EffectSet, NodeCtx};
pub use adapter_sdk::{
    AdapterCapabilities, AdapterContext, AdapterPlugin, BackendPlugin, PluginManifest,
};
pub use async_adapter::AsyncAdapter;
pub use backend_cluster::{
    artifact_collection_state, backend_ready_for_admission, canonical_k8s_terminal_events,
    capture_hpc_scheduler_version, classify_hpc_failure, classify_k8s_failure,
    effective_hpc_retry_policy, equivalent_to_local, hpc_array_job_supported,
    hpc_environment_fingerprint, hpc_log_collection_semantics, hpc_poll_response_recovered,
    hpc_replay_fidelity_from_module_fingerprints, hpc_resource_fingerprint,
    hpc_scratch_staging_semantics, k8s_capability_declaration, map_node_policy_to_k8s_job,
    map_node_resources_to_k8s, map_node_to_hpc_queue_partition, map_timeout_to_hpc_walltime,
    matches_placement_policy, normalize_backend_failure, outputs_logs_equivalent,
    quota_saturation_percent, reconcile_k8s_watch_stream,
    reject_unsupported_hpc_scheduler_features, reject_unsupported_k8s_fields,
    replay_allowed_across_backends, scratch_retention_required, staged_input_cleanup_required,
    validate_k8s_injection, workdir_semantics, AdapterExecutionOutcome, ArtifactCollectionState,
    BackendCapabilityDescriptor, BackendCleanupGuarantee, BackendConformanceSuite,
    BackendFailureMappingRule, BackendLogCollectionContract, BackendMaintenanceMode,
    BackendOutageSimulationFixture, BackendProductionReadinessChecklist, BackendQuotaMetrics,
    BackendReadinessProbe, CrossBackendReplayRule, GenericBatchExecutorContract,
    HpcFailureClassification, HpcLogCollectionSemantics, HpcNodeExecutionContract,
    HpcQueuePartitionMapping, HpcReplayFidelity, HpcResourceFingerprintInput,
    HpcRetryPolicyDecision, HpcSchedulerVersionMetadata, HpcScratchStagingSemantics,
    ImageResolutionProvenance, K8sBackendVersionMetadata, K8sCapabilityDeclaration,
    K8sFailureClass, K8sInjectionAvailability, K8sInjectionRequest, K8sJobPolicyMapping,
    K8sResourceMapping, K8sResourceRequest, K8sWatchEvent, KubernetesExecutorContractV2,
    NodeAffinityHint, NodeExecutionContract, PlacementPolicyRule, QueueBackendRoutingPolicy,
    RemoteArtifactStagingProtocol, SlurmExecutorContract, WorkdirSemantics, WorkdirVolumeKind,
};
pub use batch_execution::{
    cancel_batch_attempt, duplicate_status_delivery_detected, execution_mode_report,
    heartbeat_stale, restart_recovery_supported, retry_attempt, validate_batch_metadata,
    BatchAttemptState, BatchHeartbeat, BatchJobMetadata, BatchLifecycleEvent, BatchModeReport,
};
use bijux_dag_artifacts::schema::{
    validate_output_schema_descriptor, ArtifactSchemaDescriptor, SchemaValidationMode,
};
use bijux_dag_artifacts::{
    write_inputs_index, write_outputs_index, AdapterInfo, ArtifactError, CacheProof,
    ContainerTrace, FailureInfo, InputFile, InputsIndex, NodeCounts, NodeTrace, OutputSummary,
    OutputsIndex, ReplayProvenance, Resources as TraceResources, RunDir, RunOutputFile,
    RunOutputsIndex,
};
use bijux_dag_core::{
    Effect, FileOutput, Graph, GraphError, Node, NodeKind, RetryPolicy, Severity,
};
pub use cache::{
    cache_entry_has_required_proof, cache_key_explanation, cache_metadata_version_supported,
    CacheKeyInput,
};
use clock::{Clock, SystemClock};
pub use container_execution::{
    container_env_isolated, map_local_path_to_container, validate_container_contract,
    validate_container_relative_path, ContainerExecutionContract, ContainerMount,
};
pub use coordination::{
    merge_timeout_and_exit_events, thread_safety_audit, RunSummaryCounters,
    RuntimeCoordinationSnapshot, RuntimeCoordinationState, ThreadSafetyAuditRecord,
    TraceWriteRecord,
};
pub use execution_backend::{
    backend_registry, bind_backend_or_error, execute_with_backend, BackendBindingRequest,
    BackendCapabilities, BackendContext, BackendError, BackendKind, BackendLifecycleResult,
    EngineOutcome, ExecutionAttemptRecord, ExecutionBackend, ExecutionBackendCapabilityDescriptor,
    FakeBackend, ProcessLikeBackend,
};
pub use execution_plan::{ExecutionPlan, PlannedDependency, PlannedNode};
pub use extension_catalog::{
    compute_platform_maturity, detect_extension_compatibility_issues,
    extension_discovery_inventory, extension_failure_isolated, extension_point_status_report,
    internal_hook_ready_for_promotion, negotiate_plugin_version, register_extension,
    validate_extension_descriptor, validate_plugin_conformance, CapabilityRange,
    CodeGenerationHook, DslExtensionPoint, ExtensionCompatibilityIssue, ExtensionDescriptor,
    ExtensionDiscoveryRecord, ExtensionPointStatus, ExtensionRegistration, ExtensionStabilityLevel,
    InternalHookPromotionChecklist, OfficialPluginPolicy, PlatformMaturityScorecard,
    PluginBoundaryKind, PluginConformanceSuiteResult, PluginIsolationPolicy, PluginLifecycleState,
    PluginLoadingMode, PluginMetadata, PluginTrustPolicy,
};
pub use formal_verification::{
    artifact_integrity_holds, build_counterexample, invariant_catalog_default,
    lineage_invariants_hold, machine_checkable_invariants, policy_invariants_hold,
    replay_determinism_holds, verification_gate_passed, verification_maturity_label,
    AdversarialFixtureSet, ArtifactIntegrityInvariant, CounterexampleReport, DiffSemanticSpec,
    FormalAssuranceRoadmap, FuzzingStrategy, HaVerificationHarness, InvariantDefinition,
    LineageInvariantProof, ModelTestSuite, PolicyInvariantProof, PropertyTestSuite,
    ReplayDeterminismInvariant, SchedulerStateSpaceCheck, VerificationGate,
    VerificationMaturityLabel, VerifiedCoreScope,
};
pub use infrastructure::{
    negotiate_backend_capabilities, ArtifactStoreBackend, ArtifactTransportContract,
    ArtifactTransportMode, BackendAcceptanceGate,
    BackendCapabilities as InfrastructureBackendCapabilities, BackendCapabilityRequirement,
    BackendExecutionCompletion, BackendExecutionRequest, BackendPolicyOverlay, CapabilityDecision,
    ExecutorBackend, HighAvailabilitySchedulerPlan, HpcExecutorContract,
    KubernetesExecutorContract, MultiTenantIdentity, ObjectStorageContract, QueuePartition,
    RegistryPersistenceBackend, RuntimeSecretContract, SchedulerScalingPlan,
};
use io::{Fs, StdFs};
pub use local_executor::LocalExecutor;
pub use observability::{
    category_from_runtime_event_name, current_process_memory_bytes,
    event_contains_sensitive_material, event_names_emitted_once, required_event_fields_present,
    summarize_failure_root_causes, validate_required_event_names, write_timeline_export,
    EventCategory, EventRecord, EventSink, FileEventSink, InMemoryMetricsRegistry, MetricsRegistry,
    NodeMetrics, RemoteCollectorSink, RunMetrics, SchedulerMetrics, SpanKind, StdoutEventSink,
    TimelineEntry, TimelineExport, TraceSpan, REQUIRED_RUNTIME_EVENT_NAMES,
};
pub use observability_deep::{
    build_diagnostics, build_topology_overlay, detect_metric_drift, observability_contract_status,
    redact_event_details, render_timeline_text, root_cause_graph, sample_events, AlertRule,
    DiagnosticRecord, DiagnosticsKind, DriftDetectionReport, EventCorrelation,
    ExplainArtifactReport, ExplainNodeReport, ExplainRunReport, ExplainScheduleReport,
    FailureCauseCode, MetricsExportFormat, ObservabilityContractStatus, RedactionPolicy,
    ReplaySpanLink, SamplingPolicy, TimelineTextSummary, TopologyOverlay, TopologyOverlayNode,
};
pub use path_authorization::{authorize_input_path, authorize_output_path};
pub use performance_capacity::{
    build_cost_model, build_performance_maturity_report, compile_environment_profiles,
    derive_autoscaling_hint, detect_performance_regression, forecast_storage_growth,
    synthetic_large_dag_profiles, ArtifactStoreBenchmarkResult, AutoscalingHint, BenchmarkResult,
    CapacityModel, EnvironmentScaleProfile, PerformanceGate, PerformanceMaturityReport,
    SchedulerScalabilityResult, StorageCostModel, StorageGrowthForecast, SyntheticDagProfile,
};
pub use planner::build_plan;
pub use planner_analysis::{
    build_backfill_plan, build_planner_analysis, build_replay_plan_annotations,
    compute_partial_run_closure, diff_plans, explain_plan, fingerprint_plan, PlannerBackfillPlan,
    PlannerBuildResult, PlannerExplainReport, PlannerGuardrails, PlannerNodeAction,
    PlannerNodeAnnotation, PlannerPhase, PlannerPlanDiff, PlannerPriorityInheritance,
    PlannerResourceEstimate,
};
pub use policy::policy_allows_effects;
pub use recovery::{
    check_run_consistency, detect_stuck_run, evaluate_pause_state, reconcile_orphaned_node,
    should_quarantine_run, validate_and_repair_run_metadata, BranchRecoveryMode,
    CheckpointResumeContract, ConsistencyCheckReport, DegradedExecutionPolicy, InterruptionClass,
    ManualInterventionRecord, NodeControlMode, NodeHeartbeatPolicy, OperatorRetryPolicy,
    PersistedRunSnapshotRef, RecoveryAcceptanceSuite, RecoveryFaultBoundary,
    RecoveryFaultInjection, RecoverySimulationScenario, ResilientLogRecord, ResumePolicy,
    RunPauseMode, RunPausePolicy, RunQuarantineRecord, RunRepairOutcome, SchedulerRecoveryAction,
    SchedulerRecoveryRule, StuckRunPolicy,
};
use registry::{build_registry, AdapterRegistry};
pub use remote_execution_model::{
    execution_mode_status, remote_handoff_valid, validate_remote_identity, ExecutionModeStatus,
    RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
};
pub use remote_executor::{
    RemoteExecutionReceipt, RemoteExecutionRequest, RemoteExecutorSubmitter,
};
pub use run_state::{
    imported_run_distinguishable, node_transition_invariant_id, run_transition_invariant_id,
    terminal_transition_audit_events, validate_node_transition, validate_run_transition,
    verify_post_run_state_consistency, NodeState, NodeTransition, ReplayNodeAction,
    ReplayNodeProvenance, RunAttempt, RunCompactionPolicy, RunComparison, RunId, RunSnapshot,
    RunState, RunSummaryV2, RunTransition, StateConsistencyReport, TransitionAuditEvent,
    TransitionCause, INV_NODE_TERMINAL_NO_REVERT, INV_RUN_FAILED_CAUSAL_FAILURE,
};
pub use runtime_semantics::*;
pub use scheduler::{
    build_scheduler, compile_submission_request, deterministic_tick_order, dry_run_schedule,
    failure_allows_downstream_readiness, failure_mode_name, scheduler_contract_profile,
    scheduler_debug_event_log, scheduler_invariants_hold, validate_cron_expression,
    validate_schedule_policy_combination, validate_schedule_registry, BackfillRequest,
    CatchUpPolicy, ConcurrencyPolicyLayers, DependencyCounter, DeterministicScheduler,
    ExecutionCheckpoint, ExecutionSubmissionRequest, FailurePropagationMode,
    NoopSchedulerEventHook, PriorityClass, QueueIdentity, QueueIsolationPolicy, ReadyQueue,
    ReadyTieBreak, ScheduleAuditRecord, ScheduleDefinition, ScheduleDryRunPreview,
    ScheduleRegistry, ScheduleSubmissionStatus, ScheduledSubmission, Scheduler,
    SchedulerContractProfile, SchedulerEvent, SchedulerEventHook, SchedulerEventKind,
    SchedulerFairness, SchedulerModel, SchedulerPolicy, SchedulerPriorityModel, SchedulerState,
    SchedulerUnit, ThroughputScheduler, TriggerSpec,
};
pub use scheduler_workload::{
    apply_backfill_throttling, compute_partition_backfill_batches, deduplicate_trigger_events,
    detect_cron_conflicts, evaluate_sla_metrics, is_suppressed_by_calendar, materialize_next_runs,
    run_batches, weighted_priority_tie_break_order, BackfillThrottlingPolicy, BlackoutWindow,
    ConcurrencyScope, CronConflict, CrossSchedulerCompatibility, DagCalendar,
    DependencyTriggerBufferPolicy, EnvironmentSuppression, FairnessAlgorithm, HolidayPolicy,
    MaterializedRunPreview, PartitionBackfillOrchestration, QueueAdmissionPolicy, RunBatchPolicy,
    ScheduleOverrideRecord, ScheduleSuppressionAnnotation, SchedulerAlertRule,
    SchedulerMaturityMatrix, SchedulerSlaMetrics, SchedulingSimulationSuite, ServiceClass,
    SlaPolicy, StarvationPreventionPolicy, TriggerDedupDecision, WeightedPriorityPolicy,
};
pub use secrets_security::{
    incident_response_actions, leak_conformance_check, redact_secret_payload, secret_readiness,
    secret_scope_allows, secure_cleanup_required, secure_mode_effective, select_secret_version,
    should_materialize_secret_artifact, summarize_sensitive_classes, taint_from_secret_usage,
    validate_secret_delivery_mode, SecretArtifactPolicy, SecretDeliveryPolicy, SecretInjectionMode,
    SecretIntegrationReadiness, SecretLeakIncident, SecretMaskingPolicy, SecretResolutionTiming,
    SecretRotationRule, SecretScopeRule, SecretSource, SecretTaintRecord, SecretUsageAuditEvent,
    SecretVersionSelection, SecureExecutionMode, SecureTeardownPolicy, SecureWorkspaceRule,
    SensitiveArtifactClass, SensitiveArtifactRestriction,
};
pub use security_env::{is_allowed_env_key, is_denied_env_key, shape_environment};
pub use semantic_lineage::{
    detect_lineage_conflicts, export_lineage_format, lineage_quality_score,
    policy_hook_allows_operation, recommended_replay_set, summarize_lineage,
    ArtifactRelationshipType, ArtifactSemanticTag, CrossRunLineageStitch, FieldLevelLineageHook,
    LineageConfidence, LineageConflict, LineageExportFormat, LineageImpactReport,
    LineageMaterializationRule, LineageQualityScore, LineageReconciliationPlan, LineageSummary,
    LineageSummaryNode, PolicyLineageHookInput, RetentionProtectionRule, ReverseImpactReport,
    SemanticDependencyClass, SemanticLineageExplain, SemanticRelationship,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::io::{self as std_io, Write};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub use store::{validate_storage_relative_path, ArtifactStore, CacheStore, StorageHealthReport};
use store::{ArtifactStore as RuntimeArtifactStore, CacheStore as RuntimeCacheStore};
pub use task_contract::{
    build_task_contract, default_forced_cleanup, validate_task_contracts,
    ForcedCancellationCleanup, IdempotencyMode, NodeProvenance, OutputMaterializationPolicy,
    RuntimeState, SideEffectClassification, TaskContract, TaskFailureReason, TaskInputDescriptor,
    TaskIsolationMode, TaskOutputDescriptor, TaskResultEnvelope, TimeoutPolicy,
};
pub use task_types::{
    check_replay_adapter_compatibility, compatibility_matrix_report,
    compatibility_score_for_contract, compute_task_contract_fingerprint,
    default_task_type_registry, generate_task_contract_markdown, validate_cross_node_compatibility,
    validate_parameter_defaults, AdapterCapabilityDeclaration, CollectionType, CompatibilityScore,
    NullabilityContract, OutputEvolutionMarker, PartitionCollectionContract,
    PolymorphicTaskContract, PolymorphicVariant, ResourceReference, ScalarType, SchemaReference,
    SecretReference, TaskCompatibilityMatrixReport, TaskCompatibilityRelationship,
    TaskContractDiagnostic, TaskContractFingerprint, TaskTypeRegistry, TypeCoercionRule,
    VersionedTypeRule,
};
pub use upgrade_compatibility::{
    build_compatibility_dashboard, classify_compatibility, evaluate_release_gate,
    simulate_migration_impact, validate_upgrade_path, CompatibilityAcceptanceSuite,
    CompatibilityClass, CompatibilityDashboard, CompatibilityPolicy, CompatibilityRule,
    CrossVersionMatrixRow, DeprecationDiagnostic, DowngradeRiskReport,
    DurableStateMigrationContract, FeatureFlagRecord, FeatureLifecycleState, LongTermSupportPolicy,
    ManifestMigrationPlan, MigrationImpactEstimate, PluginVersionWindow, ReleaseGateOutcome,
    SchedulerStateCompatibilityCheck, UpgradePathPolicy, UpgradeRolloutPlan,
};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("artifact error: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("io error: {0}")]
    Io(#[from] std_io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("executor error: {0}")]
    Executor(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Success,
    Failed,
    Skipped,
    Cached,
}

pub struct RunContext {
    pub run_dir: Arc<RunDir>,
    pub graph_fingerprint: Arc<Mutex<HashMap<String, String>>>,
    pub resolved_params: HashMap<String, Value>,
    pub fs: Arc<dyn Fs>,
    pub clock: Arc<dyn Clock>,
    pub store: RuntimeArtifactStore,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub status: NodeStatus,
    pub stdout_path: String,
    pub stderr_path: String,
    pub outputs_dir: String,
    pub failure: Option<FailureInfo>,
    pub attempts: u32,
    pub attempt_events: Vec<AttemptEvent>,
    pub container_meta: Option<bijux_dag_artifacts::ContainerTrace>,
    pub adapter_binary_sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttemptEvent {
    pub attempt: u32,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub status: NodeStatus,
}

#[derive(Clone)]
pub struct ConstAdapter;

impl Adapter for ConstAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "const".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["const".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet::default()
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let params = ctx.params;
        let node_dir = exec.run_dir.node_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs
            .create_dir_all(exec.run_dir.node_outputs_dir(&node.id).as_path())?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);

        let value = params.get("value").cloned().unwrap_or(Value::Null);
        let target = node
            .outputs
            .iter()
            .find(|o| o.name == "value")
            .or_else(|| node.outputs.first())
            .ok_or_else(|| RuntimeError::Executor("no outputs declared".to_string()))?;
        let out_path = outputs_dir.join(&target.path);
        if let Some(parent) = out_path.parent() {
            exec.fs.create_dir_all(parent)?;
        }
        exec.fs
            .write(&out_path, &serde_json::to_vec_pretty(&value)?)?;
        exec.fs.write(&stdout_path, b"")?;
        exec.fs.write(&stderr_path, b"")?;
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        let output_paths = declared_output_paths(node);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Clone)]
pub struct ShellAdapter;

impl Adapter for ShellAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "shell".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["shell".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: true,
            env: false,
            network: false,
            clock: false,
        }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let params = ctx.params;
        let argv = params
            .get("argv")
            .and_then(|v| v.as_array())
            .ok_or_else(|| RuntimeError::Executor("missing argv".to_string()))?;
        if argv.is_empty() {
            return Err(RuntimeError::Executor("empty argv".to_string()));
        }
        let mut args: Vec<String> = Vec::new();
        for v in argv {
            let s = v
                .as_str()
                .ok_or_else(|| RuntimeError::Executor("argv must be strings".to_string()))?;
            args.push(s.to_string());
        }

        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let mut cmd = subprocess::command(&args[0]);
        cmd.args(&args[1..]);
        cmd.current_dir(&work_dir);
        apply_shaped_env(&mut cmd, exec.policy.clean_env, &node.env_allowlist, &[]);

        let output =
            command_output_with_timeout(&mut cmd, effective_node_timeout_ms(node, params))?;

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_paths = declared_output_paths(node);
        if let Some(failure) = validate_outputs_dir(&outputs_dir, &node.outputs) {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        let success = output.status.success();
        let failure = if success {
            None
        } else {
            Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "command failed".to_string(),
                details: None,
            })
        };

        Ok(NodeResult {
            status: if success {
                NodeStatus::Success
            } else {
                NodeStatus::Failed
            },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Clone)]
pub struct ContainerAdapter;

impl Adapter for ContainerAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "container".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["container".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: true,
            env: false,
            network: false,
            clock: false,
        }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let spec = node
            .container
            .as_ref()
            .ok_or_else(|| RuntimeError::Executor("missing container spec".to_string()))?;

        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let engine = spec.engine.as_str();
        let engine_version = engine_version(engine);
        if engine_version.is_none() {
            exec.fs.write(&stdout_path, b"")?;
            exec.fs.write(
                &stderr_path,
                format!("container engine not available: {}", engine).as_bytes(),
            )?;
            return Ok(NodeResult {
                status: NodeStatus::Skipped,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: None,
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(spec, engine, None, engine_version)),
                adapter_binary_sha256: None,
            });
        }

        let mut cmd = subprocess::command(engine);
        cmd.arg("run").arg("--rm");

        if !node.effects.contains(&Effect::Network) || exec.policy.deny_network {
            cmd.args(["--network", "none"]);
        }

        cmd.args(["-v", &format!("{}:/bijux/node", node_dir.display())]);

        let workdir = spec
            .workdir
            .clone()
            .unwrap_or_else(|| "/bijux/node/work".to_string());
        cmd.args(["--workdir", &workdir]);

        for (key, val) in shaped_environment(exec.policy.clean_env, &spec.env_allowlist, &[]) {
            cmd.arg("-e").arg(format!("{}={}", key, val));
        }

        cmd.arg(&spec.image);
        for part in &spec.argv {
            cmd.arg(part);
        }

        let output =
            command_output_with_timeout(&mut cmd, effective_node_timeout_ms(node, &Value::Null))?;
        let exit_code = output.status.code();

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_paths = declared_output_paths(node);
        if let Some(failure) = validate_outputs_dir(&outputs_dir, &node.outputs) {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(
                    spec,
                    engine,
                    exit_code,
                    engine_version.clone(),
                )),
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        let success = output.status.success();
        let failure = if success {
            None
        } else {
            Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "container command failed".to_string(),
                details: Some(serde_json::json!({ "exit_code": exit_code })),
            })
        };

        Ok(NodeResult {
            status: if success {
                NodeStatus::Success
            } else {
                NodeStatus::Failed
            },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: Some(container_trace(spec, engine, exit_code, engine_version)),
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheMode {
    Off,
    Read,
    ReadWrite,
}

struct CacheRead {
    hit: bool,
    proof: Option<CacheProof>,
}

#[derive(Clone)]
pub struct RuntimeConfig {
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub run_timeout_ms: Option<u64>,
    pub node_timeout_ms: Option<u64>,
    pub materialize_inputs: MaterializeMode,
    pub cache_mode: CacheMode,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub parent_run_id: Option<String>,
    pub submission_source: String,
    pub trigger_source: String,
    pub operator: String,
    pub labels: Vec<String>,
    pub latest_symlink: Option<PathBuf>,
    pub policy: PolicyConfig,
    pub selectors: SelectorSet,
    pub partial_rerun_dependency_closure: bool,
    pub scheduler_policy: SchedulerPolicy,
    pub failure_propagation: FailurePropagationMode,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            jobs: 1,
            cpu_budget: None,
            run_timeout_ms: None,
            node_timeout_ms: None,
            materialize_inputs: MaterializeMode::Copy,
            cache_mode: CacheMode::Off,
            cache_dir: None,
            remote_cache_dir: None,
            run_id: None,
            parent_run_id: None,
            submission_source: "manual".to_string(),
            trigger_source: "cli".to_string(),
            operator: "unknown".to_string(),
            labels: Vec::new(),
            latest_symlink: None,
            policy: PolicyConfig::default(),
            selectors: SelectorSet::default(),
            partial_rerun_dependency_closure: true,
            scheduler_policy: SchedulerPolicy::default(),
            failure_propagation: FailurePropagationMode::IsolateBranch,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectorSet {
    pub include: Vec<Selector>,
    pub exclude: Vec<Selector>,
}

#[derive(Debug, Clone)]
pub enum Selector {
    IdPrefix(String),
    Tag(String),
    Kind(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializeMode {
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
        }
    }
}

pub struct Runtime {
    registry: AdapterRegistry,
    fs: Arc<dyn Fs>,
    clock: Arc<dyn Clock>,
    init_error: Option<String>,
}

impl Runtime {
    pub fn new() -> Self {
        let registry_result = build_registry(vec![
            Arc::new(ConstAdapter),
            Arc::new(ShellAdapter),
            Arc::new(ContainerAdapter),
        ]);
        let (registry, init_error) = match registry_result {
            Ok(reg) => (reg, None),
            Err(err) => (AdapterRegistry::new(), Some(err.to_string())),
        };
        Self {
            registry,
            fs: Arc::new(StdFs),
            clock: Arc::new(SystemClock),
            init_error,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_io(fs: Arc<dyn Fs>, clock: Arc<dyn Clock>) -> Self {
        let mut runtime = Self::new();
        runtime.fs = fs;
        runtime.clock = clock;
        runtime
    }

    fn adapter_for_kind(&self, kind: &NodeKind) -> Result<Arc<dyn Adapter>, RuntimeError> {
        self.registry.resolve(kind.as_str())
    }

    fn adapter_meta_for_kind(&self, kind: &NodeKind) -> (String, String) {
        self.registry
            .resolve(kind.as_str())
            .map(|a| {
                let id = a.id();
                (id.id, id.version)
            })
            .unwrap_or_else(|_| ("unknown".to_string(), "unknown".to_string()))
    }

    fn adapter_schema_for_kind(&self, kind: &NodeKind) -> String {
        self.registry
            .resolve(kind.as_str())
            .map(|a| a.produces_outputs_schema_version())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    pub fn run(
        &self,
        graph: &Graph,
        out_dir: impl AsRef<Path>,
        options: RuntimeConfig,
    ) -> Result<PathBuf, RuntimeError> {
        if let Some(err) = &self.init_error {
            return Err(RuntimeError::Executor(err.clone()));
        }
        let diags = graph.validate_with_warnings();
        if diags.iter().any(|d| d.severity == Severity::Error) {
            return Err(GraphError::ValidationFailed.into());
        }
        let _contracts = validate_task_contracts(graph, &options)?;
        let plan = build_plan(graph, &options);
        engine::execute(self, graph, plan, out_dir, options)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn write_trace(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    status: NodeStatus,
    failure: Option<FailureInfo>,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    attempt: u32,
    cache_proof: Option<CacheProof>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
    container_meta: Option<ContainerTrace>,
    adapter_binary_sha256: Option<String>,
    skip_reason: Option<bijux_dag_artifacts::SkipReason>,
    transition_cause: Option<String>,
    replay_provenance: Option<ReplayProvenance>,
) -> Result<(), RuntimeError> {
    let node = graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?;
    ctx.store.ensure_node_dir(node_id)?;
    write_resolved_params(ctx, node_id)?;
    let inputs_index = if ctx
        .fs
        .metadata(ctx.run_dir.node_inputs_index_path(node_id).as_path())
        .is_ok()
    {
        Some("inputs/index.json".to_string())
    } else {
        None
    };
    let trace = NodeTrace {
        node_id: node_id.to_string(),
        status: status_string(&status),
        started_unix_ms,
        finished_unix_ms,
        attempt,
        fingerprint: node_fingerprint_from_ctx(ctx, node_id),
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        adapter_outputs_schema_version: adapter_outputs_schema_version.to_string(),
        adapter_binary_sha256,
        resources: node.resources.as_ref().map(|r| TraceResources {
            cpu: r.cpu,
            mem_mb: r.mem_mb,
        }),
        inputs_index,
        resolved_params: ctx.resolved_params.get(node_id).cloned(),
        container: container_meta,
        cache_proof,
        skip_reason,
        failure,
        transition_cause,
        replay_provenance,
    };
    let data = serde_json::to_vec_pretty(&trace)?;
    ctx.store.write_trace(node_id, &data)?;
    Ok(())
}

fn status_string(status: &NodeStatus) -> String {
    match status {
        NodeStatus::Success => "success".to_string(),
        NodeStatus::Failed => "failed".to_string(),
        NodeStatus::Skipped => "skipped".to_string(),
        NodeStatus::Cached => "cached".to_string(),
    }
}

pub(crate) fn transition_cause_for_status(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Success => "ExecutionSucceeded",
        NodeStatus::Failed => "ExecutionFailed",
        NodeStatus::Skipped => "SelectionFiltered",
        NodeStatus::Cached => "CachedReuse",
    }
}

fn write_resolved_params(ctx: &RunContext, node_id: &str) -> Result<(), RuntimeError> {
    let mut params = ctx
        .resolved_params
        .get(node_id)
        .cloned()
        .unwrap_or(Value::Null);
    sort_value_maps(&mut params);
    let data = serde_json::to_vec_pretty(&params)?;
    ctx.store.write_resolved_params(node_id, &data)?;
    Ok(())
}

#[allow(dead_code)]
fn node_timeout_ms(
    node: &Node,
    resolved_params: &Value,
    default_ms: Option<u64>,
) -> Option<Duration> {
    let param_timeout = resolved_params.get("timeout_ms").and_then(|v| v.as_u64());
    let ms = node.timeout_ms.or(param_timeout).or(default_ms);
    ms.map(Duration::from_millis)
}

fn node_cpu(graph: &Graph, node_id: &str) -> u32 {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .and_then(|n| n.resources.as_ref().map(|r| r.cpu))
        .unwrap_or(1)
        .max(1)
}

fn execute_with_retries(
    adapter: &dyn Adapter,
    node: &Node,
    params: &Value,
    ctx: &RunContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    let mut attempt = 0u32;
    let max = retry.max_attempts;
    let mut attempt_events = Vec::new();
    loop {
        attempt += 1;
        let started = ctx.clock.now_unix_ms();
        let node_ctx = NodeCtx {
            node,
            exec: ctx,
            params,
        };
        let mut result = adapter.execute(&node_ctx)?;
        let finished = ctx.clock.now_unix_ms();
        attempt_events.push(AttemptEvent {
            attempt,
            started_unix_ms: started,
            finished_unix_ms: finished,
            status: result.status.clone(),
        });
        result.attempts = attempt;
        if result.status != NodeStatus::Failed {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        if attempt > max {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        if retry.backoff_ms > 0 {
            let wait = retry
                .backoff_ms
                .saturating_mul(attempt.saturating_sub(1) as u64);
            if wait > 0 {
                std::thread::sleep(Duration::from_millis(wait));
            }
        }
    }
}

fn append_event(file: &mut std::fs::File, value: serde_json::Value) -> Result<(), RuntimeError> {
    let line = serde_json::to_string(&value)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

fn cache_mode_string(mode: &CacheMode) -> Option<String> {
    match mode {
        CacheMode::Off => None,
        CacheMode::Read => Some("read".to_string()),
        CacheMode::ReadWrite => Some("readwrite".to_string()),
    }
}

fn tool_version() -> String {
    let base = env!("CARGO_PKG_VERSION");
    if let Ok(out) = subprocess::output("git", &["rev-parse", "--short", "HEAD"]) {
        if out.status.success() {
            let commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !commit.is_empty() {
                return format!("{}+{}", base, commit);
            }
        }
    }
    base.to_string()
}

pub fn registered_adapters() -> Vec<AdapterInfo> {
    let registry = build_registry(vec![
        Arc::new(ConstAdapter),
        Arc::new(ShellAdapter),
        Arc::new(ContainerAdapter),
    ])
    .unwrap_or_else(|_| AdapterRegistry::new());
    registry.list()
}

pub fn adapter_registry_dump() -> serde_json::Value {
    let adapters = registered_adapters();
    serde_json::json!({
        "count": adapters.len(),
        "adapters": adapters
    })
}

fn materialize_inputs(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    mode: MaterializeMode,
) -> Result<InputsIndex, RuntimeError> {
    let inputs_dir = ctx.run_dir.node_inputs_dir(node_id);
    ctx.fs.create_dir_all(&inputs_dir)?;
    let mut files = Vec::new();
    for edge in &graph.edges {
        if edge.to.node_id != node_id {
            continue;
        }
        let from_node = graph
            .nodes
            .iter()
            .find(|n| n.id == edge.from.node_id)
            .ok_or_else(|| RuntimeError::Executor("missing source node".to_string()))?;
        let out = from_node
            .outputs
            .iter()
            .find(|o| o.name == edge.from.port)
            .ok_or_else(|| RuntimeError::Executor("missing output port".to_string()))?;
        let src_path = ctx
            .run_dir
            .node_outputs_dir(&edge.from.node_id)
            .join(&out.path);
        let dst_dir = inputs_dir.join(&edge.from.node_id);
        ctx.fs.create_dir_all(&dst_dir)?;
        let dst_path = dst_dir.join(&edge.to.port);
        if let Some(parent) = dst_path.parent() {
            ctx.fs.create_dir_all(parent)?;
        }
        if ctx.fs.metadata(&src_path).is_ok() {
            materialize_file(ctx.fs.as_ref(), &src_path, &dst_path, mode)?;
            let data = ctx.fs.read(&dst_path)?;
            let sha = sha256_bytes(&data);
            let rel = dst_path.strip_prefix(&inputs_dir).unwrap_or(&dst_path);
            let rel_str = rel.to_string_lossy().to_string();
            let from_fp = node_fingerprint_from_ctx(ctx, &edge.from.node_id);
            files.push(InputFile {
                path: rel_str,
                sha256: sha,
                from_node: edge.from.node_id.clone(),
                from_node_fingerprint: from_fp,
                from_output: edge.from.port.clone(),
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let index = InputsIndex { files };
    write_inputs_index(&inputs_dir, &index)?;
    Ok(index)
}

fn cache_dir_from_env() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

pub(crate) fn declared_output_paths(node: &Node) -> Vec<String> {
    node.outputs.iter().map(|o| o.path.clone()).collect()
}

#[allow(clippy::too_many_arguments)]
fn try_cache_read(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<CacheRead, RuntimeError> {
    if options.cache_mode == CacheMode::Off {
        return Ok(CacheRead {
            hit: false,
            proof: None,
        });
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let cache_store = match cache_dir {
        Some(d) => Some(RuntimeCacheStore::new(d, Arc::clone(&fs))),
        None => {
            return Ok(CacheRead {
                hit: false,
                proof: None,
            })
        }
    };
    if options.cache_mode == CacheMode::Read || options.cache_mode == CacheMode::ReadWrite {
        let key = node_fingerprint.to_string();
        let store = cache_store.as_ref().unwrap();
        let entry = store.entry(&key);
        if store.fs().metadata(&entry).is_ok() {
            if !verify_cache_entry(
                store.fs(),
                &entry,
                &key,
                adapter_id,
                adapter_version,
                adapter_outputs_schema_version,
            )? {
                return Ok(CacheRead {
                    hit: false,
                    proof: Some(CacheProof {
                        hit: false,
                        key,
                        source: "local".to_string(),
                        verified: false,
                        reason: "corrupt".to_string(),
                        corrupt_detected: true,
                    }),
                });
            }
            let source =
                cache_source_from_meta(store.fs(), &entry).unwrap_or_else(|| "local".to_string());
            let node_dir = ctx.run_dir.node_dir(&node.id);
            store.fs().create_dir_all(&node_dir)?;
            copy_dir_all(
                store.fs(),
                entry.join("outputs"),
                ctx.run_dir.node_outputs_dir(&node.id),
            )?;
            copy_dir_all(store.fs(), entry.join("logs"), node_dir.clone())?;
            return Ok(CacheRead {
                hit: true,
                proof: Some(CacheProof {
                    hit: true,
                    key,
                    source,
                    verified: true,
                    reason: "hit".to_string(),
                    corrupt_detected: false,
                }),
            });
        }
        if let Some(remote_dir) = options.remote_cache_dir.as_ref() {
            let remote_entry = remote_dir.join(&key);
            if store.fs().metadata(&remote_entry).is_ok() {
                if !verify_cache_entry(
                    store.fs(),
                    &remote_entry,
                    &key,
                    adapter_id,
                    adapter_version,
                    adapter_outputs_schema_version,
                )? {
                    return Ok(CacheRead {
                        hit: false,
                        proof: Some(CacheProof {
                            hit: false,
                            key,
                            source: "remote".to_string(),
                            verified: false,
                            reason: "remote_corrupt".to_string(),
                            corrupt_detected: true,
                        }),
                    });
                }
                let node_dir = ctx.run_dir.node_dir(&node.id);
                store.fs().create_dir_all(&node_dir)?;
                copy_dir_all(
                    store.fs(),
                    remote_entry.join("outputs"),
                    ctx.run_dir.node_outputs_dir(&node.id),
                )?;
                copy_dir_all(store.fs(), remote_entry.join("logs"), node_dir.clone())?;
                if let Some(local_dir) = options.cache_dir.as_ref() {
                    let local_entry = local_dir.join(&key);
                    let _ = copy_dir_all(store.fs(), &remote_entry, &local_entry);
                }
                return Ok(CacheRead {
                    hit: true,
                    proof: Some(CacheProof {
                        hit: true,
                        key,
                        source: "remote".to_string(),
                        verified: true,
                        reason: format!("fetched:{}", cache_dir_id(remote_dir)),
                        corrupt_detected: false,
                    }),
                });
            }
        }
    }
    Ok(CacheRead {
        hit: false,
        proof: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn try_cache_write(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<(), RuntimeError> {
    if options.cache_mode != CacheMode::ReadWrite {
        return Ok(());
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let store = match cache_dir {
        Some(d) => RuntimeCacheStore::new(d, Arc::clone(&fs)),
        None => return Ok(()),
    };
    let key = node_fingerprint.to_string();
    let entry = store.entry(&key);
    store.fs().create_dir_all(entry.join("outputs").as_path())?;
    store.fs().create_dir_all(entry.join("logs").as_path())?;
    let meta = serde_json::json!({
        "node_id": node.id,
        "node_fingerprint": key,
        "adapter_id": adapter_id,
        "adapter_version": adapter_version,
        "produces_outputs_schema_version": adapter_outputs_schema_version,
        "created_unix_ms": ctx.clock.now_unix_ms(),
        "cache_source": "local",
        "schema_version": "v0.1",
    });
    store.fs().write(
        entry.join("meta.json").as_path(),
        &serde_json::to_vec_pretty(&meta)?,
    )?;
    copy_dir_all(
        store.fs(),
        ctx.run_dir.node_outputs_dir(&node.id),
        entry.join("outputs"),
    )?;
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let _ = store.fs().copy(
        node_dir.join("stdout.log").as_path(),
        entry.join("logs").join("stdout.log").as_path(),
    );
    let _ = store.fs().copy(
        node_dir.join("stderr.log").as_path(),
        entry.join("logs").join("stderr.log").as_path(),
    );
    let _ = store.fs().copy(
        node_dir.join("trace.json").as_path(),
        entry.join("logs").join("trace.json").as_path(),
    );
    Ok(())
}

fn verify_cache_entry(
    fs: &dyn Fs,
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<bool, RuntimeError> {
    let index_path = entry.join("outputs").join("index.json");
    if fs.metadata(&index_path).is_err() {
        return Ok(false);
    }
    let meta_path = entry.join("meta.json");
    if fs.metadata(&meta_path).is_err() {
        return Ok(false);
    }
    let meta: serde_json::Value = serde_json::from_str(&fs.read_to_string(&meta_path)?)?;
    if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(expected_key) {
        return Ok(false);
    }
    if meta.get("adapter_id").and_then(|v| v.as_str()) != Some(adapter_id) {
        return Ok(false);
    }
    if meta.get("adapter_version").and_then(|v| v.as_str()) != Some(adapter_version) {
        return Ok(false);
    }
    if meta
        .get("produces_outputs_schema_version")
        .and_then(|v| v.as_str())
        != Some(adapter_outputs_schema_version)
    {
        return Ok(false);
    }
    let data = fs.read_to_string(&index_path)?;
    let index: OutputsIndex = serde_json::from_str(&data)?;
    for file in index.files {
        let path = entry.join("outputs").join(&file.path);
        if fs.metadata(&path).is_err() {
            return Ok(false);
        }
        let bytes = fs.read(&path)?;
        let sha = sha256_bytes(&bytes);
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

fn node_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.graph_fingerprint
        .lock()
        .ok()
        .and_then(|map| map.get(node_id).cloned())
        .unwrap_or_default()
}

fn set_node_fingerprint(ctx: &RunContext, node_id: &str, fp: String) {
    if let Ok(mut map) = ctx.graph_fingerprint.lock() {
        map.insert(node_id.to_string(), fp);
    }
}

fn node_fingerprint_with_inputs(
    base_fp: &str,
    inputs: &InputsIndex,
) -> Result<String, RuntimeError> {
    let value = serde_json::json!({
        "base": base_fp,
        "inputs": &inputs.files,
    });
    Ok(sha256_bytes(&serde_json::to_vec_pretty(&value)?))
}

fn cache_dir_id(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn cache_source_from_meta(fs: &dyn Fs, entry: &Path) -> Option<String> {
    let meta_path = entry.join("meta.json");
    let data = fs.read_to_string(&meta_path).ok()?;
    let meta: serde_json::Value = serde_json::from_str(&data).ok()?;
    meta.get("cache_source")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn sort_value_maps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut sorted: BTreeMap<String, Value> = BTreeMap::new();
            let entries = std::mem::take(map);
            for (k, mut v) in entries {
                sort_value_maps(&mut v);
                sorted.insert(k, v);
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in sorted {
                new_map.insert(k, v);
            }
            *map = new_map;
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                sort_value_maps(v);
            }
        }
        _ => {}
    }
}

pub(crate) fn validate_outputs_dir(dir: &Path, outputs: &[FileOutput]) -> Option<FailureInfo> {
    let mut declared = std::collections::BTreeSet::new();
    for out in outputs {
        declared.insert(out.path.replace('\\', "/"));
    }

    for out in outputs {
        let schema = ArtifactSchemaDescriptor {
            name: "bijux.output.file".to_string(),
            version: "v0.1".to_string(),
            media_type: "application/octet-stream".to_string(),
            encoding: "identity".to_string(),
            validation_mode: SchemaValidationMode::Strict,
        };
        if let Err(message) = validate_output_schema_descriptor(&schema) {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_SCHEMA_INVALID".to_string(),
                message,
                details: None,
            });
        }
        if out.path.contains("..") || out.path.starts_with('/') || out.path.starts_with('\\') {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_PATH_INVALID".to_string(),
                message: "invalid output path".to_string(),
                details: None,
            });
        }
        let path = dir.join(&out.path);
        if has_symlink_component(dir, Path::new(&out.path)) {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_PATH_INVALID".to_string(),
                message: format!("output path traverses symlink: {}", out.path),
                details: None,
            });
        }
        if !path.exists() {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_MISSING".to_string(),
                message: format!("missing output file: {}", out.path),
                details: None,
            });
        }
        if path.is_dir() || path.is_symlink() {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_PATH_INVALID".to_string(),
                message: format!("output must be a file: {}", out.path),
                details: None,
            });
        }
    }

    let mut actual = std::collections::BTreeSet::new();
    collect_relative_files(dir, dir, &mut actual);
    for rel in actual {
        if !declared.contains(&rel) {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_UNDECLARED".to_string(),
                message: format!("undeclared output file: {}", rel),
                details: None,
            });
        }
    }
    None
}

fn has_symlink_component(root: &Path, relative: &Path) -> bool {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if std::fs::symlink_metadata(&current)
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn collect_relative_files(
    root: &Path,
    current: &Path,
    out: &mut std::collections::BTreeSet<String>,
) {
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if std::fs::symlink_metadata(&path)
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false)
        {
            continue;
        }
        if path.is_dir() {
            collect_relative_files(root, &path, out);
            continue;
        }
        if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.insert(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

fn apply_shaped_env(
    cmd: &mut std::process::Command,
    clean_env: bool,
    allowlist: &[String],
    denylist: &[String],
) {
    cmd.env_clear();
    for (key, value) in shaped_environment(clean_env, allowlist, denylist) {
        cmd.env(key, value);
    }
}

pub(crate) fn shaped_environment(
    clean_env: bool,
    allowlist: &[String],
    denylist: &[String],
) -> BTreeMap<String, String> {
    let ambient: BTreeMap<String, String> = std::env::vars().collect();
    let mut explicit = BTreeMap::new();
    if clean_env && !allowlist.is_empty() {
        for (key, value) in &ambient {
            if is_allowed_env_key(key, allowlist) {
                explicit.insert(key.clone(), value.clone());
            }
        }
    }
    shape_environment(&ambient, clean_env, allowlist, denylist, &explicit)
}

pub(crate) fn command_output_with_timeout(
    cmd: &mut std::process::Command,
    timeout_ms: Option<u64>,
) -> Result<Output, RuntimeError> {
    let Some(limit_ms) = timeout_ms else {
        return cmd.output().map_err(RuntimeError::Io);
    };
    let mut child = cmd.spawn().map_err(RuntimeError::Io)?;
    let start = std::time::Instant::now();
    loop {
        if let Some(_status) = child.try_wait().map_err(RuntimeError::Io)? {
            return child.wait_with_output().map_err(RuntimeError::Io);
        }
        if start.elapsed().as_millis() > limit_ms as u128 {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RuntimeError::Executor(format!(
                "execution timed out after {limit_ms}ms"
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn effective_node_timeout_ms(node: &Node, params: &Value) -> Option<u64> {
    node.timeout_ms
        .or_else(|| params.get("timeout_ms").and_then(|v| v.as_u64()))
}

fn container_trace(
    spec: &bijux_dag_core::ContainerSpec,
    engine: &str,
    exit_code: Option<i32>,
    engine_version: Option<String>,
) -> ContainerTrace {
    let image_digest = subprocess::output(
        engine,
        &["image", "inspect", "--format", "{{.Id}}", &spec.image],
    )
    .ok()
    .and_then(|out| {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        } else {
            None
        }
    });
    ContainerTrace {
        image: spec.image.clone(),
        image_digest,
        engine: engine.to_string(),
        engine_version,
        exit_code,
    }
}

fn engine_version(engine: &str) -> Option<String> {
    subprocess::output(engine, &["--version"])
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            } else {
                None
            }
        })
}

fn collect_outputs_summary(
    fs: &dyn Fs,
    run_dir: &RunDir,
) -> Result<Vec<OutputSummary>, RuntimeError> {
    let mut out = Vec::new();
    let nodes_dir = run_dir.staging_path().join("nodes");
    if fs.metadata(&nodes_dir).is_ok() {
        for entry in fs.read_dir(&nodes_dir)? {
            let index_path = entry.path().join("outputs").join("index.json");
            if fs.metadata(&index_path).is_ok() {
                let data = fs.read_to_string(&index_path)?;
                let index: OutputsIndex = serde_json::from_str(&data)?;
                for f in index.files {
                    out.push(OutputSummary {
                        node_id: f.node_id,
                        node_fingerprint: f.node_fingerprint,
                        file: f.path,
                        sha256: f.sha256,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| {
        (a.node_id.clone(), a.file.clone()).cmp(&(b.node_id.clone(), b.file.clone()))
    });
    Ok(out)
}

fn build_run_outputs_index(
    run_dir: &RunDir,
    outputs: &[OutputSummary],
) -> Result<RunOutputsIndex, RuntimeError> {
    let mut files = Vec::new();
    for out in outputs {
        let rel = run_dir.node_output_relpath(&out.node_id, &out.file);
        files.push(RunOutputFile {
            node_id: out.node_id.clone(),
            node_fingerprint: out.node_fingerprint.clone(),
            sha256: out.sha256.clone(),
            path: rel,
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(RunOutputsIndex { files })
}

fn rustc_version() -> String {
    if let Ok(out) = subprocess::output("rustc", &["--version"]) {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

fn count_nodes(status_map: &HashMap<String, NodeStatus>) -> NodeCounts {
    let mut counts = NodeCounts {
        success: 0,
        failed: 0,
        skipped: 0,
        cached: 0,
    };
    for status in status_map.values() {
        match status {
            NodeStatus::Success => counts.success += 1,
            NodeStatus::Failed => counts.failed += 1,
            NodeStatus::Skipped => counts.skipped += 1,
            NodeStatus::Cached => counts.cached += 1,
        }
    }
    counts
}

fn copy_dir_all(fs: &dyn Fs, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std_io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs.create_dir_all(dst)?;
    for entry in fs.read_dir(src)? {
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(fs, entry.path(), dst_path)?;
        } else {
            let _ = fs.copy(entry.path().as_path(), dst_path.as_path())?;
        }
    }
    Ok(())
}

fn materialize_file(
    fs: &dyn Fs,
    src: &Path,
    dst: &Path,
    mode: MaterializeMode,
) -> std_io::Result<()> {
    if fs.metadata(dst).is_ok() {
        let _ = fs.remove_file(dst);
    }
    match mode {
        MaterializeMode::Copy => {
            let _ = fs.copy(src, dst)?;
        }
        MaterializeMode::Hardlink => {
            if fs.hard_link(src, dst).is_err() {
                let _ = fs.copy(src, dst)?;
            }
        }
        MaterializeMode::Symlink => {
            if fs.symlink(src, dst).is_err() {
                let _ = fs.copy(src, dst)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
include!("internal/testing/tests_runtime.in.rs");
