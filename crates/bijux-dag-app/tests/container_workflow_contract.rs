use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_json_with_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> Value {
    let (code, stdout, stderr) = support::run_dag_command_with_env(args, cwd, envs);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_with_env_allow_failure(
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
) -> (i32, Value, String) {
    let (code, stdout, stderr) = support::run_dag_command_with_env(args, cwd, envs);
    let payload = serde_json::from_str(&stdout).expect("parse json envelope");
    (code, payload, stderr)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

fn read_trace(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json")).expect("trace"),
    )
    .expect("trace json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/release-note-bundle.dag.json")
}

fn prepend_path(dir: &Path) -> String {
    let mut entries = vec![dir.to_string_lossy().into_owned()];
    if let Some(current) = std::env::var_os("PATH") {
        entries.push(current.to_string_lossy().into_owned());
    }
    entries.join(":")
}

fn copy_source_note(root: &Path, destination: &Path) -> PathBuf {
    let source = root.join("evidence/dag/authoring/examples/release-note-source/weekly-update.md");
    fs::create_dir_all(destination).expect("inputs dir");
    let note = destination.join("weekly-update.md");
    fs::copy(source, &note).expect("copy note");
    note
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod");
    }
}

fn write_docker_shim(bin_dir: &Path) -> PathBuf {
    fs::create_dir_all(bin_dir).expect("bin dir");
    let docker = bin_dir.join("docker");
    write_executable(
        &docker,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "docker fake 1.0"
  exit 0
fi
if [ "$1" = "image" ] && [ "$2" = "inspect" ]; then
  echo "sha256:feedface"
  exit 0
fi
if [ "$1" = "run" ]; then
  shift
  inputs_dir=""
  outputs_dir=""
  workdir=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --workdir)
        workdir="$2"
        shift 2
        ;;
      --network)
        shift 2
        ;;
      -v)
        mount="$2"
        host_path=$(printf '%s' "$mount" | cut -d: -f1)
        container_path=$(printf '%s' "$mount" | cut -d: -f2)
        if [ "$container_path" = "/bijux/node/inputs" ]; then
          inputs_dir="$host_path"
        elif [ "$container_path" = "/bijux/node/outputs" ]; then
          outputs_dir="$host_path"
        fi
        shift 2
        ;;
      -e)
        shift 2
        ;;
      --rm)
        shift
        ;;
      -*)
        shift
        ;;
      *)
        break
        ;;
    esac
  done
  shift
  label=""
  for arg in "$@"; do
    label="$arg"
  done
  metadata="$inputs_dir/prepare_note/metadata"
  source="$inputs_dir/prepare_note/source"
  source_bytes=$(sed -n 's/^source_bytes=//p' "$metadata")
  mkdir -p "$outputs_dir/bundle"
  {
    printf '%s\n' "$label"
    cat "$source"
  } > "$outputs_dir/bundle/release-note.txt"
  cat > "$outputs_dir/bundle/container-summary.json" <<EOF
{
  "bundle_label": "$label",
  "source_bytes": $source_bytes,
  "container_workdir": "$workdir"
}
EOF
  printf 'container-package-stdout'
  printf 'container-package-stderr' >&2
  exit 0
fi
exit 1
"#,
    );
    docker
}

fn write_unavailable_docker_shim(bin_dir: &Path) -> PathBuf {
    fs::create_dir_all(bin_dir).expect("bin dir");
    let docker = bin_dir.join("docker");
    write_executable(&docker, "#!/bin/sh\nexit 127\n");
    docker
}

#[test]
fn release_note_bundle_workflow_executes_through_container_and_records_identity() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let note = copy_source_note(&root, &temp.path().join("inputs"));
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    let bin_dir = temp.path().join("bin");
    write_docker_shim(&bin_dir);

    let graph = workflow_graph(&root);
    let path_env = prepend_path(&bin_dir);
    let source_arg = format!("source_note={}", output_path_string(&note));
    let label_arg = "bundle_label=Release Brief".to_string();

    let payload = run_json_with_env(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&runs_dir),
            "--run-id",
            "release-note-bundle",
            "--input",
            &source_arg,
            "--input",
            &label_arg,
        ],
        &root,
        &[("PATH", path_env.as_str())],
    );

    let run_dir = run_dir_from_response(&payload);
    let manifest = read_manifest(&run_dir);
    assert_eq!(manifest["status"], "success");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["source_note"], output_path_string(&note));
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["bundle_label"], "Release Brief");

    let mounted_source = run_dir
        .join("nodes")
        .join("package_bundle")
        .join("inputs")
        .join("prepare_note")
        .join("source");
    assert_eq!(
        fs::read_to_string(&mounted_source).expect("mounted source"),
        "Platform delivery status\n\n- staged the deterministic data pipeline workflow\n- kept retained-run comparison aligned with visible CLI behavior\n- prepared the next container packaging path for review\n"
    );

    let release_note = run_dir
        .join("nodes")
        .join("package_bundle")
        .join("outputs")
        .join("bundle")
        .join("release-note.txt");
    let summary = run_dir
        .join("nodes")
        .join("package_bundle")
        .join("outputs")
        .join("bundle")
        .join("container-summary.json");
    assert!(release_note.exists());
    assert!(summary.exists());

    let release_note_text = fs::read_to_string(&release_note).expect("release note");
    assert!(release_note_text.starts_with("Release Brief\n"));
    assert!(release_note_text.contains("staged the deterministic data pipeline workflow"));

    let summary_json: Value =
        serde_json::from_str(&fs::read_to_string(&summary).expect("summary json"))
            .expect("parse summary");
    assert_eq!(summary_json["bundle_label"], "Release Brief");
    assert_eq!(summary_json["container_workdir"], "/bijux/node/work/scratch");

    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("package_bundle").join("stdout.log"))
            .expect("stdout"),
        "container-package-stdout"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("package_bundle").join("stderr.log"))
            .expect("stderr"),
        "container-package-stderr"
    );

    let trace = read_trace(&run_dir, "package_bundle");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["adapter_id"], "container");
    assert_eq!(trace["container"]["engine"], "docker");
    assert_eq!(trace["container"]["engine_version"], "docker fake 1.0");
    assert_eq!(trace["container"]["image_digest"], "sha256:feedface");
    assert!(
        trace["container"]["image"].as_str().is_some_and(|image| image.contains("@sha256:")),
        "expected the retained trace to keep the pinned image reference"
    );
}

#[test]
fn release_note_bundle_workflow_reports_missing_container_engine_clearly() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let note = copy_source_note(&root, &temp.path().join("inputs"));
    let runs_dir = temp.path().join("runs");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&runs_dir).expect("runs dir");
    write_unavailable_docker_shim(&bin_dir);

    let graph = workflow_graph(&root);
    let source_arg = format!("source_note={}", output_path_string(&note));
    let label_arg = "bundle_label=Release Brief".to_string();
    let isolated_path = prepend_path(&bin_dir);

    let (code, payload, stderr) = run_json_with_env_allow_failure(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&runs_dir),
            "--run-id",
            "release-note-bundle-missing-engine",
            "--input",
            &source_arg,
            "--input",
            &label_arg,
        ],
        &root,
        &[("PATH", &isolated_path)],
    );
    assert!(code == 0 || code == 3, "unexpected command code: {code} stderr={stderr}");

    let run_dir = run_dir_from_response(&payload);
    let manifest = read_manifest(&run_dir);
    assert_eq!(manifest["status"], "failed");

    let summary = &payload["data"]["summary"];
    assert_eq!(summary["status"], "failed");
    assert_eq!(summary["failed_node_reasons"][0]["class"], "infrastructure");
    assert_eq!(summary["failed_node_reasons"][0]["node_id"], "package_bundle");
    assert!(
        summary["failed_node_reasons"][0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("container engine unavailable: docker")),
        "expected run summary to surface the missing engine clearly"
    );

    let trace = read_trace(&run_dir, "package_bundle");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["class"], "infrastructure");
    assert_eq!(trace["failure"]["code"], "CONTAINER_ENGINE_UNAVAILABLE");
    assert_eq!(trace["container"]["engine"], "docker");
    assert!(trace["container"]["engine_version"].is_null());
    assert!(trace["container"]["image_digest"].is_null());
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("package_bundle").join("stderr.log"))
            .expect("stderr"),
        "container engine unavailable: docker"
    );
}
