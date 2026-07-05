//! Execution, replay, scheduling, and policy surfaces for Bijux DAG runs.
//!
//! Prefer the crate root for focused imports, [`stable`] for the explicit
//! long-lived runtime surface, and [`prelude`] for the common execution
//! workflow. The `experimental-public-api` feature enables opt-in runtime
//! contract material that is intentionally excluded from the default docs lane.
//!
#![allow(dead_code)]

#[path = "adapters/adapter.rs"]
mod adapter;
#[doc(hidden)]
#[path = "adapters/api.rs"]
pub mod adapter_api;
#[doc(hidden)]
#[path = "adapters/conformance.rs"]
pub mod adapter_conformance;
#[cfg(test)]
#[path = "internal/testing/adapter_contract_tests.rs"]
mod adapter_contract_tests;
#[path = "adapters/sdk.rs"]
mod adapter_sdk;
#[doc(hidden)]
pub mod adapters;
#[path = "internal/analysis/adaptive_scheduler.rs"]
mod adaptive_scheduler;
#[path = "internal/workflow/ai_operator_assist.rs"]
mod ai_operator_assist;
#[doc(hidden)]
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
#[doc(hidden)]
pub mod builtins;
#[doc(hidden)]
pub mod cache;
#[path = "internal/control/clock.rs"]
mod clock;
#[doc(hidden)]
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
#[doc(hidden)]
#[path = "runtime_core/execution/flow.rs"]
pub mod execution;
#[path = "backend/runtime/execution_backend.rs"]
mod execution_backend;
#[doc(hidden)]
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
#[doc(hidden)]
#[path = "runtime_core/governance/invariants.rs"]
pub mod invariants;
#[cfg(test)]
#[path = "internal/testing/invariants_tests.rs"]
mod invariants_tests;
#[path = "internal/control/io.rs"]
mod io;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/iteration06_contracts.rs"]
mod iteration06_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/iteration09_contracts.rs"]
mod iteration09_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/planning/iteration13_contracts.rs"]
mod iteration13_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/iteration14_contracts.rs"]
mod iteration14_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/iteration15_contracts.rs"]
mod iteration15_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/iteration17_contracts.rs"]
mod iteration17_contracts;
#[path = "backend/runtime/local_executor.rs"]
mod local_executor;
#[doc(hidden)]
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
#[path = "runtime_core/planning/path_resolution.rs"]
mod path_resolution;
#[path = "internal/perf/performance_capacity.rs"]
mod performance_capacity;
#[path = "runtime_core/planning/planner.rs"]
mod planner;
#[path = "runtime_core/planning/planner_analysis.rs"]
mod planner_analysis;
#[doc(hidden)]
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
#[doc(hidden)]
#[path = "runtime_core/execution/run_context.rs"]
pub mod run_context;
#[path = "runtime_core/execution/run_state.rs"]
mod run_state;
#[path = "internal/control/runtime.rs"]
mod runtime;
#[cfg(test)]
#[path = "internal/testing/runtime_boundary_tests.rs"]
mod runtime_boundary_tests;
#[path = "internal/control/runtime_controls.rs"]
mod runtime_controls;
#[doc(hidden)]
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
#[doc(hidden)]
#[path = "internal/control/selectors.rs"]
pub mod selectors;
#[path = "artifacts/storage/semantic_lineage.rs"]
mod semantic_lineage;
#[doc(hidden)]
#[path = "internal/control/services.rs"]
pub mod services;
pub mod simulated_platform;
#[doc(hidden)]
#[path = "runtime_core/execution/state_machine.rs"]
pub mod state_machine;
#[cfg(test)]
#[path = "internal/testing/state_machine_tests.rs"]
mod state_machine_tests;
#[path = "artifacts/storage/store.rs"]
mod store;
#[doc(hidden)]
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
#[doc(hidden)]
#[path = "artifacts/storage/trace.rs"]
pub mod trace;
#[path = "artifacts/storage/upgrade_compatibility.rs"]
mod upgrade_compatibility;
#[path = "internal/workflow/workflow_product.rs"]
mod workflow_product;
use adapter::{Adapter, AdapterId, EffectSet, NodeCtx};
pub use adapter::{AdapterDescriptor, CacheCompatibilityMode};
pub use adapter_conformance::{
    build_adapter_conformance_suite, generate_adapter_reference_markdown,
    validate_output_schema_compatibility, AdapterConformanceSuiteReport,
    AdapterOutputSchemaCompatibilityReport, AdapterReferenceDocument, AdapterScenarioResult,
    AdapterScenarioStatus,
};
pub use adapter_sdk::{
    AdapterCapabilities, AdapterContext, AdapterPlugin, BackendPlugin, PluginManifest,
};
pub use async_adapter::AsyncAdapter;
pub use backend::fake::{
    fake_batch_backend_reference, fake_batch_executor_contract, FakeBatchExecutor,
    FakeBatchExecutorContract, FakeBatchJobRecord, FakeBatchJobStatus,
};
pub use backend_cluster::{
    artifact_collection_state, backend_ready_for_admission, canonical_k8s_terminal_events,
    capture_hpc_scheduler_version, classify_hpc_failure, classify_k8s_failure,
    effective_hpc_retry_policy, equivalent_to_local, hpc_array_job_supported,
    hpc_environment_fingerprint, hpc_log_collection_semantics, hpc_poll_response_recovered,
    hpc_replay_fidelity_from_module_fingerprints, hpc_resource_fingerprint,
    hpc_scratch_staging_semantics, k8s_capability_declaration, kubernetes_adapter_contract,
    map_node_policy_to_k8s_job, map_node_resources_to_k8s, map_node_to_hpc_queue_partition,
    map_timeout_to_hpc_walltime, matches_placement_policy, normalize_backend_failure,
    outputs_logs_equivalent, quota_saturation_percent, reconcile_k8s_watch_stream,
    reject_unsupported_hpc_scheduler_features, reject_unsupported_k8s_fields,
    replay_allowed_across_backends, scratch_retention_required, slurm_adapter_design_contract,
    staged_input_cleanup_required, validate_k8s_injection, workdir_semantics,
    AdapterExecutionOutcome, ArtifactCollectionState, BackendCapabilityDescriptor,
    BackendCleanupGuarantee, BackendConformanceSuite, BackendFailureMappingRule,
    BackendLogCollectionContract, BackendMaintenanceMode, BackendOutageSimulationFixture,
    BackendProductionReadinessChecklist, BackendQuotaMetrics, BackendReadinessProbe,
    CrossBackendReplayRule, GenericBatchExecutorContract, HpcFailureClassification,
    HpcLogCollectionSemantics, HpcNodeExecutionContract, HpcQueuePartitionMapping,
    HpcReplayFidelity, HpcResourceFingerprintInput, HpcRetryPolicyDecision,
    HpcSchedulerVersionMetadata, HpcScratchStagingSemantics, ImageResolutionProvenance,
    K8sBackendVersionMetadata, K8sCapabilityDeclaration, K8sFailureClass, K8sInjectionAvailability,
    K8sInjectionRequest, K8sJobPolicyMapping, K8sResourceMapping, K8sResourceRequest,
    K8sWatchEvent, KubernetesAdapterContractReport, KubernetesExecutorContractV2, NodeAffinityHint,
    NodeExecutionContract, PlacementPolicyRule, QueueBackendRoutingPolicy,
    RemoteArtifactStagingProtocol, SlurmAdapterDesignContractReport, SlurmExecutorContract,
    WorkdirSemantics, WorkdirVolumeKind,
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
    sha256_artifact_path, write_inputs_index, write_outputs_index, AdapterInfo, ArtifactError,
    CacheProof, ContainerTrace, DeclaredOutputArtifact, FailureInfo, InputFile, InputsIndex,
    NodeCounts, NodeTrace, OutputSummary, OutputsIndex, ReplayProvenance,
    Resources as TraceResources, RunDir, RunOutputFile, RunOutputsIndex, TraceOutputArtifact,
    TriggerEvaluation,
};
use bijux_dag_core::{
    Effect, FileOutput, Graph, GraphError, Node, NodeKind, OutputKind, OutputSpec, RetryPolicy,
    Severity,
};
pub use cache::{
    cache_entry_has_required_proof, cache_key_explanation, cache_metadata_version_supported,
    CacheKeyInput,
};
use clock::{Clock, SystemClock};
pub use container_execution::{
    container_engine_discovery, container_env_isolated, container_network_policy_args,
    container_volume_contract, map_local_path_to_container, supported_container_engines,
    validate_container_contract, validate_container_mount_contract,
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
pub use execution_context::{ExecutionContext, NodeExecutionContext};
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
pub use external_adapter::{
    probe_external_adapters, ExternalAdapterHandshakeReport, ExternalAdapterHandshakeStatus,
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
    negotiate_backend_capabilities, BackendCapabilities as InfrastructureBackendCapabilities,
    BackendCapabilityRequirement, BackendExecutionCompletion, BackendExecutionRequest,
    CapabilityDecision, ExecutorBackend,
};
pub use invariants::{
    run_summary_invariant_ok, terminal_run_has_terminal_node, trace_time_order_ok, RunNodeCounts,
    INVARIANT_REGISTRY,
};
use io::{Fs, StdFs};
pub use local_executor::LocalExecutor;
pub use observability::{
    category_from_runtime_event_name, current_process_memory_bytes,
    event_contains_sensitive_material, event_names_emitted_once, reconstruct_timeline_from_events,
    required_event_fields_present, summarize_failure_root_causes, validate_required_event_names,
    verify_event_log_completeness, write_timeline_export, EventCategory,
    EventLogCompletenessReport, EventRecord, EventSink, FileEventSink, InMemoryMetricsRegistry,
    MetricsRegistry, NodeMetrics, RemoteCollectorSink, RunMetrics, SchedulerMetrics, SpanKind,
    StdoutEventSink, TimelineEntry, TimelineExport, TraceSpan, REQUIRED_RUNTIME_EVENT_NAMES,
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
pub use path_resolution::AbsolutePathPolicy;
pub(crate) use path_resolution::{
    bind_path_variables_in_value, collect_container_argv_path_usages,
    collect_container_workdir_usage, collect_resolved_path_usages, resolve_container_argv,
    resolve_container_workdir, NodePathBindings, ResolvedPathUsage,
};
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
    compute_downstream_run_closure, compute_partial_run_closure, compute_upstream_run_closure,
    diff_plans, explain_plan, fingerprint_plan, PlannerBackfillPlan, PlannerBuildResult,
    PlannerExplainReport, PlannerGuardrails, PlannerNodeAction, PlannerNodeAnnotation,
    PlannerNodePathPreview, PlannerPhase, PlannerPlanDiff, PlannerPriorityInheritance,
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
    verify_post_run_state_consistency, NodeState, NodeTransition, PartialRerunContract,
    ReplayNodeAction, ReplayNodeProvenance, RunAttempt, RunCompactionPolicy, RunComparison, RunId,
    RunSnapshot, RunState, RunSummaryV2, RunTransition, StateConsistencyReport,
    TransitionAuditEvent, TransitionCause, INV_NODE_TERMINAL_NO_REVERT,
    INV_RUN_FAILED_CAUSAL_FAILURE,
};
pub use runtime_controls::{
    audit_dispatch_discipline, audit_run_event_log, build_cancellation_audit_report,
    build_execution_isolation_report, build_heartbeat_audit_report,
    build_manual_intervention_audit_report, build_pause_resume_audit_report,
    build_policy_enforcement_report, build_retry_decision_report, build_timeout_audit_report,
    build_transition_audit_report, CancellationAuditReport, DispatchAuditReport, DispatchKeyRecord,
    EventLogAuditReport, ExecutionIsolationNodeReport, ExecutionIsolationReport,
    HeartbeatAuditReport, ManualInterventionAuditReport, PauseResumeAuditReport,
    PolicyEnforcementReport, PolicyEnforcementSurfaceReport, PolicyGuardSemanticsReport,
    RetryDecisionReport, TimeoutAuditReport, TransitionAuditReport,
};
pub use runtime_semantics::*;
pub use scheduler::{
    build_scheduler, compile_submission_request, deterministic_tick_order, dry_run_schedule,
    failure_allows_downstream_readiness, failure_mode_name, replay_scheduler_checkpoint,
    scheduler_contract_profile, scheduler_debug_event_log, scheduler_invariant_violations,
    scheduler_invariants_hold, validate_cron_expression, validate_schedule_policy_combination,
    validate_schedule_registry, BackfillRequest, CatchUpPolicy, ConcurrencyPolicyLayers,
    DependencyCounter, DeterministicScheduler, ExecutionCheckpoint, ExecutionSubmissionRequest,
    FailurePropagationMode, NoopSchedulerEventHook, PriorityClass, QueueIdentity,
    QueueIsolationPolicy, ReadyQueue, ReadyTieBreak, ScheduleAuditRecord, ScheduleDefinition,
    ScheduleDryRunPreview, ScheduleRegistry, ScheduleSubmissionStatus, ScheduledSubmission,
    Scheduler, SchedulerContractProfile, SchedulerEvent, SchedulerEventHook, SchedulerEventKind,
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
pub use security_env::{
    declared_environment, effective_env_allowlist, is_allowed_env_key, is_denied_env_key,
    missing_required_env_keys, shape_environment,
};
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
pub use state_machine::{
    failure_propagation_is_deterministic, node_transition_allowed, run_transition_allowed,
    NodeLifecycleState, RunLifecycleState,
};
use std::collections::{BTreeMap, HashMap};
use std::io::{self as std_io, Write};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub use store::{validate_storage_relative_path, ArtifactStore, CacheStore, StorageHealthReport};
use store::{ArtifactStore as RuntimeArtifactStore, CacheStore as RuntimeCacheStore};
pub use task_contract::{
    build_task_contract, default_forced_cleanup, validate_task_contracts, BackoffStrategy,
    ForcedCancellationCleanup, IdempotencyMode, NodeProvenance, OutputMaterializationPolicy,
    RetryPolicyV2, RuntimeState, SideEffectClassification, TaskContract, TaskFailureReason,
    TaskInputDescriptor, TaskIsolationMode, TaskOutputDescriptor, TaskResultEnvelope,
    TimeoutPolicy,
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

/// Explicit long-lived execution, scheduling, and replay surface.
pub mod stable {
    pub use crate::{
        adapter_conformance_suite, build_plan, build_planner_analysis, build_scheduler,
        cache_key_explanation, registered_adapter_descriptors,
        registered_adapter_reference_document, registered_adapters, trace_time_order_ok,
        validate_node_transition, validate_run_transition, verify_post_run_state_consistency,
        AbsolutePathPolicy, CacheKeyInput, CacheMode, ExecutionContext, NodeExecutionContext,
        NodeLifecycleState, PlannerGuardrails, RunLifecycleState, Runtime, RuntimeConfig,
        RuntimeError, SchedulerPolicy, SelectorSet,
    };
}

/// Common imports for planning, scheduling, and executing local DAG runs.
pub mod prelude {
    pub use crate::stable::{
        build_plan, build_planner_analysis, build_scheduler, AbsolutePathPolicy, CacheMode,
        ExecutionContext, NodeExecutionContext, PlannerGuardrails, Runtime, RuntimeConfig,
        RuntimeError, SchedulerPolicy, SelectorSet,
    };
}

/// Opt-in contract and evidence helpers that are outside the stable runtime lane.
#[cfg(feature = "experimental-public-api")]
pub mod experimental {
    pub mod adapter_execution {
        pub use crate::iteration06_contracts::*;
    }
    pub mod write_boundaries {
        pub use crate::iteration09_contracts::*;
    }
    pub mod planner_admission {
        pub use crate::iteration13_contracts::*;
    }
    pub mod durable_queue {
        pub use crate::iteration14_contracts::*;
    }
    pub mod container_evidence {
        pub use crate::iteration15_contracts::*;
    }
    pub mod observability_taxonomy {
        pub use crate::iteration17_contracts::*;
    }
}

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
    pub planner_contract_version: String,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub resolved_params: HashMap<String, Value>,
    pub effective_cache_dir: Option<PathBuf>,
    pub fs: Arc<dyn Fs>,
    pub clock: Arc<dyn Clock>,
    pub store: RuntimeArtifactStore,
    pub policy: PolicyConfig,
    pub absolute_path_policy: AbsolutePathPolicy,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub status: NodeStatus,
    pub stdout_path: String,
    pub stderr_path: String,
    pub outputs_dir: String,
    pub output_evidence: Vec<TraceOutputArtifact>,
    pub failure: Option<FailureInfo>,
    pub attempts: u32,
    pub attempt_events: Vec<AttemptEvent>,
    pub container_meta: Option<bijux_dag_artifacts::ContainerTrace>,
    pub adapter_binary_sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
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
        AdapterId { id: "const".to_string(), version: "0.1".to_string() }
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
        exec.fs.create_dir_all(exec.run_dir.node_outputs_dir(&node.id).as_path())?;
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
        exec.fs.write(&out_path, &serde_json::to_vec_pretty(&value)?)?;
        exec.fs.write(&stdout_path, b"")?;
        exec.fs.write(&stderr_path, b"")?;
        let output_report = inspect_declared_outputs(&outputs_dir, &node.outputs);
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_report.present_outputs)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
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
        AdapterId { id: "shell".to_string(), version: "0.1".to_string() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["shell".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet { filesystem: true, env: false, network: false, clock: false }
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

        let env_allowlist = effective_env_allowlist(node);
        let mut cmd = subprocess::command(&args[0]);
        cmd.args(&args[1..]);
        cmd.current_dir(&work_dir);
        apply_shaped_env(&mut cmd, exec.policy.clean_env, &env_allowlist, &[]);

        let output =
            command_output_with_timeout(&mut cmd, effective_node_timeout_ms(node, params))?;

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_report = inspect_declared_outputs(&outputs_dir, &node.outputs);
        if let Some(failure) = output_report.failure {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: output_report.output_evidence,
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_report.present_outputs)?;

        let success = output.status.success();
        let exit_code = output.status.code();
        let failure = if success {
            None
        } else {
            Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "command failed".to_string(),
                details: Some(serde_json::json!({ "exit_code": exit_code })),
            })
        };

        Ok(NodeResult {
            status: if success { NodeStatus::Success } else { NodeStatus::Failed },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
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
        AdapterId { id: "container".to_string(), version: "0.1".to_string() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["container".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet { filesystem: true, env: false, network: false, clock: false }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let graph = ctx.graph;
        let node = ctx.node;
        let exec = ctx.exec;
        let params = ctx.params;
        let spec = node
            .container
            .as_ref()
            .ok_or_else(|| RuntimeError::Executor("missing container spec".to_string()))?;

        let node_dir = exec.run_dir.node_dir(&node.id);
        let inputs_dir = exec.run_dir.node_inputs_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&inputs_dir)?;
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let engine = spec.engine.as_str();
        let engine_version = match container_execution::container_engine_discovery(engine) {
            Ok(version) => version,
            Err(message) => {
                exec.fs.write(&stdout_path, b"")?;
                exec.fs.write(&stderr_path, message.as_bytes())?;
                return Ok(NodeResult {
                    status: NodeStatus::Failed,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(FailureInfo {
                        kind: "Infrastructure".to_string(),
                        code: "CONTAINER_ENGINE_UNAVAILABLE".to_string(),
                        message: message.clone(),
                        details: Some(serde_json::json!({ "engine": engine })),
                    }),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: Some(container_trace(spec, engine, None, None)),
                    adapter_binary_sha256: None,
                });
            }
        };
        let mounts = container_execution::container_volume_contract(&node_dir);
        if let Err(message) =
            container_execution::validate_container_mount_contract(&mounts, &node_dir)
        {
            exec.fs.write(&stdout_path, b"")?;
            exec.fs.write(&stderr_path, message.as_bytes())?;
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: Vec::new(),
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "CONTAINER_VOLUME_CONTRACT_INVALID".to_string(),
                    message,
                    details: Some(serde_json::json!({ "engine": engine })),
                }),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(
                    spec,
                    engine,
                    None,
                    Some(engine_version.clone()),
                )),
                adapter_binary_sha256: None,
            });
        }

        let mut cmd = subprocess::command(engine);
        cmd.arg("run").arg("--rm");

        let deny_network = !node.effects.contains(&Effect::Network) || exec.policy.deny_network;
        let network_args =
            match container_execution::container_network_policy_args(engine, deny_network) {
                Ok(args) => args,
                Err(message) => {
                    exec.fs.write(&stdout_path, b"")?;
                    exec.fs.write(&stderr_path, message.as_bytes())?;
                    return Ok(NodeResult {
                        status: NodeStatus::Failed,
                        stdout_path: stdout_path.display().to_string(),
                        stderr_path: stderr_path.display().to_string(),
                        outputs_dir: outputs_dir.display().to_string(),
                        output_evidence: Vec::new(),
                        failure: Some(FailureInfo {
                            kind: "Policy".to_string(),
                            code: "POLICY_UNENFORCEABLE".to_string(),
                            message,
                            details: Some(
                                serde_json::json!({ "engine": engine, "effect": "network" }),
                            ),
                        }),
                        attempts: 1,
                        attempt_events: Vec::new(),
                        container_meta: Some(container_trace(
                            spec,
                            engine,
                            None,
                            Some(engine_version.clone()),
                        )),
                        adapter_binary_sha256: None,
                    });
                }
            };
        for arg in network_args {
            cmd.arg(arg);
        }
        for mount in &mounts {
            let mode = if mount.readonly { "ro" } else { "rw" };
            cmd.args(["-v", &format!("{}:{}:{}", mount.local_path, mount.container_path, mode)]);
        }

        let container_bindings = NodePathBindings::for_container();
        let workdir = resolve_container_workdir(
            spec.workdir.as_deref(),
            &container_bindings,
            exec.absolute_path_policy,
        )
        .map_err(RuntimeError::Executor)?;
        cmd.args(["--workdir", &workdir]);

        let env_allowlist = effective_env_allowlist(node);
        for (key, val) in shaped_environment(exec.policy.clean_env, &env_allowlist, &[]) {
            cmd.arg("-e").arg(format!("{}={}", key, val));
        }

        cmd.arg(&spec.image);
        let stable_argv = bijux_dag_core::resolve::resolve_command_argv_templates(
            graph, node, &spec.argv, params,
        )
        .map_err(|error| RuntimeError::Executor(error.to_string()))?;
        for part in &resolve_container_argv(&stable_argv, &container_bindings)
            .map_err(RuntimeError::Executor)?
        {
            cmd.arg(part);
        }

        let output =
            command_output_with_timeout(&mut cmd, effective_node_timeout_ms(node, params))?;
        let exit_code = output.status.code();

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_report = inspect_declared_outputs(&outputs_dir, &node.outputs);
        if let Some(failure) = output_report.failure {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: output_report.output_evidence,
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(
                    spec,
                    engine,
                    exit_code,
                    Some(engine_version.clone()),
                )),
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_report.present_outputs)?;

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
            status: if success { NodeStatus::Success } else { NodeStatus::Failed },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: Some(container_trace(spec, engine, exit_code, Some(engine_version))),
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

fn cache_hit_proof(cache_read: CacheRead) -> Result<Option<CacheProof>, RuntimeError> {
    match (cache_read.hit, cache_read.proof) {
        (true, Some(proof)) => Ok(Some(proof)),
        (true, None) => {
            Err(RuntimeError::Executor("cache hit missing verification proof".to_string()))
        }
        (false, proof) => Ok(proof),
    }
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
    pub run_root: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicy,
    pub run_id: Option<String>,
    pub parent_run_id: Option<String>,
    pub submission_source: String,
    pub trigger_source: String,
    pub operator: String,
    pub labels: Vec<String>,
    pub latest_symlink: Option<PathBuf>,
    pub policy: PolicyConfig,
    pub selectors: SelectorSet,
    pub upstream_selection_targets: Vec<String>,
    pub downstream_selection_roots: Vec<String>,
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
            run_root: None,
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            run_id: None,
            parent_run_id: None,
            submission_source: "manual".to_string(),
            trigger_source: "cli".to_string(),
            operator: "unknown".to_string(),
            labels: Vec::new(),
            latest_symlink: None,
            policy: PolicyConfig::default(),
            selectors: SelectorSet::default(),
            upstream_selection_targets: Vec::new(),
            downstream_selection_roots: Vec::new(),
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
    Id(String),
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
        Self { deny_network: false, deny_env: false, deny_clock: false, clean_env: true }
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
        Self { registry, fs: Arc::new(StdFs), clock: Arc::new(SystemClock), init_error }
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
        let ambient_env = std::env::vars().collect();
        security_env::validate_graph_environment_bindings(graph, &ambient_env)
            .map_err(RuntimeError::Executor)?;
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
    output_evidence: Vec<TraceOutputArtifact>,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    attempt: u32,
    cache_proof: Option<CacheProof>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
    container_meta: Option<ContainerTrace>,
    adapter_binary_sha256: Option<String>,
    trigger_evaluation: Option<TriggerEvaluation>,
    branch_decision: Option<String>,
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
    let inputs_index =
        if ctx.fs.metadata(ctx.run_dir.node_inputs_index_path(node_id).as_path()).is_ok() {
            Some("inputs/index.json".to_string())
        } else {
            None
        };
    let outputs = if output_evidence.is_empty() {
        inspect_declared_outputs(ctx.run_dir.node_outputs_dir(node_id).as_path(), &node.outputs)
            .output_evidence
    } else {
        output_evidence
    };
    let trace = NodeTrace {
        node_id: node_id.to_string(),
        status: status_string(&status),
        started_unix_ms,
        finished_unix_ms,
        attempt,
        fingerprint: node_fingerprint_from_ctx(ctx, node_id),
        planner_contract_version: Some(ctx.planner_contract_version.clone()),
        execution_fingerprint: Some(ctx.execution_fingerprint.clone()),
        evidence_fingerprint: Some(ctx.evidence_fingerprint.clone()),
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        adapter_outputs_schema_version: adapter_outputs_schema_version.to_string(),
        adapter_binary_sha256,
        resources: node.resources.as_ref().map(|r| TraceResources { cpu: r.cpu, mem_mb: r.mem_mb }),
        inputs_index,
        resolved_params: ctx.resolved_params.get(node_id).cloned(),
        outputs,
        container: container_meta,
        cache_proof,
        branch_decision,
        trigger_evaluation,
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

pub(crate) fn transition_cause_for_failure(failure: Option<&FailureInfo>) -> &'static str {
    match failure {
        Some(failure) if failure.kind == "Policy" => "PolicyDenied",
        Some(failure) if failure.code == "UPSTREAM_FAILED" => "DependencyFailed",
        Some(failure) if failure.code == "RUN_ABORTED" => "ExecutionAborted",
        Some(failure) if failure.code == "RUN_TIMEOUT" => "TimeoutExceeded",
        Some(failure) if failure.code == "EXEC_TIMEOUT" => "TimeoutExceeded",
        Some(failure) if failure.code == "CONTAINER_ENGINE_UNAVAILABLE" => "InfrastructureFailed",
        Some(failure) if failure.code == "OUTPUT_MISSING" => "MissingRequiredOutput",
        Some(failure) if failure.code == "INPUT_MISSING" => "MissingRequiredInput",
        Some(failure) if failure.kind == "Infrastructure" => "InfrastructureFailed",
        _ => "ExecutionFailed",
    }
}

pub(crate) fn transition_cause_for_skip_reason(reason: &str) -> &'static str {
    match reason {
        "filtered"
        | "not_selected_by_include_selector"
        | "excluded_by_selector"
        | "not_selected_by_dependency_closure" => "SelectionFiltered",
        "branch_decision_not_selected" => "BranchDecisionFiltered",
        "upstream_failed" => "DependencyFailed",
        "cancelled" => "CancelRequested",
        _ => "SelectionFiltered",
    }
}

pub(crate) fn failure_propagation_cause(failure: Option<&FailureInfo>) -> &'static str {
    match transition_cause_for_failure(failure) {
        "PolicyDenied" => "policy_denied",
        "DependencyFailed" => "upstream_failed",
        "ExecutionAborted" => "execution_aborted",
        "TimeoutExceeded" => "timeout_exceeded",
        "InfrastructureFailed" => "infrastructure_failed",
        "MissingRequiredOutput" => "missing_required_output",
        "MissingRequiredInput" => "missing_required_input",
        _ => "execution_failed",
    }
}

fn write_resolved_params(ctx: &RunContext, node_id: &str) -> Result<(), RuntimeError> {
    let mut params = ctx.resolved_params.get(node_id).cloned().unwrap_or(Value::Null);
    sort_value_maps(&mut params);
    let data = serde_json::to_vec_pretty(&params)?;
    ctx.store.write_resolved_params(node_id, &data)?;
    Ok(())
}

fn write_attempt_events(
    ctx: &RunContext,
    node_id: &str,
    attempt_events: &[AttemptEvent],
) -> Result<(), RuntimeError> {
    let data = serde_json::to_vec_pretty(attempt_events)?;
    ctx.store.write_attempts(node_id, &data)?;
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
    graph: &Graph,
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
        let node_ctx = NodeCtx { graph, node, exec: ctx, params };
        let mut result = match adapter.execute(&node_ctx) {
            Ok(result) => result,
            Err(err) => failed_node_result_from_runtime_error(ctx, node, err),
        };
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
            let wait = retry.backoff_ms.saturating_mul(attempt.saturating_sub(1) as u64);
            if wait > 0 {
                std::thread::sleep(Duration::from_millis(wait));
            }
        }
    }
}

fn failed_node_result_from_runtime_error(
    ctx: &RunContext,
    node: &Node,
    error: RuntimeError,
) -> NodeResult {
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let outputs_dir = ctx.run_dir.node_outputs_dir(&node.id);
    let stdout_path = ctx.run_dir.node_stdout_path(&node.id);
    let stderr_path = ctx.run_dir.node_stderr_path(&node.id);
    let (kind, code, message) = match error {
        RuntimeError::Graph(err) => ("Internal", "GRAPH_ERROR", err.to_string()),
        RuntimeError::Artifact(err) => ("Infrastructure", "ARTIFACT_ERROR", err.to_string()),
        RuntimeError::Io(err) => ("Infrastructure", "IO_ERROR", err.to_string()),
        RuntimeError::Json(err) => ("Internal", "JSON_ERROR", err.to_string()),
        RuntimeError::Executor(message) => {
            if message.contains("timed out") {
                ("Execution", "EXEC_TIMEOUT", message)
            } else {
                ("Execution", "EXEC_ERROR", message)
            }
        }
    };
    let _ = ctx.fs.create_dir_all(&node_dir);
    let _ = ctx.fs.create_dir_all(&outputs_dir);
    let _ = ctx.fs.write(&stdout_path, b"");
    let _ = ctx.fs.write(&stderr_path, message.as_bytes());
    NodeResult {
        status: NodeStatus::Failed,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure: Some(FailureInfo {
            kind: kind.to_string(),
            code: code.to_string(),
            message,
            details: None,
        }),
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
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

fn build_git_sha() -> Option<&'static str> {
    option_env!("BIJUX_DAG_BUILD_GIT_SHA").filter(|value| !value.trim().is_empty())
}

fn compose_tool_version(package_version: &str, build_git_sha: Option<&str>) -> String {
    match build_git_sha {
        Some(commit) => format!("{package_version}+{commit}"),
        None => package_version.to_string(),
    }
}

fn tool_version() -> String {
    compose_tool_version(env!("CARGO_PKG_VERSION"), build_git_sha())
}

pub(crate) fn runtime_fingerprint(adapters: &[AdapterInfo]) -> String {
    let payload = serde_json::json!({
        "tool_version": tool_version(),
        "adapters": adapters,
    });
    sha256_bytes(payload.to_string().as_bytes())
}

pub(crate) fn policy_fingerprint(policy: &PolicyConfig) -> String {
    let payload = serde_json::json!({
        "deny_network": policy.deny_network,
        "deny_env": policy.deny_env,
        "deny_clock": policy.deny_clock,
        "clean_env": policy.clean_env,
    });
    sha256_bytes(payload.to_string().as_bytes())
}

fn selector_label(selector: &Selector) -> String {
    match selector {
        Selector::Id(v) => format!("id:{v}"),
        Selector::IdPrefix(v) => format!("id_prefix:{v}"),
        Selector::Tag(v) => format!("tag:{v}"),
        Selector::Kind(v) => format!("kind:{v}"),
    }
}

pub(crate) fn requested_selector_label(scope: &str, selector: &Selector) -> String {
    format!("{scope}:{}", selector_label(selector))
}

pub(crate) fn requested_downstream_root_label(node_id: &str) -> String {
    format!("from-node:{node_id}")
}

pub(crate) fn requested_upstream_target_label(node_id: &str) -> String {
    format!("to-node:{node_id}")
}

fn materialize_mode_label(mode: MaterializeMode) -> &'static str {
    match mode {
        MaterializeMode::Copy => "copy",
        MaterializeMode::Hardlink => "hardlink",
        MaterializeMode::Symlink => "symlink",
    }
}

fn failure_propagation_label(mode: &FailurePropagationMode) -> &'static str {
    match mode {
        FailurePropagationMode::FailFast => "fail_fast",
        FailurePropagationMode::IsolateBranch => "isolate_branch",
        FailurePropagationMode::ContinueIndependent => "continue_independent",
        FailurePropagationMode::QuorumLikeFuture => "quorum_like_future",
    }
}

fn runtime_config_fingerprint(options: &RuntimeConfig) -> String {
    let include_selectors: Vec<String> =
        options.selectors.include.iter().map(selector_label).collect();
    let exclude_selectors: Vec<String> =
        options.selectors.exclude.iter().map(selector_label).collect();
    let payload = serde_json::json!({
        "jobs": options.jobs,
        "cpu_budget": options.cpu_budget,
        "run_timeout_ms": options.run_timeout_ms,
        "node_timeout_ms": options.node_timeout_ms,
        "materialize_inputs": materialize_mode_label(options.materialize_inputs),
        "scheduler_policy": options.scheduler_policy,
        "failure_propagation": failure_propagation_label(&options.failure_propagation),
        "partial_rerun_dependency_closure": options.partial_rerun_dependency_closure,
        "upstream_selection_targets": options.upstream_selection_targets,
        "downstream_selection_roots": options.downstream_selection_roots,
        "selectors": {
            "include": include_selectors,
            "exclude": exclude_selectors,
        },
    });
    sha256_bytes(payload.to_string().as_bytes())
}

fn cache_key_input_for_run(
    options: &RuntimeConfig,
    node_fingerprint: &str,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> CacheKeyInput {
    CacheKeyInput {
        node_fingerprint: node_fingerprint.to_string(),
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        output_schema_version: adapter_outputs_schema_version.to_string(),
        policy_fingerprint: policy_fingerprint(&options.policy),
        config_fingerprint: runtime_config_fingerprint(options),
        backend_class: "local".to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AdapterAdmissionEntry {
    pub node_id: String,
    pub node_kind: String,
    pub supported: bool,
    pub adapter_id: Option<String>,
    pub adapter_version: Option<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AdapterAdmissionReport {
    pub supported: bool,
    pub entries: Vec<AdapterAdmissionEntry>,
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

pub fn registered_adapter_descriptors() -> Vec<adapter::AdapterDescriptor> {
    let registry = build_registry(vec![
        Arc::new(ConstAdapter),
        Arc::new(ShellAdapter),
        Arc::new(ContainerAdapter),
    ])
    .unwrap_or_else(|_| AdapterRegistry::new());
    registry.descriptors()
}

pub fn adapter_conformance_suite() -> Result<Vec<AdapterConformanceSuiteReport>, RuntimeError> {
    let mut descriptors = registered_adapter_descriptors();
    for handshake in probe_external_adapters()? {
        if let Some(descriptor) = handshake.descriptor {
            descriptors.push(descriptor);
        }
    }
    descriptors.sort_by(|left, right| (&left.id, &left.version).cmp(&(&right.id, &right.version)));
    Ok(descriptors
        .into_iter()
        .map(|descriptor| build_adapter_conformance_suite(&descriptor))
        .collect())
}

pub fn registered_adapter_reference_document() -> AdapterReferenceDocument {
    let mut descriptors = registered_adapter_descriptors();
    descriptors.sort_by(|left, right| (&left.id, &left.version).cmp(&(&right.id, &right.version)));
    let conformance = descriptors.iter().map(build_adapter_conformance_suite).collect::<Vec<_>>();
    AdapterReferenceDocument {
        descriptors,
        conformance,
        slurm: slurm_adapter_design_contract(),
        kubernetes: kubernetes_adapter_contract(),
        fake_batch: fake_batch_executor_contract(),
    }
}

pub fn adapter_admission_matrix(graph: &Graph) -> AdapterAdmissionReport {
    let descriptors = registered_adapter_descriptors();
    let mut by_kind = std::collections::BTreeMap::new();
    for descriptor in &descriptors {
        for kind in &descriptor.supported_kinds {
            by_kind.insert(kind.clone(), descriptor.clone());
        }
    }

    let mut entries = Vec::new();
    for node in &graph.nodes {
        let kind = node.kind.as_str().to_string();
        let descriptor = by_kind.get(&kind);
        let mut reasons = Vec::new();
        if descriptor.is_none() {
            reasons.push(format!("no registered adapter supports node kind {}", kind));
        }
        if let Some(descriptor) = descriptor {
            let conformance = adapter_conformance::validate_descriptor(descriptor);
            reasons.extend(conformance.violations);
            if matches!(node.kind, NodeKind::Container) {
                let Some(spec) = node.container.as_ref() else {
                    reasons.push("container node missing container spec".to_string());
                    entries.push(AdapterAdmissionEntry {
                        node_id: node.id.clone(),
                        node_kind: kind,
                        supported: reasons.is_empty(),
                        adapter_id: Some(descriptor.id.clone()),
                        adapter_version: Some(descriptor.version.clone()),
                        reasons,
                    });
                    continue;
                };
                if let Err(error) = container_execution::container_engine_discovery(&spec.engine) {
                    reasons.push(error);
                }
                if let Err(error) = container_execution::container_network_policy_args(
                    &spec.engine,
                    !node.effects.contains(&Effect::Network),
                ) {
                    reasons.push(error);
                }
                let mounts = container_execution::container_volume_contract(Path::new(
                    "/synthetic-node-root",
                ));
                if let Err(error) = container_execution::validate_container_mount_contract(
                    &mounts,
                    Path::new("/synthetic-node-root"),
                ) {
                    reasons.push(error);
                }
            }
        }
        let supported = reasons.is_empty();
        entries.push(AdapterAdmissionEntry {
            node_id: node.id.clone(),
            node_kind: kind,
            supported,
            adapter_id: descriptor.map(|value| value.id.clone()),
            adapter_version: descriptor.map(|value| value.version.clone()),
            reasons,
        });
    }
    let supported = entries.iter().all(|entry| entry.supported);
    AdapterAdmissionReport { supported, entries }
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
        let src_path = ctx.run_dir.node_outputs_dir(&edge.from.node_id).join(&out.path);
        let dst_dir = inputs_dir.join(&edge.from.node_id);
        ctx.fs.create_dir_all(&dst_dir)?;
        let dst_path = dst_dir.join(&edge.to.port);
        if let Some(parent) = dst_path.parent() {
            ctx.fs.create_dir_all(parent)?;
        }
        if ctx.fs.metadata(&src_path).is_ok() {
            let source_sha256 = sha256_artifact_path(&src_path).map_err(RuntimeError::Artifact)?;
            materialize_file(ctx.fs.as_ref(), &src_path, &dst_path, mode)?;
            let local_sha256 = materialized_input_sha256(ctx.fs.as_ref(), &dst_path)
                .map_err(RuntimeError::Artifact)?;
            if local_sha256 != source_sha256 {
                return Err(RuntimeError::Executor(format!(
                    "materialized input hash mismatch for {} -> {}",
                    src_path.display(),
                    dst_path.display()
                )));
            }
            let rel = dst_path.strip_prefix(&inputs_dir).unwrap_or(&dst_path);
            let rel_str = rel.to_string_lossy().to_string();
            let from_fp = node_fingerprint_from_ctx(ctx, &edge.from.node_id);
            files.push(InputFile {
                local_path: rel_str,
                source_sha256,
                source_node_id: edge.from.node_id.clone(),
                source_node_fingerprint: from_fp,
                source_output_name: edge.from.port.clone(),
                materialization_mode: materialize_mode_label(mode).to_string(),
            });
        }
    }
    files.sort_by(|a, b| a.local_path.cmp(&b.local_path));
    let index = InputsIndex { files };
    write_inputs_index(&inputs_dir, &index)?;
    Ok(index)
}

fn cache_dir_from_env() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

#[derive(Debug, Clone)]
struct OutputInspectionReport {
    pub(crate) output_evidence: Vec<TraceOutputArtifact>,
    pub(crate) present_outputs: Vec<DeclaredOutputArtifact>,
    pub(crate) failure: Option<FailureInfo>,
}

fn output_kind_label(kind: &OutputKind) -> &'static str {
    match kind {
        OutputKind::File => "file",
        OutputKind::Directory => "directory",
        OutputKind::Value => "value",
        OutputKind::Table => "table",
        OutputKind::Log => "log",
        OutputKind::Binary => "binary",
        OutputKind::Bundle => "bundle",
    }
}

pub(crate) fn declared_output_artifacts(node: &Node) -> Vec<DeclaredOutputArtifact> {
    node.outputs
        .iter()
        .map(|output| DeclaredOutputArtifact {
            name: output.name.clone(),
            path: output.path.clone(),
            kind: output_kind_label(&output.kind).to_string(),
            media_type: output.effective_media_type(),
        })
        .collect()
}

pub(crate) fn inspect_declared_outputs(
    dir: &Path,
    outputs: &[OutputSpec],
) -> OutputInspectionReport {
    let mut declared = Vec::new();
    let mut present_outputs = Vec::new();
    for output in outputs {
        declared.push(output.clone());
        let schema = ArtifactSchemaDescriptor {
            name: format!("bijux.output.{}", output_kind_label(&output.kind)),
            version: "v0.1".to_string(),
            media_type: output.effective_media_type(),
            encoding: "identity".to_string(),
            validation_mode: SchemaValidationMode::Strict,
        };
        if let Err(message) = validate_output_schema_descriptor(&schema) {
            return OutputInspectionReport {
                output_evidence: Vec::new(),
                present_outputs: Vec::new(),
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_SCHEMA_INVALID".to_string(),
                    message,
                    details: None,
                }),
            };
        }
    }

    let mut output_evidence = Vec::new();
    for output in &declared {
        if !bijux_dag_artifacts::paths::is_normalized_relative_path(&output.path) {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_PATH_INVALID".to_string(),
                    message: "invalid output path".to_string(),
                    details: Some(serde_json::json!({ "path": output.path })),
                }),
            };
        }
        if has_symlink_component(dir, Path::new(&output.path)) {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_PATH_INVALID".to_string(),
                    message: format!("output path traverses symlink: {}", output.path),
                    details: None,
                }),
            };
        }
        let path = dir.join(&output.path);
        if !path.exists() {
            output_evidence.push(TraceOutputArtifact {
                name: output.name.clone(),
                path: output.path.clone(),
                kind: output_kind_label(&output.kind).to_string(),
                required: output.required,
                present: false,
                media_type: output.effective_media_type(),
                sha256: None,
            });
            if output.required {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(FailureInfo {
                        kind: "Execution".to_string(),
                        code: "OUTPUT_MISSING".to_string(),
                        message: format!("missing required output: {}", output.path),
                        details: Some(serde_json::json!({ "output": output.name })),
                    }),
                };
            }
            continue;
        }
        if path.is_symlink() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_PATH_INVALID".to_string(),
                    message: format!("output must not be a symlink: {}", output.path),
                    details: None,
                }),
            };
        }
        if output.expects_directory() && !path.is_dir() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_PATH_INVALID".to_string(),
                    message: format!("output must be a directory: {}", output.path),
                    details: Some(serde_json::json!({ "output": output.name })),
                }),
            };
        }
        if output.expects_file() && !path.is_file() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_PATH_INVALID".to_string(),
                    message: format!("output must be a file: {}", output.path),
                    details: Some(serde_json::json!({ "output": output.name })),
                }),
            };
        }
        let sha256 = match sha256_artifact_path(&path) {
            Ok(sha256) => sha256,
            Err(error) => {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(FailureInfo {
                        kind: "Execution".to_string(),
                        code: "OUTPUT_PATH_INVALID".to_string(),
                        message: error.to_string(),
                        details: Some(serde_json::json!({ "output": output.name })),
                    }),
                };
            }
        };
        let media_type = output.effective_media_type();
        output_evidence.push(TraceOutputArtifact {
            name: output.name.clone(),
            path: output.path.clone(),
            kind: output_kind_label(&output.kind).to_string(),
            required: output.required,
            present: true,
            media_type: media_type.clone(),
            sha256: Some(sha256.clone()),
        });
        present_outputs.push(DeclaredOutputArtifact {
            name: output.name.clone(),
            path: output.path.clone(),
            kind: output_kind_label(&output.kind).to_string(),
            media_type,
        });
    }

    let mut actual = std::collections::BTreeSet::new();
    collect_relative_artifacts(dir, dir, &mut actual);
    for rel in actual {
        let declared_match = declared.iter().any(|output| {
            rel == output.path
                || (matches!(output.kind, OutputKind::Directory)
                    && rel.starts_with(&format!("{}/", output.path)))
        });
        if !declared_match {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo {
                    kind: "Execution".to_string(),
                    code: "OUTPUT_UNDECLARED".to_string(),
                    message: format!("undeclared output path: {}", rel),
                    details: None,
                }),
            };
        }
    }

    OutputInspectionReport { output_evidence, present_outputs, failure: None }
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
        return Ok(CacheRead { hit: false, proof: None });
    }
    if !node.cache.enabled {
        return Ok(CacheRead { hit: false, proof: None });
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let cache_store = match cache_dir {
        Some(d) => Some(RuntimeCacheStore::new(d, Arc::clone(&fs))),
        None => return Ok(CacheRead { hit: false, proof: None }),
    };
    if options.cache_mode == CacheMode::Read || options.cache_mode == CacheMode::ReadWrite {
        let key_input = cache_key_input_for_run(
            options,
            node_fingerprint,
            adapter_id,
            adapter_version,
            adapter_outputs_schema_version,
        );
        let key = cache_key_explanation(&key_input).key;
        let store = cache_store.as_ref().unwrap();
        let entry = store.entry(&key);
        if store.fs().metadata(&entry).is_ok() {
            if !verify_cache_entry(store.fs(), &entry, &key_input)? {
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
                if !verify_cache_entry(store.fs(), &remote_entry, &key_input)? {
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
    Ok(CacheRead { hit: false, proof: None })
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
    if !node.cache.enabled {
        return Ok(());
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let store = match cache_dir {
        Some(d) => RuntimeCacheStore::new(d, Arc::clone(&fs)),
        None => return Ok(()),
    };
    let key_input = cache_key_input_for_run(
        options,
        node_fingerprint,
        adapter_id,
        adapter_version,
        adapter_outputs_schema_version,
    );
    let key = cache_key_explanation(&key_input).key;
    let entry = store.entry(&key);
    store.fs().create_dir_all(entry.join("outputs").as_path())?;
    store.fs().create_dir_all(entry.join("logs").as_path())?;
    let meta = serde_json::json!({
        "cache_metadata_version": "cache-meta/v0.1",
        "cache_key": key,
        "node_id": node.id,
        "node_fingerprint": key_input.node_fingerprint,
        "adapter_id": key_input.adapter_id,
        "adapter_version": key_input.adapter_version,
        "produces_outputs_schema_version": key_input.output_schema_version,
        "policy_fingerprint": key_input.policy_fingerprint,
        "config_fingerprint": key_input.config_fingerprint,
        "backend_class": key_input.backend_class,
        "created_unix_ms": ctx.clock.now_unix_ms(),
        "cache_source": "local",
        "schema_version": "v0.1",
    });
    store.fs().write(entry.join("meta.json").as_path(), &serde_json::to_vec_pretty(&meta)?)?;
    copy_dir_all(store.fs(), ctx.run_dir.node_outputs_dir(&node.id), entry.join("outputs"))?;
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
    expected_input: &CacheKeyInput,
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
    if !cache_metadata_version_supported(&meta) || !cache_entry_has_required_proof(&meta) {
        return Ok(false);
    }
    let expected_key = cache_key_explanation(expected_input).key;
    if meta.get("cache_key").and_then(|v| v.as_str()) != Some(expected_key.as_str()) {
        return Ok(false);
    }
    if meta.get("node_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.node_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("adapter_id").and_then(|v| v.as_str()) != Some(expected_input.adapter_id.as_str()) {
        return Ok(false);
    }
    if meta.get("adapter_version").and_then(|v| v.as_str())
        != Some(expected_input.adapter_version.as_str())
    {
        return Ok(false);
    }
    let produced_output_schema_version = meta
        .get("produces_outputs_schema_version")
        .and_then(|v| v.as_str())
        .or_else(|| meta.get("output_schema_version").and_then(|v| v.as_str()))
        .unwrap_or_default();
    let schema_compatibility = validate_output_schema_compatibility(
        CacheCompatibilityMode::FingerprintExact,
        produced_output_schema_version,
        expected_input.output_schema_version.as_str(),
    );
    if !schema_compatibility.compatible {
        return Ok(false);
    }
    if meta.get("policy_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.policy_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("config_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.config_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("backend_class").and_then(|v| v.as_str())
        != Some(expected_input.backend_class.as_str())
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
        let sha = sha256_artifact_path(&path).map_err(RuntimeError::Artifact)?;
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
    ctx.graph_fingerprint.lock().ok().and_then(|map| map.get(node_id).cloned()).unwrap_or_default()
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
    meta.get("cache_source").and_then(|v| v.as_str()).map(|s| s.to_string())
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
    inspect_declared_outputs(dir, outputs).failure
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

fn collect_relative_artifacts(
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
            collect_relative_artifacts(root, &path, out);
            continue;
        }
        if let Ok(rel) = path.strip_prefix(root) {
            out.insert(rel.to_string_lossy().replace('\\', "/"));
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
    declared_environment(&ambient, clean_env, allowlist, denylist)
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
            #[cfg(unix)]
            {
                let pid = child.id();
                kill_child_descendants_best_effort(pid);
            }
            let _ = child.kill();
            let _ = child.wait();
            return Err(RuntimeError::Executor(format!("execution timed out after {limit_ms}ms")));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn kill_child_descendants_best_effort(pid: u32) {
    let parent = pid.to_string();
    let _ = std::process::Command::new("pkill").args(["-TERM", "-P", &parent]).status();
    let _ = std::process::Command::new("pkill").args(["-KILL", "-P", &parent]).status();
}

fn effective_node_timeout_ms(node: &Node, params: &Value) -> Option<u64> {
    node.timeout_ms.or_else(|| params.get("timeout_ms").and_then(|v| v.as_u64()))
}

fn container_trace(
    spec: &bijux_dag_core::ContainerSpec,
    engine: &str,
    exit_code: Option<i32>,
    engine_version: Option<String>,
) -> ContainerTrace {
    let image_digest =
        subprocess::output(engine, &["image", "inspect", "--format", "{{.Id}}", &spec.image])
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
                        name: f.name,
                        path: f.path,
                        kind: f.kind,
                        media_type: f.media_type,
                        sha256: f.sha256,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| {
        (a.node_id.clone(), a.path.clone()).cmp(&(b.node_id.clone(), b.path.clone()))
    });
    Ok(out)
}

fn build_run_outputs_index(
    run_dir: &RunDir,
    outputs: &[OutputSummary],
) -> Result<RunOutputsIndex, RuntimeError> {
    let mut files = Vec::new();
    for out in outputs {
        let rel = run_dir.node_output_relpath(&out.node_id, &out.path);
        files.push(RunOutputFile {
            node_id: out.node_id.clone(),
            node_fingerprint: out.node_fingerprint.clone(),
            name: out.name.clone(),
            kind: out.kind.clone(),
            media_type: out.media_type.clone(),
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
    let mut counts = NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0 };
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
    if src.is_dir() {
        if matches!(mode, MaterializeMode::Symlink) && fs.symlink(src, dst).is_ok() {
            return Ok(());
        }
        fs.create_dir_all(dst)?;
        for entry in fs.read_dir(src)? {
            let child_dst = dst.join(entry.file_name());
            materialize_file(fs, entry.path().as_path(), child_dst.as_path(), mode)?;
        }
        return Ok(());
    }
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

fn materialized_input_sha256(fs: &dyn Fs, path: &Path) -> Result<String, ArtifactError> {
    let resolved = fs.canonicalize(path)?;
    sha256_artifact_path(&resolved)
}

#[cfg(test)]
mod cache_read_contract_tests {
    use super::*;

    #[test]
    fn cache_hit_requires_proof() {
        let err = cache_hit_proof(CacheRead { hit: true, proof: None }).expect_err("invalid hit");
        assert!(err.to_string().contains("missing verification proof"));

        let proof = CacheProof {
            hit: true,
            key: "k".to_string(),
            source: "local".to_string(),
            verified: true,
            reason: "hit".to_string(),
            corrupt_detected: false,
        };
        let hit_proof =
            cache_hit_proof(CacheRead { hit: true, proof: Some(proof) }).expect("valid hit");
        let hit_proof = hit_proof.expect("proof");
        assert!(hit_proof.hit);
        assert_eq!(hit_proof.key, "k");
    }

    #[test]
    fn compose_tool_version_uses_build_git_sha_when_available() {
        assert_eq!(compose_tool_version("0.4.0", Some("abc1234")), "0.4.0+abc1234");
        assert_eq!(compose_tool_version("0.4.0", None), "0.4.0");
    }

    #[test]
    fn runtime_fingerprint_stays_stable_across_working_directories() {
        use std::sync::{Mutex, OnceLock};

        static WORKING_DIRECTORY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        let _guard = WORKING_DIRECTORY_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("working directory lock");
        let original_dir = std::env::current_dir().expect("current directory");
        let temp_dir = tempfile::tempdir().expect("temporary directory");
        let adapters = vec![AdapterInfo {
            adapter_id: "shell".to_string(),
            adapter_version: "1.0.0".to_string(),
            effects: vec!["local".to_string()],
        }];

        let original_fingerprint = runtime_fingerprint(&adapters);
        std::env::set_current_dir(temp_dir.path()).expect("switch to temp directory");
        let moved_fingerprint = runtime_fingerprint(&adapters);
        std::env::set_current_dir(&original_dir).expect("restore original directory");

        assert_eq!(original_fingerprint, moved_fingerprint);
    }
}

#[cfg(test)]
include!("internal/testing/tests_runtime.in.rs");
