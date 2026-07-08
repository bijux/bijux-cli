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
fn slurm_backend_run_executes_nodes_and_persists_batch_job_evidence() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("slurm.dag.json");
    let runs_dir = temp.path().join("runs");
    let tools_dir = temp.path().join("tools");
    let state_dir = temp.path().join("state");
    fs::create_dir_all(&tools_dir).expect("tools dir");
    fs::create_dir_all(&state_dir).expect("state dir");

    write_executable(
        &tools_dir.join("sbatch"),
        &format!(
            "#!/bin/sh\nset -eu\nSTATE_DIR={state:?}\nOUT=\"\"\nERR=\"\"\nSCRIPT=\"\"\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --parsable) shift ;;\n    --cpus-per-task|--mem|--time|--partition|--qos|--account|--output|--error)\n      if [ \"$1\" = \"--output\" ]; then OUT=\"$2\"; fi\n      if [ \"$1\" = \"--error\" ]; then ERR=\"$2\"; fi\n      shift 2 ;;\n    *) SCRIPT=\"$1\"; shift ;;\n  esac\ndone\nmkdir -p \"$STATE_DIR\" \"$(dirname \"$OUT\")\" \"$(dirname \"$ERR\")\"\nif sh \"$SCRIPT\" > \"$OUT\" 2> \"$ERR\"; then\n  printf 'COMPLETED' > \"$STATE_DIR/job-42.state\"\nelse\n  printf 'FAILED' > \"$STATE_DIR/job-42.state\"\nfi\nprintf 'job-42\\n'\n",
            state = state_dir.display().to_string(),
        ),
    );
    write_executable(
        &tools_dir.join("sacct"),
        &format!(
            "#!/bin/sh\nset -eu\nSTATE_DIR={state:?}\nSTATE=$(cat \"$STATE_DIR/job-42.state\")\nprintf '%s|0:0\\n' \"$STATE\"\n",
            state = state_dir.display().to_string(),
        ),
    );

    fs::write(
        &graph_path,
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "render",
              "kind": "shell",
              "outputs": [{"name": "report", "path": "report.txt"}],
              "params": {
                "argv": ["/bin/sh", "-c", "printf slurm-report > ../outputs/report.txt && printf node-stdout && printf node-stderr 1>&2"]
              },
              "resources": {"cpu": 2, "mem_mb": 1024},
              "tags": ["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:research"],
              "effects": ["filesystem"]
            }
          ],
          "edges": []
        }"#,
    )
    .expect("write graph");

    let output = dag_command(&root)
        .env("BIJUX_DAG_SLURM_SBATCH", tools_dir.join("sbatch"))
        .env("BIJUX_DAG_SLURM_SACCT", tools_dir.join("sacct"))
        .env("BIJUX_DAG_SLURM_POLL_INTERVAL_MS", "50")
        .args([
            "run",
            graph_path.to_string_lossy().as_ref(),
            "--out",
            runs_dir.to_string_lossy().as_ref(),
            "--run-id",
            "slurm-proof",
            "--backend",
            "slurm",
            "--slurm-queue",
            "general",
            "--slurm-partition",
            "cpu",
            "--json",
        ])
        .output()
        .expect("run slurm backend");

    assert_eq!(
        output.status.code(),
        Some(0),
        "run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse run output");
    assert_eq!(payload["ok"], true);

    let run_dir = runs_dir.join("run-slurm-proof");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("parse manifest");
    assert_eq!(manifest["status"], "success");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("render").join("trace.json"))
            .expect("trace"),
    )
    .expect("parse trace");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["stdout"]["path"], "nodes/render/stdout.log");
    assert_eq!(trace["stderr"]["path"], "nodes/render/stderr.log");

    let batch_job: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("render").join("batch-job.json"))
            .expect("batch job"),
    )
    .expect("parse batch job");
    assert_eq!(batch_job["job_id"], "job-42");
    assert_eq!(batch_job["metadata"]["scheduler_id"], "slurm");
    assert!(
        batch_job["metadata"]["resource_request"]
            .as_str()
            .expect("resource request")
            .contains("queue=priority")
    );
    assert!(
        batch_job["metadata"]["resource_request"]
            .as_str()
            .expect("resource request")
            .contains("partition=gpu")
    );
    assert!(
        batch_job["metadata"]["resource_request"]
            .as_str()
            .expect("resource request")
            .contains("account=research")
    );

    let rendered =
        fs::read_to_string(run_dir.join("nodes").join("render").join("outputs").join("report.txt"))
            .expect("rendered report");
    assert_eq!(rendered, "slurm-report");
}
