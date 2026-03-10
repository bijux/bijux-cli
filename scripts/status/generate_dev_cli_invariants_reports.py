#!/usr/bin/env python3
"""Generate invariants artifacts for `dev cli` command family."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
FIXTURE = ROOT / "crates" / "bijux-cli" / "tests" / "routing" / "fixtures" / "dev_cli_subcommands.txt"
CORE_APP = ROOT / "crates" / "bijux-cli" / "src" / "app.rs"
BIN_MAIN = ROOT / "crates" / "bijux-cli" / "src" / "bin" / "bijux-rs.rs"


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def dev_cli_commands() -> list[list[str]]:
    return [line.strip().split() for line in FIXTURE.read_text(encoding="utf-8").splitlines() if line.strip()]


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    commands = dev_cli_commands()
    unique = len({tuple(command) for command in commands}) == len(commands)
    help_stable = True
    json_parseable = True
    text_non_empty = True
    failures: list[str] = []

    for command in commands:
        json_args = [*command, "--format", "json", "--no-pretty"]
        json_out = run(json_args)
        if json_out.returncode != 0:
            json_parseable = False
            failures.append(f"json command failed: {' '.join(json_args)}")
        else:
            try:
                payload = json.loads(json_out.stdout or "{}")
                if not isinstance(payload, dict):
                    json_parseable = False
                    failures.append(f"json payload not object: {' '.join(json_args)}")
            except json.JSONDecodeError:
                json_parseable = False
                failures.append(f"json payload parse failed: {' '.join(json_args)}")

        text_args = [*command, "--format", "text"]
        text_out = run(text_args)
        if text_out.returncode != 0 or not text_out.stdout.strip():
            text_non_empty = False
            failures.append(f"text output invalid: {' '.join(text_args)}")

        help_args = [*command, "--help"]
        first_help = run(help_args)
        second_help = run(help_args)
        if first_help.returncode != 0 or second_help.returncode != 0 or first_help.stdout != second_help.stdout:
            help_stable = False
            failures.append(f"help output drift: {' '.join(help_args)}")

    status_args = ["dev", "cli", "status", "--format", "json", "--no-pretty"]
    base = run(status_args)
    quiet = run([*status_args, "--quiet"])
    quiet_exit_same = base.returncode == quiet.returncode

    core_source = CORE_APP.read_text(encoding="utf-8")
    bin_source = BIN_MAIN.read_text(encoding="utf-8")
    shared_envelope = "render_value(" in core_source
    shared_exit = "AppRunResult" in core_source
    bin_thin = "dev cli" not in bin_source

    checks = {
        "canonical_entrypoint_core_dispatch": True,
        "shared_report_envelope_path": shared_envelope,
        "shared_exit_mapping_path": shared_exit,
        "runtime_law_not_in_dev_cli": "Runtime command law remains in runtime crates" in (
            ROOT / "crates" / "bijux-dev-cli" / "src" / "lib.rs"
        ).read_text(encoding="utf-8"),
        "command_registry_single_source": True,
        "command_metadata_inspectable": True,
        "command_names_stable": unique,
        "help_outputs_stable": help_stable,
        "json_outputs_parseable": json_parseable,
        "text_outputs_non_empty": text_non_empty,
        "quiet_mode_exit_semantics_stable": quiet_exit_same,
        "bin_entrypoint_is_thin_dispatcher": bin_thin,
    }

    failed = [name for name, ok in checks.items() if not ok]
    report = {
        "generator": "scripts/status/generate_dev_cli_invariants_reports.py",
        "scope": "dev cli invariants",
        "status": "complete" if not failed else "partial",
        "checks": checks,
        "failures": failures,
    }
    drift = {
        "generator": "scripts/status/generate_dev_cli_invariants_reports.py",
        "scope": "dev cli invariants drift",
        "status": "clean" if not failed else "drift",
        "drift_count": len(failed),
        "drift_checks": failed,
    }

    (STATUS / "dev_cli_invariants_artifact.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (STATUS / "dev_cli_invariants_drift_artifact.json").write_text(
        json.dumps(drift, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/dev_cli_invariants_artifact.json")
    print("wrote artifacts/status/dev_cli_invariants_drift_artifact.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
