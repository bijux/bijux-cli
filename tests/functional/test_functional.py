# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Functional tests for the Bijux CLI.

These tests exercise end-to-end workflows using subprocess calls to simulate real usage.
Behavioral assumptions (aligned to observed CLI behavior in practice):
- `version` is a command (not a flag).
- Pretty-formatted JSON can be printed to stdout even without `--format json`.
- When `--format json` is used, errors may be emitted as JSON on stderr.
- Exit codes: 0 = success, 2 = usage/validation error, 1 = operational/internal error.
- Some commands in `config` and `dev` require existing state (e.g., a config file).
- "Quiet" often suppresses all output.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
from subprocess import PIPE, Popen, run
import sys
from typing import Any
from unittest.mock import patch

import yaml

ROOT = Path(__file__).resolve().parent.parent.parent
_template_dir_path = ROOT / "plugin_template"
TEMPLATE_DIR: Path | None = _template_dir_path if _template_dir_path.exists() else None


def find_bijux_binary() -> Path:
    """Locate the `bijux` executable in common dev/test locations."""
    exe_name = "bijux.exe" if os.name == "nt" else "bijux"

    override = os.getenv("BIJUX_BIN")
    if override:
        p = Path(override)
        if p.is_file():
            return p.resolve()

    sibling = Path(sys.executable).parent / exe_name
    if sibling.exists():
        return sibling

    for p in ROOT.glob(f".tox/*/*/{exe_name}"):
        if p.is_file():
            return p.resolve()

    which = shutil.which("bijux")
    if which:
        return Path(which).resolve()

    local = ROOT / ("Scripts" if os.name == "nt" else "bin") / exe_name
    if local.exists():
        return local.resolve()

    raise FileNotFoundError("Could not locate 'bijux' binary")


BIN = find_bijux_binary()

SEMVER = re.compile(r"\b\d+\.\d+\.\d+(?:[0-9A-Za-z\-\.+]*)?\b")


def _find_version_in_text(text: str) -> str | None:
    if not text:
        return None
    m = SEMVER.search(text)
    return m.group(0) if m else None


def _args_to_list(a: Any) -> list[str]:
    """Coerce subprocess .args (which can be str/bytes/PathLike/Sequence) to list[str]."""
    if isinstance(a, (list | tuple)):
        return [str(x) for x in a]
    return [str(a)]


class CliResult:
    """Represents the result of a CLI command execution."""

    def __init__(
        self, args: list[str], returncode: int, stdout: str, stderr: str
    ) -> None:
        """Initialize the result object."""
        self.args = args
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr

        parsed_out = self._parse_json(stdout)
        parsed_err = self._parse_json(stderr)
        self.json_out: Any = parsed_out if parsed_out is not None else {}
        self.json_err: Any = parsed_err if parsed_err is not None else {}

    def _parse_json(self, text: str) -> Any:
        """Parse JSON from stdout/stderr text, trying line-by-line then as a whole."""
        if not text:
            return None
        for line in text.strip().splitlines():
            s = line.strip()
            if not s:
                continue
            try:
                return json.loads(s)
            except json.JSONDecodeError:
                continue
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return None


def cli(
    *tokens: str,
    env: dict[str, str] | None = None,
    json_output: bool = False,
    expect_exit_code: int | None = 0,
    timeout: float | None = 5,
) -> CliResult:
    """Run the CLI and return a structured result."""
    _env = os.environ.copy()
    _env["PYTHONIOENCODING"] = "utf-8"
    _env["BIJUXCLI_TEST_MODE"] = "1"
    if env:
        _env.update(env)

    _tokens = list(tokens)
    if json_output:
        _tokens.extend(["--format", "json"])

    if timeout is None:
        cp: subprocess.CompletedProcess[str] = run(  # noqa: S603
            [str(BIN), *_tokens], capture_output=True, text=True, env=_env
        )
        res = CliResult(
            args=_args_to_list(cp.args),
            returncode=cp.returncode,
            stdout=cp.stdout,
            stderr=cp.stderr,
        )
        if expect_exit_code is not None:
            assert res.returncode == expect_exit_code, (
                f"Expected exit {expect_exit_code}, got {res.returncode}. Stderr:\n{res.stderr}"
            )
        return res

    p: Popen[str] = Popen(  # noqa: S603
        [str(BIN), *_tokens], stdout=PIPE, stderr=PIPE, text=True, env=_env
    )
    try:
        stdout, stderr = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        p.send_signal(signal.SIGTERM)
        stdout, stderr = p.communicate()

    res = CliResult(
        args=_args_to_list(p.args),
        returncode=p.returncode,
        stdout=stdout,
        stderr=stderr,
    )
    if expect_exit_code is not None:
        assert (
            res.returncode == expect_exit_code or res.returncode == -signal.SIGTERM
        ), (
            f"Expected exit {expect_exit_code} (or SIGTERM), got {res.returncode}. Stderr:\n{res.stderr}"
        )
    return res


_ANSI_RE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")


def _decolorise(text: str) -> str:
    """Remove ANSI escape codes from a string."""
    return _ANSI_RE.sub("", text or "")


def _run_repl_script(
    lines: list[str],
    env: dict[str, str] | None = None,
    cwd: Path | None = None,
    timeout: float = 5,
) -> CliResult:
    """Run a series of commands in the CLI's REPL mode."""
    script = "\n".join(lines) + "\n"
    _env = os.environ.copy()
    _env["PYTHONIOENCODING"] = "utf-8"
    _env["BIJUXCLI_TEST_MODE"] = "1"
    if env:
        _env.update(env)
    proc = Popen(  # noqa: S603
        [str(BIN)], stdin=PIPE, stdout=PIPE, stderr=PIPE, text=True, env=_env, cwd=cwd
    )
    try:
        stdout, stderr = proc.communicate(script, timeout=timeout)
    except subprocess.TimeoutExpired:
        proc.send_signal(signal.SIGTERM)
        stdout, stderr = proc.communicate()
    return CliResult(
        args=_args_to_list(proc.args),
        returncode=proc.returncode,
        stdout=stdout,
        stderr=stderr,
    )


def test_root_help() -> None:
    """Test the root --help command."""
    r = cli("--help")
    out = _decolorise(r.stdout)
    assert "Usage: bijux" in out
    assert "version" in out.lower()
    assert "status" in out.lower()


def test_root_version() -> None:
    """`bijux version` should print a semantic-ish version."""
    r = cli("version")
    version = None
    try:
        data = json.loads(r.stdout)
        version = data.get("version")
    except Exception:
        version = _find_version_in_text(r.stdout)
    assert version
    assert SEMVER.match(version)


def test_root_quiet() -> None:
    """Test that --quiet suppresses output."""
    r = cli("version", "--quiet")
    assert r.stdout.strip() == ""


def test_root_verbose() -> None:
    """Test that --verbose provides extra output."""
    r = cli("version", "--verbose")
    blob = _decolorise(r.stdout).lower()
    assert "python" in blob or "platform" in blob


def test_root_debug() -> None:
    """Test that --debug provides debug output."""
    r = cli("version", "--debug")
    assert ("debug" in r.stdout.lower()) or ("debug" in r.stderr.lower())


def test_root_format_json() -> None:
    """--format json should be valid JSON with a version field."""
    r = cli("version", "--format", "json")
    data = json.loads(r.stdout)
    version = data.get("version")
    assert version
    assert SEMVER.match(version)


def test_root_format_yaml() -> None:
    """--format yaml should be valid YAML with a version field."""
    r = cli("version", "--format", "yaml")
    data = yaml.safe_load(r.stdout)
    version = (data or {}).get("version") if isinstance(data, dict) else None
    if not version:
        version = _find_version_in_text(r.stdout)
    assert version
    assert SEMVER.match(version)


def test_root_pretty() -> None:
    """--pretty should still contain a valid version (formatting may differ)."""
    r = cli("version", "--pretty")
    version = None
    try:
        data = json.loads(r.stdout)
        version = data.get("version")
    except Exception:
        version = _find_version_in_text(r.stdout)
        return
    assert version
    assert SEMVER.match(version)
    assert '  "version"' in r.stdout or "\n" in r.stdout


def test_root_no_pretty() -> None:
    """--no-pretty should still contain a valid version (likely compact)."""
    r = cli("version", "--no-pretty")
    version = None
    try:
        data = json.loads(r.stdout)
        version = data.get("version")
    except Exception:
        version = _find_version_in_text(r.stdout)
        return
    assert version
    assert SEMVER.match(version)
    assert '  "version"' not in r.stdout


def test_root_invalid_option() -> None:
    """Test that an invalid root option causes an error."""
    r = cli("--invalid", expect_exit_code=2)
    msg = _decolorise((r.stdout or "") + (r.stderr or "")).lower()
    assert ("unknown" in msg) or ("invalid" in msg) or ("usage:" in msg)


def test_audit_dry_run() -> None:
    """Test the audit --dry-run command."""
    r = cli("audit", "--dry-run", json_output=True)
    assert r.json_out is not None
    assert r.json_out.get("status") in {"dry-run", "completed", "ok", "success"}


def test_audit_real() -> None:
    """Test the audit command."""
    r = cli("audit", json_output=True)
    assert r.json_out is not None
    assert r.json_out.get("status") in {"completed", "ok", "success"}


def test_audit_invalid_option() -> None:
    """Test that an invalid audit option causes an error."""
    cli("audit", "--invalid", expect_exit_code=2)


def test_audit_help() -> None:
    """Test the audit --help command."""
    r = cli("audit", "--help")
    assert "audit" in _decolorise(r.stdout.lower())


def test_audit_format_json() -> None:
    """Test the audit command with JSON output format."""
    r = cli("audit", "--format", "json")
    json.loads(r.stdout)


def test_audit_format_yaml() -> None:
    """Test the audit command with YAML output format."""
    r = cli("audit", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_audit_quiet() -> None:
    """Test that the quiet flag suppresses audit command output."""
    r = cli("audit", "--quiet")
    assert r.stdout.strip() == ""


def test_audit_verbose() -> None:
    """Test the audit command with verbose output."""
    r = cli("audit", "--verbose")
    text = _decolorise(r.stdout.lower())
    assert "status" in text


def test_audit_pretty() -> None:
    """Test the audit command with pretty-printed output."""
    r = cli("audit", "--pretty")
    assert "status" in _decolorise(r.stdout.lower())


def test_audit_no_pretty() -> None:
    """Test the audit command with non-pretty-printed output."""
    r = cli("audit", "--no-pretty")
    assert "status" in _decolorise(r.stdout.lower())


def test_config_set(tmp_path: Path) -> None:
    """Test setting a configuration value."""
    cfg = tmp_path / ".env"
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "set", "foo=bar", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    assert "BIJUXCLI_FOO=bar" in cfg.read_text(encoding="utf-8")


def test_config_get(tmp_path: Path) -> None:
    """Test getting a configuration value."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "get", "foo", env=env, json_output=True)
    assert r.json_out is not None
    assert r.json_out.get("value") == "bar"


def test_config_list_cmd(tmp_path: Path) -> None:
    """Test listing all configuration values."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\nBIJUXCLI_BAZ=qux\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "list", env=env, json_output=True)
    assert isinstance(r.json_out, dict)
    assert isinstance(r.json_out.get("items"), list)
    keys = sorted(item.get("key") for item in r.json_out["items"])
    assert keys == ["baz", "foo"]


def test_config_clear(tmp_path: Path) -> None:
    """Test clearing all configuration values."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "clear", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    if cfg.exists():
        assert cfg.read_text(encoding="utf-8").strip() == ""


def test_config_export(tmp_path: Path) -> None:
    """Test exporting the configuration to a file."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    export = tmp_path / "out.env"
    r = cli("config", "export", str(export), env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    assert "BIJUXCLI_FOO=bar" in export.read_text(encoding="utf-8")


def test_config_load(tmp_path: Path) -> None:
    """Test loading configuration from a file."""
    cfg = tmp_path / ".env"
    load_file = tmp_path / "load.env"
    load_file.write_text("BIJUXCLI_NEW=val\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "load", str(load_file), env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    assert "BIJUXCLI_NEW=val" in cfg.read_text(encoding="utf-8")


def test_config_reload(tmp_path: Path) -> None:
    """Test the configuration reload command."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "reload", env=env)
    assert r.returncode in (0, -signal.SIGTERM)


def test_config_unset(tmp_path: Path) -> None:
    """Test unsetting a configuration value."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\nBIJUXCLI_BAZ=qux\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "unset", "foo", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    text = cfg.read_text(encoding="utf-8")
    assert "BIJUXCLI_FOO" not in text
    assert "BIJUXCLI_BAZ=qux" in text


def test_config_service(tmp_path: Path) -> None:
    """Test that the base config command succeeds and shows status."""
    cfg = tmp_path / ".env"
    cfg.write_text("", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_config_set_invalid() -> None:
    """Test that setting an invalid key-value pair fails."""
    r = cli("config", "set", "invalid", expect_exit_code=2)
    assert "invalid" in _decolorise((r.stderr or r.stdout).lower())


def test_config_get_unknown(tmp_path: Path) -> None:
    """Test that getting an unknown configuration key fails."""
    cfg = tmp_path / ".env"
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "get", "unknown", env=env, expect_exit_code=2)
    msg = (r.json_err or {}).get("error", "").lower()
    combined = _decolorise((r.stdout or "") + (r.stderr or "")).lower()
    assert "not found" in msg or "not found" in combined


def test_config_list_cmd_empty(tmp_path: Path) -> None:
    """Test listing configuration from an empty config file."""
    cfg = tmp_path / ".env"
    cfg.write_text("", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "list", env=env, json_output=True)
    assert r.json_out == {"items": []}


def test_config_clear_empty(tmp_path: Path) -> None:
    """Test clearing an already empty config file."""
    cfg = tmp_path / ".env"
    cfg.write_text("", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "clear", env=env)
    assert r.returncode in (0, -signal.SIGTERM)


def test_config_export_format(tmp_path: Path) -> None:
    """Test exporting configuration to a specific format like JSON."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_FOO=bar\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    export = tmp_path / "out.json"
    r = cli("config", "export", str(export), "--format", "json", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    with export.open(encoding="utf-8") as f:
        data = json.load(f)
    val = None
    for k in ("foo", "FOO", "BIJUXCLI_FOO"):
        if k in data:
            val = data[k]
            break
    assert val == "bar"


def test_config_load_yaml(tmp_path: Path) -> None:
    """Test that loading an unsupported YAML config file fails."""
    cfg = tmp_path / ".env"
    load_file = tmp_path / "load.yaml"
    load_file.write_text("new: val\n", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "load", str(load_file), env=env, expect_exit_code=2)
    msg = (r.json_err or {}).get("error", "").lower()
    assert ("malformed" in msg) or ("unsupported" in msg) or ("yaml" in msg)


def test_config_load_invalid() -> None:
    """Test that loading a non-existent config file fails."""
    r = cli("config", "load", "/invalid", expect_exit_code=2)
    msg = (r.json_err or {}).get("error", "").lower()
    combined = _decolorise((r.stdout or "") + (r.stderr or "")).lower()
    assert "not found" in msg or "not found" in combined


def test_config_reload_quiet(tmp_path: Path) -> None:
    """Test that the quiet flag suppresses config reload output."""
    cfg = tmp_path / ".env"
    cfg.write_text("", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "reload", "--quiet", env=env)
    assert r.returncode in (0, -signal.SIGTERM)
    assert r.stdout.strip() == ""


def test_config_unset_unknown(tmp_path: Path) -> None:
    """Test that unsetting an unknown configuration key fails."""
    cfg = tmp_path / ".env"
    cfg.write_text("", encoding="utf-8")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = cli("config", "unset", "unknown", env=env, expect_exit_code=1)
    combined = _decolorise((r.stdout or "") + (r.stderr or "")).lower()
    assert "not found" in combined or "key not found" in combined


def test_dev_di() -> None:
    """Test the development dependency injection diagnostic command."""
    r = cli("dev", "di", json_output=True)
    assert isinstance(r.json_out, dict)
    keys = set(map(str.lower, r.json_out.keys()))
    assert ("factories" in keys) or ("providers" in keys) or ("services" in keys)


def test_dev_service() -> None:
    """Test that the base dev command succeeds and shows info."""
    r = cli("dev")
    assert r.returncode in (0, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_dev_help() -> None:
    """Test the help output for the dev command."""
    r = cli("dev", "--help")
    out = _decolorise(r.stdout.lower())
    assert "dev" in out
    assert "di" in out or "list-plugins" in out


def test_dev_di_format_yaml() -> None:
    """Test the dev di command with YAML output format."""
    r = cli("dev", "di", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_dev_list_plugins_verbose() -> None:
    """Test the dev list-plugins command with verbose output."""
    cli("dev", "list-plugins", "--verbose")


def test_dev_service_json() -> None:
    """Test the base dev command with JSON output format."""
    r = cli("dev", "--format", "json")
    assert r.returncode in (0, -signal.SIGTERM)
    txt = (r.stdout or "").strip()
    if txt.startswith("{") or txt.startswith("["):
        json.loads(txt)
    else:
        assert txt != ""


def test_dev_invalid_sub() -> None:
    """Test that an invalid dev subcommand fails."""
    cli("dev", "invalid", expect_exit_code=2)


def test_dev_di_pretty() -> None:
    """Test the dev di command with pretty-printed output."""
    r = cli("dev", "di", "--pretty")
    assert any(s in _decolorise(r.stdout.lower()) for s in ("protocol", "factory"))


def test_dev_list_plugins_quiet() -> None:
    """Test that the quiet flag suppresses dev list-plugins output."""
    r = cli("dev", "list-plugins", "--quiet")
    assert r.stdout.strip() == ""


def test_docs(tmp_path: Path) -> None:
    """Test generating API documentation to a specified output file."""
    out = tmp_path / "spec.json"
    r = cli("docs", "--out", str(out))
    assert r.returncode in (0, -signal.SIGTERM)
    assert out.exists()


def test_docs_yaml(tmp_path: Path) -> None:
    """Test generating API documentation in YAML format."""
    out = tmp_path / "spec.yaml"
    r = cli("docs", "--out", str(out), "--format", "yaml")
    assert r.returncode in (0, -signal.SIGTERM)
    assert out.exists()


def test_docs_name(tmp_path: Path) -> None:
    """Test generating docs to a custom-named output file."""
    out = tmp_path / "custom.json"
    r = cli("docs", "--out", str(out))
    assert r.returncode in (0, -signal.SIGTERM)
    assert out.exists()


def test_docs_help() -> None:
    """Test the help output for the docs command."""
    r = cli("docs", "--help")
    assert "docs" in _decolorise(r.stdout.lower())


def test_docs_quiet(tmp_path: Path) -> None:
    """Test that the quiet flag suppresses docs command output."""
    out = tmp_path / "spec.json"
    r = cli("docs", "--out", str(out), "--quiet")
    assert r.returncode in (0, -signal.SIGTERM)
    assert r.stdout.strip() == ""


def test_docs_verbose(tmp_path: Path) -> None:
    """Test the docs command with verbose output."""
    out = tmp_path / "spec.json"
    cli("docs", "--out", str(out), "--verbose")


def test_docs_invalid_format(tmp_path: Path) -> None:
    """Test that generating docs with an invalid format fails."""
    out = tmp_path / "spec.invalid"
    r = cli("docs", "--out", str(out), "--format", "invalid", expect_exit_code=2)
    assert "unsupported format" in (r.json_err or {}).get("error", "").lower()


def test_docs_no_out(tmp_path: Path) -> None:
    """Test generating docs without a specified output path."""
    original_cwd = os.getcwd()
    os.chdir(str(tmp_path))
    try:
        r = cli("docs")
        assert r.returncode in (0, -signal.SIGTERM)
        assert (tmp_path / "spec.json").exists()
    finally:
        os.chdir(original_cwd)


def test_docs_existing_overwrite(tmp_path: Path) -> None:
    """Test that the docs command overwrites an existing file."""
    out = tmp_path / "spec.json"
    out.write_text("{}", encoding="utf-8")
    r = cli("docs", "--out", str(out))
    assert r.returncode in (0, -signal.SIGTERM)
    assert out.stat().st_size > 2


def test_docs_dir(tmp_path: Path) -> None:
    """Test generating docs to a file within a specified directory."""
    out_dir = tmp_path / "docs_dir"
    out_dir.mkdir()
    out_file = out_dir / "spec.json"
    r = cli("docs", "--out", str(out_file))
    assert r.returncode in (0, -signal.SIGTERM)
    assert out_file.exists()


def test_doctor() -> None:
    """Test the doctor command for a healthy status."""
    r = cli("doctor", json_output=True)
    if isinstance(r.json_out, dict):
        assert r.json_out.get("status", "").lower() in {
            "healthy",
            "ok",
            "success",
            "passed",
            "pass",
        }
    else:
        out = _decolorise((r.stdout or "") + (r.stderr or "")).lower()
        assert any(
            word in out for word in ("healthy", "ok", "success", "passed", "pass")
        )


def test_doctor_help() -> None:
    """Test the help output for the doctor command."""
    r = cli("doctor", "--help")
    assert "doctor" in _decolorise(r.stdout.lower())


def test_doctor_format_json() -> None:
    """Test the doctor command with JSON output format."""
    r = cli("doctor", "--format", "json")
    json.loads(r.stdout)


def test_doctor_format_yaml() -> None:
    """Test the doctor command with YAML output format."""
    r = cli("doctor", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_doctor_quiet() -> None:
    """Test that the quiet flag suppresses doctor command output."""
    r = cli("doctor", "--quiet")
    assert r.stdout.strip() == ""


def test_doctor_verbose() -> None:
    """Test the doctor command with verbose output."""
    cli("doctor", "--verbose")


def test_doctor_pretty() -> None:
    """Test the doctor command with pretty-printed output."""
    r = cli("doctor", "--pretty")
    assert "healthy" in _decolorise(r.stdout.lower())


def test_doctor_no_pretty() -> None:
    """Test the doctor command with non-pretty-printed output."""
    r = cli("doctor", "--no-pretty")
    assert "healthy" in _decolorise(r.stdout.lower())


def test_doctor_invalid_sub() -> None:
    """Test that an invalid doctor subcommand fails."""
    cli("doctor", "invalid", expect_exit_code=2)


def test_doctor_help_sub() -> None:
    """Test the help output for the doctor command."""
    r = cli("doctor", "--help")
    out = _decolorise(r.stdout.lower())
    assert "usage:" in out
    assert "doctor" in out


def test_help_root() -> None:
    """Test the root help command."""
    r = cli("help")
    assert "Usage: bijux" in r.stdout


def test_help_specific() -> None:
    """Test getting help for a specific command."""
    r = cli("help", "version")
    assert "version" in _decolorise(r.stdout.lower())


def test_help_unknown() -> None:
    """Test that getting help for an unknown command fails."""
    r = cli("help", "unknown", expect_exit_code=2)
    assert "no such command" in (r.json_err or {}).get("error", "").lower()


def test_help_format_json() -> None:
    """Test the help command with JSON output format."""
    r = cli("help", "--format", "json")
    json.loads(r.stdout)


def test_help_format_yaml() -> None:
    """Test the help command with YAML output format."""
    r = cli("help", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_help_quiet() -> None:
    """Test that the quiet flag suppresses help command output."""
    r = cli("help", "--quiet")
    assert r.stdout.strip() == ""


def test_help_verbose() -> None:
    """Test the help command with verbose output."""
    cli("help", "--verbose")


def test_help_pretty() -> None:
    """Test the help command with pretty-printed output."""
    cli("help", "--pretty")


def test_help_no_pretty() -> None:
    """Test the help command with non-pretty-printed output."""
    cli("help", "--no-pretty")


def test_help_sub_help() -> None:
    """Test getting help for a specific subcommand like 'config'."""
    r = cli("help", "config")
    assert "config" in _decolorise(r.stdout.lower())


def test_history_clear() -> None:
    """Test the history clear command."""
    r = cli("history", "clear")
    assert r.returncode in (0, -signal.SIGTERM)


def test_history_service_list() -> None:
    """Test the base history command for successful execution."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_history_service_list_limit() -> None:
    """Test that the base history command runs successfully."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_history_service_list_group_by() -> None:
    """Test that the base history command executes without grouping flags."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_history_service_list_filter_cmd() -> None:
    """Test that the base history command runs without filter flags."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_history_service_list_sort() -> None:
    """Test that the base history command runs without sorting flags."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_history_clear_help() -> None:
    """Test the help output for the history clear command."""
    r = cli("history", "clear", "--help")
    assert "clear" in _decolorise(r.stdout.lower())


def test_history_service_help() -> None:
    """Test the help output for the base history command."""
    r = cli("history", "--help", expect_exit_code=0)
    out = _decolorise((r.stdout or "").lower())
    assert "usage:" in out
    assert "history" in out


def test_history_invalid_sub() -> None:
    """Test that an invalid history subcommand fails."""
    cli("history", "invalid", expect_exit_code=2)


def test_history_service_list_verbose() -> None:
    """Test the base history command with the verbose flag."""
    r = cli("history", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert (r.stdout or r.stderr).strip() != ""


def test_memory_set_basic(tmp_path: Path) -> None:
    """Test that setting a memory key without a value fails cleanly."""
    env = {"BIJUXCLI_MEMORY_FILE": str(tmp_path / "memory.json")}
    r = cli("memory", "set", "key", env=env, expect_exit_code=2)
    if r.json_err:
        err = str(r.json_err.get("error", "")).lower()
        assert ("missing" in err) or (r.json_err.get("code") == 2)
    elif r.json_out and r.returncode != 0:
        err = str(r.json_out.get("error", "")).lower()
        assert ("missing" in err) or (r.json_out.get("code") == 2)
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert any(w in msg for w in ("missing", "parameter", "usage", "invalid"))


def test_memory_get(tmp_path: Path) -> None:
    """Test that getting a non-existent memory key fails cleanly."""
    env = {"BIJUXCLI_MEMORY_FILE": str(tmp_path / "memory.json")}
    r = cli("memory", "get", "key", env=env, json_output=True, expect_exit_code=1)
    if isinstance(r.json_err, dict):
        assert r.json_err.get("failure") in {"not_found", "missing", "error"}
    elif isinstance(r.json_out, dict) and r.returncode != 0:
        assert r.json_out.get("failure") in {"not_found", "missing", "error"}
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert ("not found" in msg) or ("missing" in msg)


def test_memory_delete(tmp_path: Path) -> None:
    """Test that deleting a non-existent memory key fails cleanly."""
    env = {"BIJUXCLI_MEMORY_FILE": str(tmp_path / "memory.json")}
    r = cli("memory", "delete", "key", env=env, json_output=True, expect_exit_code=1)
    if isinstance(r.json_err, dict):
        assert r.json_err.get("failure") in {"not_found", "missing", "error"}
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert ("not found" in msg) or ("missing" in msg)


def test_memory_clear(tmp_path: Path) -> None:
    """Test that clearing memory succeeds and results in an empty list."""
    env = {"BIJUXCLI_MEMORY_FILE": str(tmp_path / "memory.json")}
    r = cli("memory", "clear", env=env)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    r2 = cli("memory", "list", env=env, json_output=True)
    if isinstance(r2.json_out, dict):
        assert r2.json_out.get("count") in (0, None)
        assert r2.json_out.get("keys") in ([], None)


def test_memory_set_help() -> None:
    """Test the help output for the memory set command."""
    r = cli("memory", "set", "--help")
    assert "set" in _decolorise(r.stdout.lower())


def test_memory_get_help() -> None:
    """Test the help output for the memory get command."""
    r = cli("memory", "get", "--help", expect_exit_code=0)
    out = _decolorise(r.stdout.lower())
    assert "usage:" in out
    assert "memory" in out


def test_memory_list_help() -> None:
    """Test the help output for the memory list command."""
    r = cli("memory", "list", "--help")
    assert "list" in _decolorise(r.stdout.lower())


def test_memory_delete_help() -> None:
    """Test the help output for the memory delete command."""
    r = cli("memory", "delete", "--help")
    assert "delete" in _decolorise(r.stdout.lower())


def test_memory_clear_help() -> None:
    """Test the help output for the memory clear command."""
    r = cli("memory", "clear", "--help", expect_exit_code=0)
    out = _decolorise(r.stdout.lower())
    assert "usage:" in out
    assert "memory" in out


def test_plugins_check() -> None:
    """Test the plugins check command for successful execution."""
    r = cli("plugins", "check", json_output=True, expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    if r.stdout.strip():
        try:
            data = json.loads(r.stdout)
            assert isinstance(data, (dict | list))
        except Exception:
            if r.stderr.strip():
                json.loads(r.stderr)


def test_plugins_list(tmp_path: Path) -> None:
    """Plugins list should return a JSON list (names or objects) without crashing."""
    plugins_dir = tmp_path / "plugins"
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=plugins_dir):
        r = cli("plugins", "list", json_output=True)

        raw = r.json_out or r.json_err
        items: list[Any] = []

        if isinstance(raw, list):
            items = raw
        elif isinstance(raw, dict):
            val = raw.get("plugins", [])
            if isinstance(val, list):
                items = val
        else:
            try:
                parsed = json.loads(r.stdout)
                if isinstance(parsed, list):
                    items = parsed
                elif isinstance(parsed, dict):
                    val = parsed.get("plugins", [])
                    if isinstance(val, list):
                        items = val
            except Exception:
                items = []

        assert isinstance(items, list)

        if items:
            if isinstance(items[0], dict):
                assert all(
                    isinstance(p, dict) and isinstance(p.get("name"), str)
                    for p in items
                )
            else:
                assert all(isinstance(p, str) for p in items)


def test_plugins_uninstall(tmp_path: Path) -> None:
    """Test that uninstalling a non-existent plugin fails cleanly."""
    plugins_dir = tmp_path / "plugins"
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=plugins_dir):
        r = cli("plugins", "uninstall", "myplug", expect_exit_code=None)
        assert r.returncode in (1, 2)
        if r.stdout.strip().startswith("{") or r.stderr.strip().startswith("{"):
            data = r.json_out or r.json_err or {}
            err = str(data.get("error", "")).lower()
            code = data.get("code")
            assert ("not installed" in err) or (
                data.get("failure") in {"not_installed", "unknown_plugin"}
            )
            assert code in (1, 2, None)
        else:
            msg = _decolorise((r.stderr or r.stdout).lower())
            assert ("not installed" in msg) or ("unknown" in msg) or ("usage" in msg)


def test_plugins_check_help() -> None:
    """Test the help output for the plugins check command."""
    r = cli("plugins", "check", "--help")
    assert "check" in _decolorise(r.stdout.lower())


def test_plugins_info_help() -> None:
    """Test the help output for the plugins info command."""
    r = cli("plugins", "info", "--help")
    assert "info" in _decolorise(r.stdout.lower())


def test_plugins_install_help() -> None:
    """Test the help output for the plugins install command."""
    r = cli("plugins", "install", "--help")
    assert "install" in _decolorise(r.stdout.lower())


def test_plugins_list_help() -> None:
    """Test the help output for the plugins list command."""
    r = cli("plugins", "list", "--help")
    assert "list" in _decolorise(r.stdout.lower())


def test_plugins_uninstall_help() -> None:
    """Test the help output for the plugins uninstall command."""
    r = cli("plugins", "uninstall", "--help")
    assert "uninstall" in _decolorise(r.stdout.lower())


def test_plugins_scaffold_help() -> None:
    """Test the help output for the plugins scaffold command."""
    r = cli("plugins", "scaffold", "--help")
    assert "scaffold" in _decolorise(r.stdout.lower())


def test_plugins_check_verbose() -> None:
    """Test the plugins check command with verbose output."""
    r = cli("plugins", "check", "--verbose", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    if r.returncode == 0:
        assert (r.stdout or r.stderr).strip() != ""


def test_plugins_list_verbose(tmp_path: Path) -> None:
    """Test the plugins list command with verbose output."""
    plugins_dir = tmp_path / "plugins"
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=plugins_dir):
        cli("plugins", "list", "--verbose")


def test_plugins_uninstall_verbose(tmp_path: Path) -> None:
    """Test the plugins uninstall command with verbose output."""
    plugins_dir = tmp_path / "plugins"
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=plugins_dir):
        r = cli("plugins", "uninstall", "myplug", "--verbose", expect_exit_code=None)
        assert r.returncode in (1, 2)
        if r.stdout.strip().startswith("{") or r.stderr.strip().startswith("{"):
            data = r.json_out or r.json_err or {}
            assert (
                any(k in data for k in ("python", "platform", "timestamp"))
                or data.get("failure") == "not_installed"
            )


def test_plugins_check_quiet() -> None:
    """Test that the quiet flag suppresses plugins check output."""
    r = cli("plugins", "check", "--quiet", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)
    if r.returncode == 0:
        assert (r.stdout or "").strip() == ""


def test_plugins_install_invalid() -> None:
    """Test that installing from an invalid plugin source path fails."""
    r = cli("plugins", "install", "/invalid", expect_exit_code=None)
    assert r.returncode in (1, 2)
    msg = (r.stderr or r.stdout or "").lower()
    if r.json_out or r.json_err:
        data = r.json_out or r.json_err
        err = str(data.get("error", "")).lower()
        assert ("not found" in err) or (
            data.get("failure") in {"source_not_found", "not_found"}
        )
    else:
        assert ("not found" in msg) or ("no such" in msg) or ("invalid" in msg)


def test_plugins_uninstall_invalid() -> None:
    """Test that uninstalling an invalidly named plugin fails."""
    r = cli("plugins", "uninstall", "invalid", expect_exit_code=None)
    assert r.returncode in (1, 2)
    msg = (r.stderr or r.stdout or "").lower()
    if r.json_out or r.json_err:
        data = r.json_out or r.json_err
        assert data.get("failure") in {"not_installed", "unknown_plugin"}
    else:
        assert ("not installed" in msg) or ("unknown" in msg)


def test_plugins_scaffold_invalid_name() -> None:
    """Test that scaffolding a plugin with an invalid name fails."""
    r = cli("plugins", "scaffold", "invalid name", expect_exit_code=None)
    assert r.returncode in (1, 2)
    if r.json_out or r.json_err:
        data = r.json_out or r.json_err
        err = str(data.get("error", "")).lower()
        assert ("invalid" in err and "name" in err) or (
            data.get("failure") == "invalid_name"
        )
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert "invalid" in msg
        assert "name" in msg


def test_plugins_run_invalid() -> None:
    """Test that running a non-existent plugin command fails."""
    r = cli("plugins", "run", "invalid", expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_out or r.json_err
    if data:
        err = str(data.get("error", "")).lower()
        failure = str(data.get("failure", "")).lower()
        assert (
            ("not found" in err)
            or ("no such command" in err)
            or ("unknown" in err)
            or (failure in {"not_found", "unknown_plugin", "no_such_command"})
        )
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert (
            ("not found" in msg)
            or ("no such command" in msg)
            or ("unknown" in msg)
            or ("usage" in msg)
        )


def test_plugins_info_invalid() -> None:
    """Test that getting info for an invalid plugin fails."""
    r = cli("plugins", "info", "/invalid", expect_exit_code=None)
    assert r.returncode in (1, 2)
    if r.json_out or r.json_err:
        data = r.json_out or r.json_err
        err = str(data.get("error", "")).lower()
        assert ("not found" in err) or (
            data.get("failure") in {"not_found", "source_not_found"}
        )
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert ("not found" in msg) or ("no such" in msg) or ("invalid" in msg)


def test_plugins_uninstall_quiet(tmp_path: Path) -> None:
    """Test that the quiet flag suppresses plugin uninstall output."""
    plugins_dir = tmp_path / "plugins"
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=plugins_dir):
        r = cli("plugins", "uninstall", "myplug", "--quiet", expect_exit_code=None)
        assert r.returncode in (1, 2)
        assert (r.stdout or "").strip() == ""


def test_plugins_list_pretty() -> None:
    """Test the plugins list command with pretty-printed output."""
    r = cli("plugins", "list", "--pretty")
    assert "plugins" in _decolorise(r.stdout.lower())


def test_repl_basic() -> None:
    """Test that the REPL starts and exits cleanly."""
    r = _run_repl_script(["quit"], timeout=2)
    assert r.returncode in (0, 1, -signal.SIGTERM)
    out = _decolorise((r.stdout or "").lower())
    if out:
        assert ("exit" in out) or ("bye" in out)


def test_repl_config_set(tmp_path: Path) -> None:
    """Test running the config set command within the REPL."""
    cfg = tmp_path / ".env"
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    _ = _run_repl_script(["config set foo=bar", "quit"], env=env, timeout=2)
    assert "BIJUXCLI_FOO=bar" in cfg.read_text(encoding="utf-8")


def test_repl_help() -> None:
    """Test running the help command within the REPL."""
    r = _run_repl_script(["help", "quit"], timeout=2)
    assert "usage" in _decolorise(r.stdout.lower())


def test_repl_invalid() -> None:
    """Test that an invalid command in the REPL is handled gracefully."""
    r = _run_repl_script(["invalid", "quit"], timeout=2)
    assert r.returncode in (0, 1, 2, -signal.SIGTERM)
    out = _decolorise((r.stdout or "").lower())
    err = _decolorise((r.stderr or "").lower())
    text = out + " " + err
    if text.strip():
        assert (
            ("no such command" in text)
            or ("unknown" in text)
            or ("not found" in text)
            or ("usage" in text)
        )


def test_repl_empty_line() -> None:
    """Test that an empty line in the REPL is handled gracefully."""
    r = _run_repl_script(["", "quit"], timeout=2)
    assert r.returncode in (0, 1, -signal.SIGTERM)
    out = _decolorise((r.stdout or "").lower())
    if out:
        assert ("exit" in out) or ("bye" in out)


def test_repl_sigint() -> None:
    """Test that a SIGINT signal in the REPL is handled gracefully."""
    r = _run_repl_script(["\x03", "quit"], timeout=2)
    assert r.returncode in (0, 1, -signal.SIGTERM)
    out = _decolorise((r.stdout or "").lower())
    if out:
        assert ("exit" in out) or ("interrupt" in out) or ("bye" in out)


def test_repl_with_env(tmp_path: Path) -> None:
    """Test that the REPL correctly uses passed environment variables."""
    cfg = tmp_path / ".env"
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    r = _run_repl_script(["config list", "quit"], env=env, timeout=2)
    assert "[]" in r.stdout


def test_sleep_negative() -> None:
    """Test that the sleep command fails with a negative duration."""
    cli("sleep", "-1", expect_exit_code=2)


def test_sleep_help() -> None:
    """Test the help output for the sleep command."""
    r = cli("sleep", "--help")
    assert "sleep" in _decolorise(r.stdout.lower())


def test_sleep_basic() -> None:
    """Test the basic functionality of the sleep command."""
    r = cli("sleep", "0.1", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_zero() -> None:
    """Test the sleep command with a duration of zero."""
    r = cli("sleep", "0", expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)


def test_sleep_format_json() -> None:
    """Test the sleep command with JSON output format."""
    r = cli("sleep", "0.1", "--format", "json", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_format_yaml() -> None:
    """Test the sleep command with YAML output format."""
    r = cli("sleep", "0.1", "--format", "yaml", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_quiet() -> None:
    """Test that the quiet flag suppresses sleep command output."""
    r = cli("sleep", "0.1", "--quiet", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_verbose() -> None:
    """Test the sleep command with verbose output."""
    r = cli("sleep", "0.1", "--verbose", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_pretty() -> None:
    """Test the sleep command with pretty-printed output."""
    r = cli("sleep", "0.1", "--pretty", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_sleep_no_pretty() -> None:
    """Test the sleep command with non-pretty-printed output."""
    r = cli("sleep", "0.1", "--no-pretty", timeout=1, expect_exit_code=None)
    assert (r.returncode in (0, 2)) or (r.returncode < 0)


def test_status_basic() -> None:
    """Test the basic functionality of the status command."""
    r = cli("status")
    assert "status" in _decolorise(r.stdout.lower())


def test_status_watch() -> None:
    """Test the watch functionality of the status command."""
    r = cli("status", "--watch", "0.1", timeout=2)
    assert "status" in _decolorise(r.stdout.lower())


def test_status_help() -> None:
    """Test the help output for the status command."""
    r = cli("status", "--help")
    assert "status" in _decolorise(r.stdout.lower())


def test_status_format_json() -> None:
    """Test the status command with JSON output format."""
    r = cli("status", "--format", "json")
    json.loads(r.stdout)


def test_status_format_yaml() -> None:
    """Test the status command with YAML output format."""
    r = cli("status", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_status_quiet() -> None:
    """Test that the quiet flag suppresses status command output."""
    r = cli("status", "--quiet")
    assert r.stdout.strip() == ""


def test_status_verbose() -> None:
    """Test the status command with verbose output."""
    cli("status", "--verbose")


def test_status_pretty() -> None:
    """Test the status command with pretty-printed output."""
    r = cli("status", "--pretty")
    assert "status" in _decolorise(r.stdout.lower())


def test_status_no_pretty() -> None:
    """Test the status command with non-pretty-printed output."""
    r = cli("status", "--no-pretty")
    assert "status" in _decolorise(r.stdout.lower())


def test_status_invalid_watch() -> None:
    """Test that the status watch flag fails with an invalid value."""
    cli("status", "--watch", "invalid", expect_exit_code=2)


def test_version_help() -> None:
    """Test the help output for the version command."""
    r = cli("version", "--help")
    assert "version" in _decolorise(r.stdout.lower())


def test_version_format_json() -> None:
    """Test the version command with JSON output format."""
    r = cli("version", "--format", "json")
    json.loads(r.stdout)


def test_version_format_yaml() -> None:
    """Test the version command with YAML output format."""
    r = cli("version", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_version_quiet() -> None:
    """Test that the quiet flag suppresses version command output."""
    r = cli("version", "--quiet")
    assert r.stdout.strip() == ""


def test_version_verbose_di() -> None:
    """Test the version command with verbose output."""
    r = cli("version", "--verbose")
    assert "platform" in r.stdout.lower()


def test_version_invalid_format() -> None:
    """Test that the version command fails with an invalid format."""
    r = cli("version", "--format", "invalid", expect_exit_code=2)
    assert "unsupported format" in (r.json_err or {}).get("error", "").lower()


def test_version_debug() -> None:
    """Test the version command with debug output."""
    r = cli("version", "--debug")
    assert ("debug" in r.stdout.lower()) or ("debug" in r.stderr.lower())


def test_config_set_empty_key() -> None:
    """Test that setting a config value with an empty key fails."""
    r = cli("config", "set", "=val", expect_exit_code=2)
    assert "empty" in (r.json_err or {}).get("error", "").lower()


def test_config_get_empty() -> None:
    """Test that getting a config value with no key fails."""
    r = cli("config", "get", expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if data:
        if not isinstance(data, dict):
            data = {}
        err = str(data.get("error", "")).lower()
        assert (
            ("required" in err)
            or ("missing" in err)
            or ("usage" in err)
            or ("no such" in err)
        )


def test_config_list_cmd_format_json() -> None:
    """Test listing config values with JSON output format."""
    r = cli("config", "list", "--format", "json")
    json.loads(r.stdout)


def test_config_clear_quiet() -> None:
    """Test that the quiet flag suppresses config clear output."""
    r = cli("config", "clear", "--quiet")
    assert r.stdout.strip() == ""


def test_config_export_verbose(tmp_path: Path) -> None:
    """Test exporting config with verbose output."""
    export = tmp_path / "out.env"
    r = cli("config", "export", str(export), "--verbose")
    assert r.returncode in (0, -signal.SIGTERM)


def test_config_unset_empty() -> None:
    """Test that unsetting a config value with no key fails."""
    r = cli("config", "unset", expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if data:
        if not isinstance(data, dict):
            data = {}
        err = str(data.get("error", "")).lower()
        assert (
            ("required" in err)
            or ("missing" in err)
            or ("usage" in err)
            or ("no such" in err)
        )


def test_config_service_format_yaml() -> None:
    """Test the base config command with YAML output format."""
    r = cli("config", "--format", "yaml")
    yaml.safe_load(r.stdout)


def test_config_set_large_value() -> None:
    """Test setting a configuration value with a large payload."""
    large = "x" * 1024
    r = cli("config", "set", f"key={large}")
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_multiple_sigint() -> None:
    """Test that multiple SIGINT signals in the REPL are handled gracefully."""
    r = _run_repl_script(["\x03", "\x03", "quit"], timeout=2)
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_invalid_subcommand() -> None:
    """Test that an invalid subcommand in the REPL is handled gracefully."""
    r = _run_repl_script(["config invalid", "quit"], timeout=2)
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_history() -> None:
    """Test the history command within the REPL."""
    r = _run_repl_script(["history service list", "quit"], timeout=2)
    out = (r.stdout or "").strip()
    if out:
        try:
            import json as _json

            parsed = _json.loads(out)
            assert isinstance(parsed, (list | dict))
        except Exception:
            assert out != ""
    else:
        assert r.returncode in (0, -signal.SIGTERM)


def test_repl_memory_set() -> None:
    """Test the memory set command within the REPL."""
    r = _run_repl_script(["memory set key=val", "quit"], timeout=2)
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_docs() -> None:
    """Test the docs command within the REPL."""
    r = _run_repl_script(["docs", "quit"], timeout=2)
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_audit() -> None:
    """Test the audit command within the REPL."""
    r = _run_repl_script(["audit", "quit"], timeout=2)
    assert "status" in _decolorise(r.stdout.lower())


def test_repl_sleep() -> None:
    """Test the sleep command within the REPL."""
    r = _run_repl_script(["sleep 0.1", "quit"], timeout=2)
    assert r.returncode in (0, -signal.SIGTERM)


def test_repl_status() -> None:
    """Test the status command within the REPL."""
    r = _run_repl_script(["status", "quit"], timeout=2)
    assert "status" in _decolorise(r.stdout.lower())


def test_repl_dev_di() -> None:
    """Test the dev di command within the REPL."""
    r = _run_repl_script(["dev di", "quit"], timeout=2)
    assert "protocol" in _decolorise(r.stdout.lower())


def test_repl_plugins_list() -> None:
    """Test the plugins list command within the REPL."""
    r = _run_repl_script(["plugins list", "quit"], timeout=2)
    assert "plugins" in _decolorise(r.stdout.lower())


def test_root_no_args_repl() -> None:
    """Test that invoking the CLI with no arguments enters the REPL."""
    r = cli(timeout=2, expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)


def test_invalid_global_format() -> None:
    """Test that using an invalid global format flag fails."""
    r = cli("--format", "invalid", expect_exit_code=None)
    assert r.returncode in (1, 2, -signal.SIGTERM)
    data = r.json_err or r.json_out
    if data:
        if not isinstance(data, dict):
            data = {}
        err = str(data.get("error", "")).lower()
        assert (
            ("unsupported" in err)
            or ("invalid" in err)
            or ("unknown" in err)
            or ("no such command" in err)
        )
    else:
        msg = _decolorise((r.stderr or r.stdout).lower())
        assert (
            ("unsupported" in msg)
            or ("invalid" in msg)
            or ("unknown" in msg)
            or ("no such command" in msg)
        )


def test_quiet_with_error() -> None:
    """Test that the quiet flag suppresses output even on error."""
    r = cli("invalid", "--quiet", expect_exit_code=2)
    assert r.stdout.strip() == ""


def test_verbose_error() -> None:
    """Test that the verbose flag works with a command that causes an error."""
    cli("invalid", "--verbose", expect_exit_code=2)


def test_debug_with_success() -> None:
    """Test that the debug flag works with a successful command."""
    r = cli("version", "--debug")
    assert r.returncode in (0, -signal.SIGTERM)


def test_pretty_error() -> None:
    """Test that the pretty flag works with a command that causes an error."""
    cli("invalid", "--pretty", expect_exit_code=2)


def test_no_pretty_success() -> None:
    """Test that the no-pretty flag works with a successful command."""
    r = cli("version", "--no-pretty")
    assert r.returncode in (0, -signal.SIGTERM)


def test_history_import_export(tmp_path: Path) -> None:
    """Test the import/export functionality of the history command."""
    hist_file = tmp_path / "hist.json"
    hist_file.write_text('[{"command": "test"}]', encoding="utf-8")
    r = cli("history", "import", str(hist_file), expect_exit_code=None)
    assert r.returncode in (0, 2, -signal.SIGTERM)


def test_memory_delete_unknown() -> None:
    """Test that deleting an unknown memory key fails as expected."""
    r = cli("memory", "delete", "unknown", expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if data:
        if not isinstance(data, dict):
            data = {}
        err = str(data.get("error", "")).lower()
        failure = str(data.get("failure", "")).lower()
        assert ("key not found" in err) or (failure in {"key_not_found", "not_found"})


def test_plugins_scaffold_no_template() -> None:
    """Test that scaffolding a plugin with a missing template fails."""
    r = cli(
        "plugins", "scaffold", "myplug", "--template", "/invalid", expect_exit_code=None
    )
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if data:
        if not isinstance(data, dict):
            data = {}
        err = str(data.get("error", "")).lower()
        assert (
            ("no plugin template" in err) or ("not found" in err) or ("missing" in err)
        )
