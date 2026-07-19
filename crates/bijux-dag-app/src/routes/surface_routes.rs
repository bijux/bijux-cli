use crate::backend_capability_surface::backend_capability_payload;
use crate::commands::DagCli;
use crate::replay_service;
use crate::{emit_json, ExitCode};
use bijux_dag_core::SPEC_VERSION;
use serde_json::json;

fn operator_commands() -> Vec<&'static str> {
    vec![
        "runs.list",
        "runs.show",
        "runs.inspect",
        "runs.history",
        "runs.stop",
        "runs.id-explain",
        "runs.tree",
        "runs.timeline",
        "runs.scheduler-checkpoint",
        "runs.diff",
        "runs.verify",
        "runs.doctor",
        "runs.explain-failure",
        "runtime.isolation",
        "runtime.dispatch",
        "runtime.state",
        "runtime.write-discipline",
        "runtime.worker-recovery",
        "runtime.control-recovery",
        "runtime.repair",
        "runtime.retry",
        "runtime.timeout",
        "runtime.heartbeat",
        "runtime.cancel",
        "runtime.pause",
        "runtime.intervention",
        "runtime.transition",
        "runtime.events",
        "artifact-inspect",
        "artifact.registry",
        "artifact.lineage",
        "artifact.retention",
        "dataset.mapping",
        "dataset.staleness",
        "enterprise.webhook",
        "enterprise.queue",
        "enterprise.service-contract",
        "enterprise.incident-hook",
        "enterprise.asset-link",
        "enterprise.calendar",
        "enterprise.approval",
        "enterprise.dependency-catalog",
        "enterprise.credentials",
        "enterprise.export",
        "control-plane.api",
        "control-plane.leadership",
        "control-plane.planning",
        "control-plane.sharding",
        "control-plane.leases",
        "control-plane.idempotency",
        "control-plane.backpressure",
        "control-plane.cache",
        "control-plane.migration",
        "control-plane.fan-in",
        "state-store.transaction",
        "state-store.journal",
        "state-store.snapshot",
        "state-store.index",
        "state-store.archive",
        "state-store.checksum",
        "state-store.amplification",
        "state-store.retention",
        "state-store.consistency",
        "state-store.clock",
        "fleet.register",
        "fleet.capabilities",
        "fleet.drain",
        "fleet.autoscale",
        "fleet.warm-pool",
        "fleet.isolation",
        "fleet.preemption",
        "fleet.trust",
        "fleet.gossip",
        "fleet.fragmentation",
        "governance.contracts",
        "governance.ownership",
        "governance.tags",
        "governance.cost",
        "governance.alerts",
        "governance.policy-check",
        "governance.catalog-export",
        "governance.audit-event",
        "governance.promotion",
        "governance.compliance",
        "incident.mode",
        "incident.blast-radius",
        "incident.safe-stop",
        "incident.degraded-mode",
        "incident.annotation",
        "incident.repair-window",
        "incident.timeline",
        "incident.replay-validation",
        "incident.readiness-review",
        "incident.scorecard",
        "federation.schedule",
        "federation.failover",
        "federation.lineage",
        "federation.sovereignty",
        "federation.replay",
        "federation.policy-distribution",
        "federation.audit-integrity",
        "federation.trust-tier",
        "federation.delegation",
        "federation.config-inheritance",
        "security.auth",
        "security.authz",
        "security.tenant",
        "security.secrets",
        "security.supply-chain",
        "security.data-access",
        "security.override",
        "security.safe-defaults",
        "release.version",
        "release.promotion",
        "release.deprecation",
        "release.checkpoint",
        "release.shadow",
        "release.canary",
        "release.rollback",
        "release.classify",
        "release.evidence",
        "release.health",
        "trace-artifact",
        "hash.run",
        "hash.artifact",
        "why-rerun",
        "why-cache-missed",
        "fsck",
    ]
}

fn capabilities_payload() -> serde_json::Value {
    let operator_commands = operator_commands();
    json!({
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
            "container": "implemented",
            "batch_slurm": "implemented",
            "remote": "simulated",
            "batch_hpc": "simulated"
        },
        "execution_lanes": {
            "local_process": "ENFORCED",
            "container": "ENFORCED",
            "batch_slurm": "ENFORCED",
            "remote": "SIMULATED",
            "batch_hpc": "SIMULATED"
        },
        "backend_capabilities": [
            backend_capability_payload("kubernetes").unwrap(),
            backend_capability_payload("slurm").unwrap(),
            backend_capability_payload("hpc").unwrap(),
            backend_capability_payload("remote").unwrap()
        ],
        "operator_commands": operator_commands
    })
}

pub(crate) fn handle_capabilities_command(
    cli: &DagCli,
    backend: &Option<String>,
) -> Result<ExitCode, ExitCode> {
    let payload = capabilities_payload();
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
                        "remediation": "use --backend slurm, --backend kubernetes, --backend hpc, or --backend remote"
                    })],
                    ExitCode::from(2),
                );
            }
        }
    } else {
        payload
    };
    if cli.json {
        return emit_json(cli, "dag.capabilities", true, payload, Vec::new(), ExitCode::SUCCESS);
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
                    json!({"message":"unsupported backend target","remediation":"use --backend slurm, --backend kubernetes, --backend hpc, or --backend remote"}),
                ]
            },
            if supported { ExitCode::SUCCESS } else { ExitCode::from(2) },
        );
    }
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(if supported { ExitCode::SUCCESS } else { ExitCode::from(2) })
}

pub(crate) fn handle_equivalence_proof_command(
    cli: &DagCli,
    run_a: &std::path::Path,
    run_b: &std::path::Path,
    backend_a: &str,
    backend_b: &str,
) -> Result<ExitCode, ExitCode> {
    let diff = replay_service::run_diff_from_dirs(run_a, run_b)?;
    let backend_supported = backend_capability_payload(backend_a).is_some()
        && backend_capability_payload(backend_b).is_some();
    let status = if diff.replay_equivalence.equivalent && backend_supported {
        "equivalent"
    } else if backend_supported {
        "fidelity-preserving"
    } else {
        "downgraded"
    };
    let payload = json!({
        "format": "equivalence-proof/v1",
        "backend_a": backend_a,
        "backend_b": backend_b,
        "status": status,
        "run_equivalent": diff.replay_equivalence.equivalent,
        "summary": diff.replay_equivalence.reason_report.summary,
        "reasons": diff.replay_equivalence.reasons
    });
    if cli.json {
        return emit_json(
            cli,
            "dag.equivalence-proof",
            status != "downgraded",
            payload,
            if status == "downgraded" {
                vec![
                    json!({"message":"equivalence proof downgraded due to unsupported backend or semantic divergence"}),
                ]
            } else {
                Vec::new()
            },
            if status == "downgraded" { ExitCode::from(2) } else { ExitCode::SUCCESS },
        );
    }
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(if status == "downgraded" { ExitCode::from(2) } else { ExitCode::SUCCESS })
}

#[cfg(test)]
mod tests {
    use super::{handle_capabilities_command, handle_semantic_portability_command};
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use std::path::PathBuf;

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
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
            command: Commands::Fsck { run_dir: PathBuf::from("."), strict: false },
        };
        let code = handle_semantic_portability_command(&cli, "hpc").unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn capabilities_without_backend_is_supported() {
        let cli = quiet_json_cli();
        let code = handle_capabilities_command(&cli, &None).expect("capabilities");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn capabilities_payload_advertises_runtime_operator_surfaces() {
        let cli = quiet_json_cli();
        let code = handle_capabilities_command(&cli, &None).expect("capabilities");
        assert_eq!(code, ExitCode::SUCCESS);
        let payload = super::capabilities_payload();
        assert_eq!(payload["execution_modes"]["container"], "implemented");
        assert_eq!(payload["execution_lanes"]["local_process"], "ENFORCED");
        assert_eq!(payload["execution_lanes"]["container"], "ENFORCED");
        assert_eq!(payload["execution_lanes"]["batch_slurm"], "ENFORCED");
        assert_eq!(payload["execution_lanes"]["remote"], "SIMULATED");
        assert_eq!(payload["execution_lanes"]["batch_hpc"], "SIMULATED");
        let operator_commands =
            payload["operator_commands"].as_array().expect("operator commands payload");
        for expected in [
            "runs.stop",
            "runtime.state",
            "runtime.write-discipline",
            "runtime.worker-recovery",
            "runtime.control-recovery",
            "runtime.repair",
            "artifact.registry",
            "artifact.lineage",
            "artifact.retention",
            "dataset.mapping",
            "dataset.staleness",
            "enterprise.webhook",
            "enterprise.queue",
            "enterprise.service-contract",
            "enterprise.incident-hook",
            "enterprise.asset-link",
            "enterprise.calendar",
            "enterprise.approval",
            "enterprise.dependency-catalog",
            "enterprise.credentials",
            "enterprise.export",
            "control-plane.api",
            "control-plane.leadership",
            "control-plane.planning",
            "control-plane.sharding",
            "control-plane.leases",
            "control-plane.idempotency",
            "control-plane.backpressure",
            "control-plane.cache",
            "control-plane.migration",
            "control-plane.fan-in",
            "state-store.transaction",
            "state-store.journal",
            "state-store.snapshot",
            "state-store.index",
            "state-store.archive",
            "state-store.checksum",
            "state-store.amplification",
            "state-store.retention",
            "state-store.consistency",
            "state-store.clock",
            "fleet.register",
            "fleet.capabilities",
            "fleet.drain",
            "fleet.autoscale",
            "fleet.warm-pool",
            "fleet.isolation",
            "fleet.preemption",
            "fleet.trust",
            "fleet.gossip",
            "fleet.fragmentation",
            "governance.contracts",
            "governance.ownership",
            "governance.tags",
            "governance.cost",
            "governance.alerts",
            "governance.policy-check",
            "governance.catalog-export",
            "governance.audit-event",
            "governance.promotion",
            "governance.compliance",
            "incident.mode",
            "incident.blast-radius",
            "incident.safe-stop",
            "incident.degraded-mode",
            "incident.annotation",
            "incident.repair-window",
            "incident.timeline",
            "incident.replay-validation",
            "incident.readiness-review",
            "incident.scorecard",
            "federation.schedule",
            "federation.failover",
            "federation.lineage",
            "federation.sovereignty",
            "federation.replay",
            "federation.policy-distribution",
            "federation.audit-integrity",
            "federation.trust-tier",
            "federation.delegation",
            "federation.config-inheritance",
            "security.auth",
            "security.authz",
            "security.tenant",
            "security.secrets",
            "security.supply-chain",
            "security.data-access",
            "security.override",
            "security.safe-defaults",
            "release.version",
            "release.promotion",
            "release.deprecation",
            "release.checkpoint",
            "release.shadow",
            "release.canary",
            "release.rollback",
            "release.classify",
            "release.evidence",
            "release.health",
        ] {
            assert!(operator_commands.iter().any(|value| value.as_str() == Some(expected)));
        }
    }

    #[test]
    fn semantic_portability_local_backend_is_supported() {
        let cli = quiet_json_cli();
        let code =
            handle_semantic_portability_command(&cli, "kubernetes").expect("semantic portability");
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
