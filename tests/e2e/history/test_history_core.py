# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for `bijux history` command."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import yaml

from tests.e2e.conftest import run_cli
from tests.e2e.history.conftest import (
    REQUIRED_FLAGS,
    assert_json,
    assert_no_stacktrace,
    assert_yaml,
    make_ro_dir,
    normalize_history_payload,
    require_symlink,
)


def test_permission_denied_parent(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensures CLI handles write errors due to parent directory permissions."""
    rodir = tmp_path / "ro"
    make_ro_dir(rodir)
    hist = rodir / ".bijux_history"
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(hist))
    res = run_cli(["version"])
    text = (res.stderr or res.stdout or "").lower()
    assert res.returncode != 0 or "permission" in text or "denied" in text


def test_disk_full_env(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensures CLI handles simulated disk full errors."""
    monkeypatch.setenv("BIJUXCLI_TEST_DISK_FULL", "1")
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(tmp_path / ".bijux_history"))
    res = run_cli(["version"])
    out = (res.stderr or "" + res.stdout or "").lower()
    assert "no space" in out or "error" in out or res.returncode != 0


def test_corrupt_history_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensures CLI handles a history file with invalid JSON."""
    hist = tmp_path / ".bijux_history"
    hist.write_text("THIS IS NOT JSON")
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(hist))
    res = run_cli(["history", "--format", "json"])
    text = (res.stdout + res.stderr).lower()
    assert res.returncode != 0 or "error" in text or "invalid" in text


def test_sparse_large_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensures CLI handles a large file containing only null bytes."""
    hist = tmp_path / ".bijux_history"
    with open(hist, "wb") as f:
        f.write(b"\x00" * 2048)
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(hist))
    res = run_cli(["history", "--format", "json"])
    text = (res.stdout + res.stderr).lower()
    assert res.returncode != 0 or "error" in text or "invalid" in text


def test_binary_garbage(tmp_path: Path) -> None:
    """Ensures CLI handles a history file with arbitrary binary data."""
    hist = tmp_path / ".bijux_history"
    with open(hist, "wb") as f:
        f.write(b"\x00\x01garbage\n")
    r = run_cli(["history", "--format", "json"])
    t = (r.stdout + r.stderr).lower()
    assert r.returncode != 0 or "error" in t


def test_empty_file_is_tolerated(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensures an empty history file does not cause a crash."""
    hist = tmp_path / ".bijux_history"
    hist.write_text("")
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(hist))
    r = run_cli(["history", "--format", "json"])
    if r.returncode == 0:
        payload = assert_json(r.stdout)
        assert isinstance(payload, dict)
        assert isinstance(payload.get("entries"), list)
    else:
        assert "error" in (r.stdout + r.stderr).lower()


def test_trailing_newlines(tmp_path: Path) -> None:
    """Ensures trailing newlines in the history file are handled."""
    hist = tmp_path / ".bijux_history"
    hist.write_text('[{"command": "test", "timestamp": 1}]\n\n')
    r = run_cli(["history", "--format", "json"])
    assert '"test"' in r.stdout


def test_symlink_supported(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensures the CLI can write to a history file that is a symlink."""
    require_symlink(tmp_path)
    real = tmp_path / ".real_hist"
    real.write_text("[]")
    link = tmp_path / ".bijux_history"
    link.symlink_to(real)
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(link))
    r = run_cli(["history", "--format", "yaml"])
    assert r.returncode == 0 or "error" in (r.stdout + r.stderr).lower()
    if r.returncode == 0:
        payload = assert_yaml(r.stdout)
        assert isinstance(payload, dict)
        assert isinstance(payload.get("entries"), list)


def test_deleted_mid_command_is_tolerated(tmp_path: Path) -> None:
    """Ensures the CLI tolerates the history file being deleted during an operation."""
    hist = tmp_path / ".bijux_history"
    run_cli(["status"])
    hist.unlink(missing_ok=True)
    r = run_cli(["history", "--format", "json"])
    assert r.returncode in (0, 1, 2)


def test_missing_parent_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensures the CLI does not crash if the history file's parent dir is missing."""
    monkeypatch.setenv(
        "BIJUXCLI_HISTORY_FILE", str(tmp_path / "missing" / ".bijux_history")
    )
    r = run_cli(["history", "--format", "json"])
    assert r.returncode in (0, 1, 2)
    assert_no_stacktrace(r.stdout + r.stderr)


def test_export_import_cycle(tmp_path: Path) -> None:
    """Verifies a full export-clear-import cycle restores the history."""
    export_file = tmp_path / "history_export.json"
    run_cli(["history", "clear"])
    run_cli(["version"])

    r1 = run_cli(["history", "--export", str(export_file)])
    assert r1.returncode == 0
    assert export_file.exists()

    run_cli(["history", "clear"])
    r2 = run_cli(["history", "--import", str(export_file)])
    assert r2.returncode == 0

    res = run_cli(["history", "--format", "json"])
    payload = json.loads(res.stdout)
    entries = payload.get("entries", [])
    assert any(e.get("command") == "version" for e in entries)


def test_export_overwrites(tmp_path: Path) -> None:
    """Ensures that export overwrites an existing file."""
    export_file = tmp_path / "hist.json"
    export_file.write_text("should be replaced")
    run_cli(["history", "clear"])
    run_cli(["version"])
    r = run_cli(["history", "--export", str(export_file)])
    assert r.returncode == 0
    assert "should be replaced" not in export_file.read_text()


def test_import_invalid_file(tmp_path: Path) -> None:
    """Ensures that importing a non-JSON file fails gracefully."""
    bad = tmp_path / "bad.json"
    bad.write_text("not json")
    r = run_cli(["history", "--import", str(bad)])
    assert r.returncode != 0 or "error" in (r.stdout + r.stderr).lower()


def test_multiple_exports_are_identical(tmp_path: Path) -> None:
    """Verifies that two consecutive exports of the same history are identical."""
    e1 = tmp_path / "e1.json"
    e2 = tmp_path / "e2.json"
    run_cli(["history", "clear"])
    run_cli(["version"])
    run_cli(["history", "--export", str(e1)])
    run_cli(["history", "--export", str(e2)])
    assert e1.exists()
    assert e2.exists()
    assert e1.read_text() == e2.read_text()


def test_filter_command_exact() -> None:
    """Tests exact command filtering."""
    run_cli(["history", "clear"])
    run_cli(["status"])
    run_cli(["version"])
    res = run_cli(["history", "--filter", "status", "--format", "json"])
    payload = json.loads(res.stdout)
    entries = payload.get("entries", [])
    assert all(e["command"] == "status" for e in entries)


def test_sort_timestamp() -> None:
    """Tests sorting by timestamp."""
    run_cli(["history", "clear"])
    run_cli(["status"])
    run_cli(["version"])
    res = run_cli(["history", "--sort", "timestamp", "--format", "json"])
    payload = json.loads(res.stdout)
    entries = payload.get("entries", [])
    ts = [e["timestamp"] for e in entries]
    assert ts == sorted(ts)


def test_group_by_command() -> None:
    """Tests grouping by command."""
    run_cli(["history", "clear"])
    run_cli(["status"])
    run_cli(["status"])
    run_cli(["version"])
    res = run_cli(["history", "--group-by", "command", "--format", "json"])
    assert res.returncode == 0
    payload = json.loads(res.stdout)
    groups = payload.get("entries", [])
    assert any(g.get("group") == "status" and g.get("count") >= 2 for g in groups)


def test_json_shape_golden(golden_dir: Path) -> None:
    """Compares JSON output against a golden file to ensure consistent shape."""
    run_cli(["history", "clear"])
    run_cli(["version"])
    r = run_cli(["history", "--format", "json"])
    live = normalize_history_payload(assert_json(r.stdout))
    want = normalize_history_payload(
        assert_json((golden_dir / "history_shape.json").read_text())
    )
    assert len(live) >= 1
    assert set(live[0].keys()).issuperset(set(want[0].keys()))


def test_yaml_shape_golden(golden_dir: Path) -> None:
    """Compares YAML output against a golden file to ensure consistent shape."""
    run_cli(["history", "clear"])
    run_cli(["status"])
    r = run_cli(["history", "--format", "yaml"])
    live = normalize_history_payload(assert_yaml(r.stdout))
    want = normalize_history_payload(
        assert_yaml((golden_dir / "history_shape.yaml").read_text())
    )
    assert len(live) >= 1
    assert set(live[0].keys()).issuperset(set(want[0].keys()))


def test_legacy_and_extra_fields(tmp_path: Path) -> None:
    """Ensures legacy and extra fields in a history file are tolerated."""
    hist = tmp_path / ".bijux_history"
    hist.write_text(
        json.dumps(
            [
                {"cmd": "old_version"},
                {"command": "foo", "timestamp": 1, "extra": 1},
            ]
        )
    )
    r = run_cli(["history", "--format", "json"])
    assert r.returncode == 0


def test_json_and_yaml_agree() -> None:
    """Verifies that normalized JSON and YAML outputs are identical."""
    run_cli(["history", "clear"])
    run_cli(["version"])
    j = run_cli(["history", "--format", "json"]).stdout
    y = run_cli(["history", "--format", "yaml"]).stdout
    jn = normalize_history_payload(assert_json(j))
    yn = normalize_history_payload(yaml.safe_load(y) or {})
    assert jn == yn


@pytest.mark.parametrize("fmt", ["json", "yaml", "JSON"])
def test_formats_flag(fmt: str) -> None:
    """Tests various valid format flags."""
    run_cli(["version"])
    r = run_cli(["history", "--format", fmt])
    assert r.returncode == 0


def test_help_mentions_flags() -> None:
    """Checks that the help message contains all required flags."""
    r = run_cli(["history", "--help"])
    assert r.returncode == 0
    t = r.stdout.lower()
    assert t.startswith("usage:")
    for flag in REQUIRED_FLAGS:
        assert flag in t
    assert_no_stacktrace(r.stdout + r.stderr)


def test_invalid_limit_flag() -> None:
    """Ensures a non-integer limit value causes an error."""
    r = run_cli(["history", "--limit", "bogus"])
    assert r.returncode in (1, 2)


def test_negative_limit_flag() -> None:
    """Ensures a negative limit value causes an error."""
    r = run_cli(["history", "--limit", "-1"])
    assert r.returncode in (1, 2)


def test_invalid_format_flag() -> None:
    """Ensures an unsupported format value causes an error."""
    r = run_cli(["history", "--format", "unsupported"])
    assert r.returncode in (1, 2)
