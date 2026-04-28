use crate::commands::{DagCli, FederationCommands};
use crate::{emit_json, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    geo_ready, GeoReadyAcceptanceGate, RegionAffinityPolicy, RegionAwareDagActivation,
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

pub(crate) fn handle_federation_command(
    cli: &DagCli,
    command: &FederationCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        FederationCommands::Schedule { simulation } => {
            let payload = serde_json::to_value(schedule_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.federation.schedule", true, payload, Vec::new(), ExitCode::SUCCESS)
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
}
