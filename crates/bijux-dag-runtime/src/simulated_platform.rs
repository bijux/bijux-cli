//! Explicitly non-stable runtime surface for modeled platform capabilities.
//!
//! These APIs are retained for evidence, experiments, and cross-crate contract
//! coverage, but they are not part of the boring deterministic runtime kernel.

pub use crate::adaptive_scheduler::{
    adaptive_cache_policy, adaptive_fallback_needed, adaptive_maturity_ready,
    adaptive_queue_throttle, choose_prefetch_hints, compare_static_and_adaptive,
    decide_adaptive_parallelism, detect_adaptive_drift, render_adaptive_explanation,
    AdaptiveBackfillPacingDecision, AdaptiveBoundsPolicy, AdaptiveCachePolicyDecision,
    AdaptiveComparisonReport, AdaptiveConcurrencyDecision, AdaptiveControlLoopGuard,
    AdaptiveDriftReport, AdaptiveExplanation, AdaptiveFallbackPolicy, AdaptiveMaturityGate,
    AdaptiveQualityMetrics, AdaptiveQueueThrottleDecision, ArtifactPrefetchHint,
    BackendSuitabilitySignal, LearnedDurationProfile, LearningWindowPolicy,
    SlaDispatchTuningDecision,
};
pub use crate::ai_operator_assist::{
    anomaly_detected, answer_failure_question, build_investigation_bundle, build_postmortem_seed,
    guardrail_allows, next_maturity_level, recommend_safe_actions, redact_for_ai_export,
    root_cause_domain_hints, suggestion_quality, AiAssistMaturityLevel, ArtifactAnomalySummary,
    DiagnosticsAnswer, EvidenceCitation, FailureSummary, IncidentSimilarityResult,
    InvestigationBundle, ObservabilityAnomalySignal, OperatorReviewDecision, PlannerReviewSummary,
    PostmortemSeed, PrivacyRedactionPolicy, RecommendationSimulationResult, ReplayRecommendation,
    RootCauseDomainHint, SafeActionGuardrail, SafeOperatorAction, ScheduleAnomalySummary,
    SuggestedAction, WhatChangedSummary,
};
pub use crate::control_plane_api::{
    authorize, check_api_compatibility, filter_resources, paginate, ApiCompatibilityRule,
    ApiVersion, ArtifactApiOperation, ArtifactResource, AuditEventResource, AuthContext,
    AuthenticationPrincipal, AuthorizationRule, ClientSdkShape, ControlPlaneMvpDefinition,
    DagResource, DagVersionResource, EnvironmentScopedConfiguration, EventSubscription, ListFilter,
    NodeAttemptResource, Page, Pagination, PolicyResource, QueueResource, RegistryOperation,
    RunControlApiOperation, RunResource, ScheduleApiOperation, ScheduleResource,
    ServiceArchitectureNote, TypedApiRequest, TypedApiResponse, VersionedResource,
};
pub use crate::cost_optimization::{
    budget_policy_action, cache_reuse_score, choose_cost_profile, cost_optimization_allowed,
    detect_cost_anomaly, run_budget_allows, scorecard_ready, ArtifactEgressEstimate,
    BackendPricingModel, CacheReuseCostScore, CostAnomaly, CostAttributionRecord,
    CostAwareRoutingPolicy, CostBackfillThrottle, CostForecast, CostObservabilityReport,
    CostPerformanceProfile, CostPlacementExplanation, CostSafetyPolicy, CostSimulationScenario,
    ExecutionCostModel, PlanCostEstimate, PlatformCostMaturityScorecard, RunBudget,
    TenantBudgetPolicy,
};
pub use crate::dataset_semantics::{
    build_dataset_provenance_report, dataset_catalog_query, dataset_consumption_satisfied,
    dataset_diff, dataset_mapping_index, dataset_ready_for_schedule,
    default_dataset_example_workflow, DatasetArtifactMapping, DatasetBinding, DatasetCatalogEntry,
    DatasetCatalogQuery, DatasetCompleteness, DatasetConsumptionContract, DatasetConsumptionMode,
    DatasetDiffReport, DatasetFreshnessPolicy, DatasetId, DatasetImmutability,
    DatasetLineageRecord, DatasetPartitionModel, DatasetPartitionStrategy, DatasetProvenanceReport,
    DatasetPublicationWorkflow, DatasetQualityState, DatasetReadinessGate, DatasetRetentionPolicy,
    DatasetSchemaContract, DatasetVersionId,
};
pub use crate::distributed::{
    artifact_upload_can_commit, cancellation_delivered_in_time, check_worker_version_compatibility,
    classify_heartbeat, classify_status_reporting, is_duplicate_dispatch, normalize_status_events,
    recover_lost_lease, reject_worker_version_mismatch, should_reassign,
    validate_task_lease_semantics, validate_worker_identity, verify_remote_artifact_integrity,
    worker_alive, worker_pool_satisfies_capability_request, DeliveryGuarantee,
    DistributedExecutionRequest, DistributedExecutionResult, DistributedFailureClass,
    DistributedReadinessChecklist, DistributedSecurityModel, HeartbeatClass, HeartbeatSemantics,
    LivenessPolicy, MockRemoteBackend, PlacementHint, ReassignmentRule,
    RemoteArtifactCommitContract, RemoteArtifactUploadContract, RemoteCancellationContract,
    RemoteLogStreamContract, RemoteStatusEvent, RetryLineageRecord, StatusReportingClass,
    TaskLeaseSemantics, WorkLease, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity, WorkerPool,
    WorkerPoolCapabilityRequest, WorkerRegistration, WorkerSandboxNegotiation,
    WorkerVersionCompatibilityRule,
};
pub use crate::distribution_readiness::{
    adoption_score, conformance_passes, integration_governance_ready, packaging_ready,
    release_note_summary, upgrade_bundle_valid, CapabilityDiscoveryReport,
    ClusterDeploymentReference, DeploymentConformanceResult, DeploymentProfileBundle,
    DistributionSignatureRecord, EcosystemCatalog, InstallationDiagnostics,
    IntegrationGovernanceRule, IntegrationSupportPolicy, OnboardingGuideCatalog, PackagingMode,
    PackagingStrategy, PlatformAdoptionScorecard, ProductTierPolicy,
    ReferenceEnvironmentVerification, ReleaseNoteRecord, ReleaseTransparencyReport,
    SampleDeploymentCatalog, StabilityClass, StabilityMap, UpgradeBundle,
    VersionedCompatibilityMatrix,
};
pub use crate::federated_scheduling::{
    cross_domain_replay_safe, default_federation_maturity_matrix, delegation_allowed,
    domain_healthy, federation_conformance_passes, select_delegation_failure_action,
    trust_tier_allows_domain, CrossClusterRoutingPolicy, CrossDomainReplaySafety,
    DelegationFailureAction, DelegationFailurePolicy, DomainCapabilityAdvertisement,
    DomainHealthSnapshot, DomainRoutingExplanation, FederatedBackfillPlan,
    FederatedConformanceGate, FederatedScheduleSuppression, FederatedSimulationScenario,
    FederationConcurrencyControl, FederationDomainIdentity, FederationMaturityMatrix,
    InterSchedulerFlowControl, PeeringObservabilityContract, RunDelegationRecord,
    SchedulerDomainId, SchedulerPeeringRule, TrustTierRoutingRule,
};
pub use crate::geo_federation::{
    build_consistency_catalog, classify_resource_consistency, default_split_brain_mitigation,
    geo_ready, region_write_allowed, ConsistencyBoundaryNote, ConsistencyClass,
    CrossRegionFailoverRule, DisasterRecoveryPlaybook, GeoReadyAcceptanceGate,
    GeoSimulationScenario, InterRegionReplicationPolicy, RegionAffinityPolicy,
    RegionAwareDagActivation, RegionBackendRegistry, RegionId, RegionLineageRecord,
    RegionMigrationWorkflow, RegionObservabilityPartition, RegionPolicyOverlay,
    RegionQueuePartition, RegionScheduleRule, RegionalReplicaOwnership, SplitBrainMitigationPlan,
    WriteRoutingRule,
};
pub use crate::ha_scheduler::{
    clock_within_assumption, conformance_no_duplicate_runs, deduplicate_across_replicas,
    evaluate_ha_conformance, failover_recovery_passes, fence_allows_mutation,
    idempotent_run_creation, is_stale_leader, next_epoch, ordering_during_failover,
    DurableInFlightDispatch, DurableRunQueueEntry, DurableSchedulerStateStore,
    DurableSchedulerTick, HaConformanceReport, HaMilestoneDefinition, HaSimulationScenario,
    LeaderElectionState, QueueOwnershipTransfer, QueueShardLease, ScheduleDedupRecord,
    SchedulerAuditEvent, SchedulerAuditEventKind, SchedulerClockAssumption, SchedulerEpoch,
    SchedulerFenceToken, SchedulerRecoveryObjectives,
};
pub use crate::operations_governance::{
    evaluate_slo, health_dashboard_score, integrated_verification_lane_default,
    release_policy_allows, AuditReadinessChecklist, ErrorBudgetPolicy, GamedayScenario,
    IncidentClassification, IncidentSeverity, IntegratedVerificationLane, LifecycleGovernanceRule,
    OperatorTrainingCatalog, PlatformAcceptanceBoard, PlatformHealthDashboard,
    PlatformInvariantCatalog, PlatformOperatingModel, PostmortemTemplate, ProductBoundary,
    ReleaseGovernancePolicy, RoadmapGovernance, RunbookEntry, ServiceLevelIndicators,
    ServiceLevelObjective, SloEvaluation, SupportabilityModel, SustainabilityOwnership,
};
pub use crate::supply_chain_trust::{
    build_provenance_drift_report, can_promote_artifact, default_supply_chain_maturity_matrix,
    evaluate_attestation_compatibility, regulated_workflow_reference_example,
    replay_trust_warnings, require_provenance_completeness, verify_attestation_or_fail,
    ArtifactTrustLabel, AttestationCompatibility, AttestationFormatRule,
    AttestationVerificationResult, BinaryComponent, BinaryProvenanceRecord,
    ComplianceEvidenceBundle, EnvironmentAttestation, PluginProvenanceRecord, PluginTrustTier,
    PromotionPolicy, ProvenanceCompletenessPolicy, ProvenanceDriftReport,
    RegulatedWorkflowReference, ReplayTrustWarning, RunProvenanceAttestation,
    SignedArtifactManifest, SupplyChainMaturityMatrix,
};
pub use crate::tenancy::{
    check_scheduler_admission, compose_tenant_run_id, enforce_tenant_plugin_allowlist,
    resolve_tenant_overlay, scope_lineage_query, tenant_index_key, tenant_provisioning_bootstrap,
    validate_tenant_isolation, TenantConcurrencyQuota, TenantConfigOverlay,
    TenantEnvironmentOverlay, TenantId, TenantIsolationConformanceReport, TenantLifecycleState,
    TenantLineageScope, TenantObservabilityView, TenantOwnershipMetadata, TenantPluginAllowlist,
    TenantPolicyBundleRef, TenantProvisioningSpec, TenantQueueIsolationPolicy,
    TenantRegistryPartition, TenantResourceBudget, TenantRetentionPolicy, TenantSchedulerAdmission,
    TenantScopedDagName, TenantSecretScope,
};
pub use crate::workflow_product::{
    approval_gate_ready, critical_workflow_ready, evolution_plan_valid, portfolio_observability,
    product_positioning_note, rollout_is_progressive, wait_state_resumable,
    workflow_blueprint_valid, workflow_quality_gate_passed, workflow_template_catalog,
    world_class_score, ApprovalGateNode, CriticalWorkflowDesignation, EvolutionPlan,
    HumanWaitState, MultiDagTransaction, PolicyComposedBlueprint, PortfolioObservabilitySummary,
    ProductPositioningNote, RolloutWorkflow, SubworkflowInvocation, WorkflowContractInheritance,
    WorkflowEvent, WorkflowFamilyImpactAnalysis, WorkflowPortfolio, WorkflowProductMetadata,
    WorkflowQualityGate, WorkflowScenarioTest, WorkflowTemplate, WorkflowTemplateKind,
    WorkflowVerificationPlan, WorldClassPlatformScorecard,
};
