fn parse_selectors(include: &[String], exclude: &[String]) -> Result<SelectorSet, ExitCode> {
    let mut set = SelectorSet {
        include: Vec::new(),
        exclude: Vec::new(),
    };
    for raw in include {
        set.include.push(parse_selector(raw)?);
    }
    for raw in exclude {
        set.exclude.push(parse_selector(raw)?);
    }
    Ok(set)
}

fn parse_selector(raw: &str) -> Result<Selector, ExitCode> {
    if let Some(rest) = raw.strip_prefix("id:") {
        return Ok(Selector::IdPrefix(rest.to_string()));
    }
    if let Some(rest) = raw.strip_prefix("tag:") {
        return Ok(Selector::Tag(rest.to_string()));
    }
    if let Some(rest) = raw.strip_prefix("kind:") {
        return Ok(Selector::Kind(rest.to_string()));
    }
    Err(ExitCode::from(2))
}

fn lint_graph(graph: &Graph) -> Vec<LintDiagnostic> {
    let mut out = Vec::new();
    for diag in graph.validate_with_warnings() {
        if diag.severity == Severity::Warning {
            out.push(LintDiagnostic {
                code: diag.code,
                message: diag.message,
                path: diag.path,
                hint: diag.hint,
            });
        }
    }
    let mut used_outputs: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for edge in &graph.edges {
        used_outputs.insert((edge.from.node_id.clone(), edge.from.port.clone()));
    }
    for node in &graph.nodes {
        for outp in &node.outputs {
            if !used_outputs.contains(&(node.id.clone(), outp.name.clone())) {
                out.push(LintDiagnostic {
                    code: "L1001".to_string(),
                    message: format!("unused output: {}", outp.name),
                    path: format!("/nodes/{}/outputs", node.id),
                    hint: Some("Remove or connect this output".to_string()),
                });
            }
        }
        if node.resources.is_none() {
            out.push(LintDiagnostic {
                code: "L1002".to_string(),
                message: "missing resource hints".to_string(),
                path: format!("/nodes/{}/resources", node.id),
                hint: Some("Set resources.cpu/mem_mb for scheduling".to_string()),
            });
        }
        if node.effects.iter().any(|e| {
            matches!(
                e,
                bijux_dag_core::Effect::Network
                    | bijux_dag_core::Effect::Env
                    | bijux_dag_core::Effect::Clock
            )
        }) {
            out.push(LintDiagnostic {
                code: "L1003".to_string(),
                message: "broad effects declared".to_string(),
                path: format!("/nodes/{}/effects", node.id),
                hint: Some("Use minimal effects required".to_string()),
            });
        }
    }
    out
}

fn graph_to_dot(graph: &Graph) -> String {
    let g = graph.canonicalize();
    let mut out = String::from("digraph bijux {\n");
    for node in &g.nodes {
        out.push_str(&format!("  \"{}\";\n", node.id));
    }
    for edge in &g.edges {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}->{}\"];\n",
            edge.from.node_id, edge.to.node_id, edge.from.port, edge.to.port
        ));
    }
    out.push_str("}\n");
    out
}

fn doctor_report() -> Result<serde_json::Value, ExitCode> {
    let cache_dir = env_cache_dir();
    let cache_status = if let Some(dir) = cache_dir.as_ref() {
        if fs::create_dir_all(dir).is_ok() {
            let test = dir.join(".__bijux_write_test");
            let writable = fs::write(&test, b"ok").is_ok();
            let _ = fs::remove_file(&test);
            if writable {
                json!({"status":"ok","path":dir})
            } else {
                json!({"status":"error","path":dir})
            }
        } else {
            json!({"status":"error","path":dir})
        }
    } else {
        json!({"status":"missing"})
    };

    let docker = check_engine("docker");
    let podman = check_engine("podman");
    let adapters = registered_adapters();

    let hardlink_ok = {
        let dir = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        let _ = fs::write(&a, b"ok");
        fs::hard_link(&a, &b).is_ok()
    };

    let status = if cache_status["status"] == "error" {
        "error"
    } else {
        "ok"
    };

    Ok(json!({
        "status": status,
        "cache": cache_status,
        "container": { "docker": docker, "podman": podman },
        "adapters": adapters,
        "filesystem": { "hardlink": hardlink_ok },
        "policy": { "clock": "allowed_by_default" }
    }))
}

fn migrate_dag(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let input = read_file(path)?;
    let graph = parse_graph(&input)?;
    if graph.spec != from {
        return Err(ExitCode::from(3));
    }
    if from == to {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

fn migrate_run(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let snapshot = load_snapshot(path)?;
    if snapshot.graph.spec != from {
        return Err(ExitCode::from(3));
    }
    if from == to {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

fn run_compat_suite() -> Result<serde_json::Value, ExitCode> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../bijux-dag-core/tests/compat/v0.1");
    if !base.exists() {
        return Ok(json!({"status":"ok","errors":[]}));
    }
    let mut errors = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(&base)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".dag.json"))
            .unwrap_or(false)
        {
            continue;
        }
        let stem = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace(".dag.json", "");
        let canonical_path = base.join(format!("{}.canonical.json", stem));
        let graph_fp_path = base.join(format!("{}.graph_fingerprint", stem));
        let node_fp_path = base.join(format!("{}.node_fingerprints.json", stem));

        let input = read_file(&path)?;
        let graph = parse_graph(&input)?;
        let canonical = graph.to_canonical_json().map_err(|_| ExitCode::from(3))?;
        let expected = read_file(&canonical_path).unwrap_or_default();
        if canonical.trim() != expected.trim() {
            errors.push(format!("canonical mismatch: {}", stem));
        }
        let fp = graph.graph_fingerprint().map_err(|_| ExitCode::from(3))?;
        let expected_fp = read_file(&graph_fp_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        if fp != expected_fp {
            errors.push(format!("graph fingerprint mismatch: {}", stem));
        }
        let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
        let mut nodes = serde_json::Map::new();
        for n in &graph.nodes {
            let fp = resolved
                .as_ref()
                .and_then(|m| m.get(&n.id))
                .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
            nodes.insert(n.id.clone(), json!(fp));
        }
        let expected_nodes = read_file(&node_fp_path).unwrap_or_default();
        let expected_val: serde_json::Value =
            serde_json::from_str(&expected_nodes).unwrap_or_else(|_| json!({}));
        if json!(nodes) != expected_val {
            errors.push(format!("node fingerprint mismatch: {}", stem));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "error" };
    Ok(json!({ "status": status, "errors": errors }))
}
