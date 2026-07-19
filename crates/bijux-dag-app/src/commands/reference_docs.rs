use crate::commands::{
    command_access_for_path, command_path_hidden_from_public_help, lane_label, CommandAvailability,
    CommandLane,
};
use crate::dag_command;
use clap::Command;
use std::path::Path;

const STABLE_REFERENCE_REL_PATH: &str = "generated-cli-reference.md";
const GATED_REFERENCE_REL_PATH: &str = "gated-command-inventory.md";

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceExample {
    purpose: &'static str,
    command: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StableCommandDoc {
    path: String,
    help: String,
    examples: Vec<ReferenceExample>,
    children: Vec<StableCommandDoc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NonStableCommandDoc {
    path: String,
    lane: CommandLane,
    availability: CommandAvailability,
    opt_in_env: Option<&'static str>,
}

pub(crate) fn render_stable_cli_reference_markdown() -> Result<String, String> {
    let root = dag_command();
    let root_help = render_help(&root)?;
    let commands = stable_commands()?;
    let stable_roots = commands.iter().map(|command| command.path.as_str()).collect::<Vec<_>>();

    let mut markdown = String::new();
    markdown.push_str(
        "---\n\
title: Generated CLI Reference\n\
audience: operators\n\
type: generated-reference\n\
status: canonical\n\
owner: bijux-dag-docs\n\
generated_from: bijux-dag clap help surface\n\
---\n\n\
# Generated CLI Reference\n\n\
This page is generated from the live `bijux-dag` Clap command definitions.\n\
It records the stable `v0.4.0` operator surface exactly as the product presents\n\
it through `bijux-dag --help`. Experimental, simulated, and internal routes are\n\
deliberately excluded from this page and listed separately in\n\
[`gated-command-inventory.md`](gated-command-inventory.md).\n\n\
## Placeholder Conventions\n\n\
- `${GRAPH}`: DAG graph file such as `evidence/workflows/file_processing/dag.json`\n\
- `${GRAPH_A}` and `${GRAPH_B}`: two graph revisions to compare\n\
- `${RUNS_ROOT}`: retained run root such as `artifacts/bijux-dag/runs`\n\
- `${RUN_DIR}`: one retained run directory such as `${RUNS_ROOT}/run-20260708-101500`\n\
- `${RUN_DIR_A}` and `${RUN_DIR_B}`: two retained run directories\n\
- `${RUN_ID}`, `${RUN_ID_A}`, `${RUN_ID_B}`: retained run ids under `${RUNS_ROOT}`\n\
- `${CACHE_DIR}`: cache root such as `artifacts/bijux-dag/cache`\n\
- `${NODE_FINGERPRINT}`: one persisted node fingerprint for cache packing\n\
- `${CACHE_KEY}`, `${CACHE_KEY_A}`, `${CACHE_KEY_B}`: cache keys reported by `cache explain` or `run` output\n\
- `${ARTIFACT_ID}`: artifact id such as `report` or `summary`\n\
- `${DELIVERABLES_ROOT}`: deliverables root such as `artifacts/bijux-dag/deliverables`\n\
- `${DIAGNOSTICS_DIR}`: diagnostics bundle output directory such as `artifacts/bijux-dag/run-diagnostics`\n\n\
## Stable Root Surface\n\n",
    );
    for path in stable_roots {
        markdown.push_str(&format!("- `{path}`\n"));
    }
    markdown.push_str("\n## Root Help\n\n```text\n");
    markdown.push_str(root_help.trim_end());
    markdown.push_str("\n```\n\n");

    for command in &commands {
        render_stable_command(command, 2, &mut markdown);
    }

    Ok(markdown.trim_end().to_string())
}

pub(crate) fn render_gated_command_inventory_markdown() -> Result<String, String> {
    let mut experimental = Vec::new();
    let mut simulated = Vec::new();
    let mut internal = Vec::new();

    for entry in gated_commands() {
        match entry.lane {
            CommandLane::Experimental => experimental.push(entry),
            CommandLane::Simulation => simulated.push(entry),
            CommandLane::Internal => internal.push(entry),
            CommandLane::Stable => {}
        }
    }

    let mut markdown = String::new();
    markdown.push_str(
        "---\n\
title: Gated Command Inventory\n\
audience: operators\n\
type: generated-reference\n\
status: canonical\n\
owner: bijux-dag-docs\n\
generated_from: bijux-dag clap help surface\n\
---\n\n\
# Gated Command Inventory\n\n\
This page is generated from the live `bijux-dag` command tree. It is the\n\
repository-owned inventory for routes that remain outside the stable\n\
`v0.4.0` operator compatibility lane.\n\n\
Stable commands belong in\n\
[`generated-cli-reference.md`](generated-cli-reference.md). This page is\n\
only for deliberate access to experimental, simulated, or internal routes.\n\n",
    );

    render_gated_section(
        &mut markdown,
        "Experimental Routes",
        "Callable by explicit path and repository-tested, but intentionally excluded from the stable public operator surface.",
        &experimental,
    );
    render_gated_section(
        &mut markdown,
        "Simulated Routes",
        "Modeled platform namespaces. Execution requires `BIJUX_DAG_ENABLE_SIMULATED=1` and does not claim production backends.",
        &simulated,
    );
    render_gated_section(
        &mut markdown,
        "Internal Routes",
        "Maintainer-only and contract-only routes. Execution requires `BIJUX_DAG_ENABLE_INTERNAL=1`.",
        &internal,
    );

    Ok(markdown.trim_end().to_string())
}

fn write_cli_reference_docs(interfaces_root: &Path) -> Result<(), String> {
    let stable = render_stable_cli_reference_markdown()?;
    let gated = render_gated_command_inventory_markdown()?;
    std::fs::create_dir_all(interfaces_root)
        .map_err(|err| format!("create {} failed: {err}", interfaces_root.display()))?;
    std::fs::write(interfaces_root.join(STABLE_REFERENCE_REL_PATH), format!("{stable}\n"))
        .map_err(|err| {
            format!(
                "write {} failed: {err}",
                interfaces_root.join(STABLE_REFERENCE_REL_PATH).display()
            )
        })?;
    std::fs::write(interfaces_root.join(GATED_REFERENCE_REL_PATH), format!("{gated}\n")).map_err(
        |err| {
            format!(
                "write {} failed: {err}",
                interfaces_root.join(GATED_REFERENCE_REL_PATH).display()
            )
        },
    )?;
    Ok(())
}

#[doc(hidden)]
pub fn write_checked_in_cli_reference_docs(repo_root: &Path) -> Result<(), String> {
    write_cli_reference_docs(&repo_root.join("docs/bijux-dag/interfaces"))
}

fn render_stable_command(command: &StableCommandDoc, depth: usize, out: &mut String) {
    let subheading = "#".repeat(depth + 1);
    out.push_str(&format!("{} `{}`\n\n", "#".repeat(depth), command.path));
    out.push_str(&format!("{subheading} Examples\n\n"));
    for example in &command.examples {
        out.push_str(&format!("- {}:\n\n```bash\n{}\n```\n\n", example.purpose, example.command));
    }
    out.push_str(&format!("{subheading} Help\n\n```text\n"));
    out.push_str(command.help.trim_end());
    out.push_str("\n```\n\n");

    for child in &command.children {
        render_stable_command(child, depth + 1, out);
    }
}

fn render_gated_section(
    out: &mut String,
    heading: &str,
    summary: &str,
    entries: &[NonStableCommandDoc],
) {
    out.push_str(&format!("## {heading}\n\n{summary}\n\n"));
    out.push_str("| Path | Lane | Availability | Opt-In |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for entry in entries {
        let opt_in = entry.opt_in_env.unwrap_or("-");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            entry.path,
            lane_label(entry.lane),
            availability_label(entry.availability),
            opt_in
        ));
    }
    out.push('\n');
}

fn stable_commands() -> Result<Vec<StableCommandDoc>, String> {
    collect_stable_commands("", &dag_command())
}

fn collect_stable_commands(
    prefix: &str,
    command: &Command,
) -> Result<Vec<StableCommandDoc>, String> {
    let mut collected = Vec::new();
    for subcommand in command.get_subcommands() {
        if is_help_command(subcommand) {
            continue;
        }
        let path = join_path(prefix, subcommand.get_name());
        let access = command_access_for_path(&path);
        if access.lane != CommandLane::Stable || command_path_hidden_from_public_help(&path) {
            continue;
        }
        let examples = stable_examples_for_path(&path)
            .ok_or_else(|| format!("stable command reference is missing examples for `{path}`"))?;
        let help = render_help(subcommand)?;
        let children = collect_stable_commands(&path, subcommand)?;
        collected.push(StableCommandDoc { path, help, examples, children });
    }
    Ok(collected)
}

fn gated_commands() -> Vec<NonStableCommandDoc> {
    let mut entries = Vec::new();
    collect_gated_commands("", &dag_command(), &mut entries);
    entries.sort_by(|left, right| {
        lane_sort_key(left.lane)
            .cmp(&lane_sort_key(right.lane))
            .then_with(|| left.path.cmp(&right.path))
    });
    entries
}

fn collect_gated_commands(prefix: &str, command: &Command, out: &mut Vec<NonStableCommandDoc>) {
    for subcommand in command.get_subcommands() {
        if is_help_command(subcommand) {
            continue;
        }
        let path = join_path(prefix, subcommand.get_name());
        let access = command_access_for_path(&path);
        if access.lane != CommandLane::Stable {
            out.push(NonStableCommandDoc {
                path: path.clone(),
                lane: access.lane,
                availability: access.availability,
                opt_in_env: access.opt_in_env,
            });
        }
        collect_gated_commands(&path, subcommand, out);
    }
}

fn render_help(command: &Command) -> Result<String, String> {
    let mut cloned = command.clone();
    let mut buffer = Vec::new();
    cloned.write_long_help(&mut buffer).map_err(|error| error.to_string())?;
    String::from_utf8(buffer)
        .map(|help| normalize_help_text(&help))
        .map_err(|error| error.to_string())
}

fn normalize_help_text(help: &str) -> String {
    let mut normalized = Vec::new();
    let mut previous_blank = false;
    for line in help.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !previous_blank {
                normalized.push(String::new());
                previous_blank = true;
            }
        } else {
            normalized.push(trimmed.to_string());
            previous_blank = false;
        }
    }
    normalized.join("\n")
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix} {name}")
    }
}

fn is_help_command(command: &Command) -> bool {
    command.get_name() == "help"
}

fn availability_label(availability: CommandAvailability) -> &'static str {
    match availability {
        CommandAvailability::Default => "default",
        CommandAvailability::ExplicitPath => "explicit-path",
        CommandAvailability::OptIn => "opt-in",
    }
}

fn lane_sort_key(lane: CommandLane) -> usize {
    match lane {
        CommandLane::Experimental => 0,
        CommandLane::Simulation => 1,
        CommandLane::Internal => 2,
        CommandLane::Stable => 3,
    }
}

fn stable_examples_for_path(path: &str) -> Option<Vec<ReferenceExample>> {
    Some(match path {
        "artifact" => vec![
            ReferenceExample {
                purpose: "List retained artifacts for one run",
                command: "bijux-dag artifact registry ${RUN_DIR}",
            },
            ReferenceExample {
                purpose: "Emit machine-readable lineage for one retained artifact",
                command: "bijux-dag --json artifact lineage ${RUN_DIR} --artifact-id ${ARTIFACT_ID}",
            },
        ],
        "artifact lineage" => vec![
            ReferenceExample {
                purpose: "Show lineage for every retained artifact in one run",
                command: "bijux-dag artifact lineage ${RUN_DIR}",
            },
            ReferenceExample {
                purpose: "Focus lineage output on one artifact id in JSON",
                command: "bijux-dag --json artifact lineage ${RUN_DIR} --artifact-id ${ARTIFACT_ID}",
            },
        ],
        "artifact promote" => vec![
            ReferenceExample {
                purpose: "Promote one retained artifact into the release deliverables tree",
                command: "bijux-dag artifact promote ${RUN_DIR} ${ARTIFACT_ID} --deliverables-root ${DELIVERABLES_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the promotion result in JSON for automation",
                command: "bijux-dag --json artifact promote ${RUN_DIR} ${ARTIFACT_ID} --deliverables-root ${DELIVERABLES_ROOT} --to release",
            },
        ],
        "artifact registry" => vec![
            ReferenceExample {
                purpose: "Inspect the retained artifact registry for one run",
                command: "bijux-dag artifact registry ${RUN_DIR}",
            },
            ReferenceExample {
                purpose: "Inspect the same registry in JSON",
                command: "bijux-dag --json artifact registry ${RUN_DIR}",
            },
        ],
        "artifact retention" => vec![
            ReferenceExample {
                purpose: "Summarize retained artifact roots under one runs directory",
                command: "bijux-dag artifact retention ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Inspect retained artifact roots in JSON",
                command: "bijux-dag --json artifact retention ${RUNS_ROOT}",
            },
        ],
        "artifact-inspect" => vec![
            ReferenceExample {
                purpose: "Inspect one retained artifact by run directory and artifact id",
                command: "bijux-dag artifact-inspect ${RUN_DIR} ${ARTIFACT_ID}",
            },
            ReferenceExample {
                purpose: "Emit artifact inspection in JSON",
                command: "bijux-dag --json artifact-inspect ${RUN_DIR} ${ARTIFACT_ID}",
            },
        ],
        "cache" => vec![
            ReferenceExample {
                purpose: "Review cache statistics before a cleanup or replay decision",
                command: "bijux-dag cache stats --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Verify cache integrity in machine-readable form",
                command: "bijux-dag --json cache verify --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache diff" => vec![
            ReferenceExample {
                purpose: "Compare two cache entries for drift",
                command: "bijux-dag cache diff --cache-dir ${CACHE_DIR} --key-a ${CACHE_KEY_A} --key-b ${CACHE_KEY_B}",
            },
            ReferenceExample {
                purpose: "Emit the cache diff in JSON",
                command: "bijux-dag --json cache diff --cache-dir ${CACHE_DIR} --key-a ${CACHE_KEY_A} --key-b ${CACHE_KEY_B}",
            },
        ],
        "cache explain" => vec![
            ReferenceExample {
                purpose: "Explain one cache key and the expected adapter contract",
                command: "bijux-dag cache explain --cache-dir ${CACHE_DIR} --key ${CACHE_KEY}",
            },
            ReferenceExample {
                purpose: "Emit the cache explanation in JSON",
                command: "bijux-dag --json cache explain --cache-dir ${CACHE_DIR} --key ${CACHE_KEY}",
            },
        ],
        "cache gc" => vec![
            ReferenceExample {
                purpose: "Run cache garbage collection on one cache root",
                command: "bijux-dag cache gc --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Capture the cleanup result in JSON",
                command: "bijux-dag --json cache gc --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache ls" => vec![
            ReferenceExample {
                purpose: "List cache entries under one cache root",
                command: "bijux-dag cache ls --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "List cache entries in JSON",
                command: "bijux-dag --json cache ls --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache pack" => vec![
            ReferenceExample {
                purpose: "Pack one cache entry for transfer or evidence capture",
                command: "bijux-dag cache pack ${NODE_FINGERPRINT} --out artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Emit the packed-entry metadata in JSON",
                command: "bijux-dag --json cache pack ${NODE_FINGERPRINT} --out artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache prune-simulate" => vec![
            ReferenceExample {
                purpose: "Preview which cache entries a cleanup policy would remove",
                command: "bijux-dag cache prune-simulate --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Review the same cleanup simulation in JSON",
                command: "bijux-dag --json cache prune-simulate --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache stats" => vec![
            ReferenceExample {
                purpose: "Summarize cache occupancy and entry counts",
                command: "bijux-dag cache stats --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Emit cache statistics in JSON",
                command: "bijux-dag --json cache stats --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache unpack" => vec![
            ReferenceExample {
                purpose: "Restore one packed cache entry into a cache root",
                command: "bijux-dag cache unpack artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Capture the unpack result in JSON",
                command: "bijux-dag --json cache unpack artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}",
            },
        ],
        "cache verify" => vec![
            ReferenceExample {
                purpose: "Verify cache integrity before relying on reuse",
                command: "bijux-dag cache verify --cache-dir ${CACHE_DIR}",
            },
            ReferenceExample {
                purpose: "Verify cache integrity and emit JSON",
                command: "bijux-dag --json cache verify --cache-dir ${CACHE_DIR}",
            },
        ],
        "commands" => vec![
            ReferenceExample {
                purpose: "List the stable public operator surface",
                command: "bijux-dag commands",
            },
            ReferenceExample {
                purpose: "Capture the stable command inventory in JSON",
                command: "bijux-dag --json commands",
            },
        ],
        "diff" => vec![
            ReferenceExample {
                purpose: "Compare two retained runs semantically",
                command: "bijux-dag diff ${RUN_DIR_A} ${RUN_DIR_B}",
            },
            ReferenceExample {
                purpose: "Capture an explained semantic diff in JSON",
                command: "bijux-dag --json diff ${RUN_DIR_A} ${RUN_DIR_B} --mode semantic --explain",
            },
        ],
        "doctor" => vec![
            ReferenceExample {
                purpose: "Inspect runtime, cache, and environment readiness",
                command: "bijux-dag doctor",
            },
            ReferenceExample {
                purpose: "Capture the doctor report in JSON",
                command: "bijux-dag --json doctor",
            },
        ],
        "explain" => vec![
            ReferenceExample {
                purpose: "Explain one retained run at a high level",
                command: "bijux-dag explain ${RUN_DIR}",
            },
            ReferenceExample {
                purpose: "Explain one retained node in JSON",
                command: "bijux-dag --json explain ${RUN_DIR} --node publish",
            },
        ],
        "plan" => vec![
            ReferenceExample {
                purpose: "Preview one run layout before execution",
                command: "bijux-dag plan explain ${GRAPH} --out ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Capture planning diagnostics in JSON",
                command: "bijux-dag --json plan diagnostics ${GRAPH}",
            },
        ],
        "plan backfill" => vec![
            ReferenceExample {
                purpose: "Plan a backfill window with explicit partition keys",
                command: "bijux-dag plan backfill --window-start-unix-ms 1711929600000 --window-end-unix-ms 1712016000000 --partition-key tenant=atlas --partition-key region=se",
            },
            ReferenceExample {
                purpose: "Emit the backfill plan in JSON",
                command: "bijux-dag --json plan backfill --window-start-unix-ms 1711929600000 --window-end-unix-ms 1712016000000 --partition-key tenant=atlas",
            },
        ],
        "plan closure" => vec![
            ReferenceExample {
                purpose: "Resolve the upstream and downstream closure for a selected node set",
                command: "bijux-dag plan closure ${GRAPH} --select id:publish",
            },
            ReferenceExample {
                purpose: "Emit the closure result in JSON",
                command: "bijux-dag --json plan closure ${GRAPH} --select id:publish",
            },
        ],
        "plan diagnostics" => vec![
            ReferenceExample {
                purpose: "Inspect planner diagnostics for one graph",
                command: "bijux-dag plan diagnostics ${GRAPH}",
            },
            ReferenceExample {
                purpose: "Capture planner diagnostics in JSON",
                command: "bijux-dag --json plan diagnostics ${GRAPH}",
            },
        ],
        "plan diff" => vec![
            ReferenceExample {
                purpose: "Compare two graph revisions through the planner surface",
                command: "bijux-dag plan diff ${GRAPH_A} ${GRAPH_B}",
            },
            ReferenceExample {
                purpose: "Emit the planner diff in JSON",
                command: "bijux-dag --json plan diff ${GRAPH_A} ${GRAPH_B}",
            },
        ],
        "plan equivalence" => vec![
            ReferenceExample {
                purpose: "Ask whether two graph files still execute the same logical workflow",
                command: "bijux-dag plan equivalence ${GRAPH_A} ${GRAPH_B}",
            },
            ReferenceExample {
                purpose: "Capture the equivalence decision in JSON",
                command: "bijux-dag --json plan equivalence ${GRAPH_A} ${GRAPH_B}",
            },
        ],
        "plan explain" => vec![
            ReferenceExample {
                purpose: "Preview run layout, path bindings, and scheduler estimates",
                command: "bijux-dag plan explain ${GRAPH} --out ${RUNS_ROOT} --run-id tutorial-run",
            },
            ReferenceExample {
                purpose: "Capture the same planning payload in JSON",
                command: "bijux-dag --json plan explain ${GRAPH} --out ${RUNS_ROOT} --run-id tutorial-run",
            },
        ],
        "replay" => vec![
            ReferenceExample {
                purpose: "Replay one retained run into a new output root",
                command: "bijux-dag replay ${RUN_DIR} --out ${RUNS_ROOT}/replay",
            },
            ReferenceExample {
                purpose: "Capture a replay proof request in JSON",
                command: "bijux-dag --json replay ${RUN_DIR} --out ${RUNS_ROOT}/replay --prove",
            },
        ],
        "run" => vec![
            ReferenceExample {
                purpose: "Execute one graph with explicit runtime inputs",
                command: "bijux-dag run ${GRAPH} --out ${RUNS_ROOT} --input source=artifacts/bijux-dag/input.txt --progress compact",
            },
            ReferenceExample {
                purpose: "Run the same workflow in machine-readable mode with streamed progress snapshots",
                command: "bijux-dag --json run ${GRAPH} --out ${RUNS_ROOT} --input source=artifacts/bijux-dag/input.txt --progress compact",
            },
        ],
        "runs" => vec![
            ReferenceExample {
                purpose: "List retained runs under one root",
                command: "bijux-dag runs list --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Summarize retained run history in JSON",
                command: "bijux-dag --json runs summary --root ${RUNS_ROOT}",
            },
        ],
        "runs compare" => vec![
            ReferenceExample {
                purpose: "Compare two retained run ids under one root",
                command: "bijux-dag runs compare ${RUN_ID_A} ${RUN_ID_B} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the retained-run comparison in JSON",
                command: "bijux-dag --json runs compare ${RUN_ID_A} ${RUN_ID_B} --root ${RUNS_ROOT}",
            },
        ],
        "runs diagnostics-bundle" => vec![
            ReferenceExample {
                purpose: "Assemble a diagnostics bundle for one retained run",
                command: "bijux-dag runs diagnostics-bundle ${RUN_ID} --root ${RUNS_ROOT} --out ${DIAGNOSTICS_DIR}",
            },
            ReferenceExample {
                purpose: "Capture diagnostics-bundle metadata in JSON",
                command: "bijux-dag --json runs diagnostics-bundle ${RUN_ID} --root ${RUNS_ROOT} --out ${DIAGNOSTICS_DIR}",
            },
        ],
        "runs diff" => vec![
            ReferenceExample {
                purpose: "Compare two retained run directories inside the runs lane",
                command: "bijux-dag runs diff ${RUN_DIR_A} ${RUN_DIR_B}",
            },
            ReferenceExample {
                purpose: "Emit the runs diff in JSON",
                command: "bijux-dag --json runs diff ${RUN_DIR_A} ${RUN_DIR_B} --mode semantic",
            },
        ],
        "runs doctor" => vec![
            ReferenceExample {
                purpose: "Diagnose one retained run by id",
                command: "bijux-dag runs doctor ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the run diagnosis in JSON",
                command: "bijux-dag --json runs doctor ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs explain-failure" => vec![
            ReferenceExample {
                purpose: "Explain the first causal failure for one retained run",
                command: "bijux-dag runs explain-failure ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Capture the same failure explanation in JSON",
                command: "bijux-dag --json runs explain-failure ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs failures" => vec![
            ReferenceExample {
                purpose: "Aggregate failed node kinds across retained history",
                command: "bijux-dag runs failures --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Capture the failure aggregation in JSON",
                command: "bijux-dag --json runs failures --root ${RUNS_ROOT}",
            },
        ],
        "runs flakes" => vec![
            ReferenceExample {
                purpose: "Find graph fingerprints with mixed retained outcomes",
                command: "bijux-dag runs flakes --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the flake report in JSON",
                command: "bijux-dag --json runs flakes --root ${RUNS_ROOT}",
            },
        ],
        "runs history" => vec![
            ReferenceExample {
                purpose: "Filter retained history by status and selection",
                command: "bijux-dag runs history --root ${RUNS_ROOT} --status failed --select id:publish",
            },
            ReferenceExample {
                purpose: "Capture filtered retained history in JSON",
                command: "bijux-dag --json runs history --root ${RUNS_ROOT} --status failed",
            },
        ],
        "runs id-explain" => vec![
            ReferenceExample {
                purpose: "Explain how one retained run id resolves under a root",
                command: "bijux-dag runs id-explain ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the run-id explanation in JSON",
                command: "bijux-dag --json runs id-explain ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs index" => vec![
            ReferenceExample {
                purpose: "Rebuild or inspect the retained run index under one root",
                command: "bijux-dag runs index --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the run index payload in JSON",
                command: "bijux-dag --json runs index --root ${RUNS_ROOT}",
            },
        ],
        "runs inspect" => vec![
            ReferenceExample {
                purpose: "Inspect one retained run by id",
                command: "bijux-dag runs inspect ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit structured run inspection in JSON",
                command: "bijux-dag --json runs inspect ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs list" => vec![
            ReferenceExample {
                purpose: "Enumerate retained runs under one root",
                command: "bijux-dag runs list --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the run list in JSON",
                command: "bijux-dag --json runs list --root ${RUNS_ROOT}",
            },
        ],
        "runs show" => vec![
            ReferenceExample {
                purpose: "Show compact status and timing for one retained run",
                command: "bijux-dag runs show ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the compact run view in JSON",
                command: "bijux-dag --json runs show ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs stop" => vec![
            ReferenceExample {
                purpose: "Request a stop for one retained run id",
                command: "bijux-dag runs stop ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Capture the stop request response in JSON",
                command: "bijux-dag --json runs stop ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs summary" => vec![
            ReferenceExample {
                purpose: "Summarize retained history under one root",
                command: "bijux-dag runs summary --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the retained summary in JSON",
                command: "bijux-dag --json runs summary --root ${RUNS_ROOT}",
            },
        ],
        "runs timeline" => vec![
            ReferenceExample {
                purpose: "Inspect ordered execution events for one retained run",
                command: "bijux-dag runs timeline ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Filter timeline events and emit them in JSON",
                command: "bijux-dag --json runs timeline ${RUN_ID} --root ${RUNS_ROOT} --node publish",
            },
        ],
        "runs scheduler-checkpoint" => vec![
            ReferenceExample {
                purpose: "Inspect the retained scheduler checkpoint for one run",
                command: "bijux-dag runs scheduler-checkpoint ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the retained scheduler checkpoint in JSON",
                command: "bijux-dag --json runs scheduler-checkpoint ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs tree" => vec![
            ReferenceExample {
                purpose: "Render the node tree for one retained run",
                command: "bijux-dag runs tree ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the retained run tree in JSON",
                command: "bijux-dag --json runs tree ${RUN_ID} --root ${RUNS_ROOT}",
            },
        ],
        "runs trend" => vec![
            ReferenceExample {
                purpose: "Review one analytics point per retained run",
                command: "bijux-dag runs trend --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit the trend report in JSON",
                command: "bijux-dag --json runs trend --root ${RUNS_ROOT}",
            },
        ],
        "runs verify" => vec![
            ReferenceExample {
                purpose: "Verify one retained run by id",
                command: "bijux-dag runs verify ${RUN_ID} --root ${RUNS_ROOT}",
            },
            ReferenceExample {
                purpose: "Emit retained-run verification in JSON",
                command: "bijux-dag --json runs verify ${RUN_ID} --root ${RUNS_ROOT} --strict",
            },
        ],
        "validate" => vec![
            ReferenceExample {
                purpose: "Validate one graph before planning or execution",
                command: "bijux-dag validate ${GRAPH}",
            },
            ReferenceExample {
                purpose: "Emit validation diagnostics in JSON",
                command: "bijux-dag --json validate ${GRAPH}",
            },
        ],
        "verify" => vec![
            ReferenceExample {
                purpose: "Verify one retained run directory directly",
                command: "bijux-dag verify ${RUN_DIR}",
            },
            ReferenceExample {
                purpose: "Capture the verification result in JSON",
                command: "bijux-dag --json verify ${RUN_DIR} --strict",
            },
        ],
        "version" => vec![
            ReferenceExample {
                purpose: "Print the product version and build identity",
                command: "bijux-dag version",
            },
            ReferenceExample {
                purpose: "Emit version identity in JSON",
                command: "bijux-dag --json version",
            },
        ],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        render_gated_command_inventory_markdown, render_stable_cli_reference_markdown,
        stable_commands, stable_examples_for_path, write_cli_reference_docs,
        GATED_REFERENCE_REL_PATH, STABLE_REFERENCE_REL_PATH,
    };

    fn docs_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/bijux-dag/interfaces")
    }

    #[test]
    fn stable_reference_examples_cover_every_stable_command_path() {
        let commands = stable_commands().expect("stable commands");
        let mut stack = commands;
        while let Some(command) = stack.pop() {
            assert!(
                stable_examples_for_path(&command.path).is_some(),
                "missing examples for stable path `{}`",
                command.path
            );
            stack.extend(command.children);
        }
    }

    #[test]
    fn stable_cli_reference_matches_checked_in_generated_reference() {
        let rendered = render_stable_cli_reference_markdown().expect("stable markdown");
        let expected =
            fs::read_to_string(docs_root().join(STABLE_REFERENCE_REL_PATH)).expect("doc file");
        assert_eq!(format!("{rendered}\n"), expected);
    }

    #[test]
    fn gated_inventory_matches_checked_in_generated_reference() {
        let rendered = render_gated_command_inventory_markdown().expect("gated markdown");
        let expected = fs::read_to_string(docs_root().join(GATED_REFERENCE_REL_PATH))
            .expect("inventory doc file");
        assert_eq!(format!("{rendered}\n"), expected);
    }

    #[test]
    fn write_cli_reference_docs_materializes_both_reference_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let interfaces_root = dir.path().join("interfaces");
        write_cli_reference_docs(&interfaces_root).expect("write reference docs");

        let stable = fs::read_to_string(interfaces_root.join(STABLE_REFERENCE_REL_PATH))
            .expect("stable file");
        let gated =
            fs::read_to_string(interfaces_root.join(GATED_REFERENCE_REL_PATH)).expect("gated file");

        assert_eq!(
            stable,
            format!(
                "{}\n",
                render_stable_cli_reference_markdown().expect("render stable markdown")
            )
        );
        assert_eq!(
            gated,
            format!(
                "{}\n",
                render_gated_command_inventory_markdown().expect("render gated markdown")
            )
        );
    }
}
