//! Execution, replay, scheduling, and policy surfaces for Bijux DAG runs.
//!
//! Prefer [`stable`] when browsing the long-lived runtime surface, [`prelude`]
//! for common execution workflows, and crate-root imports only when you
//! already know the exact item you need. Broad compatibility re-exports remain
//! callable for focused imports, but they are intentionally hidden from the
//! default docs lane. The `experimental-public-api` feature enables opt-in
//! runtime contract material that is intentionally excluded from the default
//! docs lane.
//!
#![allow(dead_code)]

#[path = "adapters/adapter.rs"]
mod adapter;
#[path = "adapters/api.rs"]
mod adapter_api;
#[path = "adapters/conformance.rs"]
mod adapter_conformance;
#[cfg(test)]
#[path = "internal/testing/adapter_contract_tests.rs"]
mod adapter_contract_tests;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/adapter_execution_contracts.rs"]
mod adapter_execution_contracts;
#[path = "adapters/sdk.rs"]
mod adapter_sdk;
mod adapters;
#[path = "internal/analysis/adaptive_scheduler.rs"]
mod adaptive_scheduler;
#[path = "internal/workflow/ai_operator_assist.rs"]
mod ai_operator_assist;
#[path = "internal/control/api.rs"]
mod api;
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
mod builtins;
mod cache;
#[path = "internal/control/clock.rs"]
mod clock;
#[path = "internal/control/config.rs"]
mod config;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/container_evidence_contracts.rs"]
mod container_evidence_contracts;
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
#[path = "runtime_core/execution/cron_calendar.rs"]
mod cron_calendar;
#[path = "internal/analysis/dataset_semantics.rs"]
mod dataset_semantics;
mod diagnostics;
#[path = "backend/distributed/distributed.rs"]
mod distributed;
#[path = "backend/distributed/distribution_readiness.rs"]
mod distribution_readiness;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/durable_queue_contracts.rs"]
mod durable_queue_contracts;
#[path = "runtime_core/execution/engine.rs"]
mod engine;
mod error;
#[path = "runtime_core/execution/flow.rs"]
mod execution;
#[path = "backend/runtime/execution_backend.rs"]
mod execution_backend;
#[path = "runtime_core/execution/context.rs"]
mod execution_context;
#[path = "runtime_core/planning/execution_plan.rs"]
mod execution_plan;
#[path = "internal/ext/extension_catalog.rs"]
mod extension_catalog;
#[path = "adapters/external.rs"]
mod external_adapter;
#[path = "runtime_core/execution/failure_summary.rs"]
mod failure_summary;
#[path = "backend/distributed/federated_scheduling.rs"]
mod federated_scheduling;
#[path = "adapters/file_transform.rs"]
mod file_transform_adapter;
#[path = "internal/ext/formal_verification.rs"]
mod formal_verification;
#[path = "backend/distributed/geo_federation.rs"]
mod geo_federation;
#[path = "backend/distributed/ha_scheduler.rs"]
mod ha_scheduler;
#[path = "adapters/http.rs"]
mod http_adapter;
#[path = "backend/distributed/infrastructure.rs"]
mod infrastructure;
mod internal;
#[path = "runtime_core/governance/invariants.rs"]
mod invariants;
#[cfg(test)]
#[path = "internal/testing/invariants_tests.rs"]
mod invariants_tests;
#[path = "internal/control/io.rs"]
mod io;
#[path = "backend/runtime/kubernetes_execution.rs"]
mod kubernetes_execution;
#[path = "backend/runtime/local_executor.rs"]
mod local_executor;
#[path = "backend/runtime/local_worker_pool.rs"]
mod local_worker_pool;
#[path = "runtime_core/execution/node_result.rs"]
mod node_result;
#[path = "diagnostics/runtime/observability.rs"]
mod observability;
#[path = "diagnostics/runtime/observability_deep.rs"]
mod observability_deep;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/observability_taxonomy_contracts.rs"]
mod observability_taxonomy_contracts;
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
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/planning/planner_admission_contracts.rs"]
mod planner_admission_contracts;
#[path = "runtime_core/planning/planner_analysis.rs"]
mod planner_analysis;
mod policy;
#[path = "adapters/python.rs"]
mod python_adapter;
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
mod run_context;
#[path = "runtime_core/execution/run_state.rs"]
mod run_state;
#[path = "internal/control/runtime.rs"]
mod runtime;
#[cfg(test)]
#[path = "internal/testing/runtime_boundary_tests.rs"]
mod runtime_boundary_tests;
#[path = "internal/control/runtime_controls.rs"]
mod runtime_controls;
mod runtime_core;
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
mod selectors;
#[path = "artifacts/storage/semantic_lineage.rs"]
mod semantic_lineage;
#[path = "internal/control/services.rs"]
mod services;
pub mod simulated_platform;
#[path = "backend/runtime/slurm_execution.rs"]
mod slurm_execution;
#[path = "runtime_core/execution/state_machine.rs"]
mod state_machine;
#[cfg(test)]
#[path = "internal/testing/state_machine_tests.rs"]
mod state_machine_tests;
#[path = "artifacts/storage/store.rs"]
mod store;
#[path = "backend/runtime/subprocess.rs"]
mod subprocess;
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
mod trace;
#[path = "artifacts/storage/upgrade_compatibility.rs"]
mod upgrade_compatibility;
#[path = "internal/workflow/workflow_product.rs"]
mod workflow_product;
#[cfg(feature = "experimental-public-api")]
#[path = "runtime_core/execution/write_boundary_contracts.rs"]
mod write_boundary_contracts;
use adapter::{Adapter, AdapterId, EffectSet, NodeCtx};
#[doc(hidden)]
pub use adapter::{AdapterDescriptor, CacheCompatibilityMode};
#[doc(hidden)]
pub use adapter_conformance::{
    build_adapter_conformance_suite, generate_adapter_reference_markdown,
    validate_output_schema_compatibility, AdapterConformanceSuiteReport,
    AdapterOutputSchemaCompatibilityReport, AdapterReferenceDocument, AdapterScenarioObservation,
    AdapterScenarioResult, AdapterScenarioStatus,
};
#[doc(hidden)]
pub use adapter_sdk::{
    AdapterCapabilities, AdapterContext, AdapterPlugin, BackendPlugin, PluginManifest,
};
#[doc(hidden)]
pub use async_adapter::AsyncAdapter;
#[doc(hidden)]
pub use backend::fake::{
    fake_batch_backend_reference, fake_batch_executor_contract, FakeBatchExecutor,
    FakeBatchExecutorContract, FakeBatchJobRecord, FakeBatchJobStatus,
};
#[doc(hidden)]
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
#[doc(hidden)]
pub use batch_execution::{
    cancel_batch_attempt, duplicate_status_delivery_detected, execution_mode_report,
    heartbeat_stale, restart_recovery_supported, retry_attempt, validate_batch_metadata,
    BatchAttemptState, BatchHeartbeat, BatchJobMetadata, BatchLifecycleEvent, BatchModeReport,
};
use bijux_dag_artifacts::schema::{
    validate_output_schema_descriptor, ArtifactSchemaDescriptor, SchemaValidationMode,
};
#[doc(hidden)]
pub use bijux_dag_artifacts::ContainerImageReferencePolicy;
use bijux_dag_artifacts::{
    artifact_size_bytes, sha256_artifact_path, write_inputs_index, write_outputs_index,
    AdapterInfo, ArtifactError, CacheIdentity, CacheProof, ContainerTrace, DeclaredOutputArtifact,
    FailureClass, FailureInfo, InputCollection, InputCollectionItem, InputFile, InputsIndex,
    NodeCounts, NodeLifecycleTransition, NodeLogEvidence, NodeTrace, OutputSummary, OutputsIndex,
    ReplayProvenance, Resources as TraceResources, RunDir, RunDirLayout, RunOutputFile,
    RunOutputsIndex, TraceOutputArtifact, TriggerEvaluation,
};
use bijux_dag_core::{
    Effect, FileOutput, Graph, GraphError, Node, NodeKind, OutputKind, OutputSpec, RetryPolicy,
    SemanticNodeKind, Severity,
};
#[doc(hidden)]
pub use cache::{
    cache_entry_has_required_proof, cache_entry_manifest_version_supported,
    cache_explainability_proof_from_meta, cache_key_explanation, cache_key_input_from_meta,
    cache_metadata_version_supported, CacheEntryManifest, CacheExplainabilityProof, CacheKeyInput,
    CacheManifestOutput, CACHE_ENTRY_MANIFEST_VERSION, CACHE_METADATA_VERSION,
};
use clock::{Clock, SystemClock};
#[doc(hidden)]
pub use container_execution::{
    container_engine_discovery, container_env_isolated, container_gpu_runtime_args,
    container_network_policy_args, container_volume_contract, map_local_path_to_container,
    supported_container_engines, validate_container_contract, validate_container_mount_contract,
    validate_container_relative_path, ContainerExecutionContract, ContainerMount,
};
#[doc(hidden)]
pub use coordination::{
    merge_timeout_and_exit_events, thread_safety_audit, RunSummaryCounters,
    RuntimeCoordinationSnapshot, RuntimeCoordinationState, ThreadSafetyAuditRecord,
    TraceWriteRecord,
};
#[doc(hidden)]
pub use execution_backend::{
    backend_registry, bind_backend_or_error, execute_with_backend, BackendBindingRequest,
    BackendCapabilities, BackendContext, BackendError, BackendKind, BackendLifecycleResult,
    EngineOutcome, ExecutionAttemptRecord, ExecutionBackend, ExecutionBackendCapabilityDescriptor,
    FakeBackend, ProcessLikeBackend,
};
#[doc(hidden)]
pub use execution_context::{ExecutionContext, NodeExecutionContext};
#[doc(hidden)]
pub use execution_plan::{ExecutionPlan, PlannedDependency, PlannedNode};
#[doc(hidden)]
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
#[doc(hidden)]
pub use external_adapter::{
    probe_external_adapters, ExternalAdapterHandshakeReport, ExternalAdapterHandshakeStatus,
};
use file_transform_adapter::FileTransformAdapter;
#[doc(hidden)]
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
use http_adapter::HttpRequestAdapter;
#[doc(hidden)]
pub use infrastructure::{
    negotiate_backend_capabilities, BackendCapabilities as InfrastructureBackendCapabilities,
    BackendCapabilityRequirement, BackendExecutionCompletion, BackendExecutionRequest,
    CapabilityDecision, ExecutorBackend,
};
#[doc(hidden)]
pub use invariants::{
    run_summary_invariant_ok, terminal_run_has_terminal_node, trace_time_order_ok, RunNodeCounts,
    INVARIANT_REGISTRY,
};
use io::{Fs, StdFs};
#[doc(hidden)]
pub use kubernetes_execution::{
    build_kubernetes_execution_request, kubernetes_pod_status_from_node_result,
    map_kubernetes_pod_status_to_node_status, validate_kubernetes_execution_request,
    KubernetesBackendExecutor, KubernetesExecutionRequest, KubernetesExecutionResult,
    KubernetesJobRecord, KubernetesLogCapture, KubernetesPodLifecycleEvent, KubernetesPodPhase,
    KubernetesPodStatus, KubernetesVolumeMount, KubernetesWorkloadDescriptor,
    KubernetesWorkloadKind, KubernetesWorkspaceTransfer, KubernetesWorkspaceTransferMode,
    MockKubernetesBackend, SystemKubernetesBackend, SystemKubernetesBackendConfig,
    SystemKubernetesPaths,
};
#[doc(hidden)]
pub use local_executor::LocalExecutor;
#[doc(hidden)]
pub use local_worker_pool::{
    LocalWorkerAssignment, LocalWorkerCompletion, LocalWorkerExecution, LocalWorkerPool,
    LocalWorkerState, LocalWorkerStatus,
};
#[doc(hidden)]
pub use observability::{
    canonicalize_event_records, category_from_runtime_event_name, current_process_memory_bytes,
    event_contains_sensitive_material, event_names_emitted_once, reconstruct_timeline_from_events,
    required_event_fields_present, serialize_timeline_export, summarize_failure_root_causes,
    validate_required_event_names, validate_required_timeline_labels,
    verify_event_log_completeness, write_timeline_export, EventCategory,
    EventLogCompletenessReport, EventRecord, EventSink, FileEventSink, InMemoryMetricsRegistry,
    MetricsRegistry, NodeMetrics, RemoteCollectorSink, RunMetrics, SchedulerMetrics, SpanKind,
    StdoutEventSink, TimelineEntry, TimelineExport, TraceSpan, REQUIRED_RUNTIME_EVENT_NAMES,
};
#[doc(hidden)]
pub use observability_deep::{
    build_diagnostics, build_topology_overlay, detect_metric_drift, observability_contract_status,
    redact_event_details, render_timeline_text, root_cause_graph, sample_events, AlertRule,
    DiagnosticRecord, DiagnosticsKind, DriftDetectionReport, EventCorrelation,
    ExplainArtifactReport, ExplainNodeReport, ExplainRunReport, ExplainScheduleReport,
    FailureCauseCode, MetricsExportFormat, ObservabilityContractStatus, RedactionPolicy,
    ReplaySpanLink, SamplingPolicy, TimelineTextSummary, TopologyOverlay, TopologyOverlayNode,
};
#[doc(hidden)]
pub use path_authorization::{authorize_input_path, authorize_output_path};
#[doc(hidden)]
pub use path_resolution::AbsolutePathPolicy;
pub(crate) use path_resolution::{
    bind_path_variables_in_value, collect_container_argv_path_usages,
    collect_container_workdir_usage, collect_resolved_path_usages, resolve_container_argv,
    resolve_container_workdir, NodePathBindings, ResolvedPathUsage,
};
#[doc(hidden)]
pub use performance_capacity::{
    build_cost_model, build_performance_maturity_report, compile_environment_profiles,
    derive_autoscaling_hint, detect_performance_regression, forecast_storage_growth,
    synthetic_large_dag_profiles, ArtifactStoreBenchmarkResult, AutoscalingHint, BenchmarkResult,
    CapacityModel, EnvironmentScaleProfile, PerformanceGate, PerformanceMaturityReport,
    SchedulerScalabilityResult, StorageCostModel, StorageGrowthForecast, SyntheticDagProfile,
};
#[doc(hidden)]
pub use planner::build_plan;
#[doc(hidden)]
pub use planner_analysis::{
    build_backfill_plan, build_planner_analysis, build_replay_plan_annotations,
    compare_plan_equivalence, compute_downstream_run_closure, compute_partial_run_closure,
    compute_upstream_run_closure, diff_plans, explain_plan, fingerprint_plan, PlannerBackfillPlan,
    PlannerBlockedNodeEstimate, PlannerBuildResult, PlannerCriticalPathEstimate,
    PlannerCriticalPathNode, PlannerDurationSource, PlannerEquivalenceClass,
    PlannerEquivalenceReport, PlannerExecutionCostEstimate, PlannerExplainReport,
    PlannerGuardrails, PlannerNodeAction, PlannerNodeAnnotation, PlannerNodePathPreview,
    PlannerPhase, PlannerPlanDiff, PlannerPriorityInheritance, PlannerResourceBottleneck,
    PlannerSchedulingBound, PlannerSchedulingSimulation,
};
#[doc(hidden)]
pub use policy::policy_allows_effects;
use python_adapter::PythonFunctionAdapter;
#[doc(hidden)]
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
#[doc(hidden)]
pub use remote_execution_model::{
    execute_remote_payload_in_place, execution_mode_status, remote_handoff_valid,
    remote_input_artifact_digest_matches, serialize_node_result_payload,
    validate_remote_execution_fingerprint_set, validate_remote_execution_payload,
    validate_remote_execution_workspace, validate_remote_identity, validate_remote_input_artifact,
    ExecutionModeStatus, MockRemoteWorker, RemoteArtifactHandoff, RemoteExecutionFingerprintSet,
    RemoteExecutionIdentity, RemoteExecutionWorkspace, RemoteInputArtifact,
    RemoteNodeExecutionPayload, RemoteNodeExecutionResult, RemoteObservabilityHandoff,
    RemoteWorkerExecutor,
};
#[doc(hidden)]
pub use remote_executor::{
    RemoteExecutionReceipt, RemoteExecutionRequest, RemoteExecutorSubmitter,
};
#[doc(hidden)]
pub use run_state::{
    imported_run_distinguishable, node_transition_invariant_id, run_transition_invariant_id,
    terminal_transition_audit_events, validate_node_transition, validate_run_transition,
    verify_post_run_state_consistency, NodeState, NodeTransition, PartialRerunContract,
    ReplayNodeAction, ReplayNodeProvenance, ResumeFailureMode, ResumeSummary, RunAttempt,
    RunCompactionPolicy, RunComparison, RunId, RunSnapshot, RunState, RunSummaryV2, RunTransition,
    StateConsistencyReport, TransitionAuditEvent, TransitionCause, INV_NODE_TERMINAL_NO_REVERT,
    INV_RUN_FAILED_CAUSAL_FAILURE,
};
#[doc(hidden)]
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
#[doc(hidden)]
pub use runtime_semantics::*;
#[doc(hidden)]
pub use scheduler::{
    advance_backfill_operation, apply_submission_status_updates, build_schedule_override_status,
    build_schedule_queue_state, build_scheduler, cancel_backfill_operation,
    compile_backfill_operation, compile_submission_request, deterministic_tick_order,
    dispatch_schedule_queue_runs, dry_run_schedule, evaluate_schedule_submissions,
    evaluate_schedule_submissions_with_overrides, failure_allows_downstream_readiness,
    failure_mode_name, pause_backfill_operation, pause_schedule, record_schedule_override,
    replay_scheduler_checkpoint, resume_backfill_operation, resume_schedule,
    retry_failed_backfill_runs, scheduler_contract_profile, scheduler_debug_event_log,
    scheduler_invariant_violations, scheduler_invariants_hold, summarize_backfill_operation,
    validate_cron_expression, validate_schedule_policy_combination, validate_schedule_registry,
    BackfillAdvanceReport, BackfillAdvanceRequest, BackfillAuditRecord, BackfillFailurePolicy,
    BackfillLifecycleStatus, BackfillOperation, BackfillOperationSummary, BackfillPartitionSummary,
    BackfillRequest, BackfillRunRecord, BackfillRunStatus, BackfillStatusUpdate,
    BackfillStatusUpdateBatch, CatchUpPolicy, ConcurrencyPolicyLayers, DependencyCompletionRecord,
    DependencyCounter, DependencyTriggerCondition, DeterministicScheduler, ExecutionCheckpoint,
    ExecutionSubmissionRequest, FailurePropagationMode, ManualSubmissionRequest,
    NoopSchedulerEventHook, PriorityClass, QueueIdentity, QueueIsolationPolicy, ReadyQueue,
    ReadyTieBreak, ScheduleAuditRecord, ScheduleDefinition, ScheduleDispatchRecord,
    ScheduleDispatchReport, ScheduleDryRunPreview, ScheduleEvaluationInputs,
    ScheduleEvaluationReport, ScheduleEventLineage, ScheduleEventRecord, ScheduleInputSource,
    SchedulePriorityDispatchPolicy, ScheduleQueueRunRecord, ScheduleQueueState,
    ScheduleQueueStateEntry, ScheduleQueueTenantState, ScheduleRegistry, ScheduleSubmissionLedger,
    ScheduleSubmissionLedgerEntry, ScheduleSubmissionStatus, ScheduleSubmissionStatusUpdate,
    ScheduleSubmissionStatusUpdateBatch, ScheduledSubmission, Scheduler, SchedulerContractProfile,
    SchedulerEvent, SchedulerEventHook, SchedulerEventKind, SchedulerFairness, SchedulerModel,
    SchedulerPolicy, SchedulerPriorityModel, SchedulerState, SchedulerUnit, SignalRecord,
    SubmissionTriggerKind, ThroughputScheduler, TriggerSpec,
};
#[doc(hidden)]
pub use scheduler_workload::{
    apply_backfill_throttling, compute_partition_backfill_batches, deduplicate_trigger_events,
    detect_cron_conflicts, evaluate_sla_metrics, is_suppressed_by_calendar, materialize_next_runs,
    run_batches, weighted_priority_tie_break_order, BackfillThrottlingPolicy, BlackoutWindow,
    ConcurrencyScope, CronConflict, CrossSchedulerCompatibility, DagCalendar,
    DependencyTriggerBufferPolicy, EnvironmentSuppression, FairnessAlgorithm, HolidayPolicy,
    MaterializedRunPreview, PartitionBackfillOrchestration, QueueAdmissionPolicy, RunBatchPolicy,
    ScheduleOverrideAction, ScheduleOverrideRecord, ScheduleOverrideState, ScheduleOverrideStatus,
    ScheduleSuppressionAnnotation, SchedulerAlertRule, SchedulerMaturityMatrix,
    SchedulerSlaMetrics, SchedulingSimulationSuite, ServiceClass, SlaPolicy,
    StarvationPreventionPolicy, TriggerDedupDecision, WeightedPriorityPolicy,
};
#[doc(hidden)]
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
#[doc(hidden)]
pub use security_env::{
    declared_environment, effective_env_allowlist, is_allowed_env_key, is_denied_env_key,
    missing_required_env_keys, shape_environment,
};
#[doc(hidden)]
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
#[doc(hidden)]
pub use slurm_execution::{
    build_slurm_execution_request, build_slurm_scheduler_request,
    map_slurm_job_status_to_node_status, validate_slurm_execution_request,
    validate_slurm_scheduler_request, MockSlurmBackend, SlurmBackendExecutor,
    SlurmExecutionRequest, SlurmExecutionResult, SlurmJobLifecycleEvent, SlurmJobRecord,
    SlurmJobStatus, SlurmLogCapture, SlurmSchedulerRequest, SystemSlurmBackend,
    SystemSlurmBackendConfig, SystemSlurmPaths,
};
#[doc(hidden)]
pub use state_machine::{
    failure_propagation_is_deterministic, node_transition_allowed, run_transition_allowed,
    NodeLifecycleState, RunLifecycleState,
};
use std::collections::{BTreeMap, HashMap};
use std::io::{self as std_io, Read, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;
#[doc(hidden)]
pub use store::{validate_storage_relative_path, ArtifactStore, CacheStore, StorageHealthReport};
use store::{ArtifactStore as RuntimeArtifactStore, CacheStore as RuntimeCacheStore};
#[doc(hidden)]
pub use task_contract::{
    build_retry_policy, build_task_contract, default_forced_cleanup, evaluate_retry_decision,
    retry_backoff_ms as contract_retry_backoff_ms, retry_jitter_ms as contract_retry_jitter_ms,
    retry_observation, retry_observation_from_failure, retry_wait_ms as contract_retry_wait_ms,
    validate_task_contracts, BackoffStrategy, ForcedCancellationCleanup, IdempotencyMode,
    NodeProvenance, OutputMaterializationPolicy, RetryDecision, RetryFailureObservation,
    RetryPolicyV2, RuntimeState, SideEffectClassification, TaskContract, TaskFailureReason,
    TaskInputDescriptor, TaskIsolationMode, TaskOutputDescriptor, TaskResultEnvelope,
    TimeoutPolicy, TimeoutRetryPolicy,
};
#[doc(hidden)]
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
#[doc(hidden)]
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
        AbsolutePathPolicy, CacheKeyInput, CacheMode, ExecutionBackendTarget, ExecutionContext,
        NodeExecutionContext, NodeLifecycleState, PlannerGuardrails, RunLifecycleState, Runtime,
        RuntimeConfig, RuntimeError, SchedulerPolicy, SelectorSet, SlurmRuntimeConfig,
    };
}

/// Common imports for planning, scheduling, and executing local DAG runs.
pub mod prelude {
    pub use crate::stable::{
        build_plan, build_planner_analysis, build_scheduler, AbsolutePathPolicy, CacheMode,
        ExecutionBackendTarget, ExecutionContext, NodeExecutionContext, PlannerGuardrails, Runtime,
        RuntimeConfig, RuntimeError, SchedulerPolicy, SelectorSet, SlurmRuntimeConfig,
    };
}

/// Opt-in contract and evidence helpers that are outside the stable runtime lane.
#[cfg(feature = "experimental-public-api")]
pub mod experimental {
    pub mod adapter_execution {
        pub use crate::adapter_execution_contracts::*;
    }
    pub mod write_boundaries {
        pub use crate::write_boundary_contracts::*;
    }
    pub mod planner_admission {
        pub use crate::planner_admission_contracts::*;
    }
    pub mod durable_queue {
        pub use crate::durable_queue_contracts::*;
    }
    pub mod container_evidence {
        pub use crate::container_evidence_contracts::*;
    }
    pub mod observability_taxonomy {
        pub use crate::observability_taxonomy_contracts::*;
    }
}

/// Runtime-level failure classification for planning, execution, and artifact work.
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

/// Terminal status reported for a node after execution or cache reuse.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Success,
    Failed,
    Skipped,
    Cached,
    Cancelled,
}

/// Execution-scoped state shared across node adapter invocations for one run.
pub struct RunContext {
    pub run_dir: Arc<RunDir>,
    pub replay_source_run_dir: Option<PathBuf>,
    pub graph_fingerprint: Arc<Mutex<HashMap<String, String>>>,
    pub node_definition_fingerprints: Arc<HashMap<String, String>>,
    pub declared_environment_fingerprints: Arc<HashMap<String, String>>,
    pub params_fingerprints: Arc<HashMap<String, String>>,
    pub command_fingerprints: Arc<HashMap<String, Option<String>>>,
    pub planner_contract_version: String,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub execution_contract_fingerprint: String,
    pub resolved_params: HashMap<String, Value>,
    pub effective_cache_dir: Option<PathBuf>,
    pub fs: Arc<dyn Fs>,
    pub clock: Arc<dyn Clock>,
    pub store: RuntimeArtifactStore,
    pub policy: PolicyConfig,
    pub absolute_path_policy: AbsolutePathPolicy,
    pub cancellation_requested: Arc<std::sync::atomic::AtomicBool>,
}

/// Artifact, status, and failure evidence recorded for one node execution result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MapExecutionSummary {
    schema_version: String,
    map_node_id: String,
    input_port: String,
    item_count: usize,
    successful_item_count: usize,
    failed_item_count: usize,
    cancelled_item_count: usize,
    items: Vec<MapExecutionItemSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MapExecutionItemSummary {
    item_id: String,
    item_sha256: String,
    status: String,
    run_dir: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    outputs: Vec<MapExecutionOutputSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure: Option<FailureInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MapExecutionOutputSummary {
    output_name: String,
    item_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReduceExecutionMode {
    AllSuccess,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReduceEmptyPolicy {
    Forbid,
    Allow,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReduceExecutionConfig {
    pub mode: ReduceExecutionMode,
    pub empty_policy: ReduceEmptyPolicy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ReduceExecutionSummary {
    schema_version: String,
    reduce_node_id: String,
    mode: String,
    empty_policy: String,
    usable_input_count: usize,
    failed_input_count: usize,
    skipped_input_count: usize,
    cancelled_input_count: usize,
    collection: InputCollection,
}

/// Timestamped attempt evidence for a retried or single-shot node execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttemptEvent {
    pub attempt: u32,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<FailureInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_backoff_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_decision: Option<RetryDecision>,
}

#[derive(Debug)]
pub(crate) enum ControlledCommandResult {
    Exited(ControlledCommandOutput),
    TimedOut(ControlledCommandOutput),
    Cancelled(ControlledCommandOutput),
}

impl ControlledCommandResult {
    fn output(&self) -> &ControlledCommandOutput {
        match self {
            Self::Exited(output) | Self::TimedOut(output) | Self::Cancelled(output) => output,
        }
    }

    pub(crate) fn persist_streams(
        &self,
        fs: &dyn Fs,
        stdout_path: &Path,
        stderr_path: &Path,
    ) -> Result<(), RuntimeError> {
        let output = self.output();
        output.stdout.copy_to(fs, stdout_path)?;
        output.stderr.copy_to(fs, stderr_path)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ControlledCommandOutput {
    status: std::process::ExitStatus,
    stdout: ControlledCommandStream,
    stderr: ControlledCommandStream,
}

impl ControlledCommandOutput {
    fn read_tail_bytes(&self, max_bytes: u64) -> Result<Vec<u8>, RuntimeError> {
        self.stderr.read_tail_bytes(max_bytes).map_err(RuntimeError::Io)
    }
}

#[derive(Debug)]
struct ControlledCommandStream {
    path: PathBuf,
}

impl ControlledCommandStream {
    fn copy_to(&self, fs: &dyn Fs, destination: &Path) -> Result<(), RuntimeError> {
        fs.copy(&self.path, destination).map(|_| ()).map_err(RuntimeError::Io)
    }

    fn read_tail_bytes(&self, max_bytes: u64) -> std_io::Result<Vec<u8>> {
        read_file_tail_bytes(&self.path, max_bytes)
    }

    fn append_cleanup_diagnostics(&self, cleanup_diagnostics: &[String]) -> std_io::Result<()> {
        if cleanup_diagnostics.is_empty() {
            return Ok(());
        }

        let mut file = std::fs::OpenOptions::new().append(true).open(&self.path)?;
        if file.metadata()?.len() > 0 {
            file.write_all(b"\n")?;
        }
        for diagnostic in cleanup_diagnostics {
            file.write_all(b"[bijux cleanup] ")?;
            file.write_all(diagnostic.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl Drop for ControlledCommandStream {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlledCommandOutcomeKind {
    Exited,
    TimedOut,
    Cancelled,
}

#[derive(Debug)]
struct ControlledCommandTermination {
    status: std::process::ExitStatus,
    cleanup_diagnostics: Vec<String>,
}

impl ControlledCommandTermination {
    fn new(status: std::process::ExitStatus) -> Self {
        Self { status, cleanup_diagnostics: Vec::new() }
    }
}

/// Built-in adapter that writes constant JSON payloads into declared outputs.
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
        if let Err(failure) = preflight_declared_output_targets(&outputs_dir, &node.outputs) {
            return node_failure_result(
                exec.fs.as_ref(),
                &stdout_path,
                &stderr_path,
                &outputs_dir,
                NodeStatus::Failed,
                failure,
                b"declared output path preflight failed",
            );
        }

        let value = params.get("value").cloned().unwrap_or(Value::Null);
        let target = node
            .outputs
            .iter()
            .find(|o| o.name == "value")
            .or_else(|| node.outputs.first())
            .ok_or_else(|| RuntimeError::Executor("no outputs declared".to_string()))?;
        let out_path = authorized_declared_output_path(&outputs_dir, target)
            .map_err(|failure| RuntimeError::Executor(failure.message))?;
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

/// Built-in adapter that executes local shell commands inside the run boundary.
#[derive(Clone)]
pub struct ShellAdapter;

fn shell_argv_failure(message: impl Into<String>, reason: &'static str) -> FailureInfo {
    FailureInfo::new(
        FailureClass::User,
        "User",
        "EXEC_ERROR",
        message,
        Some(serde_json::json!({
            "field": "argv",
            "reason": reason,
        })),
    )
}

fn shell_argv(params: &Value) -> Result<Vec<String>, FailureInfo> {
    let Some(argv_value) = params.get("argv") else {
        return Err(shell_argv_failure("argv is required", "missing"));
    };
    let Some(argv) = argv_value.as_array() else {
        return Err(shell_argv_failure("argv must be an array of strings", "expected_array"));
    };
    if argv.is_empty() {
        return Err(shell_argv_failure("argv must not be empty", "empty"));
    }

    let mut args = Vec::with_capacity(argv.len());
    for (index, value) in argv.iter().enumerate() {
        let Some(arg) = value.as_str() else {
            return Err(FailureInfo::new(
                FailureClass::User,
                "User",
                "EXEC_ERROR",
                format!("argv[{index}] must be a string"),
                Some(serde_json::json!({
                    "field": "argv",
                    "reason": "non_string_entry",
                    "index": index,
                })),
            ));
        };
        if index == 0 && arg.trim().is_empty() {
            return Err(shell_argv_failure(
                "argv[0] must resolve to a non-empty executable",
                "blank_executable",
            ));
        }
        args.push(arg.to_string());
    }
    Ok(args)
}

fn node_failure_result(
    fs: &dyn Fs,
    stdout_path: &Path,
    stderr_path: &Path,
    outputs_dir: &Path,
    status: NodeStatus,
    failure: FailureInfo,
    stderr_contents: &[u8],
) -> Result<NodeResult, RuntimeError> {
    fs.write(stdout_path, b"")?;
    fs.write(stderr_path, stderr_contents)?;
    Ok(NodeResult {
        status,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure: Some(failure),
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
    })
}

pub(crate) fn authorized_declared_output_path(
    output_root: &Path,
    output: &FileOutput,
) -> Result<PathBuf, FailureInfo> {
    crate::path_authorization::authorize_declared_output_target(output_root, &output.path).map_err(
        |message| {
            FailureInfo::new(
                FailureClass::User,
                "User",
                "OUTPUT_PATH_INVALID",
                message,
                Some(serde_json::json!({
                    "output": output.name,
                    "path": output.path,
                })),
            )
        },
    )
}

pub(crate) fn preflight_declared_output_targets(
    output_root: &Path,
    outputs: &[FileOutput],
) -> Result<(), FailureInfo> {
    for output in outputs {
        authorized_declared_output_path(output_root, output)?;
    }
    Ok(())
}

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
        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        if let Err(failure) = preflight_declared_output_targets(&outputs_dir, &node.outputs) {
            return node_failure_result(
                exec.fs.as_ref(),
                &stdout_path,
                &stderr_path,
                &outputs_dir,
                NodeStatus::Failed,
                failure,
                b"declared output path preflight failed",
            );
        }
        let args = match shell_argv(params) {
            Ok(args) => args,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };

        let env_allowlist = effective_env_allowlist(node);
        let mut cmd = subprocess::command(&args[0]);
        cmd.args(&args[1..]);
        cmd.current_dir(&work_dir);
        apply_shaped_env(&mut cmd, exec.policy.clean_env, &env_allowlist, &[]);
        apply_temp_env(&mut cmd, &exec.run_dir.node_temp_dir(&node.id));

        let output = match command_output_with_controls(
            &mut cmd,
            effective_node_timeout_ms(node, params),
            Some(exec.cancellation_requested.as_ref()),
        ) {
            Ok(output) => output,
            Err(RuntimeError::Io(error)) if error.kind() == std_io::ErrorKind::NotFound => {
                let failure = FailureInfo::new(
                    FailureClass::Infrastructure,
                    "Infrastructure",
                    "MISSING_EXECUTABLE",
                    format!("executable could not be resolved: {}", args[0]),
                    Some(serde_json::json!({
                        "executable": args[0],
                        "io_error_kind": "not_found",
                        "os_error_code": error.raw_os_error(),
                    })),
                );
                let stderr_message = failure.message.clone();
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
            Err(error) => return Err(error),
        };

        output.persist_streams(exec.fs.as_ref(), &stdout_path, &stderr_path)?;
        match output {
            ControlledCommandResult::TimedOut(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Failed,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(FailureInfo::new(
                        FailureClass::Timeout,
                        "Timeout",
                        "EXEC_TIMEOUT",
                        "execution timed out after configured node timeout",
                        Some(serde_json::json!({ "exit_code": output.status.code() })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Cancelled(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Cancelled,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(FailureInfo::new(
                        FailureClass::Execution,
                        "Execution",
                        "EXEC_CANCELLED",
                        "execution cancelled by operator",
                        Some(serde_json::json!({ "exit_code": output.status.code() })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Exited(output) => {
                let success = output.status.success();
                let exit_code = output.status.code();
                if !success {
                    return Ok(NodeResult {
                        status: NodeStatus::Failed,
                        stdout_path: stdout_path.display().to_string(),
                        stderr_path: stderr_path.display().to_string(),
                        outputs_dir: outputs_dir.display().to_string(),
                        output_evidence: Vec::new(),
                        failure: Some(FailureInfo::new(
                            FailureClass::Execution,
                            "Execution",
                            "EXEC_FAIL",
                            "command failed",
                            Some(serde_json::json!({ "exit_code": exit_code })),
                        )),
                        attempts: 1,
                        attempt_events: Vec::new(),
                        container_meta: None,
                        adapter_binary_sha256: None,
                    });
                }
            }
        }

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

/// Built-in adapter that executes container workloads through a supported engine.
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
        if let Err(failure) = preflight_declared_output_targets(&outputs_dir, &node.outputs) {
            return node_failure_result(
                exec.fs.as_ref(),
                &stdout_path,
                &stderr_path,
                &outputs_dir,
                NodeStatus::Failed,
                failure,
                b"declared output path preflight failed",
            );
        }

        let engine = spec.engine.as_str();
        if let Err(failure) = enforce_container_image_reference_policy(
            spec.image.as_str(),
            exec.policy.container_image_reference_policy,
        ) {
            exec.fs.write(&stdout_path, b"")?;
            exec.fs.write(&stderr_path, failure.message.as_bytes())?;
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: Vec::new(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(spec, engine, None, None)),
                adapter_binary_sha256: None,
            });
        }
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
                    failure: Some(FailureInfo::new(
                        FailureClass::Infrastructure,
                        "Infrastructure",
                        "CONTAINER_ENGINE_UNAVAILABLE",
                        message.clone(),
                        Some(serde_json::json!({ "engine": engine })),
                    )),
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
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "CONTAINER_VOLUME_CONTRACT_INVALID",
                    message,
                    Some(serde_json::json!({ "engine": engine })),
                )),
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
                        failure: Some(FailureInfo::new(
                            FailureClass::Policy,
                            "Policy",
                            "POLICY_UNENFORCEABLE",
                            message,
                            Some(serde_json::json!({ "engine": engine, "effect": "network" })),
                        )),
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
        let gpu_devices = bijux_dag_core::resources::node_gpu_devices(node);
        let gpu_args = match container_execution::container_gpu_runtime_args(engine, gpu_devices) {
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
                    failure: Some(FailureInfo::new(
                        FailureClass::Infrastructure,
                        "Infrastructure",
                        "CONTAINER_GPU_UNSUPPORTED",
                        message,
                        Some(serde_json::json!({ "engine": engine, "gpu_devices": gpu_devices })),
                    )),
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
        for arg in gpu_args {
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
        let temp_dir = container_temp_dir(&workdir);
        cmd.arg("-e").arg(format!("TMPDIR={temp_dir}"));
        cmd.arg("-e").arg(format!("TMP={temp_dir}"));
        cmd.arg("-e").arg(format!("TEMP={temp_dir}"));

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

        let output = command_output_with_controls(
            &mut cmd,
            effective_node_timeout_ms(node, params),
            Some(exec.cancellation_requested.as_ref()),
        )?;
        output.persist_streams(exec.fs.as_ref(), &stdout_path, &stderr_path)?;
        let timeout_failure = || {
            FailureInfo::new(
                FailureClass::Timeout,
                "Timeout",
                "EXEC_TIMEOUT",
                "container execution timed out after configured node timeout",
                None,
            )
        };
        let cancelled_failure = || {
            FailureInfo::new(
                FailureClass::Execution,
                "Execution",
                "EXEC_CANCELLED",
                "execution cancelled by operator",
                None,
            )
        };
        let exit_code = match output {
            ControlledCommandResult::TimedOut(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Failed,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(timeout_failure()),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: Some(container_trace(
                        spec,
                        engine,
                        output.status.code(),
                        Some(engine_version.clone()),
                    )),
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Cancelled(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Cancelled,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(cancelled_failure()),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: Some(container_trace(
                        spec,
                        engine,
                        output.status.code(),
                        Some(engine_version.clone()),
                    )),
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Exited(output) => {
                let success = output.status.success();
                let exit_code = output.status.code();
                if !success {
                    return Ok(NodeResult {
                        status: NodeStatus::Failed,
                        stdout_path: stdout_path.display().to_string(),
                        stderr_path: stderr_path.display().to_string(),
                        outputs_dir: outputs_dir.display().to_string(),
                        output_evidence: Vec::new(),
                        failure: Some(FailureInfo::new(
                            FailureClass::Execution,
                            "Execution",
                            "EXEC_FAIL",
                            "container command failed",
                            Some(serde_json::json!({ "exit_code": exit_code })),
                        )),
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
                exit_code
            }
        };

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

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: Some(container_trace(spec, engine, exit_code, Some(engine_version))),
            adapter_binary_sha256: None,
        })
    }
}

/// Cache read and write policy for runtime execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheMode {
    Off,
    Read,
    ReadWrite,
}

/// Behavior to apply when a whole-run timeout is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTimeoutBehavior {
    FinishRunning,
    CancelRunning,
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

/// Runtime configuration for planning, selection, caching, policy, and scheduling.
#[derive(Clone)]
pub struct RuntimeConfig {
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub memory_budget_mb: Option<u32>,
    pub gpu_device_budget: Option<u32>,
    pub named_resource_capacities: BTreeMap<String, u32>,
    pub run_timeout_ms: Option<u64>,
    pub run_timeout_behavior: RunTimeoutBehavior,
    pub node_timeout_ms: Option<u64>,
    pub materialize_inputs: MaterializeMode,
    pub cache_mode: CacheMode,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
    pub run_root: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicy,
    pub run_id: Option<String>,
    pub resume_run_id: Option<String>,
    pub resume_failure_mode: ResumeFailureMode,
    pub parent_run_id: Option<String>,
    pub replay_source_run_dir: Option<PathBuf>,
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
    pub execution_backend: ExecutionBackendTarget,
    pub kubernetes: KubernetesRuntimeConfig,
    pub slurm: SlurmRuntimeConfig,
}

/// Selected execution backend for node launches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionBackendTarget {
    #[default]
    Local,
    Kubernetes,
    Slurm,
}

/// Runtime configuration for the Kubernetes Job backend.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KubernetesRuntimeConfig {
    pub default_namespace: String,
    pub shared_volume_claim: String,
    pub shared_local_root: PathBuf,
    pub poll_interval_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kubectl_command: Option<String>,
}

impl Default for KubernetesRuntimeConfig {
    fn default() -> Self {
        Self {
            default_namespace: "bijux".to_string(),
            shared_volume_claim: String::new(),
            shared_local_root: PathBuf::new(),
            poll_interval_ms: 250,
            kubectl_command: None,
        }
    }
}

/// Runtime configuration for the SLURM backend.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlurmRuntimeConfig {
    pub default_queue: String,
    pub default_partition: String,
    pub poll_interval_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worker_command: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbatch_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sacct_command: Option<String>,
}

impl Default for SlurmRuntimeConfig {
    fn default() -> Self {
        Self {
            default_queue: "general".to_string(),
            default_partition: "cpu".to_string(),
            poll_interval_ms: 250,
            worker_command: Vec::new(),
            sbatch_command: None,
            sacct_command: None,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            jobs: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            named_resource_capacities: BTreeMap::new(),
            run_timeout_ms: None,
            run_timeout_behavior: RunTimeoutBehavior::FinishRunning,
            node_timeout_ms: None,
            materialize_inputs: MaterializeMode::Copy,
            cache_mode: CacheMode::Off,
            cache_dir: None,
            remote_cache_dir: None,
            run_root: None,
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            run_id: None,
            resume_run_id: None,
            resume_failure_mode: ResumeFailureMode::RerunIncomplete,
            parent_run_id: None,
            replay_source_run_dir: None,
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
            failure_propagation: FailurePropagationMode::ContinueIndependent,
            execution_backend: ExecutionBackendTarget::Local,
            kubernetes: KubernetesRuntimeConfig::default(),
            slurm: SlurmRuntimeConfig::default(),
        }
    }
}

/// Include and exclude selectors applied before execution begins.
#[derive(Debug, Clone, Default)]
pub struct SelectorSet {
    pub include: Vec<Selector>,
    pub exclude: Vec<Selector>,
}

/// Node selection rule used for partial execution and rerun workflows.
#[derive(Debug, Clone)]
pub enum Selector {
    Id(String),
    IdPrefix(String),
    Tag(String),
    Kind(String),
}

/// Input materialization strategy for upstream artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializeMode {
    Copy,
    Hardlink,
    Symlink,
}

/// Policy flags that constrain ambient effects during runtime execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyConfig {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    pub container_image_reference_policy: ContainerImageReferencePolicy,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
            container_image_reference_policy: ContainerImageReferencePolicy::RequireDigest,
        }
    }
}

/// Runtime entrypoint for executing validated graphs against registered adapters.
pub struct Runtime {
    registry: AdapterRegistry,
    fs: Arc<dyn Fs>,
    clock: Arc<dyn Clock>,
    init_error: Option<String>,
}

impl Runtime {
    /// Builds a runtime with the default adapter registry, filesystem, and clock.
    pub fn new() -> Self {
        let registry_result = build_registry(vec![
            Arc::new(ConstAdapter),
            Arc::new(FileTransformAdapter),
            Arc::new(HttpRequestAdapter),
            Arc::new(ShellAdapter),
            Arc::new(PythonFunctionAdapter),
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

    /// Executes a validated graph with the supplied runtime configuration.
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
        engine::execute(self, graph, out_dir, options)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_gpu_runtime_capacity(
    plan: &ExecutionPlan,
    options: &RuntimeConfig,
) -> Result<(), RuntimeError> {
    let gpu_device_budget =
        options.scheduler_policy.gpu_device_budget.or(options.gpu_device_budget);
    let mut required_nodes = Vec::new();
    let mut oversized_nodes = Vec::new();

    for node in &plan.nodes {
        let requested = bijux_dag_core::resources::node_gpu_devices(node);
        if requested == 0 {
            continue;
        }
        required_nodes.push((node.id.clone(), requested));
        if gpu_device_budget.is_some_and(|budget| requested > budget) {
            oversized_nodes.push((node.id.clone(), requested));
        }
    }

    if required_nodes.is_empty() {
        return Ok(());
    }

    let Some(gpu_device_budget) = gpu_device_budget.filter(|budget| *budget > 0) else {
        let requested = required_nodes
            .iter()
            .map(|(node_id, requested)| format!("{node_id}={requested}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(RuntimeError::Executor(format!(
            "selected nodes require gpu devices ({requested}), but runtime gpu_device_budget is unset"
        )));
    };

    if oversized_nodes.is_empty() {
        return Ok(());
    }

    let requested = oversized_nodes
        .iter()
        .map(|(node_id, requested)| format!("{node_id}={requested}"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(RuntimeError::Executor(format!(
        "selected nodes require more gpu devices than runtime gpu_device_budget={gpu_device_budget}: {requested}"
    )))
}

fn validate_named_resource_runtime_capacity(
    plan: &ExecutionPlan,
    options: &RuntimeConfig,
) -> Result<(), RuntimeError> {
    let mut capacities = options.named_resource_capacities.clone();
    for (name, amount) in &options.scheduler_policy.named_resource_capacities {
        capacities.insert(name.clone(), *amount);
    }

    let mut missing = BTreeMap::<String, Vec<String>>::new();
    let mut oversized = Vec::new();
    for node in &plan.nodes {
        for (name, requested) in bijux_dag_core::resources::node_named_resources(node) {
            match capacities.get(&name).copied().filter(|capacity| *capacity > 0) {
                Some(capacity) if requested > capacity => {
                    oversized.push((node.id.clone(), name, requested, capacity));
                }
                Some(_) => {}
                None => {
                    missing.entry(name).or_default().push(format!("{}={requested}", node.id));
                }
            }
        }
    }

    if !missing.is_empty() {
        let requested = missing
            .into_iter()
            .map(|(name, nodes)| format!("{name}({})", nodes.join(", ")))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(RuntimeError::Executor(format!(
            "selected nodes require named resources without runtime capacity: {requested}"
        )));
    }

    if oversized.is_empty() {
        return Ok(());
    }

    let requested = oversized
        .into_iter()
        .map(|(node_id, name, requested, capacity)| {
            format!("{node_id}:{name}={requested} exceeds capacity {capacity}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    Err(RuntimeError::Executor(format!(
        "selected nodes require more named resources than runtime capacities allow: {requested}"
    )))
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
    lifecycle_state: Option<String>,
    lifecycle_transitions: Vec<NodeLifecycleTransition>,
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
    let cache_identity = Some(cache_identity_for_trace(
        ctx,
        node_id,
        adapter_id,
        adapter_version,
        adapter_binary_sha256.as_deref(),
        adapter_outputs_schema_version,
    )?);
    let exit_code = terminal_exit_code(node, &status, failure.as_ref(), container_meta.as_ref());
    let stdout = collect_node_log_evidence(
        ctx.fs.as_ref(),
        &ctx.run_dir,
        &ctx.run_dir.node_stdout_path(node_id),
    );
    let stderr = collect_node_log_evidence(
        ctx.fs.as_ref(),
        &ctx.run_dir,
        &ctx.run_dir.node_stderr_path(node_id),
    );
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
        resources: node.resources.as_ref().map(|r| TraceResources {
            cpu: r.cpu,
            mem_mb: r.mem_mb,
            gpu_devices: r.gpu_devices,
        }),
        inputs_index,
        resolved_params: ctx.resolved_params.get(node_id).cloned(),
        exit_code,
        stdout,
        stderr,
        outputs,
        container: container_meta,
        cache_proof,
        cache_identity,
        branch_decision,
        trigger_evaluation,
        skip_reason,
        failure,
        transition_cause,
        lifecycle_state,
        lifecycle_transitions,
        replay_provenance,
    };
    let data = serde_json::to_vec_pretty(&trace)?;
    ctx.store.write_trace(node_id, &data)?;
    Ok(())
}

const NODE_LOG_TAIL_LINE_LIMIT: usize = 20;
const NODE_LOG_TAIL_READ_BYTES: u64 = 16 * 1024;

fn terminal_exit_code(
    node: &Node,
    status: &NodeStatus,
    failure: Option<&FailureInfo>,
    container_meta: Option<&ContainerTrace>,
) -> Option<i32> {
    if let Some(exit_code) = failure.and_then(failure_exit_code) {
        return Some(exit_code);
    }
    if let Some(exit_code) = container_meta.and_then(|trace| trace.exit_code) {
        return Some(exit_code);
    }
    if supports_terminal_exit_code(node) && matches!(status, NodeStatus::Success) {
        return Some(0);
    }
    None
}

fn supports_terminal_exit_code(node: &Node) -> bool {
    matches!(
        node.kind,
        NodeKind::Shell | NodeKind::Python | NodeKind::Container | NodeKind::External(_)
    )
}

fn failure_exit_code(failure: &FailureInfo) -> Option<i32> {
    failure
        .details
        .as_ref()
        .and_then(|details| details.get("exit_code"))
        .and_then(Value::as_i64)
        .and_then(|code| i32::try_from(code).ok())
}

fn collect_node_log_evidence(
    fs: &dyn Fs,
    run_dir: &RunDir,
    path: &Path,
) -> Option<NodeLogEvidence> {
    let size_bytes = fs.metadata(path).ok()?.len();
    let tail_lines =
        read_log_tail_lines(path, NODE_LOG_TAIL_LINE_LIMIT, NODE_LOG_TAIL_READ_BYTES).ok()?;
    Some(NodeLogEvidence { path: run_relative_path(run_dir, path), size_bytes, tail_lines })
}

fn run_relative_path(run_dir: &RunDir, path: &Path) -> String {
    path.strip_prefix(run_dir.staging_path())
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn read_log_tail_lines(
    path: &Path,
    max_lines: usize,
    max_bytes: u64,
) -> std_io::Result<Vec<String>> {
    let mut file = std::fs::File::open(path)?;
    let file_len = file.metadata()?.len();
    let start = file_len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    let mut lines = content.lines().map(ToString::to_string).collect::<Vec<_>>();
    if start > 0 && !content.starts_with('\n') && !lines.is_empty() {
        lines.remove(0);
    }
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    Ok(lines)
}

fn status_string(status: &NodeStatus) -> String {
    match status {
        NodeStatus::Success => "success".to_string(),
        NodeStatus::Failed => "failed".to_string(),
        NodeStatus::Skipped => "skipped".to_string(),
        NodeStatus::Cached => "cached".to_string(),
        NodeStatus::Cancelled => "cancelled".to_string(),
    }
}

pub(crate) fn node_state_string(state: &NodeState) -> String {
    match state {
        NodeState::Pending => "pending",
        NodeState::Eligible => "eligible",
        NodeState::Queued => "queued",
        NodeState::Running => "running",
        NodeState::Success => "success",
        NodeState::Failed => "failed",
        NodeState::Skipped => "skipped",
        NodeState::Cached => "cached",
        NodeState::Cancelled => "cancelled",
        NodeState::TimedOut => "timed_out",
    }
    .to_string()
}

pub(crate) fn trace_lifecycle_state_string(state: &NodeState) -> String {
    match state {
        NodeState::Pending => "pending",
        NodeState::Eligible => "ready",
        NodeState::Queued => "queued",
        NodeState::Running => "running",
        NodeState::Success => "succeeded",
        NodeState::Failed => "failed",
        NodeState::Skipped => "skipped",
        NodeState::Cached => "cached",
        NodeState::Cancelled => "cancelled",
        NodeState::TimedOut => "timed_out",
    }
    .to_string()
}

pub(crate) fn transition_cause_string(cause: &TransitionCause) -> String {
    match cause {
        TransitionCause::Submission => "submission",
        TransitionCause::PlanningCompleted => "planning_completed",
        TransitionCause::SchedulerEligible => "scheduler_eligible",
        TransitionCause::SchedulerQueued => "scheduler_queued",
        TransitionCause::ExecutionStarted => "execution_started",
        TransitionCause::ExecutionSucceeded => "execution_succeeded",
        TransitionCause::ExecutionFailed => "execution_failed",
        TransitionCause::CachedReuse => "cached_reuse",
        TransitionCause::PolicyDenied => "policy_denied",
        TransitionCause::DependencyFailed => "dependency_failed",
        TransitionCause::SelectionFiltered => "selection_filtered",
        TransitionCause::ExecutionAborted => "execution_aborted",
        TransitionCause::CancelRequested => "cancel_requested",
        TransitionCause::TimeoutExceeded => "timeout_exceeded",
        TransitionCause::ReplayReused => "replay_reused",
        TransitionCause::ReplayReexecuted => "replay_reexecuted",
        TransitionCause::ResumeRequested => "resume_requested",
    }
    .to_string()
}

pub(crate) fn transition_cause_for_status(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Success => "ExecutionSucceeded",
        NodeStatus::Failed => "ExecutionFailed",
        NodeStatus::Skipped => "SelectionFiltered",
        NodeStatus::Cached => "CachedReuse",
        NodeStatus::Cancelled => "CancelRequested",
    }
}

pub(crate) fn transition_cause_for_failure(failure: Option<&FailureInfo>) -> &'static str {
    match failure {
        Some(failure) if failure.kind == "Policy" => "PolicyDenied",
        Some(failure) if failure.code == "UPSTREAM_FAILED" => "DependencyFailed",
        Some(failure) if failure.code == "RUN_ABORTED" => "ExecutionAborted",
        Some(failure) if failure.code == "EXEC_CANCELLED" => "CancelRequested",
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
        "upstream_failed" | "isolated_branch_failure" => "DependencyFailed",
        "cancelled" => "CancelRequested",
        _ => "SelectionFiltered",
    }
}

pub(crate) fn failure_propagation_cause(failure: Option<&FailureInfo>) -> &'static str {
    match transition_cause_for_failure(failure) {
        "PolicyDenied" => "policy_denied",
        "DependencyFailed" => "upstream_failed",
        "ExecutionAborted" => "execution_aborted",
        "CancelRequested" => "cancel_requested",
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

fn attempt_log_relative_path(attempt: u32, file_name: &str) -> String {
    format!("attempts/{attempt}/{file_name}")
}

fn persist_attempt_logs(
    ctx: &RunContext,
    node_id: &str,
    attempt: u32,
    stdout_path: &str,
    stderr_path: &str,
) -> Result<(String, String), RuntimeError> {
    let attempt_dir = ctx.run_dir.node_attempt_dir(node_id, attempt);
    let attempt_stdout_path = ctx.run_dir.node_attempt_stdout_path(node_id, attempt);
    let attempt_stderr_path = ctx.run_dir.node_attempt_stderr_path(node_id, attempt);
    ctx.fs.create_dir_all(&attempt_dir)?;
    match ctx.fs.copy(Path::new(stdout_path), &attempt_stdout_path) {
        Ok(_) => {}
        Err(error) if error.kind() == std_io::ErrorKind::NotFound => {
            ctx.fs.write(&attempt_stdout_path, b"")?;
        }
        Err(error) => return Err(RuntimeError::Io(error)),
    }
    match ctx.fs.copy(Path::new(stderr_path), &attempt_stderr_path) {
        Ok(_) => {}
        Err(error) if error.kind() == std_io::ErrorKind::NotFound => {
            ctx.fs.write(&attempt_stderr_path, b"")?;
        }
        Err(error) => return Err(RuntimeError::Io(error)),
    }
    Ok((
        attempt_log_relative_path(attempt, "stdout.log"),
        attempt_log_relative_path(attempt, "stderr.log"),
    ))
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

fn map_execution_summary_path(ctx: &RunContext, node_id: &str) -> PathBuf {
    ctx.run_dir.node_dir(node_id).join("map.execution.json")
}

fn map_execution_version() -> String {
    "map-execution/v0.1".to_string()
}

fn reduce_collection_manifest_name() -> &'static str {
    "reduce.collection.json"
}

fn reduce_execution_summary_path(ctx: &RunContext, node_id: &str) -> PathBuf {
    ctx.run_dir.node_dir(node_id).join("reduce.execution.json")
}

fn reduce_execution_version() -> String {
    "reduce-execution/v0.1".to_string()
}

fn reduce_mode_label(mode: ReduceExecutionMode) -> &'static str {
    match mode {
        ReduceExecutionMode::AllSuccess => "all_success",
        ReduceExecutionMode::Partial => "partial",
    }
}

fn reduce_empty_policy_label(policy: ReduceEmptyPolicy) -> &'static str {
    match policy {
        ReduceEmptyPolicy::Forbid => "forbid",
        ReduceEmptyPolicy::Allow => "allow",
        ReduceEmptyPolicy::Skip => "skip",
    }
}

pub(crate) fn reduce_execution_config(node: &Node) -> Result<ReduceExecutionConfig, FailureInfo> {
    let reduce = match &node.params {
        bijux_dag_core::ParamValue::Object(params) => match params.get("reduce") {
            Some(bijux_dag_core::ParamValue::Object(reduce)) => Some(reduce),
            Some(_) => {
                return Err(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "REDUCE_CONFIG_INVALID",
                    format!("reduce params on node {} must be an object", node.id),
                    Some(serde_json::json!({ "node_id": node.id })),
                ));
            }
            None => None,
        },
        _ => None,
    };

    let mode = match reduce.and_then(|value| value.get("mode")) {
        None => ReduceExecutionMode::AllSuccess,
        Some(bijux_dag_core::ParamValue::Literal(Value::String(value)))
            if value == "all_success" =>
        {
            ReduceExecutionMode::AllSuccess
        }
        Some(bijux_dag_core::ParamValue::Literal(Value::String(value))) if value == "partial" => {
            ReduceExecutionMode::Partial
        }
        Some(_) => {
            return Err(FailureInfo::new(
                FailureClass::User,
                "User",
                "REDUCE_MODE_INVALID",
                format!("reduce.mode on node {} must be 'all_success' or 'partial'", node.id),
                Some(serde_json::json!({ "node_id": node.id })),
            ));
        }
    };

    let empty_policy = match reduce.and_then(|value| value.get("empty")) {
        None => ReduceEmptyPolicy::Forbid,
        Some(bijux_dag_core::ParamValue::Literal(Value::String(value))) if value == "forbid" => {
            ReduceEmptyPolicy::Forbid
        }
        Some(bijux_dag_core::ParamValue::Literal(Value::String(value))) if value == "allow" => {
            ReduceEmptyPolicy::Allow
        }
        Some(bijux_dag_core::ParamValue::Literal(Value::String(value))) if value == "skip" => {
            ReduceEmptyPolicy::Skip
        }
        Some(_) => {
            return Err(FailureInfo::new(
                FailureClass::User,
                "User",
                "REDUCE_EMPTY_POLICY_INVALID",
                format!("reduce.empty on node {} must be 'forbid', 'allow', or 'skip'", node.id),
                Some(serde_json::json!({ "node_id": node.id })),
            ));
        }
    };

    Ok(ReduceExecutionConfig { mode, empty_policy })
}

fn map_input_port(node: &Node, params: &Value) -> Result<String, FailureInfo> {
    if let Some(input) =
        params.get("map").and_then(|value| value.get("input")).and_then(Value::as_str)
    {
        if node.inputs.iter().any(|candidate| candidate == input) {
            return Ok(input.to_string());
        }
        return Err(FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_INVALID",
            format!("map input '{}' is not declared on node {}", input, node.id),
            Some(serde_json::json!({
                "node_id": node.id,
                "input": input,
                "declared_inputs": node.inputs,
            })),
        ));
    }

    match node.inputs.as_slice() {
        [input] => Ok(input.clone()),
        [] => Err(FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_MISSING",
            format!("map node {} requires at least one declared input", node.id),
            Some(serde_json::json!({ "node_id": node.id })),
        )),
        _ => Err(FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_AMBIGUOUS",
            format!(
                "map node {} requires params.map.input when more than one input is declared",
                node.id
            ),
            Some(serde_json::json!({
                "node_id": node.id,
                "declared_inputs": node.inputs,
            })),
        )),
    }
}

fn map_input_binding(
    graph: &Graph,
    node_id: &str,
    input_port: &str,
) -> Result<(String, String), FailureInfo> {
    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.to.node_id == node_id && edge.to.port == input_port)
        .ok_or_else(|| {
            FailureInfo::new(
                FailureClass::User,
                "User",
                "MAP_INPUT_UNBOUND",
                format!("map input {}.{} is not bound to an upstream output", node_id, input_port),
                Some(serde_json::json!({
                    "node_id": node_id,
                    "input_port": input_port,
                })),
            )
        })?;
    Ok((edge.from.node_id.clone(), edge.from.port.clone()))
}

fn load_map_items(
    ctx: &RunContext,
    graph: &Graph,
    node: &Node,
    input_port: &str,
) -> Result<Vec<Value>, FailureInfo> {
    let (source_node_id, _) = map_input_binding(graph, &node.id, input_port)?;
    let item_path = ctx.run_dir.node_inputs_dir(&node.id).join(&source_node_id).join(input_port);
    let raw = ctx.fs.read_to_string(&item_path).map_err(|error| {
        FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_UNREADABLE",
            format!("map input could not be read from {}: {}", item_path.display(), error),
            Some(serde_json::json!({
                "node_id": node.id,
                "input_port": input_port,
                "source_node_id": source_node_id,
            })),
        )
    })?;
    let payload = serde_json::from_str::<Value>(&raw).map_err(|error| {
        FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_INVALID",
            format!("map input for {} must be valid json array: {}", node.id, error),
            Some(serde_json::json!({
                "node_id": node.id,
                "input_port": input_port,
                "source_node_id": source_node_id,
            })),
        )
    })?;
    payload.as_array().cloned().ok_or_else(|| {
        FailureInfo::new(
            FailureClass::User,
            "User",
            "MAP_INPUT_INVALID",
            format!("map input for {} must be a json array", node.id),
            Some(serde_json::json!({
                "node_id": node.id,
                "input_port": input_port,
                "source_node_id": source_node_id,
            })),
        )
    })
}

fn read_inputs_index(ctx: &RunContext, node_id: &str) -> Result<InputsIndex, RuntimeError> {
    let raw = ctx.fs.read_to_string(&ctx.run_dir.node_inputs_index_path(node_id))?;
    serde_json::from_str(&raw).map_err(RuntimeError::from)
}

fn map_item_identity(index: usize, item: &Value) -> Result<(String, String), RuntimeError> {
    let item_bytes = serde_json::to_vec(item)?;
    let item_sha256 = sha256_bytes(&item_bytes);
    Ok((format!("position-{index:06}-{}", &item_sha256[..8]), item_sha256))
}

fn write_item_inputs_index(
    ctx: &RunContext,
    graph: &Graph,
    node: &Node,
    input_port: &str,
    item_run_dir: &RunDir,
    item_value: &Value,
) -> Result<InputsIndex, RuntimeError> {
    let parent_inputs_dir = ctx.run_dir.node_inputs_dir(&node.id);
    let item_inputs_dir = item_run_dir.node_inputs_dir(&node.id);
    copy_dir_all(ctx.fs.as_ref(), &parent_inputs_dir, &item_inputs_dir)?;

    let parent_index = read_inputs_index(ctx, &node.id)?;
    let (source_node_id, source_output_name) = map_input_binding(graph, &node.id, input_port)
        .map_err(|failure| RuntimeError::Executor(failure.message))?;
    let item_input_path = item_inputs_dir.join(&source_node_id).join(input_port);
    if let Some(parent) = item_input_path.parent() {
        ctx.fs.create_dir_all(parent)?;
    }
    let item_bytes = serde_json::to_vec_pretty(item_value)?;
    ctx.fs.write(&item_input_path, &item_bytes)?;
    let item_sha256 = sha256_bytes(&item_bytes);
    let source_node_fingerprint = node_fingerprint_from_ctx(ctx, &source_node_id);
    let local_path = format!("{source_node_id}/{input_port}");

    let mut updated = false;
    let mut files = parent_index
        .files
        .into_iter()
        .map(|mut file| {
            if file.local_path == local_path {
                file.source_sha256.clone_from(&item_sha256);
                file.source_node_id.clone_from(&source_node_id);
                file.source_node_fingerprint.clone_from(&source_node_fingerprint);
                file.source_output_name.clone_from(&source_output_name);
                file.materialization_mode = "copy".to_string();
                updated = true;
            }
            file
        })
        .collect::<Vec<_>>();
    if !updated {
        files.push(InputFile {
            local_path,
            source_sha256: item_sha256,
            source_node_id,
            source_node_fingerprint,
            source_output_name,
            materialization_mode: "copy".to_string(),
        });
    }
    files.sort_by(|left, right| left.local_path.cmp(&right.local_path));
    let index = InputsIndex { collections: Vec::new(), files };
    write_inputs_index(&item_inputs_dir, &index)?;
    Ok(index)
}

fn write_map_summary(
    ctx: &RunContext,
    node_id: &str,
    summary: &MapExecutionSummary,
) -> Result<(), RuntimeError> {
    let bytes = serde_json::to_vec_pretty(summary)?;
    ctx.fs.write(&map_execution_summary_path(ctx, node_id), &bytes)?;
    Ok(())
}

fn aggregate_map_item_outputs(
    fs: &dyn Fs,
    node: &Node,
    parent_outputs_dir: &Path,
    item_node_outputs_dir: &Path,
    item_id: &str,
) -> Result<Vec<MapExecutionOutputSummary>, RuntimeError> {
    let mut outputs = Vec::new();
    for output in &node.outputs {
        if output.expects_file() {
            return Err(RuntimeError::Executor(format!(
                "map node {} output {} must be a directory output",
                node.id, output.name
            )));
        }
        let item_output_path = authorized_declared_output_path(item_node_outputs_dir, output)
            .map_err(|failure| RuntimeError::Executor(failure.message))?;
        let aggregate_root = authorized_declared_output_path(parent_outputs_dir, output)
            .map_err(|failure| RuntimeError::Executor(failure.message))?;
        let aggregate_item_path = aggregate_root.join("items").join(item_id);
        fs.create_dir_all(&aggregate_root)?;
        copy_dir_all(fs, &item_output_path, &aggregate_item_path)?;
        outputs.push(MapExecutionOutputSummary {
            output_name: output.name.clone(),
            item_path: format!("{}/items/{}", output.path, item_id),
        });
    }
    Ok(outputs)
}

fn execute_map_node(
    adapter: &dyn Adapter,
    graph: &Graph,
    node: &Node,
    params: &Value,
    ctx: &RunContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    prepare_node_execution_dirs(ctx, &node.id)?;
    let started = ctx.clock.now_unix_ms();
    let stdout_path = ctx.run_dir.node_stdout_path(&node.id);
    let stderr_path = ctx.run_dir.node_stderr_path(&node.id);
    let parent_outputs_dir = ctx.run_dir.node_outputs_dir(&node.id);

    let input_port = match map_input_port(node, params) {
        Ok(input_port) => input_port,
        Err(failure) => {
            let message = failure.message.clone();
            let mut result = node_failure_result(
                ctx.fs.as_ref(),
                &stdout_path,
                &stderr_path,
                &parent_outputs_dir,
                NodeStatus::Failed,
                failure,
                message.as_bytes(),
            )?;
            let finished = ctx.clock.now_unix_ms();
            let (attempt_stdout_path, attempt_stderr_path) =
                persist_attempt_logs(ctx, &node.id, 1, &result.stdout_path, &result.stderr_path)?;
            result.attempts = 1;
            result.attempt_events = vec![AttemptEvent {
                attempt: 1,
                started_unix_ms: started,
                finished_unix_ms: finished,
                status: result.status.clone(),
                stdout_path: Some(attempt_stdout_path),
                stderr_path: Some(attempt_stderr_path),
                failure: result.failure.clone(),
                scheduled_backoff_ms: None,
                retry_decision: None,
            }];
            return Ok(result);
        }
    };

    let items = load_map_items(ctx, graph, node, &input_port)
        .map_err(|failure| RuntimeError::Executor(failure.message))?;
    for output in &node.outputs {
        let aggregate_root = authorized_declared_output_path(&parent_outputs_dir, output)
            .map_err(|failure| RuntimeError::Executor(failure.message))?;
        ctx.fs.create_dir_all(&aggregate_root)?;
    }

    let map_runs_dir = ctx.run_dir.node_dir(&node.id).join("mapped_items");
    ctx.fs.create_dir_all(&map_runs_dir)?;
    let resolved = graph.resolve_graph()?;
    let base_node_definition_fp = node_definition_fingerprint_from_ctx(ctx, &node.id);
    let base_declared_env_fp = declared_environment_fingerprint_from_ctx(ctx, &node.id);
    let base_fp =
        sha256_bytes(format!("{base_node_definition_fp}:{base_declared_env_fp}").as_bytes());

    let mut summaries = Vec::new();
    let mut successful_item_count = 0usize;
    let mut failed_item_count = 0usize;
    let mut cancelled_item_count = 0usize;

    for (index, item) in items.into_iter().enumerate() {
        let (item_id, item_sha256) = map_item_identity(index, &item)?;
        let item_layout =
            RunDirLayout::preview(&map_runs_dir, Some(&item_id)).map_err(|error| {
                RuntimeError::Executor(format!("invalid map item identity {}: {}", item_id, error))
            })?;
        let item_run_dir = RunDir::create_with_id(&map_runs_dir, &item_id)?;
        let item_inputs =
            write_item_inputs_index(ctx, graph, node, &input_port, &item_run_dir, &item)?;
        let params_template =
            resolved.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        let item_bindings =
            NodePathBindings::for_host(&item_layout, &node.id, ctx.effective_cache_dir.as_deref());
        let item_params = bind_path_variables_in_value(&params_template, &item_bindings)
            .map_err(RuntimeError::Executor)?;
        let item_params_fingerprint = params_fingerprint(&item_params)?;
        let item_command_fingerprint = command_fingerprint(graph, node, &item_params)?;
        let item_fp = node_fingerprint_with_inputs(&base_fp, &item_inputs)?;
        let item_run_dir_arc = Arc::new(item_run_dir.clone());
        let item_ctx = RunContext {
            run_dir: Arc::clone(&item_run_dir_arc),
            replay_source_run_dir: ctx.replay_source_run_dir.clone(),
            graph_fingerprint: Arc::new(Mutex::new(HashMap::from([(node.id.clone(), item_fp)]))),
            node_definition_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                base_node_definition_fp.clone(),
            )])),
            declared_environment_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                base_declared_env_fp.clone(),
            )])),
            params_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                item_params_fingerprint,
            )])),
            command_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                item_command_fingerprint,
            )])),
            planner_contract_version: ctx.planner_contract_version.clone(),
            execution_fingerprint: ctx.execution_fingerprint.clone(),
            evidence_fingerprint: ctx.evidence_fingerprint.clone(),
            execution_contract_fingerprint: ctx.execution_contract_fingerprint.clone(),
            resolved_params: HashMap::from([(node.id.clone(), item_params.clone())]),
            effective_cache_dir: ctx.effective_cache_dir.clone(),
            fs: Arc::clone(&ctx.fs),
            clock: Arc::clone(&ctx.clock),
            store: RuntimeArtifactStore::new(item_run_dir_arc, Arc::clone(&ctx.fs)),
            policy: ctx.policy.clone(),
            absolute_path_policy: ctx.absolute_path_policy,
            cancellation_requested: Arc::clone(&ctx.cancellation_requested),
        };
        let mut item_node = node.clone();
        item_node.semantic_kind = bijux_dag_core::SemanticNodeKind::Task;
        let item_result =
            execute_with_retries(adapter, graph, &item_node, &item_params, &item_ctx, retry)?;
        let item_final_dir = item_run_dir.finalize()?;
        let item_outputs_dir = item_final_dir.join("nodes").join(&node.id).join("outputs");
        let outputs = if item_result.status == NodeStatus::Success {
            successful_item_count += 1;
            aggregate_map_item_outputs(
                ctx.fs.as_ref(),
                node,
                &parent_outputs_dir,
                &item_outputs_dir,
                &item_id,
            )?
        } else {
            if item_result.status == NodeStatus::Cancelled {
                cancelled_item_count += 1;
            } else {
                failed_item_count += 1;
            }
            Vec::new()
        };
        summaries.push(MapExecutionItemSummary {
            item_id,
            item_sha256,
            status: status_string(&item_result.status),
            run_dir: item_final_dir
                .strip_prefix(ctx.run_dir.node_dir(&node.id))
                .unwrap_or(item_final_dir.as_path())
                .to_string_lossy()
                .replace('\\', "/"),
            outputs,
            failure: item_result.failure.clone(),
        });
    }

    summaries.sort_by(|left, right| left.item_id.cmp(&right.item_id));
    let status = if failed_item_count > 0 {
        NodeStatus::Failed
    } else if cancelled_item_count > 0 {
        NodeStatus::Cancelled
    } else {
        NodeStatus::Success
    };
    let failure = match status {
        NodeStatus::Failed => Some(FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "MAP_ITEMS_FAILED",
            format!(
                "map node {} failed for {} of {} items",
                node.id,
                failed_item_count,
                summaries.len()
            ),
            Some(serde_json::json!({
                "failed_items": summaries
                    .iter()
                    .filter(|item| item.status == "failed")
                    .map(|item| serde_json::json!({
                        "item_id": item.item_id,
                        "failure": item.failure,
                    }))
                    .collect::<Vec<_>>(),
            })),
        )),
        NodeStatus::Cancelled => Some(FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "MAP_ITEMS_CANCELLED",
            format!("map node {} cancelled while processing {} items", node.id, summaries.len()),
            Some(serde_json::json!({
                "cancelled_items": cancelled_item_count,
            })),
        )),
        _ => None,
    };
    let summary = MapExecutionSummary {
        schema_version: map_execution_version(),
        map_node_id: node.id.clone(),
        input_port,
        item_count: summaries.len(),
        successful_item_count,
        failed_item_count,
        cancelled_item_count,
        items: summaries,
    };
    write_map_summary(ctx, &node.id, &summary)?;

    let finished = ctx.clock.now_unix_ms();
    let output_report = inspect_declared_outputs(&parent_outputs_dir, &node.outputs);
    if let Some(output_failure) = output_report.failure {
        let message = output_failure.message.clone();
        let mut result = node_failure_result(
            ctx.fs.as_ref(),
            &stdout_path,
            &stderr_path,
            &parent_outputs_dir,
            NodeStatus::Failed,
            output_failure,
            message.as_bytes(),
        )?;
        let (attempt_stdout_path, attempt_stderr_path) =
            persist_attempt_logs(ctx, &node.id, 1, &result.stdout_path, &result.stderr_path)?;
        result.attempts = 1;
        result.attempt_events = vec![AttemptEvent {
            attempt: 1,
            started_unix_ms: started,
            finished_unix_ms: finished,
            status: result.status.clone(),
            stdout_path: Some(attempt_stdout_path),
            stderr_path: Some(attempt_stderr_path),
            failure: result.failure.clone(),
            scheduled_backoff_ms: None,
            retry_decision: None,
        }];
        return Ok(result);
    }

    let stdout = format!(
        "mapped {} items for {} (success={}, failed={}, cancelled={})\n",
        summary.item_count,
        summary.map_node_id,
        summary.successful_item_count,
        summary.failed_item_count,
        summary.cancelled_item_count,
    );
    let stderr = if matches!(status, NodeStatus::Failed | NodeStatus::Cancelled) {
        summary
            .items
            .iter()
            .filter_map(|item| {
                item.failure.as_ref().map(|failure| {
                    format!("{}: {} ({})", item.item_id, failure.message, failure.code)
                })
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };
    ctx.fs.write(&stdout_path, stdout.as_bytes())?;
    ctx.fs.write(&stderr_path, stderr.as_bytes())?;
    let fp = node_fingerprint_from_ctx(ctx, &node.id);
    write_outputs_index(&parent_outputs_dir, &node.id, &fp, &output_report.present_outputs)?;
    let (attempt_stdout_path, attempt_stderr_path) = persist_attempt_logs(
        ctx,
        &node.id,
        1,
        &stdout_path.display().to_string(),
        &stderr_path.display().to_string(),
    )?;
    Ok(NodeResult {
        status: status.clone(),
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: parent_outputs_dir.display().to_string(),
        output_evidence: output_report.output_evidence,
        failure: failure.clone(),
        attempts: 1,
        attempt_events: vec![AttemptEvent {
            attempt: 1,
            started_unix_ms: started,
            finished_unix_ms: finished,
            status: status.clone(),
            stdout_path: Some(attempt_stdout_path),
            stderr_path: Some(attempt_stderr_path),
            failure,
            scheduled_backoff_ms: None,
            retry_decision: None,
        }],
        container_meta: None,
        adapter_binary_sha256: adapter.binary_hash(),
    })
}

fn execute_with_retries(
    adapter: &dyn Adapter,
    graph: &Graph,
    node: &Node,
    params: &Value,
    ctx: &RunContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    if node.semantic_kind == bijux_dag_core::SemanticNodeKind::Map {
        return execute_map_node(adapter, graph, node, params, ctx, retry);
    }
    execute_with_retry_operation(ctx, node, retry, |_attempt| {
        let node_ctx = NodeCtx { graph, node, exec: ctx, params };
        Ok(match adapter.execute(&node_ctx) {
            Ok(result) => result,
            Err(error) => failed_node_result_from_runtime_error(ctx, node, error),
        })
    })
}

pub(crate) fn execute_with_retry_operation<F>(
    ctx: &RunContext,
    node: &Node,
    _retry: &RetryPolicy,
    mut operation: F,
) -> Result<NodeResult, RuntimeError>
where
    F: FnMut(u32) -> Result<NodeResult, RuntimeError>,
{
    let mut attempt = 0u32;
    let retry_policy = build_retry_policy(node);
    let mut attempt_events = Vec::new();
    loop {
        attempt += 1;
        prepare_node_execution_dirs(ctx, &node.id)?;
        let started = ctx.clock.now_unix_ms();
        let mut result = operation(attempt)
            .unwrap_or_else(|error| failed_node_result_from_runtime_error(ctx, node, error));
        let finished = ctx.clock.now_unix_ms();
        let retry_decision = result.failure.as_ref().and_then(|failure| {
            (result.status == NodeStatus::Failed).then(|| {
                evaluate_retry_decision(
                    &node.id,
                    &retry_policy,
                    attempt,
                    &retry_observation_from_failure(failure),
                )
            })
        });
        let retry_allowed = retry_decision.as_ref().is_some_and(|decision| decision.retry_allowed);
        let scheduled_backoff_ms = retry_decision
            .as_ref()
            .filter(|decision| decision.retry_allowed)
            .and_then(|_| {
                result.failure.as_ref().map(|failure| {
                    contract_retry_wait_ms(
                        &node.id,
                        &retry_policy,
                        attempt,
                        failure.operator_class().as_str(),
                    )
                })
            })
            .filter(|wait| *wait > 0);
        let (attempt_stdout_path, attempt_stderr_path) =
            persist_attempt_logs(ctx, &node.id, attempt, &result.stdout_path, &result.stderr_path)?;
        attempt_events.push(AttemptEvent {
            attempt,
            started_unix_ms: started,
            finished_unix_ms: finished,
            status: result.status.clone(),
            stdout_path: Some(attempt_stdout_path),
            stderr_path: Some(attempt_stderr_path),
            failure: result.failure.clone(),
            scheduled_backoff_ms,
            retry_decision: retry_decision.clone(),
        });
        result.attempts = attempt;
        if result.status != NodeStatus::Failed {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        if !retry_allowed {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        let wait = scheduled_backoff_ms.unwrap_or(0);
        if wait > 0 {
            std::thread::sleep(Duration::from_millis(wait));
        }
    }
}

pub(crate) fn failed_node_result_from_runtime_error(
    ctx: &RunContext,
    node: &Node,
    error: RuntimeError,
) -> NodeResult {
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let outputs_dir = ctx.run_dir.node_outputs_dir(&node.id);
    let stdout_path = ctx.run_dir.node_stdout_path(&node.id);
    let stderr_path = ctx.run_dir.node_stderr_path(&node.id);
    let (class, kind, code, message) = match error {
        RuntimeError::Graph(err) => (FailureClass::User, "User", "GRAPH_ERROR", err.to_string()),
        RuntimeError::Artifact(err) => {
            (FailureClass::Infrastructure, "Infrastructure", "ARTIFACT_ERROR", err.to_string())
        }
        RuntimeError::Io(err) if err.kind() == std_io::ErrorKind::NotFound => {
            (FailureClass::Infrastructure, "Infrastructure", "MISSING_EXECUTABLE", err.to_string())
        }
        RuntimeError::Io(err) => {
            (FailureClass::Infrastructure, "Infrastructure", "IO_ERROR", err.to_string())
        }
        RuntimeError::Json(err) => {
            (FailureClass::Infrastructure, "Infrastructure", "JSON_ERROR", err.to_string())
        }
        RuntimeError::Executor(message) => {
            if message.contains("timed out") {
                (FailureClass::Timeout, "Timeout", "EXEC_TIMEOUT", message)
            } else if message.contains("cancelled") {
                (FailureClass::Execution, "Execution", "EXEC_CANCELLED", message)
            } else if matches!(
                message.as_str(),
                "missing argv" | "empty argv" | "argv must be strings" | "missing container spec"
            ) {
                (FailureClass::User, "User", "EXEC_ERROR", message)
            } else {
                (FailureClass::Execution, "Execution", "EXEC_ERROR", message)
            }
        }
    };
    let _ = ctx.fs.create_dir_all(&node_dir);
    let _ = ctx.fs.create_dir_all(&outputs_dir);
    let _ = ctx.fs.write(&stdout_path, b"");
    let _ = ctx.fs.write(&stderr_path, message.as_bytes());
    NodeResult {
        status: if code == "EXEC_CANCELLED" { NodeStatus::Cancelled } else { NodeStatus::Failed },
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure: Some(FailureInfo::new(class, kind, code, message, None)),
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
        "container_image_reference_policy": container_image_reference_policy_label(
            policy.container_image_reference_policy
        ),
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

fn container_image_reference_policy_label(policy: ContainerImageReferencePolicy) -> &'static str {
    match policy {
        ContainerImageReferencePolicy::RequireDigest => "require_digest",
        ContainerImageReferencePolicy::AllowUnpinned => "allow_unpinned",
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

fn execution_contract_fingerprint(options: &RuntimeConfig) -> String {
    let payload = serde_json::json!({
        "node_timeout_ms": options.node_timeout_ms,
        "materialize_inputs": materialize_mode_label(options.materialize_inputs),
    });
    sha256_bytes(payload.to_string().as_bytes())
}

fn params_fingerprint(params: &Value) -> Result<String, RuntimeError> {
    let mut normalized = params.clone();
    sort_value_maps(&mut normalized);
    Ok(sha256_bytes(&serde_json::to_vec(&normalized)?))
}

fn command_fingerprint(
    graph: &Graph,
    node: &Node,
    params: &Value,
) -> Result<Option<String>, RuntimeError> {
    let command_surface = if matches!(node.kind, NodeKind::Shell) {
        Some(serde_json::json!({
            "kind": "shell",
            "argv": params.get("argv").cloned().unwrap_or(Value::Null),
        }))
    } else if matches!(node.kind, NodeKind::FileTransform) {
        Some(serde_json::json!({
            "kind": "file_transform",
            "params": params,
        }))
    } else if matches!(node.kind, NodeKind::Python) {
        Some(serde_json::json!({
            "kind": "python",
            "params": params,
        }))
    } else if matches!(node.kind, NodeKind::Http) {
        Some(serde_json::json!({
            "kind": "http",
            "method": params.get("method").cloned().unwrap_or(Value::Null),
            "url": params.get("url").cloned().unwrap_or(Value::Null),
            "headers": params.get("headers").cloned().unwrap_or(Value::Null),
            "body": params.get("body").cloned().unwrap_or(Value::Null),
        }))
    } else if let Some(container) = node.container.as_ref() {
        let argv = bijux_dag_core::resolve::resolve_command_argv_templates(
            graph,
            node,
            &container.argv,
            params,
        )
        .map_err(|error| RuntimeError::Executor(error.to_string()))?;
        Some(serde_json::json!({
            "kind": "container",
            "engine": container.engine,
            "image": container.image,
            "workdir": container.workdir,
            "argv": argv,
        }))
    } else {
        None
    };

    command_surface
        .map(|surface| serde_json::to_vec(&surface).map(|bytes| sha256_bytes(&bytes)))
        .transpose()
        .map_err(RuntimeError::from)
}

fn cache_key_input_for_run(
    options: &RuntimeConfig,
    node: &Node,
    execution_fingerprint: &str,
    ctx: &RunContext,
    adapter_id: &str,
    adapter_version: &str,
    adapter_binary_sha256: Option<&str>,
    adapter_outputs_schema_version: &str,
) -> Result<CacheKeyInput, RuntimeError> {
    Ok(CacheKeyInput {
        execution_fingerprint: execution_fingerprint.to_string(),
        node_definition_fingerprint: node_definition_fingerprint_from_ctx(ctx, &node.id),
        declared_environment_fingerprint: declared_environment_fingerprint_from_ctx(ctx, &node.id),
        input_lineage_fingerprint: input_lineage_fingerprint_from_run(ctx, &node.id)?,
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        adapter_binary_sha256: adapter_binary_sha256.map(ToString::to_string),
        output_schema_version: adapter_outputs_schema_version.to_string(),
        policy_fingerprint: policy_fingerprint(&options.policy),
        execution_contract_fingerprint: execution_contract_fingerprint(options),
        backend_class: "local".to_string(),
    })
}

/// Admission record for one graph node against the currently registered adapters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AdapterAdmissionEntry {
    pub node_id: String,
    pub node_kind: String,
    pub supported: bool,
    pub adapter_id: Option<String>,
    pub adapter_version: Option<String>,
    pub reasons: Vec<String>,
}

/// Summary of whether every node in a graph can be admitted by the runtime.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AdapterAdmissionReport {
    pub supported: bool,
    pub entries: Vec<AdapterAdmissionEntry>,
}

/// Lists the currently registered adapter records from the default runtime registry.
pub fn registered_adapters() -> Vec<AdapterInfo> {
    let registry = build_registry(vec![
        Arc::new(ConstAdapter),
        Arc::new(FileTransformAdapter),
        Arc::new(HttpRequestAdapter),
        Arc::new(ShellAdapter),
        Arc::new(PythonFunctionAdapter),
        Arc::new(ContainerAdapter),
    ])
    .unwrap_or_else(|_| AdapterRegistry::new());
    registry.list()
}

/// Lists adapter descriptors that define the public runtime adapter contract surface.
pub fn registered_adapter_descriptors() -> Vec<adapter::AdapterDescriptor> {
    let registry = build_registry(vec![
        Arc::new(ConstAdapter),
        Arc::new(FileTransformAdapter),
        Arc::new(HttpRequestAdapter),
        Arc::new(ShellAdapter),
        Arc::new(PythonFunctionAdapter),
        Arc::new(ContainerAdapter),
    ])
    .unwrap_or_else(|_| AdapterRegistry::new());
    registry.descriptors()
}

/// Builds conformance results for every registered adapter descriptor.
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

/// Builds the checked-in adapter reference document payload from live descriptors.
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

/// Evaluates whether each node in a graph is supported by the current adapter registry.
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

/// Serializes the default adapter registry into a JSON inventory report.
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
    node: &Node,
    mode: MaterializeMode,
    parent_statuses: &HashMap<String, NodeStatus>,
) -> Result<InputsIndex, RuntimeError> {
    let node_id = &node.id;
    let inputs_dir = ctx.run_dir.node_inputs_dir(node_id);
    recreate_dir(ctx.fs.as_ref(), &inputs_dir)?;
    let mut files = Vec::new();
    let mut materialized_inputs = BTreeMap::<(String, String, String), (String, String)>::new();
    for edge in &graph.edges {
        if edge.to.node_id != *node_id {
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
        let mut src_path = authorized_declared_output_path(
            ctx.run_dir.node_outputs_dir(&edge.from.node_id).as_path(),
            out,
        )
        .map_err(|failure| RuntimeError::Executor(failure.message))?;
        let mut from_fp = node_fingerprint_from_ctx(ctx, &edge.from.node_id);
        if ctx.fs.metadata(&src_path).is_err() {
            if let Some(source_run_dir) = ctx.replay_source_run_dir.as_deref() {
                let replay_outputs_dir =
                    source_run_dir.join("nodes").join(&edge.from.node_id).join("outputs");
                let replay_src_path = authorized_declared_output_path(&replay_outputs_dir, out)
                    .map_err(|failure| RuntimeError::Executor(failure.message))?;
                if ctx.fs.metadata(&replay_src_path).is_ok() {
                    src_path = replay_src_path;
                    if from_fp.is_empty() {
                        from_fp = replay_source_node_fingerprint(
                            ctx.fs.as_ref(),
                            source_run_dir,
                            &edge.from.node_id,
                        )
                        .unwrap_or_default();
                    }
                }
            }
        }
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
            materialized_inputs.insert(
                (edge.to.port.clone(), edge.from.node_id.clone(), edge.from.port.clone()),
                (rel_str.clone(), source_sha256.clone()),
            );
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
    let mut collections = Vec::new();
    if node.semantic_kind == SemanticNodeKind::Reduce {
        let summary = build_reduce_summary(graph, node, parent_statuses, &materialized_inputs)?;
        write_reduce_collection_manifest(ctx, node_id, &summary.collection)?;
        write_reduce_summary(ctx, node_id, &summary)?;
        collections.push(summary.collection);
    }
    let index = InputsIndex { collections, files };
    write_inputs_index(&inputs_dir, &index)?;
    Ok(index)
}

fn replay_source_node_fingerprint(
    fs: &dyn Fs,
    source_run_dir: &Path,
    node_id: &str,
) -> Option<String> {
    let trace_path = source_run_dir.join("nodes").join(node_id).join("trace.json");
    let bytes = fs.read(&trace_path).ok()?;
    let trace: Value = serde_json::from_slice(&bytes).ok()?;
    trace.get("fingerprint").and_then(Value::as_str).map(str::to_string)
}

fn cache_dir_from_env() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ReduceDependencyBinding {
    input_port: String,
    source_node_id: String,
    source_output_name: String,
}

fn reduce_dependency_bindings(graph: &Graph, node: &Node) -> Vec<ReduceDependencyBinding> {
    let input_positions = node
        .inputs
        .iter()
        .enumerate()
        .map(|(index, input)| (input.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut bindings = graph
        .edges
        .iter()
        .filter(|edge| edge.to.node_id == node.id)
        .map(|edge| ReduceDependencyBinding {
            input_port: edge.to.port.clone(),
            source_node_id: edge.from.node_id.clone(),
            source_output_name: edge.from.port.clone(),
        })
        .collect::<Vec<_>>();
    bindings.sort_by(|left, right| {
        input_positions
            .get(&left.input_port)
            .unwrap_or(&usize::MAX)
            .cmp(input_positions.get(&right.input_port).unwrap_or(&usize::MAX))
            .then_with(|| left.input_port.cmp(&right.input_port))
            .then_with(|| left.source_node_id.cmp(&right.source_node_id))
            .then_with(|| left.source_output_name.cmp(&right.source_output_name))
    });
    bindings
}

fn write_reduce_collection_manifest(
    ctx: &RunContext,
    node_id: &str,
    collection: &InputCollection,
) -> Result<(), RuntimeError> {
    let bytes = serde_json::to_vec_pretty(collection)?;
    let path = ctx.run_dir.node_inputs_dir(node_id).join(reduce_collection_manifest_name());
    ctx.fs.write(&path, &bytes)?;
    Ok(())
}

fn write_reduce_summary(
    ctx: &RunContext,
    node_id: &str,
    summary: &ReduceExecutionSummary,
) -> Result<(), RuntimeError> {
    let bytes = serde_json::to_vec_pretty(summary)?;
    ctx.fs.write(&reduce_execution_summary_path(ctx, node_id), &bytes)?;
    Ok(())
}

fn build_reduce_summary(
    graph: &Graph,
    node: &Node,
    parent_statuses: &HashMap<String, NodeStatus>,
    materialized_inputs: &BTreeMap<(String, String, String), (String, String)>,
) -> Result<ReduceExecutionSummary, RuntimeError> {
    let config = reduce_execution_config(node)
        .map_err(|failure| RuntimeError::Executor(failure.message.clone()))?;
    let mut usable_input_count = 0usize;
    let mut failed_input_count = 0usize;
    let mut skipped_input_count = 0usize;
    let mut cancelled_input_count = 0usize;
    let mut items = Vec::new();

    for binding in reduce_dependency_bindings(graph, node) {
        let status = parent_statuses.get(&binding.source_node_id).cloned().ok_or_else(|| {
            RuntimeError::Executor(format!(
                "missing terminal status for reduce dependency {} -> {}",
                binding.source_node_id, node.id
            ))
        })?;
        let key = (
            binding.input_port.clone(),
            binding.source_node_id.clone(),
            binding.source_output_name.clone(),
        );
        let (local_path, source_sha256) = materialized_inputs
            .get(&key)
            .cloned()
            .map(|(path, sha)| (Some(path), Some(sha)))
            .unwrap_or((None, None));
        match status {
            NodeStatus::Success | NodeStatus::Cached => usable_input_count += 1,
            NodeStatus::Failed => failed_input_count += 1,
            NodeStatus::Skipped => skipped_input_count += 1,
            NodeStatus::Cancelled => cancelled_input_count += 1,
        }
        items.push(InputCollectionItem {
            input_port: binding.input_port,
            source_node_id: binding.source_node_id,
            source_output_name: binding.source_output_name,
            status: status_string(&status),
            local_path,
            source_sha256,
        });
    }

    let collection = InputCollection {
        name: "reduce_inputs".to_string(),
        semantic_kind: "reduce".to_string(),
        manifest_path: reduce_collection_manifest_name().to_string(),
        mode: Some(reduce_mode_label(config.mode).to_string()),
        empty_policy: Some(reduce_empty_policy_label(config.empty_policy).to_string()),
        items,
    };
    Ok(ReduceExecutionSummary {
        schema_version: reduce_execution_version(),
        reduce_node_id: node.id.clone(),
        mode: reduce_mode_label(config.mode).to_string(),
        empty_policy: reduce_empty_policy_label(config.empty_policy).to_string(),
        usable_input_count,
        failed_input_count,
        skipped_input_count,
        cancelled_input_count,
        collection,
    })
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
            promotable: output.promotable,
        })
        .collect()
}

fn is_managed_output_metadata_path(rel: &str) -> bool {
    rel == "index.json"
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
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_SCHEMA_INVALID",
                    message,
                    None,
                )),
            };
        }
    }

    let mut output_evidence = Vec::new();
    for output in &declared {
        let path = match authorized_declared_output_path(dir, output) {
            Ok(path) => path,
            Err(failure) => {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(failure),
                };
            }
        };
        if path.is_symlink() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_PATH_INVALID",
                    format!("output must not be a symlink: {}", output.path),
                    None,
                )),
            };
        }
        if !path.exists() {
            output_evidence.push(TraceOutputArtifact {
                name: output.name.clone(),
                path: output.path.clone(),
                kind: output_kind_label(&output.kind).to_string(),
                required: output.required,
                present: false,
                media_type: output.effective_media_type(),
                size_bytes: None,
                sha256: None,
                promotable: output.promotable,
            });
            if output.required {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(FailureInfo::new(
                        FailureClass::User,
                        "User",
                        "OUTPUT_MISSING",
                        format!("missing required output: {}", output.path),
                        Some(serde_json::json!({ "output": output.name })),
                    )),
                };
            }
            continue;
        }
        if path.is_symlink() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_PATH_INVALID",
                    format!("output must not be a symlink: {}", output.path),
                    None,
                )),
            };
        }
        if output.expects_directory() && !path.is_dir() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_PATH_INVALID",
                    format!("output must be a directory: {}", output.path),
                    Some(serde_json::json!({ "output": output.name })),
                )),
            };
        }
        if output.expects_file() && !path.is_file() {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_PATH_INVALID",
                    format!("output must be a file: {}", output.path),
                    Some(serde_json::json!({ "output": output.name })),
                )),
            };
        }
        let size_bytes = match artifact_size_bytes(&path) {
            Ok(size_bytes) => size_bytes,
            Err(error) => {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(FailureInfo::new(
                        FailureClass::User,
                        "User",
                        "OUTPUT_PATH_INVALID",
                        error.to_string(),
                        Some(serde_json::json!({ "output": output.name })),
                    )),
                };
            }
        };
        let sha256 = match sha256_artifact_path(&path) {
            Ok(sha256) => sha256,
            Err(error) => {
                return OutputInspectionReport {
                    output_evidence,
                    present_outputs,
                    failure: Some(FailureInfo::new(
                        FailureClass::User,
                        "User",
                        "OUTPUT_PATH_INVALID",
                        error.to_string(),
                        Some(serde_json::json!({ "output": output.name })),
                    )),
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
            size_bytes: Some(size_bytes),
            sha256: Some(sha256.clone()),
            promotable: output.promotable,
        });
        present_outputs.push(DeclaredOutputArtifact {
            name: output.name.clone(),
            path: output.path.clone(),
            kind: output_kind_label(&output.kind).to_string(),
            media_type,
            promotable: output.promotable,
        });
    }

    let mut actual = std::collections::BTreeSet::new();
    collect_relative_artifacts(dir, dir, &mut actual);
    for rel in actual {
        if is_managed_output_metadata_path(&rel) {
            continue;
        }
        let declared_match = declared.iter().any(|output| {
            rel == output.path
                || (matches!(output.kind, OutputKind::Directory)
                    && rel.starts_with(&format!("{}/", output.path)))
        });
        if !declared_match {
            return OutputInspectionReport {
                output_evidence,
                present_outputs,
                failure: Some(FailureInfo::new(
                    FailureClass::User,
                    "User",
                    "OUTPUT_UNDECLARED",
                    format!("undeclared output path: {}", rel),
                    None,
                )),
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
    adapter_binary_sha256: Option<&str>,
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
            node,
            node_fingerprint,
            ctx,
            adapter_id,
            adapter_version,
            adapter_binary_sha256,
            adapter_outputs_schema_version,
        )?;
        let key = cache_key_explanation(&key_input).key;
        let store = cache_store.as_ref().unwrap();
        let entry = store.entry(&key);
        let mut local_corrupt_entry: Option<PathBuf> = None;
        let mut local_corrupt_proof: Option<CacheProof> = None;
        if store.fs().metadata(&entry).is_ok() {
            if !verify_cache_entry(store.fs(), &entry, node, &key_input)? {
                local_corrupt_entry = Some(entry.clone());
                local_corrupt_proof = Some(CacheProof {
                    hit: false,
                    key: key.clone(),
                    source: "local".to_string(),
                    verified: false,
                    reason: "corrupt".to_string(),
                    corrupt_detected: true,
                });
            } else {
                let source = cache_source_from_meta(store.fs(), &entry)
                    .unwrap_or_else(|| "local".to_string());
                prepare_node_execution_dirs(ctx, &node.id)?;
                let node_dir = ctx.run_dir.node_dir(&node.id);
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
        }
        if let Some(remote_dir) = options.remote_cache_dir.as_ref() {
            let remote_entry = remote_dir.join(&key);
            if store.fs().metadata(&remote_entry).is_ok() {
                if !verify_cache_entry(store.fs(), &remote_entry, node, &key_input)? {
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
                prepare_node_execution_dirs(ctx, &node.id)?;
                let node_dir = ctx.run_dir.node_dir(&node.id);
                copy_dir_all(
                    store.fs(),
                    remote_entry.join("outputs"),
                    ctx.run_dir.node_outputs_dir(&node.id),
                )?;
                copy_dir_all(store.fs(), remote_entry.join("logs"), node_dir.clone())?;
                if let Some(corrupt_entry) = local_corrupt_entry.as_ref() {
                    let _ = store.fs().remove_dir_all(corrupt_entry);
                }
                if let Some(local_dir) = options.cache_dir.as_ref() {
                    let local_entry = local_dir.join(&key);
                    if let Ok(outcome) = copy_cache_entry_atomically(
                        store.fs(),
                        &remote_entry,
                        &local_entry,
                        "hydrate",
                    ) {
                        if matches!(outcome, CachePublishOutcome::Published)
                            && !verify_cache_entry(store.fs(), &local_entry, node, &key_input)?
                        {
                            let _ = store.fs().remove_dir_all(&local_entry);
                        }
                    }
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
        if let Some(proof) = local_corrupt_proof {
            return Ok(CacheRead { hit: false, proof: Some(proof) });
        }
    }
    Ok(CacheRead { hit: false, proof: None })
}

fn prepare_node_execution_dirs(ctx: &RunContext, node_id: &str) -> Result<(), RuntimeError> {
    let node_dir = ctx.run_dir.node_dir(node_id);
    ctx.fs.create_dir_all(&node_dir)?;
    recreate_dir(ctx.fs.as_ref(), &ctx.run_dir.node_outputs_dir(node_id))?;
    ctx.fs.create_dir_all(&ctx.run_dir.node_work_dir(node_id))?;
    recreate_dir(ctx.fs.as_ref(), &ctx.run_dir.node_temp_dir(node_id))?;
    Ok(())
}

fn recreate_dir(fs: &dyn Fs, path: &Path) -> std_io::Result<()> {
    match fs.metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs.remove_dir_all(path)?;
            } else {
                fs.remove_file(path)?;
            }
        }
        Err(err) if err.kind() == std_io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    fs.create_dir_all(path)
}

pub(crate) fn apply_temp_env(cmd: &mut std::process::Command, temp_dir: &Path) {
    let temp_dir = temp_dir.display().to_string();
    cmd.env("TMPDIR", &temp_dir);
    cmd.env("TMP", &temp_dir);
    cmd.env("TEMP", &temp_dir);
}

fn container_temp_dir(workdir: &str) -> String {
    format!("{workdir}/temp")
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
    adapter_binary_sha256: Option<&str>,
    adapter_outputs_schema_version: &str,
) -> Result<(), RuntimeError> {
    if options.cache_mode != CacheMode::ReadWrite {
        return Ok(());
    }
    if !node.cache.enabled {
        return Ok(());
    }
    let local_cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let remote_cache_dir = options.remote_cache_dir.clone();
    let staging_root = match local_cache_dir.clone().or_else(|| remote_cache_dir.clone()) {
        Some(d) => d,
        None => return Ok(()),
    };
    let store = RuntimeCacheStore::new(staging_root.clone(), Arc::clone(&fs));
    let key_input = cache_key_input_for_run(
        options,
        node,
        node_fingerprint,
        ctx,
        adapter_id,
        adapter_version,
        adapter_binary_sha256,
        adapter_outputs_schema_version,
    )?;
    let key = cache_key_explanation(&key_input).key;
    let staging_entry = cache_staging_entry_path(&staging_root, &key, "publish");
    populate_cache_entry_dir(store.fs(), &staging_entry, node, ctx, &key_input, &key)?;

    let mut canonical_entry: Option<PathBuf> = None;
    if let Some(local_dir) = local_cache_dir.as_ref() {
        let local_entry = local_dir.join(&key);
        let _ = if local_dir == &staging_root {
            publish_staged_cache_entry(store.fs(), &staging_entry, &local_entry)?
        } else {
            copy_cache_entry_atomically(store.fs(), &staging_entry, &local_entry, "publish")?
        };
        canonical_entry = Some(local_entry);
    }
    if canonical_entry.is_none() {
        if let Some(remote_dir) = remote_cache_dir.as_ref() {
            let remote_entry = remote_dir.join(&key);
            let _ = if remote_dir == &staging_root {
                publish_staged_cache_entry(store.fs(), &staging_entry, &remote_entry)?
            } else {
                copy_cache_entry_atomically(store.fs(), &staging_entry, &remote_entry, "publish")?
            };
            canonical_entry = Some(remote_entry);
        }
    }
    if let (Some(source_entry), Some(remote_dir)) =
        (canonical_entry.as_ref(), remote_cache_dir.as_ref())
    {
        let remote_entry = remote_dir.join(&key);
        if &remote_entry != source_entry {
            let _ =
                copy_cache_entry_atomically(store.fs(), source_entry, &remote_entry, "publish")?;
        }
    }
    if store.fs().metadata(&staging_entry).is_ok() {
        let _ = store.fs().remove_dir_all(&staging_entry);
    }
    Ok(())
}

fn populate_cache_entry_dir(
    fs: &dyn Fs,
    entry: &Path,
    node: &Node,
    ctx: &RunContext,
    key_input: &CacheKeyInput,
    key: &str,
) -> Result<(), RuntimeError> {
    if fs.metadata(entry).is_ok() {
        fs.remove_dir_all(entry)?;
    }
    fs.create_dir_all(entry.join("outputs").as_path())?;
    fs.create_dir_all(entry.join("logs").as_path())?;
    let manifest = cache_entry_manifest_for_node(node, key);
    let meta = serde_json::json!({
        "cache_metadata_version": crate::cache::CACHE_METADATA_VERSION,
        "cache_key": key,
        "node_id": node.id,
        "node_fingerprint": key_input.execution_fingerprint,
        "node_definition_fingerprint": key_input.node_definition_fingerprint,
        "declared_environment_fingerprint": key_input.declared_environment_fingerprint,
        "input_lineage_fingerprint": key_input.input_lineage_fingerprint,
        "params_fingerprint": params_fingerprint_from_ctx(ctx, &node.id),
        "command_fingerprint": command_fingerprint_from_ctx(ctx, &node.id),
        "adapter_id": key_input.adapter_id,
        "adapter_version": key_input.adapter_version,
        "adapter_binary_sha256": key_input.adapter_binary_sha256,
        "produces_outputs_schema_version": key_input.output_schema_version,
        "policy_fingerprint": key_input.policy_fingerprint,
        "execution_contract_fingerprint": key_input.execution_contract_fingerprint,
        "backend_class": key_input.backend_class,
        "created_unix_ms": ctx.clock.now_unix_ms(),
        "cache_source": "local",
        "schema_version": "v0.1",
    });
    fs.write(entry.join("manifest.json").as_path(), &serde_json::to_vec_pretty(&manifest)?)?;
    fs.write(entry.join("meta.json").as_path(), &serde_json::to_vec_pretty(&meta)?)?;
    copy_dir_all(fs, ctx.run_dir.node_outputs_dir(&node.id), entry.join("outputs"))?;
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let _ = fs.copy(
        node_dir.join("stdout.log").as_path(),
        entry.join("logs").join("stdout.log").as_path(),
    );
    let _ = fs.copy(
        node_dir.join("stderr.log").as_path(),
        entry.join("logs").join("stderr.log").as_path(),
    );
    let _ = fs.copy(
        node_dir.join("trace.json").as_path(),
        entry.join("logs").join("trace.json").as_path(),
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CachePublishOutcome {
    Published,
    AlreadyPresent,
}

fn publish_staged_cache_entry(
    fs: &dyn Fs,
    staging_entry: &Path,
    target_entry: &Path,
) -> std_io::Result<CachePublishOutcome> {
    if fs.metadata(target_entry).is_ok() {
        let _ = fs.remove_dir_all(staging_entry);
        return Ok(CachePublishOutcome::AlreadyPresent);
    }
    if let Some(parent) = target_entry.parent() {
        fs.create_dir_all(parent)?;
    }
    match fs.rename(staging_entry, target_entry) {
        Ok(()) => Ok(CachePublishOutcome::Published),
        Err(error) => {
            let target_exists = fs.metadata(target_entry).is_ok();
            let _ = fs.remove_dir_all(staging_entry);
            if target_exists {
                Ok(CachePublishOutcome::AlreadyPresent)
            } else {
                Err(error)
            }
        }
    }
}

fn copy_cache_entry_atomically(
    fs: &dyn Fs,
    src_entry: &Path,
    target_entry: &Path,
    operation: &str,
) -> std_io::Result<CachePublishOutcome> {
    let cache_root = target_entry.parent().ok_or_else(|| {
        std_io::Error::new(std_io::ErrorKind::InvalidInput, "cache target missing parent directory")
    })?;
    let key = target_entry.file_name().and_then(|value| value.to_str()).ok_or_else(|| {
        std_io::Error::new(std_io::ErrorKind::InvalidInput, "cache target missing cache key")
    })?;
    let staging_entry = cache_staging_entry_path(cache_root, key, operation);
    copy_dir_all(fs, src_entry, &staging_entry)?;
    publish_staged_cache_entry(fs, &staging_entry, target_entry)
}

fn cache_staging_entry_path(cache_root: &Path, cache_key: &str, operation: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    cache_root.join(format!(".cache-{cache_key}-{operation}-{}-{nonce}", std::process::id()))
}

fn cache_entry_manifest_for_node(node: &Node, cache_key: &str) -> CacheEntryManifest {
    let mut outputs = node
        .outputs
        .iter()
        .map(|output| CacheManifestOutput {
            name: output.name.clone(),
            path: output.path.clone(),
            kind: output_kind_label(&output.kind).to_string(),
            media_type: output.effective_media_type(),
            required: output.required,
        })
        .collect::<Vec<_>>();
    outputs.sort_by(|a, b| a.path.cmp(&b.path));
    CacheEntryManifest {
        manifest_version: CACHE_ENTRY_MANIFEST_VERSION.to_string(),
        cache_key: cache_key.to_string(),
        node_id: node.id.clone(),
        outputs,
    }
}

fn verify_cache_entry(
    fs: &dyn Fs,
    entry: &Path,
    node: &Node,
    expected_input: &CacheKeyInput,
) -> Result<bool, RuntimeError> {
    let index_path = entry.join("outputs").join("index.json");
    if fs.metadata(&index_path).is_err() {
        return Ok(false);
    }
    let manifest_path = entry.join("manifest.json");
    if fs.metadata(&manifest_path).is_err() {
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
    let manifest: CacheEntryManifest = serde_json::from_str(&fs.read_to_string(&manifest_path)?)?;
    if !cache_entry_manifest_version_supported(&manifest) {
        return Ok(false);
    }
    let expected_manifest = cache_entry_manifest_for_node(node, &expected_key);
    if manifest != expected_manifest {
        return Ok(false);
    }
    if meta.get("node_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.execution_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("node_definition_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.node_definition_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("declared_environment_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.declared_environment_fingerprint.as_str())
    {
        return Ok(false);
    }
    if meta.get("input_lineage_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.input_lineage_fingerprint.as_str())
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
    if meta.get("adapter_binary_sha256").and_then(|v| v.as_str())
        != expected_input.adapter_binary_sha256.as_deref()
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
    if meta.get("execution_contract_fingerprint").and_then(|v| v.as_str())
        != Some(expected_input.execution_contract_fingerprint.as_str())
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
    for expected_output in &manifest.outputs {
        let indexed = index.files.iter().find(|file| file.path == expected_output.path);
        if expected_output.required && indexed.is_none() {
            return Ok(false);
        }
        let Some(file) = indexed else {
            continue;
        };
        if file.name != expected_output.name
            || file.kind != expected_output.kind
            || file.media_type != expected_output.media_type
            || file.node_id != node.id
            || file.node_fingerprint != expected_input.execution_fingerprint
        {
            return Ok(false);
        }
        let path = entry.join("outputs").join(&file.path);
        if fs.metadata(&path).is_err() {
            return Ok(false);
        }
        let sha = sha256_artifact_path(&path).map_err(RuntimeError::Artifact)?;
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    for file in index.files {
        if !manifest.outputs.iter().any(|output| {
            output.path == file.path
                && output.name == file.name
                && output.kind == file.kind
                && output.media_type == file.media_type
        }) {
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

fn cache_identity_for_trace(
    ctx: &RunContext,
    node_id: &str,
    adapter_id: &str,
    adapter_version: &str,
    adapter_binary_sha256: Option<&str>,
    adapter_outputs_schema_version: &str,
) -> Result<CacheIdentity, RuntimeError> {
    let key_input = CacheKeyInput {
        execution_fingerprint: node_fingerprint_from_ctx(ctx, node_id),
        node_definition_fingerprint: node_definition_fingerprint_from_ctx(ctx, node_id),
        declared_environment_fingerprint: declared_environment_fingerprint_from_ctx(ctx, node_id),
        input_lineage_fingerprint: input_lineage_fingerprint_from_run(ctx, node_id)?,
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        adapter_binary_sha256: adapter_binary_sha256.map(ToString::to_string),
        output_schema_version: adapter_outputs_schema_version.to_string(),
        policy_fingerprint: policy_fingerprint(&ctx.policy),
        execution_contract_fingerprint: ctx.execution_contract_fingerprint.clone(),
        backend_class: "local".to_string(),
    };
    Ok(CacheIdentity {
        cache_key: cache_key_explanation(&key_input).key,
        node_definition_fingerprint: key_input.node_definition_fingerprint,
        declared_environment_fingerprint: key_input.declared_environment_fingerprint,
        input_lineage_fingerprint: key_input.input_lineage_fingerprint,
        adapter_binary_sha256: key_input.adapter_binary_sha256,
        params_fingerprint: params_fingerprint_from_ctx(ctx, node_id),
        command_fingerprint: command_fingerprint_from_ctx(ctx, node_id),
        policy_fingerprint: key_input.policy_fingerprint,
        execution_contract_fingerprint: key_input.execution_contract_fingerprint,
        backend_class: key_input.backend_class,
    })
}

pub(crate) fn node_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.graph_fingerprint.lock().ok().and_then(|map| map.get(node_id).cloned()).unwrap_or_default()
}

fn node_definition_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.node_definition_fingerprints.get(node_id).cloned().unwrap_or_default()
}

fn declared_environment_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.declared_environment_fingerprints.get(node_id).cloned().unwrap_or_default()
}

fn params_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.params_fingerprints.get(node_id).cloned().unwrap_or_default()
}

fn command_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> Option<String> {
    ctx.command_fingerprints.get(node_id).cloned().flatten()
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
    let value = if inputs.collections.is_empty() {
        serde_json::json!({
            "base": base_fp,
            "inputs": &inputs.files,
        })
    } else {
        serde_json::json!({
            "base": base_fp,
            "inputs": &inputs.files,
            "collections": &inputs.collections,
        })
    };
    Ok(sha256_bytes(&serde_json::to_vec_pretty(&value)?))
}

fn input_lineage_fingerprint(inputs: &InputsIndex) -> Result<String, RuntimeError> {
    if inputs.collections.is_empty() {
        return Ok(sha256_bytes(&serde_json::to_vec(&inputs.files)?));
    }
    Ok(sha256_bytes(&serde_json::to_vec(inputs)?))
}

fn input_lineage_fingerprint_from_run(
    ctx: &RunContext,
    node_id: &str,
) -> Result<String, RuntimeError> {
    let index_path = ctx.run_dir.node_inputs_dir(node_id).join("index.json");
    if ctx.fs.metadata(&index_path).is_err() {
        return input_lineage_fingerprint(&InputsIndex {
            collections: Vec::new(),
            files: Vec::new(),
        });
    }
    let raw = ctx.fs.read_to_string(&index_path)?;
    let index: InputsIndex = serde_json::from_str(&raw)?;
    input_lineage_fingerprint(&index)
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

pub(crate) fn apply_shaped_env(
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
) -> Result<ControlledCommandResult, RuntimeError> {
    command_output_with_controls(cmd, timeout_ms, None)
}

fn cancellation_registry() -> &'static Mutex<Vec<Weak<AtomicBool>>> {
    static REGISTRY: OnceLock<Mutex<Vec<Weak<AtomicBool>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn broadcast_runtime_cancellation() {
    let mut registry = cancellation_registry().lock().expect("cancellation registry");
    registry.retain(|entry| {
        if let Some(flag) = entry.upgrade() {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    });
}

pub(crate) fn install_runtime_cancellation_handler() {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    INSTALLED.get_or_init(|| {
        let _ = ctrlc::set_handler(broadcast_runtime_cancellation);
    });
}

pub(crate) fn register_runtime_cancellation_flag(flag: &Arc<AtomicBool>) {
    let mut registry = cancellation_registry().lock().expect("cancellation registry");
    registry.retain(|entry| entry.strong_count() > 0);
    registry.push(Arc::downgrade(flag));
}

pub(crate) fn command_output_with_controls(
    cmd: &mut std::process::Command,
    timeout_ms: Option<u64>,
    cancellation_requested: Option<&AtomicBool>,
) -> Result<ControlledCommandResult, RuntimeError> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    configure_controlled_subprocess(cmd);
    let mut child = cmd.spawn().map_err(RuntimeError::Io)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RuntimeError::Executor("failed to capture process stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RuntimeError::Executor("failed to capture process stderr".to_string()))?;
    let stdout_reader = spawn_output_reader(stdout, "stdout")?;
    let stderr_reader = spawn_output_reader(stderr, "stderr")?;
    let timeout_limit = timeout_ms.map(|limit| (limit, std::time::Instant::now()));
    let (termination, outcome_kind) = loop {
        if let Some(status) = child.try_wait().map_err(RuntimeError::Io)? {
            break (
                ControlledCommandTermination::new(status),
                ControlledCommandOutcomeKind::Exited,
            );
        }
        if cancellation_requested.is_some_and(|requested| requested.load(Ordering::SeqCst)) {
            let termination = terminate_child_best_effort(&mut child).map_err(RuntimeError::Io)?;
            break (termination, ControlledCommandOutcomeKind::Cancelled);
        }
        if timeout_limit
            .is_some_and(|(limit_ms, started)| started.elapsed().as_millis() > limit_ms as u128)
        {
            let termination = terminate_child_best_effort(&mut child).map_err(RuntimeError::Io)?;
            break (termination, ControlledCommandOutcomeKind::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = join_output_reader(stdout_reader, "stdout")?;
    let stderr = join_output_reader(stderr_reader, "stderr")?;
    stderr
        .append_cleanup_diagnostics(&termination.cleanup_diagnostics)
        .map_err(RuntimeError::Io)?;
    let output = ControlledCommandOutput { status: termination.status, stdout, stderr };
    Ok(match outcome_kind {
        ControlledCommandOutcomeKind::Exited => ControlledCommandResult::Exited(output),
        ControlledCommandOutcomeKind::Cancelled => ControlledCommandResult::Cancelled(output),
        ControlledCommandOutcomeKind::TimedOut => ControlledCommandResult::TimedOut(output),
    })
}

fn configure_controlled_subprocess(cmd: &mut std::process::Command) {
    #[cfg(unix)]
    {
        cmd.process_group(0);
    }
}

struct ControlledCommandReader {
    path: PathBuf,
    handle: std::thread::JoinHandle<std_io::Result<()>>,
}

fn spawn_output_reader<T>(
    mut stream: T,
    stream_name: &str,
) -> std_io::Result<ControlledCommandReader>
where
    T: Read + Send + 'static,
{
    let (mut file, path) = create_capture_file(stream_name)?;
    let handle = std::thread::spawn(move || {
        std_io::copy(&mut stream, &mut file)?;
        Ok(())
    });
    Ok(ControlledCommandReader { path, handle })
}

fn join_output_reader(
    reader: ControlledCommandReader,
    stream_name: &str,
) -> Result<ControlledCommandStream, RuntimeError> {
    reader
        .handle
        .join()
        .map_err(|_| {
            RuntimeError::Executor(format!("failed to join {stream_name} capture thread"))
        })?
        .map_err(RuntimeError::Io)?;
    Ok(ControlledCommandStream { path: reader.path })
}

fn terminate_child_best_effort(
    child: &mut std::process::Child,
) -> std_io::Result<ControlledCommandTermination> {
    if let Some(status) = child.try_wait()? {
        return Ok(ControlledCommandTermination::new(status));
    }

    #[cfg(unix)]
    {
        return terminate_process_group_best_effort(child);
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
        Ok(ControlledCommandTermination::new(child.wait()?))
    }
}

#[cfg(unix)]
fn terminate_process_group_best_effort(
    child: &mut std::process::Child,
) -> std_io::Result<ControlledCommandTermination> {
    const SIGNAL_GRACE_PERIOD: Duration = Duration::from_millis(250);

    let process_group_id = child.id();
    let mut termination = ControlledCommandTermination::new(unreachable_exit_status());

    if let Err(error) = signal_process_group(process_group_id, "TERM") {
        termination.cleanup_diagnostics.push(format!(
            "failed to send SIGTERM to subprocess group {process_group_id}: {error}"
        ));
    }
    if let Some(status) = wait_for_child_exit(child, SIGNAL_GRACE_PERIOD)? {
        termination.status = status;
        return Ok(termination);
    }

    if let Err(error) = signal_process_group(process_group_id, "KILL") {
        termination.cleanup_diagnostics.push(format!(
            "failed to send SIGKILL to subprocess group {process_group_id}: {error}"
        ));
    }
    if let Some(status) = wait_for_child_exit(child, SIGNAL_GRACE_PERIOD)? {
        termination.status = status;
        return Ok(termination);
    }

    if let Err(error) = child.kill() {
        termination
            .cleanup_diagnostics
            .push(format!("failed to kill subprocess leader {process_group_id}: {error}"));
    }
    termination.status = child.wait()?;
    Ok(termination)
}

#[cfg(unix)]
fn signal_process_group(process_group_id: u32, signal: &str) -> std_io::Result<()> {
    let target = format!("-{process_group_id}");
    let status =
        std::process::Command::new("kill").args([format!("-{signal}"), target]).status()?;
    if status.success() {
        return Ok(());
    }
    Err(std_io::Error::other(format!("kill exited with status {status}")))
}

fn wait_for_child_exit(
    child: &mut std::process::Child,
    grace_period: Duration,
) -> std_io::Result<Option<std::process::ExitStatus>> {
    let deadline = std::time::Instant::now() + grace_period;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if std::time::Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn create_capture_file(stream_name: &str) -> std_io::Result<(std::fs::File, PathBuf)> {
    static CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    for _ in 0..32 {
        let sequence = CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bijux-dag-{stream_name}-{}-{timestamp}-{sequence}.log",
            std::process::id()
        ));
        match std::fs::OpenOptions::new().create_new(true).read(true).write(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == std_io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }

    Err(std_io::Error::new(
        std_io::ErrorKind::AlreadyExists,
        format!("failed to allocate unique capture path for {stream_name}"),
    ))
}

fn read_file_tail_bytes(path: &Path, max_bytes: u64) -> std_io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let file_len = file.metadata()?.len();
    let start = file_len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(unix)]
fn unreachable_exit_status() -> std::process::ExitStatus {
    std::os::unix::process::ExitStatusExt::from_raw(0)
}

pub(crate) fn effective_node_timeout_ms(node: &Node, params: &Value) -> Option<u64> {
    node.timeout_ms.or_else(|| params.get("timeout_ms").and_then(|v| v.as_u64()))
}

fn enforce_container_image_reference_policy(
    image_reference: &str,
    policy: ContainerImageReferencePolicy,
) -> Result<(), FailureInfo> {
    if matches!(policy, ContainerImageReferencePolicy::AllowUnpinned)
        || container_image_reference_has_digest(image_reference)
    {
        return Ok(());
    }

    Err(FailureInfo::new(
        FailureClass::Policy,
        "Policy",
        "POLICY_CONTAINER_IMAGE_REFERENCE_DENIED",
        "container image reference must include an @sha256 digest under the active policy",
        Some(serde_json::json!({
            "image": image_reference,
            "container_image_reference_policy": container_image_reference_policy_label(policy),
        })),
    ))
}

fn container_image_reference_has_digest(image_reference: &str) -> bool {
    image_reference.contains("@sha256:")
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
                        size_bytes: f.size_bytes,
                        sha256: f.sha256,
                        promotable: f.promotable,
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
            size_bytes: out.size_bytes,
            sha256: out.sha256.clone(),
            path: rel,
            promotable: out.promotable,
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
    let mut counts = NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0, cancelled: 0 };
    for status in status_map.values() {
        match status {
            NodeStatus::Success => counts.success += 1,
            NodeStatus::Failed => counts.failed += 1,
            NodeStatus::Skipped => counts.skipped += 1,
            NodeStatus::Cached => counts.cached += 1,
            NodeStatus::Cancelled => counts.cancelled += 1,
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
mod controlled_command_cleanup_contract_tests {
    use super::*;
    use std::sync::{Arc, Mutex, OnceLock};

    fn cleanup_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[cfg(unix)]
    fn nested_background_marker_command(marker_path: &Path) -> std::process::Command {
        let mut cmd = std::process::Command::new("/bin/sh");
        cmd.arg("-c")
            .arg("( /bin/sh -c 'sleep 1; printf orphan > \"$MARKER_PATH\"' & wait ) & sleep 5");
        cmd.env("MARKER_PATH", marker_path);
        cmd
    }

    #[cfg(unix)]
    #[test]
    fn timeout_kills_nested_background_descendants() {
        let _guard = cleanup_test_lock().lock().expect("cleanup test lock");
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let marker_path = temp_dir.path().join("orphan.txt");
        let mut cmd = nested_background_marker_command(&marker_path);

        let output = command_output_with_timeout(&mut cmd, Some(100)).expect("timeout result");
        assert!(matches!(output, ControlledCommandResult::TimedOut(_)));

        std::thread::sleep(Duration::from_millis(1_500));
        assert!(!marker_path.exists(), "timed out subprocess group left a descendant running");
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_kills_nested_background_descendants() {
        let _guard = cleanup_test_lock().lock().expect("cleanup test lock");
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let marker_path = temp_dir.path().join("orphan.txt");
        let mut cmd = nested_background_marker_command(&marker_path);
        let cancellation_requested = Arc::new(AtomicBool::new(false));
        let trigger = Arc::clone(&cancellation_requested);
        let notifier = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            trigger.store(true, Ordering::SeqCst);
        });

        let output =
            command_output_with_controls(&mut cmd, None, Some(cancellation_requested.as_ref()))
                .expect("cancelled result");
        notifier.join().expect("cancellation notifier");
        assert!(matches!(output, ControlledCommandResult::Cancelled(_)));

        std::thread::sleep(Duration::from_millis(1_500));
        assert!(!marker_path.exists(), "cancelled subprocess group left a descendant running");
    }

    #[cfg(unix)]
    #[test]
    fn termination_is_harmless_after_process_exits() {
        let _guard = cleanup_test_lock().lock().expect("cleanup test lock");
        let mut child = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg("exit 0")
            .spawn()
            .expect("spawn child");

        std::thread::sleep(Duration::from_millis(50));
        let termination = terminate_child_best_effort(&mut child).expect("terminate exited child");

        assert!(termination.status.success());
        assert!(termination.cleanup_diagnostics.is_empty());
    }
}

#[cfg(test)]
include!("internal/testing/tests_runtime.in.rs");
