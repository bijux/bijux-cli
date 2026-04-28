use crate::commands::{DagCli, FederationCommands};
use crate::{emit_json, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    geo_ready, CrossRegionFailoverRule, DisasterRecoveryPlaybook, GeoReadyAcceptanceGate,
    GeoSimulationScenario, RegionAffinityPolicy, RegionAwareDagActivation, RegionLineageRecord,
    RegionQueuePartition, RegionScheduleRule,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
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
    let schedule_rules_complete = affinity_preserved && selected.iter().all(|region| {
        simulation.schedule_rules.iter().any(|rule| {
            &rule.region == region && !rule.timezone.trim().is_empty() && (!rule.utc_anchor_required || !rule.failover_regions.is_empty())
        })
    });
    let queue_partitioned = affinity_preserved && selected.iter().all(|region| {
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
    let within_rto = simulation.scenario.delayed_failover_seconds <= simulation.rule.max_failover_seconds;
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
        visible_consumer_regions: visible_consumer_regions.into_iter().map(|region| region.0).collect(),
        queryable,
        regional_boundary_preserved,
        gaps,
    })
}

pub(crate) fn handle_federation_command(
    cli: &DagCli,
    command: &FederationCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        FederationCommands::Schedule { simulation } => {
            let payload = serde_json::to_value(schedule_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.schedule", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::Failover { simulation } => {
            let payload = serde_json::to_value(failover_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.failover", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        FederationCommands::Lineage { simulation } => {
            let payload = serde_json::to_value(lineage_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.lineage", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        _ => emit_json(
            cli,
            "dag.federation",
            false,
            json!({"status":"not-yet-implemented"}),
            vec![json!({"message":"federation surface not yet implemented for this command in the current commit boundary"})],
            ExitCode::from(2),
        ),
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
}
