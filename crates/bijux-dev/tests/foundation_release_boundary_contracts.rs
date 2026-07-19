use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct DagReleaseTruthTable {
    schema_version: String,
    release: String,
    owner: String,
    stable_operator_surface: SurfaceEntry,
    experimental_operator_surface: SurfaceEntry,
    simulated_surface: SurfaceEntry,
    internal_surface: SurfaceEntry,
    unreleased_surface: UnreleasedSurfaceEntry,
}

#[derive(Debug, Deserialize)]
struct SurfaceEntry {
    summary: String,
    root_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UnreleasedSurfaceEntry {
    summary: String,
    capabilities: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_truth_table() -> DagReleaseTruthTable {
    let path = repo_root().join("contracts/foundation/dag_release_truth_table.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

fn dag_root_help() -> String {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--", "--help"])
        .current_dir(repo_root())
        .output()
        .expect("run bijux-dag --help");
    assert!(output.status.success(), "bijux-dag --help failed");
    String::from_utf8(output.stdout).expect("help output must be utf8")
}

fn parse_root_help_commands(help: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let mut in_commands = false;

    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line == "Options:" {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command = trimmed.split_whitespace().next().expect("command token").to_string();
        commands.insert(command);
    }

    commands
}

fn assert_contains_all(text: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(text.contains(needle), "{context} must contain `{needle}`");
    }
}

#[test]
fn dag_release_truth_table_contract_is_current() {
    let truth_table = read_truth_table();
    assert_eq!(truth_table.schema_version, "foundation-dag-release-truth-table/v1");
    assert_eq!(truth_table.release, "v0.4.0");
    assert_eq!(truth_table.owner, "bijux-dag");
    assert!(!truth_table.stable_operator_surface.summary.trim().is_empty());
    assert!(!truth_table.experimental_operator_surface.summary.trim().is_empty());
    assert!(!truth_table.simulated_surface.summary.trim().is_empty());
    assert!(!truth_table.internal_surface.summary.trim().is_empty());
    assert!(!truth_table.unreleased_surface.summary.trim().is_empty());
    assert!(!truth_table.unreleased_surface.capabilities.is_empty());
    assert!(truth_table.simulated_surface.summary.contains("BIJUX_DAG_ENABLE_SIMULATED=1"));
    assert!(truth_table.internal_surface.summary.contains("BIJUX_DAG_ENABLE_INTERNAL=1"));
    assert!(truth_table.simulated_surface.root_commands.contains(&"governance".to_string()));
}

#[test]
fn dag_root_help_matches_stable_release_boundary() {
    let truth_table = read_truth_table();
    let help = dag_root_help();
    let commands = parse_root_help_commands(&help);

    let expected = truth_table
        .stable_operator_surface
        .root_commands
        .into_iter()
        .chain(std::iter::once("help".to_string()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        commands, expected,
        "visible bijux-dag --help command inventory drifted from the stable release boundary"
    );

    assert_contains_all(
        &help,
        &[
            "v0.4.0 surface truth table:",
            "stable: validate, plan, run, replay, runs ..., artifact, artifact-inspect, diff, explain, verify, doctor, cache, version, commands",
            "commands --lane experimental",
            "commands --lane simulated",
            "commands --lane internal",
            "BIJUX_DAG_ENABLE_SIMULATED=1",
            "BIJUX_DAG_ENABLE_INTERNAL=1",
            "Use `bijux-dag commands` for the stable operator surface",
        ],
        "bijux-dag --help",
    );
}

#[test]
fn dag_release_boundary_docs_and_examples_stay_honest() {
    let readme = read_repo_file("README.md");
    let handbook = read_repo_file("docs/bijux-dag/index.md");
    let cli_surface = read_repo_file("docs/bijux-dag/interfaces/cli-surface.md");
    let release_boundary = read_repo_file("docs/bijux-dag/foundation/release-boundary.md");

    assert_contains_all(
        &readme,
        &[
            "### `bijux-dag` v0.4.0 Surface Truth Table",
            "contracts/foundation/dag_release_truth_table.v1.json",
            "BIJUX_DAG_ENABLE_SIMULATED=1",
            "BIJUX_DAG_ENABLE_INTERNAL=1",
        ],
        "README.md",
    );
    assert_contains_all(
        &handbook,
        &[
            "## v0.4.0 Surface Truth Table",
            "[Release Boundary](foundation/release-boundary.md)",
            "../bijux-core/foundation/package-boundary.md",
        ],
        "docs/bijux-dag/index.md",
    );
    assert_contains_all(
        &cli_surface,
        &[
            "## v0.4.0 Surface Truth Table",
            "../foundation/release-boundary.md",
            "generated-cli-reference.md",
            "gated-command-inventory.md",
            "BIJUX_DAG_ENABLE_SIMULATED=1",
            "BIJUX_DAG_ENABLE_INTERNAL=1",
        ],
        "docs/bijux-dag/interfaces/cli-surface.md",
    );
    assert_contains_all(
        &release_boundary,
        &[
            "| stable |",
            "| experimental |",
            "| simulated |",
            "| internal |",
            "| unreleased |",
            "BIJUX_DAG_ENABLE_SIMULATED=1",
            "BIJUX_DAG_ENABLE_INTERNAL=1",
            "contracts/foundation/workspace_package_boundary.v1.json",
            "../../bijux-core/foundation/package-boundary.md",
        ],
        "docs/bijux-dag/foundation/release-boundary.md",
    );

    let package_index = read_repo_file("docs/bijux-dag/packages/index.md");
    assert_contains_all(
        &package_index,
        &[
            "../../bijux-core/foundation/package-boundary.md",
            "contracts/foundation/workspace_package_boundary.v1.json",
        ],
        "docs/bijux-dag/packages/index.md",
    );

    for path in [
        "docs/bijux-dag/interfaces/entrypoints-and-examples.md",
        "docs/bijux-dag/interfaces/operator-workflows.md",
        "docs/bijux-dag/operations/installation-and-setup.md",
        "docs/bijux-dag/operations/common-workflows.md",
        "docs/bijux-dag/operations/failure-recovery.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            !content.contains("bijux-dag status"),
            "{path} must not advertise hidden experimental `status` as a stable example"
        );
        assert!(
            !content.contains("bijux-dag inspect"),
            "{path} must not advertise a nonexistent root `inspect` command"
        );
    }

    let recipes = read_repo_file("docs/bijux-dag/interfaces/runnable-examples.md");
    assert_contains_all(
        &recipes,
        &["experimental explicit-path routes", "bijux-dag explain --json ${RUN_DIR}"],
        "docs/bijux-dag/interfaces/runnable-examples.md",
    );

    let release_binary = read_repo_file("docs/spec/RELEASE_BINARY_VERIFICATION.md");
    assert_contains_all(
        &release_binary,
        &[
            "internal probe (`capabilities`)",
            "BIJUX_DAG_ENABLE_INTERNAL=1 bijux-dag capabilities --json",
            "bijux-dag explain --json ${RUN_DIR}",
        ],
        "docs/spec/RELEASE_BINARY_VERIFICATION.md",
    );

    let support_matrix = read_repo_file("docs/bijux-dag/interfaces/support-matrix.md");
    assert_contains_all(
        &support_matrix,
        &[
            "| `commands` | stable | visible CLI | route inventory for stable and non-stable command discovery |",
            "`commands` defaults to the stable lane",
            "`--lane experimental`",
            "`--lane simulated`",
            "`--lane internal`",
            "do not promote those commands into the stable operator promise",
            "| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |",
        ],
        "docs/bijux-dag/interfaces/support-matrix.md",
    );

    let first_run = read_repo_file("docs/bijux-dag/operations/first-run-tutorial.md");
    assert_contains_all(
        &first_run,
        &[
            "cargo run -p bijux-dag-cli --bin bijux-dag -- commands",
            "Maintainer-only probes such as `capabilities` remain outside this",
        ],
        "docs/bijux-dag/operations/first-run-tutorial.md",
    );
    assert!(
        !first_run.contains("cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json"),
        "first-run tutorial must not present `capabilities --json` as part of the public operator lane"
    );

    let ci = read_repo_file("docs/bijux-dag/operations/ci-integration.md");
    assert_contains_all(
        &ci,
        &[
            "cargo run -p bijux-dag-cli --bin bijux-dag -- commands",
            "BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json",
            "not part of the public operator boundary",
        ],
        "docs/bijux-dag/operations/ci-integration.md",
    );
}

#[test]
fn release_boundary_references_route_readers_to_handbook_pages() {
    for path in [
        "docs/bijux-cli/interfaces/entrypoints-and-examples.md",
        "docs/bijux-cli/interfaces/command-examples.md",
        "docs/bijux-cli/operations/migration-guide.md",
        "docs/bijux-dag/interfaces/runnable-examples.md",
        "docs/bijux-dag/interfaces/support-matrix.md",
        "docs/bijux-dag/operations/ci-integration.md",
        "docs/bijux-dag/operations/first-run-tutorial.md",
        "docs/spec/RELEASE_BINARY_VERIFICATION.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            !content.contains("](../../../contracts/foundation/dag_release_truth_table.v1.json)")
                && !content
                    .contains("](../../../../contracts/foundation/dag_release_truth_table.v1.json)")
                && !content.contains("](../../contracts/foundation/dag_release_truth_table.v1.json)"),
            "{path} must not link handbook readers directly to the raw release truth-table contract"
        );
        assert!(
            content.contains("release-boundary.md"),
            "{path} must route release-boundary readers to a handbook page"
        );
        assert!(
            content.contains("contracts/foundation/dag_release_truth_table.v1.json"),
            "{path} must still name the machine-readable release truth-table contract"
        );
    }
}

#[test]
fn dag_operator_reference_docs_use_public_binary_examples() {
    for path in [
        "docs/bijux-dag/interfaces/generated-cli-reference.md",
        "docs/bijux-dag/interfaces/configuration-surface.md",
        "docs/bijux-dag/interfaces/node-inspection.md",
        "docs/bijux-dag/interfaces/gated-command-inventory.md",
        "docs/bijux-dag/interfaces/operator-command-index.md",
        "docs/bijux-dag/operations/failure-recovery.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            !content.contains("`dag "),
            "{path} must use `bijux-dag ...` in public command examples"
        );
    }
}

#[test]
fn dag_interface_indexes_link_generated_cli_references() {
    let handbook = read_repo_file("docs/bijux-dag/index.md");
    assert_contains_all(
        &handbook,
        &[
            "interfaces/graph-schema.md",
            "interfaces/run-evidence-layout.md",
            "interfaces/generated-cli-reference.md",
            "interfaces/gated-command-inventory.md",
        ],
        "docs/bijux-dag/index.md",
    );

    let interfaces = read_repo_file("docs/bijux-dag/interfaces/index.md");
    assert_contains_all(
        &interfaces,
        &[
            "Graph Schema Reference",
            "Run Evidence Layout",
            "Generated CLI Reference",
            "Authoring Guide",
            "Gated Command Inventory",
        ],
        "docs/bijux-dag/interfaces/index.md",
    );

    let entrypoints = read_repo_file("docs/bijux-dag/interfaces/entrypoints-and-examples.md");
    assert_contains_all(
        &entrypoints,
        &["Generated CLI Reference", "Gated Command Inventory"],
        "docs/bijux-dag/interfaces/entrypoints-and-examples.md",
    );
}

#[test]
fn backend_release_docs_keep_shipped_lanes_and_future_lanes_separate() {
    let release_boundary = read_repo_file("docs/bijux-dag/foundation/release-boundary.md");
    let support_matrix = read_repo_file("docs/bijux-dag/interfaces/support-matrix.md");

    assert_contains_all(
        &release_boundary,
        &[
            "Kubernetes Job submission through `run --backend kubernetes`",
            "shared-filesystem SLURM submission through `run --backend slurm`",
            "generic hpc execution beyond the shared-filesystem slurm lane, public remote workers, public enterprise or federation APIs, full scheduler service",
        ],
        "docs/bijux-dag/foundation/release-boundary.md",
    );

    assert_contains_all(
        &support_matrix,
        &[
            "| `run --backend slurm` on a shared filesystem | stable | visible `run` surface with explicit backend selection | submits through `sbatch`, polls `sacct`, and records retained batch evidence when the scheduled worker can reopen the same run directory |",
            "| `run --backend kubernetes` for container nodes | stable | visible `run` surface with explicit backend selection | requires `--kubernetes-volume-claim`, `--kubernetes-shared-root`, and a shared persistent volume claim mounted into Job pods |",
            "| Generic HPC beyond the shared-filesystem SLURM lane, public remote workers, full scheduler service | unreleased | not part of first-run adoption | broader portability and distributed control are not a `v0.4.0` product promise |",
        ],
        "docs/bijux-dag/interfaces/support-matrix.md",
    );
}

#[test]
fn dag_run_evidence_layout_reference_covers_retained_surfaces() {
    let layout = read_repo_file("docs/bijux-dag/interfaces/run-evidence-layout.md");
    assert_contains_all(
        &layout,
        &[
            "manifest.json",
            "graph.snapshot.json",
            "outputs/index.json",
            "nodes/<node_id>/trace.json",
            "nodes/<node_id>/inputs/index.json",
            "nodes/<node_id>/outputs/index.json",
            "run.log.jsonl",
            "observability.timeline.json",
            "plan.json",
            "cache_mode",
            "cache_dir",
            "meta.json",
            "promotions/index.json",
            "run_summary.promoted_outputs",
            "crates/bijux-dag-app/tests/snapshots/run_dir_hello.json",
        ],
        "docs/bijux-dag/interfaces/run-evidence-layout.md",
    );

    let artifact_contracts = read_repo_file("docs/bijux-dag/interfaces/artifact-contracts.md");
    assert_contains_all(
        &artifact_contracts,
        &["run-evidence-layout.md", "cache-entry manifests", "promotion ledgers"],
        "docs/bijux-dag/interfaces/artifact-contracts.md",
    );

    let artifact_package = read_repo_file("docs/bijux-dag/packages/bijux-dag-artifacts.md");
    assert_contains_all(
        &artifact_package,
        &["../interfaces/run-evidence-layout.md", "../interfaces/artifact-contracts.md"],
        "docs/bijux-dag/packages/bijux-dag-artifacts.md",
    );
}

#[test]
fn dag_graph_schema_reference_covers_live_authoring_surface() {
    let graph_schema = read_repo_file("docs/bijux-dag/interfaces/graph-schema.md");
    assert_contains_all(
        &graph_schema,
        &[
            "configs/dag/schema/dag.schema.json",
            "crates/bijux-dag-core/src/graph/model.rs",
            "`graph.inputs`",
            "## Node Fields",
            "`graph_input`",
            "`node_output`",
            "`path_var`",
            "`resources`",
            "`retry`",
            "`timeout_ms`",
            "`cache`",
            "`env_allowlist`",
            "`branch`",
            "`subgraphs`",
            "`subgraph_instances`",
            "Validation diagnostics always carry",
        ],
        "docs/bijux-dag/interfaces/graph-schema.md",
    );

    let dag_schema = read_repo_file("configs/dag/schema/dag.schema.json");
    assert_contains_all(
        &dag_schema,
        &[
            "\"subgraphs\"",
            "\"subgraph_instances\"",
            "\"dynamic\"",
            "\"gpu_devices\"",
            "\"named_resources\"",
            "\"promotable\"",
        ],
        "configs/dag/schema/dag.schema.json",
    );
}

#[test]
fn dag_registry_and_root_cli_docs_preserve_public_binary_identity() {
    let registry: serde_json::Value =
        serde_json::from_str(&read_repo_file("contracts/official_product_namespace_registry.json"))
            .expect("official product registry json");
    let dag = registry["entries"]
        .as_array()
        .expect("registry entries")
        .iter()
        .find(|entry| entry["namespace"] == "dag")
        .expect("dag registry entry");
    assert_eq!(dag["runtime_binary"], "bijux-dag");
    assert_eq!(dag["runtime_package"], "bijux-dag-cli");

    let examples = read_repo_file("docs/bijux-cli/interfaces/command-examples.md");
    assert_contains_all(&examples, &["bijux-dag --help", "bijux apps which dag"], "examples.md");
    assert!(
        !examples.contains("bijux dag --help"),
        "root CLI examples must not present `bijux dag --help` as the public DAG operator surface"
    );

    let migration = read_repo_file("docs/bijux-cli/operations/migration-guide.md");
    assert_contains_all(
        &migration,
        &[
            "use `bijux-dag ...` for the public DAG command surface",
            "use `bijux dag ...` when you intentionally want root-managed app routing",
            "`bijux-workflow`",
        ],
        "migration-guide.md",
    );
    assert!(
        !migration.contains("`bijux-dag ...` -> `bijux dag ...`"),
        "migration guide must not rewrite the public DAG binary into the routed root namespace"
    );
}

#[test]
fn root_cli_help_for_dag_points_back_to_the_public_binary() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-cli", "--", "help", "dag"])
        .current_dir(repo_root())
        .output()
        .expect("run bijux help dag");
    assert!(output.status.success(), "bijux help dag failed");
    let stdout = String::from_utf8(output.stdout).expect("help output must be utf8");
    assert_contains_all(
        &stdout,
        &[
            "Official app help: Bijux DAG",
            "root route: bijux dag <command> ...",
            "product binary: bijux-dag",
            "cargo install bijux-dag-cli",
            "bijux-dag --help",
        ],
        "bijux help dag",
    );
}
