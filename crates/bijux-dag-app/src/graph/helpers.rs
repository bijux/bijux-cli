use crate::routes::selector_grammar::{parse_selector_expression, SelectorField};
use crate::run_data::{env_cache_dir, load_snapshot};
use crate::{
    check_engine, config_fingerprint, default_runtime_config, parse_graph, read_file, Graph,
    LintDiagnostic, Severity, SPEC_VERSION,
};
use bijux_dag_runtime::{
    compute_downstream_run_closure, compute_upstream_run_closure, registered_adapters, Selector,
    SelectorSet,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(crate) fn parse_selectors(
    include: &[String],
    exclude: &[String],
) -> Result<SelectorSet, ExitCode> {
    let mut set = SelectorSet { include: Vec::new(), exclude: Vec::new() };
    for raw in include {
        set.include.push(parse_selector(raw)?);
    }
    for raw in exclude {
        set.exclude.push(parse_selector(raw)?);
    }
    Ok(set)
}

pub(crate) fn parse_selector(raw: &str) -> Result<Selector, ExitCode> {
    let selector = parse_selector_expression(raw)?;
    match selector.field {
        SelectorField::Id | SelectorField::Node => Ok(Selector::Id(selector.value)),
        SelectorField::IdPrefix | SelectorField::NodePrefix => {
            Ok(Selector::IdPrefix(selector.value))
        }
        SelectorField::Tag => Ok(Selector::Tag(selector.value)),
        SelectorField::Kind => Ok(Selector::Kind(selector.value)),
        SelectorField::Run
        | SelectorField::Graph
        | SelectorField::State
        | SelectorField::Artifact
        | SelectorField::Branch
        | SelectorField::Attempt => Err(ExitCode::from(2)),
    }
}

pub(crate) fn validate_partial_selection_surface(
    from_node: &[String],
    to_node: &[String],
    include: &[String],
    exclude: &[String],
    dependency_closure: bool,
) -> Result<(), ExitCode> {
    if from_node.is_empty() && to_node.is_empty() {
        return Ok(());
    }
    if !from_node.is_empty() && !to_node.is_empty() {
        return Err(ExitCode::from(2));
    }
    if !include.is_empty() || !exclude.is_empty() || dependency_closure {
        return Err(ExitCode::from(2));
    }
    Ok(())
}

pub(crate) fn validate_downstream_selection_surface(
    from_node: &[String],
    include: &[String],
    exclude: &[String],
    dependency_closure: bool,
) -> Result<(), ExitCode> {
    validate_partial_selection_surface(from_node, &[], include, exclude, dependency_closure)
}

pub(crate) fn resolve_downstream_run_selection(
    graph: &Graph,
    from_node: &[String],
) -> Result<(Vec<String>, Vec<String>), ExitCode> {
    if from_node.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let declared = graph.nodes.iter().map(|node| node.id.as_str()).collect::<BTreeSet<_>>();
    let mut roots = BTreeSet::new();
    for node_id in from_node {
        if !declared.contains(node_id.as_str()) {
            return Err(ExitCode::from(2));
        }
        roots.insert(node_id.clone());
    }

    let roots = roots.into_iter().collect::<Vec<_>>();
    let selected = compute_downstream_run_closure(graph, &roots).into_iter().collect::<Vec<_>>();
    Ok((roots, selected))
}

pub(crate) fn resolve_upstream_run_selection(
    graph: &Graph,
    to_node: &[String],
) -> Result<(Vec<String>, Vec<String>), ExitCode> {
    if to_node.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let declared = graph.nodes.iter().map(|node| node.id.as_str()).collect::<BTreeSet<_>>();
    let mut targets = BTreeSet::new();
    for node_id in to_node {
        if !declared.contains(node_id.as_str()) {
            return Err(ExitCode::from(2));
        }
        targets.insert(node_id.clone());
    }

    let targets = targets.into_iter().collect::<Vec<_>>();
    let selected = compute_upstream_run_closure(graph, &targets).into_iter().collect::<Vec<_>>();
    Ok((targets, selected))
}

pub(crate) fn lint_graph(graph: &Graph) -> Vec<LintDiagnostic> {
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

pub(crate) fn graph_to_dot(graph: &Graph) -> String {
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

pub(crate) fn doctor_report() -> Result<serde_json::Value, ExitCode> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf();
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
    let schema_root = repo_root.join("configs").join("dag").join("schema");
    let schema_files =
        if schema_root.exists() { walk_json_files(&schema_root)? } else { Vec::new() };
    let runtime_schema = schema_root.join("runtime_config.schema.json");
    let env_overrides_present = [
        "BIJUX_DAG_JOBS",
        "BIJUX_DAG_CACHE_MODE",
        "BIJUX_DAG_MATERIALIZE_INPUTS",
        "BIJUX_DAG_POLICY_JSON",
    ]
    .into_iter()
    .filter(|key| std::env::var(key).is_ok())
    .collect::<Vec<_>>();
    let runtime_config = json!({
        "schema_found": runtime_schema.exists(),
        "defaults_fingerprint": config_fingerprint(&default_runtime_config()),
        "env_overrides_present": env_overrides_present,
    });

    let hardlink_ok = {
        let dir = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        let _ = fs::write(&a, b"ok");
        fs::hard_link(&a, &b).is_ok()
    };

    let status =
        if cache_status["status"] == "error" || !runtime_schema.exists() { "error" } else { "ok" };

    Ok(json!({
        "status": status,
        "cache": cache_status,
        "container": { "docker": docker, "podman": podman },
        "adapters": adapters,
        "schema_files": {
            "root": schema_root,
            "count": schema_files.len(),
            "runtime_config_schema_found": runtime_schema.exists(),
            "files": schema_files,
        },
        "runtime_config": runtime_config,
        "filesystem": { "hardlink": hardlink_ok },
        "policy": { "clock": "allowed_by_default" }
    }))
}

fn walk_json_files(root: &Path) -> Result<Vec<String>, ExitCode> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).map_err(|_| ExitCode::from(3))? {
            let entry = entry.map_err(|_| ExitCode::from(3))?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                files.push(path.strip_prefix(root).unwrap_or(&path).display().to_string());
            }
        }
    }
    files.sort();
    Ok(files)
}

pub(crate) fn migrate_dag(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let input = read_file(path)?;
    let graph = parse_graph(&input)?;
    let from_normalized =
        if from == "0.1" || from == "v0.1" { SPEC_VERSION.to_string() } else { from.to_string() };
    let to_normalized =
        if to == "0.1" || to == "v0.1" { SPEC_VERSION.to_string() } else { to.to_string() };
    if graph.spec != from_normalized {
        return Err(ExitCode::from(3));
    }
    if from_normalized == to_normalized {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

pub(crate) fn migrate_run(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let snapshot = load_snapshot(path)?;
    let from_normalized =
        if from == "0.1" || from == "v0.1" { SPEC_VERSION.to_string() } else { from.to_string() };
    let to_normalized =
        if to == "0.1" || to == "v0.1" { SPEC_VERSION.to_string() } else { to.to_string() };
    if snapshot.graph.spec != from_normalized {
        return Err(ExitCode::from(3));
    }
    if from_normalized == to_normalized {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

pub(crate) fn inspect_migrate_dag(
    path: &Path,
    from: &str,
    to: &str,
) -> Result<serde_json::Value, ExitCode> {
    let input = read_file(path)?;
    let graph = parse_graph(&input)?;
    let from_normalized =
        if from == "0.1" || from == "v0.1" { SPEC_VERSION.to_string() } else { from.to_string() };
    let to_normalized =
        if to == "0.1" || to == "v0.1" { SPEC_VERSION.to_string() } else { to.to_string() };
    let migration_required = from_normalized != to_normalized;
    let apply_supported = from_normalized == to_normalized && graph.spec == from_normalized;
    Ok(json!({
        "target": "dag",
        "path": path,
        "current_spec": graph.spec,
        "requested_from": from_normalized,
        "requested_to": to_normalized,
        "migration_required": migration_required,
        "apply_supported": apply_supported,
        "decision": if apply_supported { "no-op" } else { "refuse-unsupported" },
        "reason": if apply_supported {
            "migration is a no-op for this version lane"
        } else if graph.spec != from_normalized {
            "requested from-version does not match graph spec"
        } else {
            "cross-version migration is unavailable"
        }
    }))
}

pub(crate) fn inspect_migrate_run(
    path: &Path,
    from: &str,
    to: &str,
) -> Result<serde_json::Value, ExitCode> {
    let snapshot = load_snapshot(path)?;
    let from_normalized =
        if from == "0.1" || from == "v0.1" { SPEC_VERSION.to_string() } else { from.to_string() };
    let to_normalized =
        if to == "0.1" || to == "v0.1" { SPEC_VERSION.to_string() } else { to.to_string() };
    let migration_required = from_normalized != to_normalized;
    let apply_supported =
        from_normalized == to_normalized && snapshot.graph.spec == from_normalized;
    Ok(json!({
        "target": "run",
        "path": path,
        "current_spec": snapshot.graph.spec,
        "requested_from": from_normalized,
        "requested_to": to_normalized,
        "migration_required": migration_required,
        "apply_supported": apply_supported,
        "decision": if apply_supported { "no-op" } else { "refuse-unsupported" },
        "reason": if apply_supported {
            "migration is a no-op for this version lane"
        } else if snapshot.graph.spec != from_normalized {
            "requested from-version does not match run snapshot graph spec"
        } else {
            "cross-version migration is unavailable"
        }
    }))
}

pub(crate) fn run_compat_suite() -> Result<serde_json::Value, ExitCode> {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bijux-dag-core/tests/compat/v0.1");
    if !base.exists() {
        return Ok(json!({"status":"ok","errors":[]}));
    }
    let mut errors = Vec::new();
    let mut entries: Vec<_> =
        fs::read_dir(&base).map_err(|_| ExitCode::from(3))?.filter_map(|e| e.ok()).collect();
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
        let stem = path.file_name().unwrap().to_string_lossy().replace(".dag.json", "");
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
        let expected_fp = read_file(&graph_fp_path).unwrap_or_default().trim().to_string();
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

#[cfg(test)]
mod tests {
    use super::{
        doctor_report, inspect_migrate_dag, inspect_migrate_run, parse_selector, parse_selectors,
        resolve_downstream_run_selection, resolve_upstream_run_selection,
        validate_partial_selection_surface,
    };
    use crate::parse_graph;
    use serde_json::json;
    use std::process::ExitCode;

    #[test]
    fn selector_parser_accepts_supported_prefixes() {
        let by_id = parse_selector("id:train").expect("id selector");
        let by_tag = parse_selector("tag:gpu").expect("tag selector");
        let by_kind = parse_selector("kind:shell").expect("kind selector");
        let set = parse_selectors(
            &["id:a".to_string(), "tag:b".to_string()],
            &["kind:const".to_string()],
        )
        .expect("selector set");

        let by_id_prefix = parse_selector("id-prefix:train").expect("id-prefix selector");

        assert_eq!(format!("{by_id:?}"), "Id(\"train\")");
        assert_eq!(format!("{by_id_prefix:?}"), "IdPrefix(\"train\")");
        assert_eq!(format!("{by_tag:?}"), "Tag(\"gpu\")");
        assert_eq!(format!("{by_kind:?}"), "Kind(\"shell\")");
        assert_eq!(set.include.len(), 2);
        assert_eq!(set.exclude.len(), 1);
    }

    #[test]
    fn selector_parser_rejects_invalid_syntax() {
        for raw in ["", "id", "tag", "kind", "name:node", "id=", "attempt:latest"] {
            let err = parse_selector(raw).expect_err("invalid selector must fail");
            assert_eq!(err, ExitCode::from(2), "selector should reject: {raw}");
        }
    }

    #[test]
    fn selector_parser_accepts_node_alias_for_execution_selectors() {
        let by_node = parse_selector("node:train").expect("node selector");
        let by_node_prefix = parse_selector("node-prefix:train").expect("node-prefix selector");
        assert_eq!(format!("{by_node:?}"), "Id(\"train\")");
        assert_eq!(format!("{by_node_prefix:?}"), "IdPrefix(\"train\")");
    }

    #[test]
    fn partial_selection_rejects_selector_mode_mixes() {
        let error = validate_partial_selection_surface(
            &["train".to_string()],
            &[],
            &["id:report".to_string()],
            &[],
            false,
        )
        .expect_err("mixed mode must fail");
        assert_eq!(error, ExitCode::from(2));
    }

    #[test]
    fn partial_selection_rejects_multiple_direction_modes() {
        let error = validate_partial_selection_surface(
            &["train".to_string()],
            &["report".to_string()],
            &[],
            &[],
            false,
        )
        .expect_err("conflicting mode must fail");
        assert_eq!(error, ExitCode::from(2));
    }

    #[test]
    fn downstream_selection_resolves_exact_root_and_descendants() {
        let graph = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out"}],"params":{"value":1}},
                {"id":"branch","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"branch/out"}],"params":{"value":2}},
                {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{"value":3}}
              ],
              "edges":[
                {"from":{"node_id":"source","port":"out"},"to":{"node_id":"branch","port":"in"}},
                {"from":{"node_id":"branch","port":"out"},"to":{"node_id":"sink","port":"in"}}
              ]
            }"#,
        )
        .expect("graph");
        let (roots, selected_nodes) =
            resolve_downstream_run_selection(&graph, &["branch".to_string()]).expect("selection");
        assert_eq!(roots, vec!["branch"]);
        assert_eq!(selected_nodes, vec!["branch".to_string(), "sink".to_string()]);
    }

    #[test]
    fn upstream_selection_resolves_exact_target_and_ancestors() {
        let graph = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out"}],"params":{"value":1}},
                {"id":"branch","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"branch/out"}],"params":{"value":2}},
                {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{"value":3}},
                {"id":"side","kind":"const","outputs":[{"name":"out","path":"side/out"}],"params":{"value":4}}
              ],
              "edges":[
                {"from":{"node_id":"source","port":"out"},"to":{"node_id":"branch","port":"in"}},
                {"from":{"node_id":"branch","port":"out"},"to":{"node_id":"sink","port":"in"}}
              ]
            }"#,
        )
        .expect("graph");
        let (targets, selected_nodes) =
            resolve_upstream_run_selection(&graph, &["sink".to_string()]).expect("selection");
        assert_eq!(targets, vec!["sink"]);
        assert_eq!(
            selected_nodes,
            vec!["branch".to_string(), "sink".to_string(), "source".to_string()]
        );
    }

    #[test]
    fn doctor_report_exposes_schema_and_runtime_config_status() {
        let report = doctor_report().expect("doctor report");
        assert!(report["schema_files"]["count"].as_u64().is_some());
        assert!(report["runtime_config"]["defaults_fingerprint"].as_str().is_some());
    }

    #[test]
    fn migrate_inspect_surfaces_decision_without_mutation() {
        let temp = tempfile::tempdir().expect("tmp");
        let dag_path = temp.path().join("dag.json");
        std::fs::write(
            &dag_path,
            serde_json::to_vec_pretty(&json!({
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"x","owners":[],"tags":[]},
                "nodes":[],
                "edges":[]
            }))
            .expect("dag"),
        )
        .expect("write dag");
        let run_dir = temp.path().join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(
            run_dir.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph":{"spec":"bijux-dag/v0.1","meta":{"name":"x","owners":[],"tags":[]},"nodes":[],"edges":[]},
                "graph_fingerprint":"g1"
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        let dag_report = inspect_migrate_dag(&dag_path, "v0.1", "v0.1").expect("inspect dag");
        let run_report = inspect_migrate_run(&run_dir, "v0.1", "v0.2").expect("inspect run");
        assert_eq!(dag_report["decision"], "no-op");
        assert_eq!(run_report["decision"], "refuse-unsupported");
    }
}
