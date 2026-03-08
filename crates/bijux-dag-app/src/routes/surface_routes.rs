use crate::capability_matrix::backend_capability_payload;
use crate::commands::DagCli;
use crate::{emit_json, ExitCode};
use bijux_dag_core::SPEC_VERSION;
use serde_json::json;

pub(crate) fn handle_capabilities_command(
    cli: &DagCli,
    backend: &Option<String>,
) -> Result<ExitCode, ExitCode> {
    let payload = json!({
        "format": "capabilities/v1",
        "binary_version": env!("CARGO_PKG_VERSION"),
        "graph_schema_version": SPEC_VERSION,
        "surfaces": {
            "cli": {"status": "supported"},
            "run_directory": {"status": "supported"},
            "export_bundle": {"status": "supported"},
            "library_crates": {"status": "experimental"}
        },
        "execution_modes": {
            "local_process": "implemented",
            "container": "simulated",
            "remote": "simulated",
            "batch_hpc": "simulated"
        },
        "backend_capability_matrix": [
            backend_capability_payload("kubernetes").unwrap(),
            backend_capability_payload("hpc").unwrap(),
            backend_capability_payload("remote").unwrap()
        ],
        "operator_commands": [
            "runs.list","runs.show","runs.inspect","runs.history","runs.id-explain","runs.tree","runs.timeline","runs.diff","runs.verify","runs.doctor","runs.explain-failure","artifact-inspect","trace-artifact","hash.run","hash.artifact","why-rerun","why-cache-missed","fsck"
        ]
    });
    let payload = if let Some(name) = backend.as_deref() {
        match backend_capability_payload(name) {
            Some(entry) => entry,
            None => {
                return emit_json(
                    cli,
                    "dag.capabilities",
                    false,
                    json!({
                        "format": "capabilities/v1",
                        "backend": name,
                        "status": "unsupported-backend-query"
                    }),
                    vec![json!({
                        "message": format!("unsupported backend query: {name}"),
                        "remediation": "use --backend kubernetes, --backend hpc, or --backend remote"
                    })],
                    ExitCode::from(2),
                );
            }
        }
    } else {
        payload
    };
    if cli.json {
        return emit_json(
            cli,
            "dag.capabilities",
            true,
            payload,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_semantic_portability_command(
    cli: &DagCli,
    backend: &str,
) -> Result<ExitCode, ExitCode> {
    let capability = backend_capability_payload(backend);
    let supported = capability.is_some();
    let payload = if let Some(capability) = capability {
        json!({
            "format": "semantic-portability/v1",
            "backend": capability["backend"],
            "status": "fidelity-preserving",
            "equivalence_class": "contract-equivalent",
            "downgrade_conditions": [
                "missing artifacts",
                "environment fingerprint drift",
                "backend-specific unsupported requirement"
            ],
            "capability_reference": capability
        })
    } else {
        json!({
            "format": "semantic-portability/v1",
            "backend": backend,
            "status": "downgraded",
            "equivalence_class": "unsupported-backend-query",
            "downgrade_conditions": ["unsupported backend target"]
        })
    };
    if cli.json {
        return emit_json(
            cli,
            "dag.semantic-portability",
            supported,
            payload,
            if supported {
                Vec::new()
            } else {
                vec![
                    json!({"message":"unsupported backend target","remediation":"use --backend kubernetes, --backend hpc, or --backend remote"}),
                ]
            },
            if supported {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            },
        );
    }
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(if supported {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}

#[cfg(test)]
mod tests {
    use super::{handle_capabilities_command, handle_semantic_portability_command};
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use std::path::PathBuf;

    fn quiet_json_cli() -> DagCli {
        DagCli {
            json: true,
            quiet: true,
            command: Commands::Version,
        }
    }

    #[test]
    fn unsupported_backend_query_returns_compatibility_exit() {
        let cli = quiet_json_cli();
        let code = handle_capabilities_command(&cli, &Some("unknown".to_string())).unwrap_err();
        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn semantic_portability_unknown_backend_is_rejected() {
        let cli = quiet_json_cli();
        let code = handle_semantic_portability_command(&cli, "unknown").unwrap_err();
        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn semantic_portability_known_backend_is_supported() {
        let cli = DagCli {
            json: true,
            quiet: true,
            command: Commands::Fsck {
                run_dir: PathBuf::from("."),
                strict: false,
            },
        };
        let code = handle_semantic_portability_command(&cli, "hpc").unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
