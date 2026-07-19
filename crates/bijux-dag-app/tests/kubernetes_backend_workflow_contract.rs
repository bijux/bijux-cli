use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn dag_command(root: &Path) -> Command {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.current_dir(root).env("CARGO_TARGET_DIR", root.join("artifacts/target")).args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--",
    ]);
    command
}

fn write_executable(path: &Path, script: &str) {
    fs::write(path, script).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("stat script").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod script");
    }
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__kubernetes_backend_run_executes_container_nodes_and_persists_batch_job_evidence() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("kubernetes.dag.json");
    let runs_dir = temp.path().join("runs");
    let state_dir = temp.path().join("state");
    let kubectl = temp.path().join("kubectl");
    fs::create_dir_all(&state_dir).expect("state dir");

    write_executable(
        &kubectl,
        &format!(
            r#"#!/bin/sh
set -eu
STATE_DIR={state:?}
SHARED_ROOT={shared_root:?}
command="$1"
shift
case "$command" in
  create)
    spec=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -f) spec="$2"; shift 2 ;;
        -o) shift 2 ;;
        *) shift ;;
      esac
    done
    job_id=$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["metadata"]["name"])' "$spec")
    outputs_sub=$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); mounts=data["spec"]["template"]["spec"]["containers"][0]["volumeMounts"]; print(next(m["subPath"] for m in mounts if m["mountPath"]=="/bijux/node/outputs"))' "$spec")
    mkdir -p "$STATE_DIR" "$SHARED_ROOT/$outputs_sub"
    printf 'kubernetes-report' > "$SHARED_ROOT/$outputs_sub/report.txt"
    printf 'Succeeded' > "$STATE_DIR/$job_id.phase"
    printf 'Completed' > "$STATE_DIR/$job_id.reason"
    printf 'kubernetes workflow log\n' > "$STATE_DIR/$job_id.log"
    python3 - "$job_id" <<'PY'
import json, sys
print(json.dumps({{"metadata": {{"name": sys.argv[1]}}}}))
PY
    ;;
  get)
    label=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -l) label="$2"; shift 2 ;;
        -o) shift 2 ;;
        -n) shift 2 ;;
        pods) shift ;;
        *) shift ;;
      esac
    done
    job_id=${{label#job-name=}}
    phase=$(cat "$STATE_DIR/$job_id.phase")
    reason=$(cat "$STATE_DIR/$job_id.reason")
    python3 - "$phase" "$reason" <<'PY'
import json, sys
phase, reason = sys.argv[1], sys.argv[2]
print(json.dumps({{
  "items": [{{
    "status": {{
      "phase": phase,
      "containerStatuses": [{{
        "state": {{"terminated": {{"reason": reason}}}}
      }}]
    }}
  }}]
}}))
PY
    ;;
  logs)
    target=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -n) shift 2 ;;
        *) target="$1"; shift ;;
      esac
    done
    job_id=${{target#job/}}
    cat "$STATE_DIR/$job_id.log"
    ;;
  *)
    echo "unexpected kubectl command: $command" >&2
    exit 1
    ;;
esac
"#,
            state = state_dir.display().to_string(),
            shared_root = temp.path().display().to_string(),
        ),
    );

    fs::write(
        &graph_path,
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "render",
              "kind": "container",
              "outputs": [{"name": "report", "path": "report.txt"}],
              "container": {
                "image": "example.local/runner@sha256:feedface",
                "argv": ["/bin/sh", "-c", "printf ignored > /bijux/node/outputs/report.txt"],
                "workdir": "scratch",
                "engine": "docker"
              },
              "effects": ["filesystem"],
              "resources": {"cpu": 2, "mem_mb": 1024}
            }
          ],
          "edges": []
        }"#,
    )
    .expect("write graph");

    let output = dag_command(&root)
        .env("BIJUX_DAG_KUBECTL", &kubectl)
        .env("BIJUX_DAG_KUBERNETES_POLL_INTERVAL_MS", "50")
        .args([
            "run",
            graph_path.to_string_lossy().as_ref(),
            "--out",
            runs_dir.to_string_lossy().as_ref(),
            "--run-id",
            "kubernetes-proof",
            "--backend",
            "kubernetes",
            "--kubernetes-namespace",
            "bijux-jobs",
            "--kubernetes-volume-claim",
            "bijux-run-pvc",
            "--kubernetes-shared-root",
            temp.path().to_string_lossy().as_ref(),
            "--json",
        ])
        .output()
        .expect("run kubernetes backend");

    assert_eq!(
        output.status.code(),
        Some(0),
        "run failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse run output");
    assert_eq!(payload["ok"], true);

    let run_dir = runs_dir.join("run-kubernetes-proof");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("parse manifest");
    assert_eq!(manifest["status"], "success");

    let batch_job: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("render").join("batch-job.json"))
            .expect("batch job"),
    )
    .expect("parse batch job");
    assert_eq!(batch_job["metadata"]["scheduler_id"], "kubernetes");
    assert_eq!(batch_job["metadata"]["run_id"], "kubernetes-proof");
    assert_eq!(batch_job["workspace"]["mode"], "mounted_workdir");
    assert!(batch_job["metadata"]["resource_request"]
        .as_str()
        .expect("resource request")
        .contains("namespace=bijux-jobs"));

    let rendered =
        fs::read_to_string(run_dir.join("nodes").join("render").join("outputs").join("report.txt"))
            .expect("rendered report");
    assert_eq!(rendered, "kubernetes-report");
}
