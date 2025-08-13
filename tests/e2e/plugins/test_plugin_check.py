# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin check command."""

from __future__ import annotations

import json
import os
from pathlib import Path

import yaml

from tests.e2e.conftest import TEST_TEMPLATE, last_json_with, run_cli


def test_plugin_check_ok(tmp_path: Path) -> None:
    """Test a successful health check on a valid plugin."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "healthplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0, scf.stdout
    plug_py = next((tmp_path / "healthplug").glob("**/plugin.py"))
    plug_py.write_text(plug_py.read_text() + "\ndef health(di):\n    return True\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "healthplug")], env=env)
    assert ins.returncode == 0, ins.stdout
    res = run_cli(["plugins", "check", "healthplug", "--format", "json"], env=env)
    assert res.returncode == 0, res.stdout
    data = json.loads(res.stdout)
    assert data.get("status") == "healthy"


def test_plugin_check_nonexistent() -> None:
    """Test that checking a non-existent plugin fails correctly."""
    res = run_cli(["plugins", "check", "missing"])
    assert res.returncode != 0
    data = last_json_with(res.stderr, "error", "plugin")
    assert data.get("plugin") == "missing"
    assert "not found" in data.get("error", "").lower()


def test_plugin_check_no_health_hook(tmp_path: Path) -> None:
    """Test checking a plugin that is missing the health() hook."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "nohealth",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    plug_py = next((tmp_path / "nohealth").glob("**/plugin.py"))
    content = plug_py.read_text().replace("def health", "def _no_health")
    plug_py.write_text(content)
    ins = run_cli(["plugins", "install", str(tmp_path / "nohealth")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "nohealth"], env=env)
    assert res.returncode != 0


def test_plugin_check_unhealthy(tmp_path: Path) -> None:
    """Test checking a plugin that reports an unhealthy status."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "badhealth",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "badhealth").glob("**/plugin.py"))
    plug_py.write_text(plug_py.read_text() + "\ndef health(di):\n    return False\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "badhealth")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "badhealth"], env=env)
    assert res.returncode != 0


def test_plugin_check_yaml(tmp_path: Path) -> None:
    """Test that the check command works with YAML output format."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "healthyml",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "healthyml")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "healthyml", "--format", "yaml"], env=env)
    data = yaml.safe_load(res.stdout)
    assert data.get("status") == "healthy"


def test_plugin_check_quiet(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses output."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkquiet",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "chkquiet")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "chkquiet", "--quiet"], env=env)
    assert res.stdout.strip() == ""


def test_plugin_check_debug(tmp_path: Path) -> None:
    """Test the check command with the --debug flag."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkdebug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "chkdebug")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "chkdebug", "--debug"], env=env)
    assert res.returncode == 0


def test_plugin_check_invalid_output_format(tmp_path: Path) -> None:
    """Test that an invalid format value fails."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "fmtfail",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "fmtfail")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "fmtfail", "--format", "foobar"], env=env)
    assert res.returncode != 0


def test_plugin_check_permission_denied(tmp_path: Path) -> None:
    """Test a graceful failure when a plugin file is not readable."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "permchk",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_dir = tmp_path / "permchk"
    plug_py = next(plug_dir.glob("**/plugin.py"))
    os.chmod(plug_py, 0o000)
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    res = run_cli(["plugins", "check", "permchk"], env=env)
    assert res.returncode != 0 or "error" in res.stdout.lower()
    os.chmod(plug_py, 0o644)


def test_plugin_check_invalid_format(tmp_path: Path) -> None:
    """Test that a bad format value fails gracefully."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkfmt",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "chkfmt")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "chkfmt", "--format", "badfmt"], env=env)
    assert res.returncode != 0


def test_plugin_check_quiet_and_debug(tmp_path: Path) -> None:
    """Test that the --quiet flag overrides the --debug flag."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkqd",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins = run_cli(["plugins", "install", str(tmp_path / "chkqd")], env=env)
    assert ins.returncode == 0
    res = run_cli(["plugins", "check", "chkqd", "--quiet", "--debug"], env=env)
    assert res.stdout.strip() == ""


def test_plugin_check_with_broken_code(tmp_path: Path) -> None:
    """Test checking a plugin with invalid Python syntax."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "broken",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "broken").glob("**/plugin.py"))
    plug_py.write_text("def totally_invalid_python(:\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "broken")], env=env)
    res = run_cli(["plugins", "check", "broken"], env=env)
    assert res.returncode != 0
    assert "error" in res.stderr.lower() or "failed" in res.stderr.lower()


def test_plugin_check_with_partial_metadata(tmp_path: Path) -> None:
    """
    Check that plugin 'check' command fails for incomplete metadata.

    This ensures the check command detects and rejects plugins with invalid or
    insufficient metadata (e.g., 'plugin.json' missing required fields).
    """
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "partialmeta",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0, f"Scaffold failed: {res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir(exist_ok=True)
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install_res = run_cli(
        ["plugins", "install", str(tmp_path / "partialmeta")], env=env
    )
    assert install_res.returncode == 0, f"Install failed: {install_res.stdout}"
    candidates = list(plugins_dir.glob("partialmeta*"))
    assert candidates, "Installed plugin directory not found"
    meta_file = candidates[0] / "plugin.json"
    meta_file.write_text('{"incomplete": true}')
    check_res = run_cli(["plugins", "check", "partialmeta"], env=env)
    assert check_res.returncode != 0, (
        f"'check' should fail for incomplete metadata, got: {check_res.stdout}"
    )


def test_plugin_check_crashes_should_not_kill_cli(tmp_path: Path) -> None:
    """Test that a crashing health() hook doesn't crash the CLI."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkcrash",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "chkcrash").glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text()
        + '\ndef health(di):\n    raise Exception("Health failed!")\n'
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "chkcrash")], env=env)
    res = run_cli(["plugins", "check", "chkcrash"], env=env)
    assert res.returncode != 0
    assert "Health failed" in res.stderr or "Exception" in res.stderr


def test_plugin_check_returns_non_json(tmp_path: Path) -> None:
    """Test a health() hook that prints non-JSON output."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "badjsonchk",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "badjsonchk").glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text()
        + '\ndef health(self, di): print("I am not JSON"); return True\n'
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "badjsonchk")], env=env)
    res = run_cli(["plugins", "check", "badjsonchk"], env=env)
    assert res.returncode in (0, 1)


def test_plugin_check_after_uninstall(tmp_path: Path) -> None:
    """Test that checking a plugin after it has been uninstalled fails."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "chkplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "chkplug")], env=env)
    run_cli(["plugins", "uninstall", "chkplug"], env=env)
    res = run_cli(["plugins", "check", "chkplug"], env=env)
    assert res.returncode != 0


def test_plugin_check_health_returns_unexpected_type(tmp_path: Path) -> None:
    """Test a health() hook that returns an unexpected data type."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "weirdhealth",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "weirdhealth").glob("**/plugin.py"))
    plug_py.write_text(plug_py.read_text() + "\ndef health(di):\n    return 'maybe'\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "weirdhealth")], env=env)
    res = run_cli(["plugins", "check", "weirdhealth", "--format", "json"], env=env)
    data = json.loads(res.stdout)
    assert data.get("status") == "unhealthy"


def test_plugin_check_async_health(tmp_path: Path) -> None:
    """Test an asynchronous health() hook."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "asynchealth",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "asynchealth").glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text()
        + "\nimport asyncio\nasync def health(di):\n    await asyncio.sleep(0.01)\n    return True\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "asynchealth")], env=env)
    res = run_cli(["plugins", "check", "asynchealth", "--format", "json"], env=env)
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data.get("status") == "healthy"


def test_plugin_check_health_raises_non_exception(tmp_path: Path) -> None:
    """Test a health() hook that raises a BaseException (like SystemExit)."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "panicplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0
    plug_py = next((tmp_path / "panicplug").glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text() + "\ndef health(di):\n    raise SystemExit('bail out')\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "panicplug")], env=env)
    res = run_cli(["plugins", "check", "panicplug", "--format", "json"], env=env)
    assert res.returncode != 0
    assert "bail out" in res.stderr or "SystemExit" in res.stderr


def test_plugin_check_valid_and_invalid(tmp_path: Path) -> None:
    """Test checking a valid plugin, then corrupting it and checking again."""
    scf = run_cli(
        [
            "plugins",
            "scaffold",
            "checker",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf.returncode == 0, scf.stdout
    plugin_dir = tmp_path / "checker"
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(plugin_dir)], env=env)

    check_res = run_cli(["plugins", "check", "checker"], env=env)
    assert check_res.returncode == 0
    assert "healthy" in check_res.stdout.lower()

    plug_py = next(plugin_dir.glob("**/plugin.py"))
    plug_py.write_text("def broken(:\n")

    run_cli(["plugins", "install", str(plugin_dir), "--force"], env=env)
    check_res2 = run_cli(["plugins", "check", "checker"], env=env)
    assert check_res2.returncode != 0
    assert (
        "error" in check_res2.stderr.lower() or "invalid" in check_res2.stderr.lower()
    )
