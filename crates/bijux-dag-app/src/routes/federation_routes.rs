use crate::commands::{DagCli, FederationCommands};
use crate::{emit_json, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    build_consistency_catalog, classify_resource_consistency, delegation_allowed, domain_healthy,
    federation_conformance_passes, geo_ready, region_write_allowed, replay_trust_warnings,
    resolve_tenant_overlay, select_delegation_failure_action, trust_tier_allows_domain,
    ConsistencyBoundaryNote, ConsistencyClass, CrossDomainReplaySafety, CrossRegionFailoverRule,
    DelegationFailureAction, DelegationFailurePolicy, DisasterRecoveryPlaybook,
    DomainHealthSnapshot, FederatedConformanceGate, FederationDomainIdentity,
    GeoReadyAcceptanceGate, GeoSimulationScenario, InterSchedulerFlowControl,
    PeeringObservabilityContract, RegionAffinityPolicy, RegionAwareDagActivation,
    RegionLineageRecord, RegionPolicyOverlay, RegionQueuePartition, RegionScheduleRule,
    ReplayTrustWarning, RunProvenanceAttestation, TenantConfigOverlay, TrustTierRoutingRule,
    WriteRoutingRule,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct ScheduleSimulation {
    activation: RegionAwareDagActivation,
    schedule_rules: Vec<RegionScheduleRule>,
    affinity: RegionAffinityPolicy,
    queue_partitions: Vec<RegionQueuePartition>,
    gate: GeoReadyAcceptanceGate,
}

#[derive(Debug, Serialize)]
struct ScheduleReport {
    selected_regions: Vec<String>,
    geo_ready: bool,
    affinity_preserved: bool,
    schedule_rules_complete: bool,
    queue_partitioned: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct FailoverSimulation {
    rule: CrossRegionFailoverRule,
    playbook: DisasterRecoveryPlaybook,
    scenario: GeoSimulationScenario,
}

#[derive(Debug, Serialize)]
struct FailoverReport {
    target_region: Option<String>,
    within_rto: bool,
    playbook_complete: bool,
    scenario_covered: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct LineageSimulation {
    record: RegionLineageRecord,
    allowed_regions: BTreeSet<bijux_dag_runtime::simulated_platform::RegionId>,
}

#[derive(Debug, Serialize)]
struct LineageReport {
    producer_region: String,
    visible_consumer_regions: Vec<String>,
    queryable: bool,
    regional_boundary_preserved: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SovereigntySimulation {
    resource: String,
    write_rule: WriteRoutingRule,
    requested_write_region: bijux_dag_runtime::simulated_platform::RegionId,
    consistency_notes: Vec<ConsistencyBoundaryNote>,
    overlay: RegionPolicyOverlay,
}

#[derive(Debug, Serialize)]
struct SovereigntyReport {
    write_allowed: bool,
    consistency_class: String,
    consistency_catalog_size: usize,
    regulatory_profile: String,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReplaySimulation {
    run_id: String,
    safety: CrossDomainReplaySafety,
    baseline: RunProvenanceAttestation,
    candidate: RunProvenanceAttestation,
}

#[derive(Debug, Serialize)]
struct ReplayReport {
    replay_safe: bool,
    warning_count: usize,
    warnings: ReplayTrustWarning,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PolicyDistributionSimulation {
    active_regions: BTreeSet<bijux_dag_runtime::simulated_platform::RegionId>,
    overlays: Vec<RegionPolicyOverlay>,
    gate: FederatedConformanceGate,
}

#[derive(Debug, Serialize)]
struct PolicyDistributionReport {
    all_regions_covered: bool,
    conformance_passed: bool,
    distinct_profiles: usize,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AuditIntegritySimulation {
    gate: FederatedConformanceGate,
    observability: PeeringObservabilityContract,
}

#[derive(Debug, Serialize)]
struct AuditIntegrityReport {
    audit_exchange_enabled: bool,
    metrics_exchange_enabled: bool,
    redaction_profile: String,
    integrity_passed: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TrustTierSimulation {
    rule: TrustTierRoutingRule,
    domain: FederationDomainIdentity,
}

#[derive(Debug, Serialize)]
struct TrustTierReport {
    domain_allowed: bool,
    tier_sufficient: bool,
    selected_domain: String,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct DelegationSimulation {
    flow: InterSchedulerFlowControl,
    target_domain: bijux_dag_runtime::simulated_platform::SchedulerDomainId,
    health: Vec<DomainHealthSnapshot>,
    inflight: usize,
    per_minute: usize,
    failure_policy: DelegationFailurePolicy,
    persistent_failure: bool,
}

#[derive(Debug, Serialize)]
struct DelegationReport {
    delegation_allowed: bool,
    target_domain_healthy: bool,
    failure_action: String,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ConfigInheritanceSimulation {
    global_defaults: std::collections::BTreeMap<String, String>,
    region_overlay: RegionPolicyOverlay,
    region_values: std::collections::BTreeMap<String, String>,
    tenant_overlay: TenantConfigOverlay,
    explicit_overrides: std::collections::BTreeMap<String, String>,
    review_required_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ConfigInheritanceReport {
    merged: std::collections::BTreeMap<String, String>,
    review_required_keys_present: bool,
    explicit_override_count: usize,
    gaps: Vec<String>,
}

fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
}

fn region_intersection(
    sets: [&BTreeSet<bijux_dag_runtime::simulated_platform::RegionId>; 4],
) -> BTreeSet<bijux_dag_runtime::simulated_platform::RegionId> {
    let mut current = sets[0].clone();
    for set in &sets[1..] {
        current = current.intersection(set).cloned().collect();
    }
    current
}

fn schedule_payload(simulation: &Path) -> Result<ScheduleReport, ExitCode> {
    let simulation: ScheduleSimulation = load_json_file(simulation)?;
    let selected = region_intersection([
        &simulation.activation.active_regions,
        &simulation.affinity.dag_regions,
        &simulation.affinity.run_regions,
        &simulation.affinity.artifact_regions,
    ])
    .intersection(&simulation.affinity.tenant_regions)
    .cloned()
    .collect::<BTreeSet<_>>();
    let affinity_preserved = !selected.is_empty();
    let schedule_rules_complete = affinity_preserved
        && selected.iter().all(|region| {
            simulation.schedule_rules.iter().any(|rule| {
                &rule.region == region
                    && !rule.timezone.trim().is_empty()
                    && (!rule.utc_anchor_required || !rule.failover_regions.is_empty())
            })
        });
    let queue_partitioned = affinity_preserved
        && selected.iter().all(|region| {
            simulation
                .queue_partitions
                .iter()
                .filter(|partition| &partition.region == region)
                .count()
                == 1
        });
    let geo_ready = geo_ready(&simulation.gate);
    let mut gaps = Vec::new();
    if !geo_ready {
        gaps.push("geo readiness gate is incomplete".to_string());
    }
    if !affinity_preserved {
        gaps.push("no common region satisfies dag, run, artifact, and tenant affinity".to_string());
    }
    if !schedule_rules_complete {
        gaps.push("multi-region schedule rules are incomplete for selected regions".to_string());
    }
    if !queue_partitioned {
        gaps.push("selected regions are missing dedicated queue partitions".to_string());
    }
    Ok(ScheduleReport {
        selected_regions: selected.into_iter().map(|region| region.0).collect(),
        geo_ready,
        affinity_preserved,
        schedule_rules_complete,
        queue_partitioned,
        gaps,
    })
}

fn failover_payload(simulation: &Path) -> Result<FailoverReport, ExitCode> {
    let simulation: FailoverSimulation = load_json_file(simulation)?;
    let target_region = simulation.rule.secondary_regions.first().map(|region| region.0.clone());
    let within_rto =
        simulation.scenario.delayed_failover_seconds <= simulation.rule.max_failover_seconds;
    let playbook_complete = simulation.playbook.region == simulation.rule.primary_region
        && !simulation.playbook.control_plane_outage_steps.is_empty()
        && !simulation.playbook.artifact_store_outage_steps.is_empty();
    let scenario_covered = simulation
        .scenario
        .region_loss
        .as_ref()
        .is_some_and(|region| region == &simulation.rule.primary_region);
    let mut gaps = Vec::new();
    if target_region.is_none() {
        gaps.push("no secondary region is configured for failover".to_string());
    }
    if !within_rto {
        gaps.push("failover exceeds the configured recovery time objective".to_string());
    }
    if !playbook_complete {
        gaps.push("disaster recovery playbook is incomplete for the primary region".to_string());
    }
    if !scenario_covered {
        gaps.push("simulation does not cover primary-region loss".to_string());
    }
    Ok(FailoverReport { target_region, within_rto, playbook_complete, scenario_covered, gaps })
}

fn lineage_payload(simulation: &Path) -> Result<LineageReport, ExitCode> {
    let simulation: LineageSimulation = load_json_file(simulation)?;
    let visible_consumer_regions = simulation
        .record
        .consumer_regions
        .intersection(&simulation.allowed_regions)
        .cloned()
        .collect::<BTreeSet<_>>();
    let regional_boundary_preserved =
        visible_consumer_regions.len() == simulation.record.consumer_regions.len();
    let queryable = simulation.record.lineage_queryable;
    let mut gaps = Vec::new();
    if !queryable {
        gaps.push("cross-region lineage is not queryable".to_string());
    }
    if !regional_boundary_preserved {
        gaps.push("consumer-region visibility exceeds the approved regional scope".to_string());
    }
    Ok(LineageReport {
        producer_region: simulation.record.producer_region.0,
        visible_consumer_regions: visible_consumer_regions
            .into_iter()
            .map(|region| region.0)
            .collect(),
        queryable,
        regional_boundary_preserved,
        gaps,
    })
}

fn consistency_class_name(class: &ConsistencyClass) -> &'static str {
    match class {
        ConsistencyClass::StronglyConsistent => "strongly-consistent",
        ConsistencyClass::RegionallyConsistent => "regionally-consistent",
        ConsistencyClass::EventuallyReplicated => "eventually-replicated",
    }
}

fn sovereignty_payload(simulation: &Path) -> Result<SovereigntyReport, ExitCode> {
    let simulation: SovereigntySimulation = load_json_file(simulation)?;
    let write_allowed =
        region_write_allowed(&simulation.write_rule, &simulation.requested_write_region);
    let consistency_class =
        classify_resource_consistency(&simulation.resource, &simulation.consistency_notes);
    let consistency_catalog = build_consistency_catalog(&simulation.consistency_notes);
    let mut gaps = Vec::new();
    if !write_allowed {
        gaps.push(
            "requested region is not allowed to perform writes for this resource".to_string(),
        );
    }
    if simulation.overlay.regulatory_profile.trim().is_empty() {
        gaps.push("region overlay is missing a regulatory profile".to_string());
    }
    if simulation.overlay.regulatory_profile != "unrestricted"
        && matches!(consistency_class, ConsistencyClass::EventuallyReplicated)
    {
        gaps.push(
            "regulated region cannot rely on eventually replicated writes for this resource"
                .to_string(),
        );
    }
    Ok(SovereigntyReport {
        write_allowed,
        consistency_class: consistency_class_name(&consistency_class).to_string(),
        consistency_catalog_size: consistency_catalog.len(),
        regulatory_profile: simulation.overlay.regulatory_profile,
        gaps,
    })
}

fn replay_payload(simulation: &Path) -> Result<ReplayReport, ExitCode> {
    let simulation: ReplaySimulation = load_json_file(simulation)?;
    let replay_safe =
        bijux_dag_runtime::simulated_platform::cross_domain_replay_safe(&simulation.safety);
    let warnings =
        replay_trust_warnings(&simulation.run_id, &simulation.baseline, &simulation.candidate);
    let mut gaps = Vec::new();
    if !replay_safe {
        gaps.push("cross-domain replay safety contract does not hold".to_string());
    }
    if !warnings.warnings.is_empty() {
        gaps.push("replay provenance changed across domains".to_string());
    }
    Ok(ReplayReport { warning_count: warnings.warnings.len(), warnings, replay_safe, gaps })
}

fn policy_distribution_payload(simulation: &Path) -> Result<PolicyDistributionReport, ExitCode> {
    let simulation: PolicyDistributionSimulation = load_json_file(simulation)?;
    let covered_regions =
        simulation.overlays.iter().map(|overlay| overlay.region.clone()).collect::<BTreeSet<_>>();
    let all_regions_covered = simulation.active_regions.is_subset(&covered_regions);
    let conformance_passed = federation_conformance_passes(&simulation.gate);
    let distinct_profiles = simulation
        .overlays
        .iter()
        .map(|overlay| {
            format!(
                "{}|{}|{}",
                overlay.regulatory_profile, overlay.cost_profile, overlay.infrastructure_profile
            )
        })
        .collect::<BTreeSet<_>>()
        .len();
    let mut gaps = Vec::new();
    if !all_regions_covered {
        gaps.push("active regions are missing policy overlays".to_string());
    }
    if !conformance_passed {
        gaps.push("federated policy distribution conformance gate is incomplete".to_string());
    }
    if simulation.overlays.iter().any(|overlay| {
        overlay.regulatory_profile.trim().is_empty()
            || overlay.cost_profile.trim().is_empty()
            || overlay.infrastructure_profile.trim().is_empty()
    }) {
        gaps.push("one or more region overlays are incomplete".to_string());
    }
    Ok(PolicyDistributionReport {
        all_regions_covered,
        conformance_passed,
        distinct_profiles,
        gaps,
    })
}

fn audit_integrity_payload(simulation: &Path) -> Result<AuditIntegrityReport, ExitCode> {
    let simulation: AuditIntegritySimulation = load_json_file(simulation)?;
    let audit_exchange_enabled = simulation.observability.exchange_audit_events;
    let metrics_exchange_enabled = simulation.observability.exchange_metrics;
    let integrity_passed = federation_conformance_passes(&simulation.gate)
        && audit_exchange_enabled
        && !simulation.observability.redaction_profile.trim().is_empty();
    let mut gaps = Vec::new();
    if !federation_conformance_passes(&simulation.gate) {
        gaps.push("federated audit conformance gate is incomplete".to_string());
    }
    if !audit_exchange_enabled {
        gaps.push("audit events are not exchanged across domains".to_string());
    }
    if simulation.observability.redaction_profile.trim().is_empty() {
        gaps.push("audit exchange is missing a redaction profile".to_string());
    }
    Ok(AuditIntegrityReport {
        audit_exchange_enabled,
        metrics_exchange_enabled,
        redaction_profile: simulation.observability.redaction_profile,
        integrity_passed,
        gaps,
    })
}

fn trust_tier_rank(value: &str) -> usize {
    match value {
        "bronze" => 1,
        "silver" => 2,
        "gold" => 3,
        "platinum" => 4,
        _ => 0,
    }
}

fn trust_tier_payload(simulation: &Path) -> Result<TrustTierReport, ExitCode> {
    let simulation: TrustTierSimulation = load_json_file(simulation)?;
    let domain_allowed = trust_tier_allows_domain(&simulation.rule, &simulation.domain.domain_id);
    let tier_sufficient = trust_tier_rank(&simulation.domain.trust_tier)
        >= trust_tier_rank(&simulation.rule.min_trust_tier);
    let mut gaps = Vec::new();
    if !domain_allowed {
        gaps.push("selected domain is not in the allowed trust-tier routing set".to_string());
    }
    if !tier_sufficient {
        gaps.push("selected domain trust tier is below the workflow minimum".to_string());
    }
    Ok(TrustTierReport {
        domain_allowed,
        tier_sufficient,
        selected_domain: simulation.domain.domain_id.0,
        gaps,
    })
}

fn delegation_action_name(action: &DelegationFailureAction) -> &'static str {
    match action {
        DelegationFailureAction::RetrySameDomain => "retry-same-domain",
        DelegationFailureAction::Reroute => "reroute",
        DelegationFailureAction::Quarantine => "quarantine",
    }
}

fn delegation_payload(simulation: &Path) -> Result<DelegationReport, ExitCode> {
    let simulation: DelegationSimulation = load_json_file(simulation)?;
    let delegation_allowed =
        delegation_allowed(&simulation.flow, simulation.inflight, simulation.per_minute);
    let target_domain_healthy = domain_healthy(&simulation.target_domain, &simulation.health);
    let failure_action =
        select_delegation_failure_action(&simulation.failure_policy, simulation.persistent_failure);
    let mut gaps = Vec::new();
    if !delegation_allowed {
        gaps.push("delegation exceeds configured inflight or rate limits".to_string());
    }
    if !target_domain_healthy {
        gaps.push("target domain is not healthy enough to receive delegation".to_string());
    }
    Ok(DelegationReport {
        delegation_allowed,
        target_domain_healthy,
        failure_action: delegation_action_name(&failure_action).to_string(),
        gaps,
    })
}

fn config_inheritance_payload(simulation: &Path) -> Result<ConfigInheritanceReport, ExitCode> {
    let simulation: ConfigInheritanceSimulation = load_json_file(simulation)?;
    let mut regional_defaults = simulation.global_defaults.clone();
    for (key, value) in simulation.region_values {
        regional_defaults.insert(key, value);
    }
    let mut merged = resolve_tenant_overlay(&regional_defaults, &simulation.tenant_overlay);
    let explicit_override_count = simulation.explicit_overrides.len();
    for (key, value) in simulation.explicit_overrides {
        merged.insert(key, value);
    }
    let review_required_keys_present =
        simulation.review_required_keys.iter().all(|key| merged.contains_key(key));
    let mut gaps = Vec::new();
    if simulation.region_overlay.regulatory_profile.trim().is_empty()
        || simulation.region_overlay.infrastructure_profile.trim().is_empty()
    {
        gaps.push("region overlay metadata is incomplete".to_string());
    }
    if !review_required_keys_present {
        gaps.push("merged configuration is missing review-required keys".to_string());
    }
    Ok(ConfigInheritanceReport {
        merged,
        review_required_keys_present,
        explicit_override_count,
        gaps,
    })
}

pub(crate) fn handle_federation_command(
    cli: &DagCli,
    command: &FederationCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        FederationCommands::Schedule { simulation } => {
            let payload = serde_json::to_value(schedule_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.schedule", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::Failover { simulation } => {
            let payload = serde_json::to_value(failover_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.failover", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::Lineage { simulation } => {
            let payload = serde_json::to_value(lineage_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.lineage", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::Sovereignty { simulation } => {
            let payload = serde_json::to_value(sovereignty_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.sovereignty",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        FederationCommands::Replay { simulation } => {
            let payload =
                serde_json::to_value(replay_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.replay", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::PolicyDistribution { simulation } => {
            let payload = serde_json::to_value(policy_distribution_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.policy-distribution",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        FederationCommands::AuditIntegrity { simulation } => {
            let payload = serde_json::to_value(audit_integrity_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.audit-integrity",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        FederationCommands::TrustTier { simulation } => {
            let payload = serde_json::to_value(trust_tier_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.trust-tier",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        FederationCommands::Delegation { simulation } => {
            let payload = serde_json::to_value(delegation_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.delegation",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        FederationCommands::ConfigInheritance { simulation } => {
            let payload = serde_json::to_value(config_inheritance_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.federation.config-inheritance",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_federation_command;
    use crate::commands::{Commands, DagCli, FederationCommands};
    use crate::ExitCode;

    fn quiet_json_cli(command: FederationCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Federation { command } }
    }

    #[test]
    fn federation_schedule_accepts_geo_ready_partitioned_regions() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("schedule.json");
        std::fs::write(
            &simulation,
            r#"{
              "activation":{"dag_name":"wf","version":"v1","global_visibility":true,"active_regions":["eu","us"]},
              "schedule_rules":[
                {"region":"eu","timezone":"Europe/Stockholm","failover_regions":["us"],"utc_anchor_required":true},
                {"region":"us","timezone":"America/New_York","failover_regions":["eu"],"utc_anchor_required":true}
              ],
              "affinity":{
                "dag_regions":["eu","us"],
                "run_regions":["eu","us"],
                "artifact_regions":["eu","us"],
                "tenant_regions":["eu","us"]
              },
              "queue_partitions":[
                {"region":"eu","queue_name":"eu-main","shared_with_regions":[]},
                {"region":"us","queue_name":"us-main","shared_with_regions":[]}
              ],
              "gate":{"registry_ready":true,"scheduler_ready":true,"lineage_ready":true,"observability_ready":true}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::Schedule { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Schedule { simulation: simulation.clone() },
        )
        .expect("schedule");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::schedule_payload(&simulation).expect("report");
        assert!(report.geo_ready);
        assert!(report.affinity_preserved);
        assert!(report.schedule_rules_complete);
        assert!(report.queue_partitioned);
        assert_eq!(report.selected_regions, vec!["eu".to_string(), "us".to_string()]);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_schedule_flags_missing_gate_and_partition_gaps() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("schedule.json");
        std::fs::write(
            &simulation,
            r#"{
              "activation":{"dag_name":"wf","version":"v1","global_visibility":false,"active_regions":["eu","us"]},
              "schedule_rules":[
                {"region":"eu","timezone":"","failover_regions":[],"utc_anchor_required":true}
              ],
              "affinity":{
                "dag_regions":["eu","us"],
                "run_regions":["eu"],
                "artifact_regions":["us"],
                "tenant_regions":["eu","us"]
              },
              "queue_partitions":[
                {"region":"eu","queue_name":"eu-main","shared_with_regions":[]}
              ],
              "gate":{"registry_ready":true,"scheduler_ready":false,"lineage_ready":true,"observability_ready":false}
            }"#,
        )
        .expect("write simulation");
        let report = super::schedule_payload(&simulation).expect("report");
        assert!(!report.geo_ready);
        assert!(!report.affinity_preserved);
        assert!(!report.schedule_rules_complete);
        assert!(!report.queue_partitioned);
        for expected in [
            "geo readiness gate is incomplete",
            "no common region satisfies dag, run, artifact, and tenant affinity",
            "multi-region schedule rules are incomplete for selected regions",
            "selected regions are missing dedicated queue partitions",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_failover_accepts_documented_secondary_region() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("failover.json");
        std::fs::write(
            &simulation,
            r#"{
              "rule":{"service":"scheduler","primary_region":"eu","secondary_regions":["us"],"max_failover_seconds":120},
              "playbook":{"region":"eu","control_plane_outage_steps":["freeze-writes","promote-secondary"],"artifact_store_outage_steps":["switch-read-replica"]},
              "scenario":{"name":"loss-of-eu","replication_lag_seconds":5,"region_loss":"eu","delayed_failover_seconds":90}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::Failover { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Failover { simulation: simulation.clone() },
        )
        .expect("failover");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::failover_payload(&simulation).expect("report");
        assert_eq!(report.target_region.as_deref(), Some("us"));
        assert!(report.within_rto);
        assert!(report.playbook_complete);
        assert!(report.scenario_covered);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_failover_flags_missing_secondary_and_incomplete_playbook() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("failover.json");
        std::fs::write(
            &simulation,
            r#"{
              "rule":{"service":"scheduler","primary_region":"eu","secondary_regions":[],"max_failover_seconds":60},
              "playbook":{"region":"us","control_plane_outage_steps":[],"artifact_store_outage_steps":[]},
              "scenario":{"name":"loss-of-us","replication_lag_seconds":30,"region_loss":"us","delayed_failover_seconds":180}
            }"#,
        )
        .expect("write simulation");
        let report = super::failover_payload(&simulation).expect("report");
        for expected in [
            "no secondary region is configured for failover",
            "failover exceeds the configured recovery time objective",
            "disaster recovery playbook is incomplete for the primary region",
            "simulation does not cover primary-region loss",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_lineage_accepts_queryable_scoped_record() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("lineage.json");
        std::fs::write(
            &simulation,
            r#"{
              "record":{
                "artifact_id":"a1",
                "producer_region":"eu",
                "consumer_regions":["eu","us"],
                "lineage_queryable":true
              },
              "allowed_regions":["eu","us"]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::Lineage { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Lineage { simulation: simulation.clone() },
        )
        .expect("lineage");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::lineage_payload(&simulation).expect("report");
        assert!(report.queryable);
        assert!(report.regional_boundary_preserved);
        assert_eq!(report.visible_consumer_regions, vec!["eu".to_string(), "us".to_string()]);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_lineage_flags_hidden_or_out_of_scope_consumers() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("lineage.json");
        std::fs::write(
            &simulation,
            r#"{
              "record":{
                "artifact_id":"a1",
                "producer_region":"eu",
                "consumer_regions":["eu","us"],
                "lineage_queryable":false
              },
              "allowed_regions":["eu"]
            }"#,
        )
        .expect("write simulation");
        let report = super::lineage_payload(&simulation).expect("report");
        assert!(!report.queryable);
        assert!(!report.regional_boundary_preserved);
        for expected in [
            "cross-region lineage is not queryable",
            "consumer-region visibility exceeds the approved regional scope",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_sovereignty_accepts_region_bound_strong_write_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("sovereignty.json");
        std::fs::write(
            &simulation,
            r#"{
              "resource":"artifact-registry",
              "write_rule":{"resource":"artifact-registry","global_visible":false,"write_regions":["eu"]},
              "requested_write_region":"eu",
              "consistency_notes":[{"resource":"artifact-registry","class":"StronglyConsistent","rationale":"regulated metadata"}],
              "overlay":{"region":"eu","regulatory_profile":"gdpr","cost_profile":"premium","infrastructure_profile":"primary"}
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(FederationCommands::Sovereignty { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Sovereignty { simulation: simulation.clone() },
        )
        .expect("sovereignty");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::sovereignty_payload(&simulation).expect("report");
        assert!(report.write_allowed);
        assert_eq!(report.consistency_class, "strongly-consistent");
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_sovereignty_rejects_out_of_region_eventual_write_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("sovereignty.json");
        std::fs::write(
            &simulation,
            r#"{
              "resource":"artifact-registry",
              "write_rule":{"resource":"artifact-registry","global_visible":true,"write_regions":["us"]},
              "requested_write_region":"eu",
              "consistency_notes":[{"resource":"artifact-registry","class":"EventuallyReplicated","rationale":"cheap async mirror"}],
              "overlay":{"region":"eu","regulatory_profile":"gdpr","cost_profile":"cheap","infrastructure_profile":"secondary"}
            }"#,
        )
        .expect("write simulation");
        let report = super::sovereignty_payload(&simulation).expect("report");
        for expected in [
            "requested region is not allowed to perform writes for this resource",
            "regulated region cannot rely on eventually replicated writes for this resource",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_replay_accepts_stable_cross_domain_provenance() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("replay.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-1",
              "safety":{"artifact_compatible":true,"policy_compatible":true,"backend_compatible":true},
              "baseline":{
                "run_id":"run-1",
                "dag_snapshot_id":"dag-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"bundle-v1",
                "output_digests":["sha256:o1"],
                "binaries":[{"component":"Scheduler","version":"1.0.0","build_id":"build-a","source_revision":"rev-a","build_timestamp_utc":"2026-04-28T00:00:00Z"}],
                "plugins":[{"plugin_name":"builtin-python","version":"1.0.0","source":"registry","trust_tier":"Official","approved":true}],
                "environment":{"backend":"kubernetes","capability_class":"standard","trust_domain":"prod-eu"}
              },
              "candidate":{
                "run_id":"run-1",
                "dag_snapshot_id":"dag-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"bundle-v1",
                "output_digests":["sha256:o1"],
                "binaries":[{"component":"Scheduler","version":"1.0.0","build_id":"build-a","source_revision":"rev-a","build_timestamp_utc":"2026-04-28T00:00:00Z"}],
                "plugins":[{"plugin_name":"builtin-python","version":"1.0.0","source":"registry","trust_tier":"Official","approved":true}],
                "environment":{"backend":"kubernetes","capability_class":"standard","trust_domain":"prod-eu"}
              }
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::Replay { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Replay { simulation: simulation.clone() },
        )
        .expect("replay");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::replay_payload(&simulation).expect("report");
        assert!(report.replay_safe);
        assert_eq!(report.warning_count, 0);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_replay_flags_cross_domain_provenance_drift() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("replay.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-1",
              "safety":{"artifact_compatible":true,"policy_compatible":false,"backend_compatible":true},
              "baseline":{
                "run_id":"run-1",
                "dag_snapshot_id":"dag-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"bundle-v1",
                "output_digests":["sha256:o1"],
                "binaries":[{"component":"Scheduler","version":"1.0.0","build_id":"build-a","source_revision":"rev-a","build_timestamp_utc":"2026-04-28T00:00:00Z"}],
                "plugins":[{"plugin_name":"builtin-python","version":"1.0.0","source":"registry","trust_tier":"Official","approved":true}],
                "environment":{"backend":"kubernetes","capability_class":"standard","trust_domain":"prod-eu"}
              },
              "candidate":{
                "run_id":"run-1",
                "dag_snapshot_id":"dag-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"bundle-v2",
                "output_digests":["sha256:o2"],
                "binaries":[{"component":"Scheduler","version":"1.1.0","build_id":"build-b","source_revision":"rev-b","build_timestamp_utc":"2026-04-29T00:00:00Z"}],
                "plugins":[{"plugin_name":"builtin-python","version":"1.0.0","source":"registry","trust_tier":"Official","approved":false}],
                "environment":{"backend":"kubernetes","capability_class":"standard","trust_domain":"prod-us"}
              }
            }"#,
        )
        .expect("write simulation");
        let report = super::replay_payload(&simulation).expect("report");
        assert!(!report.replay_safe);
        assert!(report.warning_count > 0);
        for expected in [
            "cross-domain replay safety contract does not hold",
            "replay provenance changed across domains",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_policy_distribution_accepts_complete_region_overlay_set() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("policy.json");
        std::fs::write(
            &simulation,
            r#"{
              "active_regions":["eu","us"],
              "overlays":[
                {"region":"eu","regulatory_profile":"gdpr","cost_profile":"premium","infrastructure_profile":"primary"},
                {"region":"us","regulatory_profile":"ccpa","cost_profile":"standard","infrastructure_profile":"secondary"}
              ],
              "gate":{"lineage_auditable":true,"routing_deterministic":true,"audit_events_complete":true}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::PolicyDistribution {
            simulation: simulation.clone(),
        });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::PolicyDistribution { simulation: simulation.clone() },
        )
        .expect("policy distribution");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::policy_distribution_payload(&simulation).expect("report");
        assert!(report.all_regions_covered);
        assert!(report.conformance_passed);
        assert_eq!(report.distinct_profiles, 2);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_policy_distribution_flags_missing_or_incomplete_overlays() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("policy.json");
        std::fs::write(
            &simulation,
            r#"{
              "active_regions":["eu","us"],
              "overlays":[
                {"region":"eu","regulatory_profile":"","cost_profile":"premium","infrastructure_profile":"primary"}
              ],
              "gate":{"lineage_auditable":true,"routing_deterministic":false,"audit_events_complete":false}
            }"#,
        )
        .expect("write simulation");
        let report = super::policy_distribution_payload(&simulation).expect("report");
        for expected in [
            "active regions are missing policy overlays",
            "federated policy distribution conformance gate is incomplete",
            "one or more region overlays are incomplete",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_audit_integrity_accepts_complete_cross_domain_audit_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("audit.json");
        std::fs::write(
            &simulation,
            r#"{
              "gate":{"lineage_auditable":true,"routing_deterministic":true,"audit_events_complete":true},
              "observability":{"exchange_metrics":true,"exchange_audit_events":true,"redaction_profile":"tenant-safe"}
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(FederationCommands::AuditIntegrity { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::AuditIntegrity { simulation: simulation.clone() },
        )
        .expect("audit integrity");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::audit_integrity_payload(&simulation).expect("report");
        assert!(report.integrity_passed);
        assert!(report.audit_exchange_enabled);
        assert_eq!(report.redaction_profile, "tenant-safe");
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_audit_integrity_flags_missing_audit_exchange() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("audit.json");
        std::fs::write(
            &simulation,
            r#"{
              "gate":{"lineage_auditable":true,"routing_deterministic":false,"audit_events_complete":false},
              "observability":{"exchange_metrics":true,"exchange_audit_events":false,"redaction_profile":""}
            }"#,
        )
        .expect("write simulation");
        let report = super::audit_integrity_payload(&simulation).expect("report");
        for expected in [
            "federated audit conformance gate is incomplete",
            "audit events are not exchanged across domains",
            "audit exchange is missing a redaction profile",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_trust_tier_accepts_allowed_high_assurance_domain() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("trust-tier.json");
        std::fs::write(
            &simulation,
            r#"{
              "rule":{"min_trust_tier":"gold","allowed_domains":["domain-eu","domain-us"]},
              "domain":{"domain_id":"domain-eu","trust_tier":"platinum","issuer":"oidc-prod"}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::TrustTier { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::TrustTier { simulation: simulation.clone() },
        )
        .expect("trust tier");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::trust_tier_payload(&simulation).expect("report");
        assert!(report.domain_allowed);
        assert!(report.tier_sufficient);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_trust_tier_flags_unapproved_or_weak_domain() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("trust-tier.json");
        std::fs::write(
            &simulation,
            r#"{
              "rule":{"min_trust_tier":"gold","allowed_domains":["domain-eu"]},
              "domain":{"domain_id":"domain-us","trust_tier":"silver","issuer":"oidc-dev"}
            }"#,
        )
        .expect("write simulation");
        let report = super::trust_tier_payload(&simulation).expect("report");
        for expected in [
            "selected domain is not in the allowed trust-tier routing set",
            "selected domain trust tier is below the workflow minimum",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_delegation_accepts_healthy_target_within_limits() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("delegation.json");
        std::fs::write(
            &simulation,
            r#"{
              "flow":{"source_domain":"domain-eu","target_domain":"domain-us","max_inflight_delegations":5,"max_delegations_per_minute":20},
              "target_domain":"domain-us",
              "health":[{"domain_id":"domain-us","healthy":true,"impairment_reason":null}],
              "inflight":2,
              "per_minute":4,
              "failure_policy":{"transient_action":"RetrySameDomain","persistent_action":"Reroute"},
              "persistent_failure":false
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::Delegation { simulation: simulation.clone() });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::Delegation { simulation: simulation.clone() },
        )
        .expect("delegation");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::delegation_payload(&simulation).expect("report");
        assert!(report.delegation_allowed);
        assert!(report.target_domain_healthy);
        assert_eq!(report.failure_action, "retry-same-domain");
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_delegation_flags_unhealthy_or_rate_limited_target() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("delegation.json");
        std::fs::write(
            &simulation,
            r#"{
              "flow":{"source_domain":"domain-eu","target_domain":"domain-us","max_inflight_delegations":1,"max_delegations_per_minute":2},
              "target_domain":"domain-us",
              "health":[{"domain_id":"domain-us","healthy":false,"impairment_reason":"storage-pressure"}],
              "inflight":2,
              "per_minute":3,
              "failure_policy":{"transient_action":"RetrySameDomain","persistent_action":"Quarantine"},
              "persistent_failure":true
            }"#,
        )
        .expect("write simulation");
        let report = super::delegation_payload(&simulation).expect("report");
        assert_eq!(report.failure_action, "quarantine");
        for expected in [
            "delegation exceeds configured inflight or rate limits",
            "target domain is not healthy enough to receive delegation",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn federation_config_inheritance_merges_global_region_tenant_and_explicit_layers() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("config.json");
        std::fs::write(
            &simulation,
            r#"{
              "global_defaults":{"retry":"2","queue":"global","region":"global"},
              "region_overlay":{"region":"eu","regulatory_profile":"gdpr","cost_profile":"premium","infrastructure_profile":"primary"},
              "region_values":{"queue":"eu-main","region":"eu"},
              "tenant_overlay":{"tenant_id":"atlas","values":{"retention":"30d"},"overrides":{"retry":"5"}},
              "explicit_overrides":{"queue":"eu-priority"},
              "review_required_keys":["retry","queue","retention","region"]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(FederationCommands::ConfigInheritance {
            simulation: simulation.clone(),
        });
        let code = handle_federation_command(
            &cli,
            &FederationCommands::ConfigInheritance { simulation: simulation.clone() },
        )
        .expect("config inheritance");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::config_inheritance_payload(&simulation).expect("report");
        assert!(report.review_required_keys_present);
        assert_eq!(report.explicit_override_count, 1);
        assert_eq!(report.merged.get("retry").map(String::as_str), Some("5"));
        assert_eq!(report.merged.get("queue").map(String::as_str), Some("eu-priority"));
        assert_eq!(report.merged.get("region").map(String::as_str), Some("eu"));
        assert_eq!(report.merged.get("retention").map(String::as_str), Some("30d"));
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn federation_config_inheritance_flags_missing_review_keys_or_overlay_metadata() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("config.json");
        std::fs::write(
            &simulation,
            r#"{
              "global_defaults":{"retry":"2"},
              "region_overlay":{"region":"eu","regulatory_profile":"","cost_profile":"premium","infrastructure_profile":""},
              "region_values":{},
              "tenant_overlay":{"tenant_id":"atlas","values":{},"overrides":{}},
              "explicit_overrides":{},
              "review_required_keys":["retry","queue"]
            }"#,
        )
        .expect("write simulation");
        let report = super::config_inheritance_payload(&simulation).expect("report");
        for expected in [
            "region overlay metadata is incomplete",
            "merged configuration is missing review-required keys",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }
}
