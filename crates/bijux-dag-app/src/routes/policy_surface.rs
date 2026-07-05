use crate::ExitCode;
use bijux_dag_core::Graph;
use bijux_dag_runtime::{build_policy_enforcement_report, CacheMode, RuntimeConfig};
use serde_json::{json, Value};

pub(crate) fn policy_surface_payload(
    graph: &Graph,
    options: &RuntimeConfig,
    hermetic: bool,
) -> Result<Value, ExitCode> {
    let enforcement =
        build_policy_enforcement_report(graph, options).map_err(|_| ExitCode::from(3))?;
    Ok(json!({
        "profile": hermetic_profile_payload(hermetic),
        "enforcement": enforcement,
    }))
}

pub(crate) fn hermetic_profile_payload(hermetic: bool) -> Value {
    if hermetic {
        return json!({
            "enabled": true,
            "mode": "best_effort_local_policy_profile",
            "summary": "forces deny-network, deny-clock, and clean-env for local execution",
            "limitations": [
                "shell execution still relies on declared-effect gates and environment shaping",
                "hermetic mode does not claim syscall sandboxing or host filesystem isolation"
            ]
        });
    }

    json!({
        "enabled": false,
        "mode": "explicit_policy_flags",
        "summary": "uses only the policy flags requested by the operator plus the default clean environment",
        "limitations": []
    })
}

pub(crate) fn replay_sandbox_scope_payload(sandbox: bool) -> Value {
    if sandbox {
        return json!({
            "enabled": true,
            "mode": "source_run_write_boundary",
            "summary": "forbids replay outputs from being written inside the source run directory",
            "limitations": [
                "sandbox mode does not create a process sandbox for replay execution",
                "sandbox mode does not claim network, clock, or filesystem syscall isolation"
            ]
        });
    }

    json!({
        "enabled": false,
        "mode": "standard",
        "summary": "replay uses the requested output directory without the source-run write boundary",
        "limitations": []
    })
}

pub(crate) fn cache_surface_payload(options: &RuntimeConfig) -> Value {
    let read_order = if options.remote_cache_dir.is_some() && options.cache_dir.is_some() {
        vec!["local", "shared"]
    } else if options.remote_cache_dir.is_some() {
        vec!["shared"]
    } else if options.cache_dir.is_some() {
        vec!["local"]
    } else {
        Vec::new()
    };
    let write_targets = if matches!(options.cache_mode, CacheMode::ReadWrite) {
        if options.remote_cache_dir.is_some() && options.cache_dir.is_some() {
            vec!["local", "shared"]
        } else if options.remote_cache_dir.is_some() {
            vec!["shared"]
        } else if options.cache_dir.is_some() {
            vec!["local"]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    json!({
        "mode": match options.cache_mode {
            CacheMode::Off => "off",
            CacheMode::Read => "read",
            CacheMode::ReadWrite => "readwrite",
        },
        "local_dir": options.cache_dir.as_ref().map(|path| path.display().to_string()),
        "shared_dir": options.remote_cache_dir.as_ref().map(|path| path.display().to_string()),
        "read_order": read_order,
        "write_targets": write_targets,
    })
}
