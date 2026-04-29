use crate::commands::{AdaptersCommands, DagCli};
use crate::routes::preconditions::require_file;
use crate::{check_engine, emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_runtime::{
    adapter_admission_matrix, adapter_registry_dump, probe_external_adapters,
    registered_adapter_descriptors, registered_adapters,
};
use serde_json::json;

pub(crate) fn handle_adapters_command(
    cli: &DagCli,
    command: &AdaptersCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        AdaptersCommands::Ls => {
            let adapters = registered_adapters();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.adapters.ls",
                    true,
                    json!(adapters),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for adapter in adapters {
                println!(
                    "{} {} effects={:?}",
                    adapter.adapter_id, adapter.adapter_version, adapter.effects
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        AdaptersCommands::Dump => {
            let data = json!({
                "registry": adapter_registry_dump(),
                "descriptors": registered_adapter_descriptors(),
                "external_handshakes": probe_external_adapters().map_err(|_| ExitCode::from(3))?,
            });
            if cli.json {
                return emit_json(
                    cli,
                    "dag.adapters.dump",
                    true,
                    data,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&data).map_err(|_| ExitCode::from(3))?);
            Ok(ExitCode::SUCCESS)
        }
        AdaptersCommands::Describe => {
            let descriptors = registered_adapter_descriptors();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.adapters.describe",
                    true,
                    json!({ "descriptors": descriptors }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for descriptor in descriptors {
                println!(
                    "{} {} kinds={:?} effects={:?} timeout={} cancel={} cache={:?}",
                    descriptor.id,
                    descriptor.version,
                    descriptor.supported_kinds,
                    descriptor.required_effects,
                    descriptor.supports_timeout,
                    descriptor.supports_cancel,
                    descriptor.cache_compatibility,
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        AdaptersCommands::Admit { dag } => {
            require_file(dag)?;
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let report = adapter_admission_matrix(&graph);
            let code = if report.supported { ExitCode::SUCCESS } else { ExitCode::from(3) };
            if cli.json {
                return emit_json(
                    cli,
                    "dag.adapters.admit",
                    report.supported,
                    json!(report),
                    Vec::new(),
                    code,
                );
            }
            for entry in &report.entries {
                if entry.supported {
                    println!("ok {} kind={}", entry.node_id, entry.node_kind);
                } else {
                    println!(
                        "unsupported {} kind={} reasons={}",
                        entry.node_id,
                        entry.node_kind,
                        entry.reasons.join("; ")
                    );
                }
            }
            if report.supported {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        AdaptersCommands::Doctor => {
            let docker = check_engine("docker");
            let podman = check_engine("podman");
            let handshakes = probe_external_adapters().map_err(|_| ExitCode::from(3))?;
            let descriptors = registered_adapter_descriptors();
            let ok = (docker.get("status").and_then(|v| v.as_str()) == Some("ok")
                || podman.get("status").and_then(|v| v.as_str()) == Some("ok"))
                && handshakes.iter().all(|report| {
                    report.status == bijux_dag_runtime::ExternalAdapterHandshakeStatus::Ok
                });
            let payload = json!({
                "docker": docker,
                "podman": podman,
                "descriptors": descriptors,
                "external_handshakes": handshakes,
            });
            if cli.json {
                return emit_json(
                    cli,
                    "dag.adapters.doctor",
                    ok,
                    payload,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("docker: {}", payload["docker"]["status"]);
            if let Some(version) = payload["docker"].get("version").and_then(|value| value.as_str())
            {
                println!("docker_version: {}", version);
            }
            println!("podman: {}", payload["podman"]["status"]);
            if let Some(version) = payload["podman"].get("version").and_then(|value| value.as_str())
            {
                println!("podman_version: {}", version);
            }
            for report in payload["external_handshakes"].as_array().into_iter().flatten() {
                println!(
                    "handshake {} status={}",
                    report["path"].as_str().unwrap_or("<unknown>"),
                    report["status"].as_str().unwrap_or("unknown")
                );
                if let Some(reason) = report.get("reason").and_then(|value| value.as_str()) {
                    println!("reason: {}", reason);
                }
            }
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
    use super::handle_adapters_command;
    use crate::commands::{AdaptersCommands, Commands, DagCli};
    use std::fs;

    fn cli(json: bool) -> DagCli {
        DagCli {
            json,
            quiet: true,
            command: Commands::Adapters { command: AdaptersCommands::Describe },
        }
    }

    #[test]
    fn adapter_describe_json_exposes_descriptor_contract_fields() {
        let result = handle_adapters_command(&cli(true), &AdaptersCommands::Describe);
        assert!(result.is_ok());
    }

    #[test]
    fn adapter_admit_reports_missing_adapter_kind() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"x","kind":"missing.kind","outputs":[{"name":"out","path":"out"}],"params":{}}],
              "edges":[]
            }"#,
        )
        .expect("write graph");
        let result = handle_adapters_command(&cli(true), &AdaptersCommands::Admit { dag });
        assert!(result.is_err());
    }
}
