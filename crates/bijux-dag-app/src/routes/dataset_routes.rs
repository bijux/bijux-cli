use crate::commands::{DagCli, DatasetCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    dataset_consumption_satisfied, dataset_mapping_index, DatasetArtifactMapping,
    DatasetConsumptionContract, DatasetFreshnessPolicy, DatasetVersionId,
};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct MappingSimulation {
    mappings: Vec<DatasetArtifactMapping>,
}

#[derive(Debug, Deserialize)]
struct StalenessSimulation {
    contract: DatasetConsumptionContract,
    available_version: DatasetVersionId,
    approved_latest: DatasetVersionId,
    freshness_minutes: u32,
    freshness_policy: DatasetFreshnessPolicy,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

pub(crate) fn handle_dataset_command(
    cli: &DagCli,
    command: &DatasetCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        DatasetCommands::Mapping { simulation } => {
            let simulation: MappingSimulation = parse_json_file(simulation)?;
            let index = dataset_mapping_index(&simulation.mappings);
            let index_entries = index
                .into_iter()
                .map(|((dataset_id, version_id), artifact_ids)| {
                    json!({
                        "dataset_id": dataset_id.0,
                        "version_id": version_id.0,
                        "artifact_ids": artifact_ids,
                    })
                })
                .collect::<Vec<_>>();
            let payload = json!({
                "mapping_count": simulation.mappings.len(),
                "dataset_version_count": index_entries.len(),
                "index": index_entries,
            });
            if cli.json {
                return emit_json(
                    cli,
                    "dag.dataset.mapping",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        DatasetCommands::Staleness { simulation } => {
            let simulation: StalenessSimulation = parse_json_file(simulation)?;
            let consumption_satisfied = dataset_consumption_satisfied(
                &simulation.contract,
                &simulation.available_version,
                &simulation.approved_latest,
                simulation.freshness_minutes,
            );
            let stale = simulation.freshness_minutes > simulation.freshness_policy.max_age_minutes;
            let payload = json!({
                "dataset_id": simulation.contract.dataset_id.0,
                "available_version": simulation.available_version.0,
                "approved_latest": simulation.approved_latest.0,
                "freshness_minutes": simulation.freshness_minutes,
                "consumption_satisfied": consumption_satisfied,
                "stale": stale,
                "staleness_action": simulation.freshness_policy.staleness_action,
                "max_age_minutes": simulation.freshness_policy.max_age_minutes,
            });
            let ok = consumption_satisfied && !stale;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.dataset.staleness",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"dataset_staleness_gate_failed",
                            "severity":"error",
                            "message":"dataset freshness or consumption contract is not satisfied",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_dataset_command;
    use crate::commands::{DagCli, DatasetCommands};
    use crate::ExitCode;
    use clap::Parser;

    fn quiet_json_cli(command: DatasetCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: crate::commands::Commands::Dataset { command } }
    }

    #[test]
    fn dataset_mapping_route_builds_dataset_version_index() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(
            dir.path().join("mapping.json"),
            r#"{
              "mappings": [
                {
                  "dataset_id": "sales",
                  "version_id": "v1",
                  "artifact_ids": ["extract:raw.csv", "report:weekly.json"]
                },
                {
                  "dataset_id": "sales",
                  "version_id": "v2",
                  "artifact_ids": ["extract:raw-v2.csv"]
                }
              ]
            }"#,
        )
        .expect("mapping");
        let cli = quiet_json_cli(DatasetCommands::Mapping {
            simulation: dir.path().join("mapping.json"),
        });
        let code = handle_dataset_command(
            &cli,
            &DatasetCommands::Mapping { simulation: dir.path().join("mapping.json") },
        )
        .expect("mapping");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn dataset_staleness_route_fails_when_freshness_budget_is_exceeded() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(
            dir.path().join("staleness.json"),
            r#"{
              "contract": {
                "dataset_id": "sales",
                "mode": {"FreshnessBounded": 30}
              },
              "available_version": "v1",
              "approved_latest": "v1",
              "freshness_minutes": 45,
              "freshness_policy": {
                "max_age_minutes": 30,
                "staleness_action": "block"
              }
            }"#,
        )
        .expect("staleness");
        let cli = quiet_json_cli(DatasetCommands::Staleness {
            simulation: dir.path().join("staleness.json"),
        });
        let exit = handle_dataset_command(
            &cli,
            &DatasetCommands::Staleness { simulation: dir.path().join("staleness.json") },
        )
        .expect_err("should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn dataset_routes_reject_missing_simulation_without_panic() {
        let cli =
            DagCli::parse_from(["bijux-dag", "--json", "dataset", "mapping", "/missing/file.json"]);
        let result = std::panic::catch_unwind(|| {
            let _ = handle_dataset_command(
                &cli,
                &DatasetCommands::Mapping { simulation: "/missing/file.json".into() },
            );
        });
        assert!(result.is_ok());
    }
}
