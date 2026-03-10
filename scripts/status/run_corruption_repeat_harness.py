#!/usr/bin/env python3
"""Run repeated corrupted-state command probes and emit a stability report."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_once(args: list[str], env: dict[str, str]) -> dict[str, object]:
    binary = ROOT / "target" / "debug" / "bijux-rs"
    if not binary.exists():
        subprocess.run(
            ["cargo", "build", "-q", "-p", "bijux-cli-core"],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            check=True,
        )
    proc = subprocess.run(
        [str(binary), *args],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    return {
        "code": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
    }


def main() -> None:
    STATUS.mkdir(parents=True, exist_ok=True)

    temp = Path(tempfile.mkdtemp(prefix="bijux-corruption-harness-"))
    try:
        home = temp / "home"
        plugins = temp / "plugins"
        config = temp / "broken.env"
        history = temp / "broken.history"

        (home / ".bijux").mkdir(parents=True, exist_ok=True)
        plugins.mkdir(parents=True, exist_ok=True)
        config.write_text("BROKEN_LINE\n", encoding="utf-8")
        history.write_text("{oops:true}", encoding="utf-8")
        (plugins / "registry.json").write_text("{broken-json", encoding="utf-8")

        base_env = os.environ.copy()
        base_env.update(
            {
                "HOME": str(home),
                "BIJUXCLI_PLUGINS_DIR": str(plugins),
                "BIJUXCLI_HISTORY_FILE": str(history),
            }
        )

        probes = [
            {
                "name": "state_doctor_json_corrupt_config",
                "args": [
                    "dev",
                    "cli",
                    "state-doctor",
                    "--format",
                    "json",
                    "--no-pretty",
                    "--config-path",
                    str(config),
                ],
            },
            {
                "name": "plugin_doctor_json_corrupt_registry",
                "args": ["cli", "plugins", "doctor", "--format", "json", "--no-pretty"],
            },
            {
                "name": "history_json_corrupt_history",
                "args": ["history", "--format", "json", "--no-pretty"],
            },
            {
                "name": "runtime_identity_ambiguous_path",
                "args": ["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"],
            },
        ]

        results: list[dict[str, object]] = []
        for probe in probes:
            if probe["name"] == "runtime_identity_ambiguous_path":
                a = temp / "bin-a"
                b = temp / "bin-b"
                a.mkdir(parents=True, exist_ok=True)
                b.mkdir(parents=True, exist_ok=True)
                (a / "bijux").write_text("#!/bin/sh\n", encoding="utf-8")
                (b / "bijux").write_text("#!/bin/sh\n", encoding="utf-8")
                probe_env = dict(base_env)
                probe_env["PATH"] = f"{a}:{b}:{probe_env.get('PATH', '')}"
            else:
                probe_env = base_env

            first = run_once(probe["args"], probe_env)
            second = run_once(probe["args"], probe_env)
            stable = (
                first["code"] == second["code"]
                and first["stdout"] == second["stdout"]
                and first["stderr"] == second["stderr"]
            )
            results.append(
                {
                    "name": probe["name"],
                    "args": probe["args"],
                    "first": first,
                    "second": second,
                    "stable": stable,
                }
            )

        payload = {
            "generator": "scripts/status/run_corruption_repeat_harness.py",
            "results": results,
            "summary": {
                "stable": sum(1 for row in results if row["stable"]),
                "unstable": sum(1 for row in results if not row["stable"]),
            },
        }

        (STATUS / "repeated_run_corruption_harness.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print("wrote artifacts/status/repeated_run_corruption_harness.json")
    finally:
        shutil.rmtree(temp, ignore_errors=True)


if __name__ == "__main__":
    main()
