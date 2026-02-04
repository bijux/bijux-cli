use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn write_dag(path: &std::path::Path, value: i32) {
    let data = format!(
        r#"{{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {{"id": "a", "kind": "const", "inputs": [], "outputs": ["out"], "params": {{"value": {}}}}},
    {{"id": "b", "kind": "shell", "inputs": ["in"], "outputs": ["out_b"], "params": {{"argv": ["echo", "ok"]}}, "effects": ["filesystem"]}}
  ],
  "edges": [
    {{"from": {{"node_id": "a", "port": "out"}}, "to": {{"node_id": "b", "port": "in"}}}}
  ]
}}"#,
        value
    );
    fs::write(path, data).unwrap();
}

#[test]
fn replay_golden() {
    let bin = env!("CARGO_BIN_EXE_bijux-dag");
    let dir = tempdir().unwrap();
    let dag_path = dir.path().join("dag.json");
    write_dag(&dag_path, 1);

    let runs_dir = dir.path().join("runs");
    fs::create_dir_all(&runs_dir).unwrap();

    let out = Command::new(bin)
        .args([
            "run",
            dag_path.to_str().unwrap(),
            "--out",
            runs_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let run_dirs = fs::read_dir(&runs_dir).unwrap();
    let run_dir = run_dirs
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
        .unwrap();

    let replay_dir = dir.path().join("replay");
    fs::create_dir_all(&replay_dir).unwrap();

    let out = Command::new(bin)
        .args([
            "replay",
            run_dir.to_str().unwrap(),
            "--out",
            replay_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let replay_run = fs::read_dir(&replay_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
        .unwrap();

    let snap_a = fs::read_to_string(run_dir.join("graph.snapshot.json")).unwrap();
    let snap_b = fs::read_to_string(replay_run.join("graph.snapshot.json")).unwrap();
    assert_eq!(snap_a, snap_b);

    let trace_a = fs::read_to_string(run_dir.join("nodes").join("a").join("trace.json")).unwrap();
    let trace_b =
        fs::read_to_string(replay_run.join("nodes").join("a").join("trace.json")).unwrap();
    let va: serde_json::Value = serde_json::from_str(&trace_a).unwrap();
    let vb: serde_json::Value = serde_json::from_str(&trace_b).unwrap();
    assert_eq!(va["fingerprint"], vb["fingerprint"]);
    assert_eq!(va["status"], vb["status"]);
}

#[test]
fn diff_golden() {
    let bin = env!("CARGO_BIN_EXE_bijux-dag");
    let dir = tempdir().unwrap();
    let dag_a = dir.path().join("a.json");
    let dag_b = dir.path().join("b.json");
    write_dag(&dag_a, 1);
    write_dag(&dag_b, 2);

    let runs_dir = dir.path().join("runs");
    fs::create_dir_all(&runs_dir).unwrap();

    let out = Command::new(bin)
        .args([
            "run",
            dag_a.to_str().unwrap(),
            "--out",
            runs_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = Command::new(bin)
        .args([
            "run",
            dag_b.to_str().unwrap(),
            "--out",
            runs_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let mut run_dirs: Vec<_> = fs::read_dir(&runs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
        .collect();
    run_dirs.sort();
    let run_a = run_dirs[0].clone();
    let run_b = run_dirs[1].clone();

    let out = Command::new(bin)
        .args([
            "--json",
            "diff",
            run_a.to_str().unwrap(),
            run_b.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let val: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let nodes = val["node_changes"].as_object().unwrap();
    assert!(nodes.contains_key("a"));
}
