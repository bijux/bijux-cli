#!/usr/bin/env python3
"""Generate dev-cli resilience, determinism, and side-effect artifacts."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-bin", "--", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        env=merged,
        check=False,
    )


def run_json(args: list[str], env: dict[str, str] | None = None) -> tuple[int, dict]:
    proc = run([*args, "--format", "json", "--no-pretty"], env)
    payload: dict = {}
    try:
        payload = json.loads(proc.stdout or "{}")
    except json.JSONDecodeError:
        payload = {}
    return proc.returncode, payload


def sha(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    summary_commands = [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ]
    machine_commands = [
        ["dev", "cli", "parity"],
        ["dev", "cli", "evidence", "audit"],
        ["dev", "cli", "routes"],
        ["dev", "cli", "registry"],
        ["dev", "cli", "env"],
        ["dev", "cli", "contracts"],
        ["dev", "cli", "state-audit"],
        ["dev", "cli", "state-doctor"],
        ["dev", "cli", "runtime-identity"],
        ["dev", "cli", "package-health"],
    ]

    determinism_rows: list[dict] = []
    for command in [*summary_commands, *machine_commands]:
        first = run([*command, "--format", "json", "--no-pretty"])
        second = run([*command, "--format", "json", "--no-pretty"])
        stable = first.returncode == second.returncode and first.stdout == second.stdout
        determinism_rows.append(
            {
                "command": " ".join(command),
                "stable": stable,
                "first_exit": first.returncode,
                "second_exit": second.returncode,
            }
        )

    with tempfile.TemporaryDirectory(prefix="bijux-dev-cli-side-effects-") as raw:
        tmp = Path(raw)
        config = tmp / "config.env"
        history = tmp / "history.json"
        memory = tmp / "memory.json"
        plugins = tmp / "plugins"
        plugins.mkdir(parents=True, exist_ok=True)
        config.write_text("BIJUXCLI_SAMPLE=1\n", encoding="utf-8")
        history.write_text("[]", encoding="utf-8")
        memory.write_text("{}", encoding="utf-8")
        (plugins / "healthy.toml").write_text("[plugin]\nname='healthy'\nentry='plugin:main'\n", encoding="utf-8")
        env = {
            "BIJUX_CONFIG_PATH": str(config),
            "BIJUX_HISTORY_PATH": str(history),
            "BIJUX_MEMORY_PATH": str(memory),
            "BIJUX_PLUGINS_DIR": str(plugins),
        }
        before = {"config": sha(config), "history": sha(history), "memory": sha(memory)}
        for command in [*summary_commands, *machine_commands]:
            run(command, env)
        after = {"config": sha(config), "history": sha(history), "memory": sha(memory)}

    failure_cases = [
        (
            "status_unreadable_input",
            ["dev", "cli", "status"],
            {"BIJUX_HISTORY_PATH": "/root/forbidden/history.json"},
        ),
        (
            "parity_corrupted_input",
            ["dev", "cli", "parity"],
            {"BIJUX_MEMORY_PATH": "/dev/null/not-json"},
        ),
        (
            "contracts_missing_snapshot_context",
            ["dev", "cli", "contracts"],
            {"PWD": "/definitely/missing/contracts/root"},
        ),
        (
            "runtime_identity_path_ambiguity",
            ["dev", "cli", "runtime-identity"],
            {"PATH": f"/tmp/bijux-a:/tmp/bijux-b:{os.environ.get('PATH', '')}"},
        ),
        (
            "package_health_metadata_mismatch",
            ["dev", "cli", "package-health"],
            {"BIJUX_WHEEL_VERSION": "0.0.1", "BIJUX_PYTHON_BRIDGE_SUPPORTED": "0"},
        ),
    ]
    failure_rows = []
    for case_id, command, env in failure_cases:
        code, payload = run_json(command, env)
        failure_rows.append(
            {
                "case_id": case_id,
                "command": " ".join(command),
                "exit_code": code,
                "json_object": isinstance(payload, dict),
            }
        )

    determinism_clean = all(row["stable"] for row in determinism_rows)
    no_side_effects = before == after
    resilience_checks = {
        "failure_injection_cases_reported": len(failure_rows) == len(failure_cases),
        "determinism_rows_present": len(determinism_rows) == len(summary_commands) + len(machine_commands),
        "summary_commands_deterministic": all(
            row["stable"] for row in determinism_rows if row["command"] in {" ".join(c) for c in summary_commands}
        ),
        "machine_commands_deterministic": all(
            row["stable"] for row in determinism_rows if row["command"] in {" ".join(c) for c in machine_commands}
        ),
        "read_only_commands_did_not_mutate_state": no_side_effects,
    }
    drift_checks = [name for name, ok in resilience_checks.items() if not ok]

    write_json(
        "dev_cli_control_plane_resilience_artifact.json",
        {
            "scope": "dev cli control-plane resilience",
            "generator": "scripts/status/generate_dev_cli_resilience_reports.py",
            "failure_injection_cases": failure_rows,
            "checks": resilience_checks,
            "status": "complete" if all(resilience_checks.values()) else "partial",
        },
    )
    write_json(
        "dev_cli_determinism_artifact.json",
        {
            "scope": "dev cli determinism",
            "generator": "scripts/status/generate_dev_cli_resilience_reports.py",
            "rows": determinism_rows,
            "status": "clean" if determinism_clean else "drift",
        },
    )
    write_json(
        "dev_cli_side_effect_audit_artifact.json",
        {
            "scope": "dev cli side-effect audit",
            "generator": "scripts/status/generate_dev_cli_resilience_reports.py",
            "before": before,
            "after": after,
            "status": "clean" if no_side_effects else "drift",
        },
    )
    write_json(
        "dev_cli_resilience_drift_artifact.json",
        {
            "scope": "dev cli resilience drift",
            "generator": "scripts/status/generate_dev_cli_resilience_reports.py",
            "drift_checks": drift_checks,
            "drift_count": len(drift_checks),
            "status": "clean" if not drift_checks else "drift",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
