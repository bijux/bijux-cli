#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0

"""Capture current Python CLI behavior into reproducible artifacts."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import textwrap


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts" / "python-behavior"
GOLDEN = OUT / "golden"
RUNTIME = OUT / "runtime"
SANDBOX = RUNTIME / "sandbox"


def _run_case(
    name: str,
    argv: list[str],
    *,
    base_env: dict[str, str],
    stdin: str | None = None,
    env_extra: dict[str, str] | None = None,
) -> dict[str, object]:
    env = dict(base_env)
    if env_extra:
        env.update(env_extra)
    proc = subprocess.run(
        [str(ROOT / "bin" / "bijux"), *argv],
        input=stdin,
        text=True,
        capture_output=True,
        env=env,
        cwd=ROOT,
        check=False,
    )
    rec: dict[str, object] = {
        "name": name,
        "argv": ["bijux", *argv],
        "exit_code": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "env_overrides": {
            k: env[k]
            for k in (
                "BIJUXCLI_CONFIG",
                "BIJUXCLI_HISTORY_FILE",
                "BIJUXCLI_PLUGINS_DIR",
                "NO_COLOR",
            )
            if k in env
        },
    }
    (GOLDEN / f"{name}.json").write_text(json.dumps(rec, indent=2), encoding="utf-8")
    return rec


def main() -> int:
    GOLDEN.mkdir(parents=True, exist_ok=True)
    RUNTIME.mkdir(parents=True, exist_ok=True)

    if SANDBOX.exists():
        shutil.rmtree(SANDBOX)

    (SANDBOX / "home").mkdir(parents=True)
    (SANDBOX / "plugins").mkdir(parents=True)
    (SANDBOX / "tmp").mkdir(parents=True)

    config_path = SANDBOX / "home" / ".bijux" / "config.env"
    history_path = SANDBOX / "home" / ".bijux" / "history.json"
    plugins_dir = SANDBOX / "plugins"
    config_path.parent.mkdir(parents=True, exist_ok=True)

    base_env = os.environ.copy()
    base_env.update(
        {
            "BIJUXCLI_CONFIG": str(config_path),
            "BIJUXCLI_HISTORY_FILE": str(history_path),
            "BIJUXCLI_PLUGINS_DIR": str(plugins_dir),
            "HOME": str(SANDBOX / "home"),
            "NO_COLOR": "1",
        }
    )

    captures: dict[str, dict[str, object]] = {}

    # Golden command outputs
    captures["bijux_help"] = _run_case("bijux_help", ["--help"], base_env=base_env)
    captures["bijux_version"] = _run_case(
        "bijux_version", ["version"], base_env=base_env
    )
    captures["bijux_doctor"] = _run_case("bijux_doctor", ["doctor"], base_env=base_env)
    captures["bijux_status_text"] = _run_case(
        "bijux_status_text", ["status"], base_env=base_env
    )
    captures["bijux_status_json_no_pretty"] = _run_case(
        "bijux_status_json_no_pretty",
        ["status", "-f", "json", "--no-pretty"],
        base_env=base_env,
    )
    captures["bijux_status_yaml_pretty"] = _run_case(
        "bijux_status_yaml_pretty",
        ["status", "-f", "yaml", "--pretty"],
        base_env=base_env,
    )
    captures["bijux_plugins_list"] = _run_case(
        "bijux_plugins_list", ["plugins", "list"], base_env=base_env
    )
    captures["bijux_config_root"] = _run_case(
        "bijux_config_root", ["config"], base_env=base_env
    )
    captures["bijux_history_root"] = _run_case(
        "bijux_history_root", ["history"], base_env=base_env
    )
    captures["bijux_dev_help"] = _run_case(
        "bijux_dev_help", ["dev", "--help"], base_env=base_env
    )

    # Behavior captures
    captures["behavior_success_streams"] = _run_case(
        "behavior_success_streams", ["version"], base_env=base_env
    )
    captures["behavior_validation_failure_streams"] = _run_case(
        "behavior_validation_failure_streams",
        ["status", "--format", "toml"],
        base_env=base_env,
    )
    captures["behavior_internal_failure_streams"] = _run_case(
        "behavior_internal_failure_streams",
        ["dev", "di"],
        base_env=base_env,
        env_extra={"BIJUXCLI_TEST_FORCE_SERIALIZE_FAIL": "1"},
    )
    captures["behavior_quiet_mode"] = _run_case(
        "behavior_quiet_mode", ["plugins", "list", "--quiet"], base_env=base_env
    )
    captures["behavior_debug_log_level"] = _run_case(
        "behavior_debug_log_level",
        ["plugins", "list", "--log-level", "debug"],
        base_env=base_env,
    )
    captures["behavior_help_short_circuit"] = _run_case(
        "behavior_help_short_circuit",
        ["--help", "--format", "toml"],
        base_env=base_env,
    )
    captures["behavior_repl_startup_piped"] = _run_case(
        "behavior_repl_startup_piped", ["repl"], base_env=base_env, stdin="exit\n"
    )

    # Interactive REPL startup capture via pseudo-terminal helper.
    interactive_file = RUNTIME / "repl-interactive.txt"
    cmd = "script -q {out} ./bin/bijux repl <<'EOT'\nexit\nEOT".format(
        out=interactive_file
    )
    proc = subprocess.run(
        ["zsh", "-lc", cmd],
        cwd=ROOT,
        env=base_env,
        capture_output=True,
        text=True,
        check=False,
    )
    captures["behavior_repl_startup_interactive"] = {
        "name": "behavior_repl_startup_interactive",
        "argv": ["bijux", "repl"],
        "exit_code": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "transcript_file": str(interactive_file),
    }

    # Plugin lifecycle captures (install/check/uninstall).
    plug_name = "capture_plugin"
    plug_dir = SANDBOX / "tmp" / plug_name
    plug_dir.mkdir(parents=True, exist_ok=True)
    (plug_dir / "plugin.py").write_text(
        textwrap.dedent(
            """\
            class Plugin:
                name = "capture_plugin"
                version = "0.1.0"
                requires_cli_version = ">=0.1.0"

                def cli(self):
                    return None

            def health(_di):
                return True
            """
        ),
        encoding="utf-8",
    )
    (plug_dir / "plugin.json").write_text(
        json.dumps(
            {
                "name": plug_name,
                "version": "0.1.0",
                "bijux_cli_version": ">=0.1.0",
                "enabled": True,
                "schema_version": "1",
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    captures["behavior_plugins_install"] = _run_case(
        "behavior_plugins_install", ["plugins", "install", str(plug_dir)], base_env=base_env
    )
    captures["behavior_plugins_check"] = _run_case(
        "behavior_plugins_check", ["plugins", "check", plug_name], base_env=base_env
    )
    captures["behavior_plugins_uninstall"] = _run_case(
        "behavior_plugins_uninstall", ["plugins", "uninstall", plug_name], base_env=base_env
    )

    # Config discovery precedence captures.
    config_path.write_text(
        "BIJUXCLI_SAMPLE_KEY=from_config\nBIJUXCLI_LOG_LEVEL=warning\n", encoding="utf-8"
    )
    captures["behavior_config_precedence_config_only"] = _run_case(
        "behavior_config_precedence_config_only",
        ["config", "get", "sample_key"],
        base_env=base_env,
    )
    captures["behavior_config_precedence_env_override"] = _run_case(
        "behavior_config_precedence_env_override",
        ["config", "get", "sample_key"],
        base_env=base_env,
        env_extra={"BIJUXCLI_SAMPLE_KEY": "from_env"},
    )
    captures["behavior_config_precedence_cli_override"] = _run_case(
        "behavior_config_precedence_cli_override",
        ["plugins", "list", "--log-level", "debug"],
        base_env=base_env,
        env_extra={"BIJUXCLI_LOG_LEVEL": "warning"},
    )

    (OUT / "behavior" / "capture-index.json").parent.mkdir(parents=True, exist_ok=True)
    (OUT / "behavior" / "capture-index.json").write_text(
        json.dumps({"captured": list(captures.keys())}, indent=2), encoding="utf-8"
    )

    captured_at = subprocess.check_output(
        ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True
    ).strip()
    lock = {
        "schema_version": "1",
        "captured_at": captured_at,
        "environment": {
            "python": sys.version.split()[0],
            "platform": sys.platform,
            "sandbox_home": str(SANDBOX / "home"),
        },
        "captures": captures,
    }
    (ROOT / "artifacts" / "current-python-behavior-lock.json").write_text(
        json.dumps(lock, indent=2), encoding="utf-8"
    )

    print("Captured", len(captures), "behavior records")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
