# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin uninstall command."""

from __future__ import annotations

from pathlib import Path
import shutil
import threading
import time

from tests.e2e.conftest import TEST_TEMPLATE, assert_text, run_cli


def test_plugin_uninstall_ok(tmp_path: Path) -> None:
    """Test a successful plugin uninstall operation."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "unplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "unplug")], env=env)
    res = run_cli(["plugins", "uninstall", "unplug"], env=env)
    assert res.returncode == 0


def test_plugin_uninstall_nonexistent() -> None:
    """Test that uninstalling a non-existent plugin fails correctly."""
    res = run_cli(["plugins", "uninstall", "nonexistent"])
    assert res.returncode == 1
    assert_text(res, "not installed")


def test_plugin_uninstall_twice(tmp_path: Path) -> None:
    """Test that uninstalling the same plugin twice fails on the second attempt."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "twiceunplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "twiceunplug")], env=env)
    run_cli(["plugins", "uninstall", "twiceunplug"], env=env)
    res = run_cli(["plugins", "uninstall", "twiceunplug"], env=env)
    assert res.returncode == 1


def test_plugin_uninstall_quiet(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses output during uninstall."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "quietun",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "quietun")], env=env)
    res = run_cli(["plugins", "uninstall", "quietun", "--quiet"], env=env)
    assert res.returncode == 0
    assert res.stdout.strip() == ""


def test_plugin_uninstall_wrong_case(tmp_path: Path) -> None:
    """Test that plugin uninstallation is case-sensitive."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "CamelCase",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "CamelCase")], env=env)
    res = run_cli(["plugins", "uninstall", "camelcase"], env=env)
    assert res.returncode == 1


def test_plugin_uninstall_with_partial_permissions(tmp_path: Path) -> None:
    """Test uninstalling a plugin with read-only files."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "permunplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "permunplug")], env=env)
    installed_plugin_dirs = list((tmp_path / "plugs").glob("permunplug*"))
    assert installed_plugin_dirs
    plug_dir = installed_plugin_dirs[0]
    for file in plug_dir.rglob("*"):
        if file.is_file():
            file.chmod(0o400)
    res = run_cli(["plugins", "uninstall", "permunplug"], env=env)
    assert res.returncode in (0, 1)
    assert not plug_dir.exists()


def test_plugin_uninstall_when_in_use(tmp_path: Path) -> None:
    """Test uninstalling a plugin while one of its commands is running."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "busyplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "busyplug")], env=env)

    def run_cmd() -> None:
        """Run a plugin command."""
        run_cli(["busyplug", "run", "--input", "wait"], env=env)

    t = threading.Thread(target=run_cmd)
    t.start()
    time.sleep(0.1)
    res = run_cli(["plugins", "uninstall", "busyplug"], env=env)
    t.join()
    assert res.returncode in (0, 1)


def test_plugin_uninstall_symlink_dir(tmp_path: Path) -> None:
    """Test that uninstalling fails if the plugins directory is a symlink."""
    real_dir = tmp_path / "realplugs"
    real_dir.mkdir()
    symlink = tmp_path / "plugs"
    symlink.symlink_to(real_dir)
    env = {"BIJUXCLI_PLUGINS_DIR": str(symlink)}
    res = run_cli(["plugins", "uninstall", "whatever"], env=env)
    assert res.returncode == 1
    assert "symlink" in res.stderr or "refuse" in res.stderr.lower()


def test_plugin_uninstall_non_dir(tmp_path: Path) -> None:
    """Test that uninstall fails if the target plugin path is a file, not a directory."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / "notaplug").write_text("not a dir")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "uninstall", "notaplug"], env=env)
    assert res.returncode == 1
    assert (
        "not a directory" in res.stderr.lower()
        or "invalid" in res.stderr.lower()
        or "not installed" in res.stderr.lower()
    )


def test_plugin_uninstall_idempotent(tmp_path: Path) -> None:
    """Test that uninstalling a manually deleted plugin still reports it's not installed."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "goneplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "goneplug")], env=env)
    # Find the installed path (with hash) before deleting
    installed_dirs = list((tmp_path / "plugs").glob("goneplug*"))
    assert installed_dirs, "Plugin directory not found after install"
    shutil.rmtree(installed_dirs[0])
    res = run_cli(["plugins", "uninstall", "goneplug"], env=env)
    assert res.returncode == 1
    assert "not installed" in res.stderr or "not found" in res.stderr.lower()


def test_plugin_uninstall_empty_plugins_dir(tmp_path: Path) -> None:
    """Test uninstalling from an empty plugins directory."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "uninstall", "ghost"], env=env)
    assert res.returncode == 1
    assert "not installed" in res.stderr.lower() or "not found" in res.stderr.lower()


def test_plugin_uninstall_handles_broken_symlink(tmp_path: Path) -> None:
    """Test that a broken symlink in the plugins directory is handled gracefully."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    broken_link = plugins_dir / "broken"
    broken_link.symlink_to(tmp_path / "no_such_dir")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "uninstall", "broken"], env=env)
    assert res.returncode == 1
    assert "not installed" in res.stderr.lower() or "not found" in res.stderr.lower()


def test_plugin_uninstall_in_non_writable_dir(tmp_path: Path) -> None:
    """Test that uninstalling fails when the plugins directory is not writable."""
    plugins_dir = tmp_path / "nowrite"
    plugins_dir.mkdir()
    plugins_dir.chmod(0o400)  # Read-only
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    try:
        res = run_cli(["plugins", "uninstall", "foo"], env=env)
        assert res.returncode == 1
    finally:
        plugins_dir.chmod(0o700)


def test_plugin_uninstall_non_ascii_name_rejected(tmp_path: Path) -> None:
    """Test that a non-ASCII plugin name is rejected at the scaffold stage."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "ユニプラグ",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0
    assert "Invalid plugin name" in res.stdout or "Invalid plugin name" in res.stderr


def test_plugin_uninstall_with_existing_file(tmp_path: Path) -> None:
    """Test that uninstall fails if the target plugin path is a file, not a directory."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / "plainfile").write_text("not a plugin dir")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "uninstall", "plainfile"], env=env)
    assert res.returncode == 1


def test_plugin_uninstall_removes_all_files(tmp_path: Path) -> None:
    """Test that uninstall correctly removes the entire plugin directory."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "fullplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "fullplug")], env=env)
    installed_plugin_dirs = list((tmp_path / "plugs").glob("fullplug*"))
    assert installed_plugin_dirs, "Plugin directory not found after install"
    plug_dir = installed_plugin_dirs[0]
    (plug_dir / "extra.txt").write_text("extra file")
    res = run_cli(["plugins", "uninstall", "fullplug"], env=env)
    assert res.returncode == 0
    assert not plug_dir.exists()


def test_plugin_uninstall_symlinked_plugin_json(tmp_path: Path) -> None:
    """Test uninstalling a plugin that contains internal symlinks."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "symlinkplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "symlinkplug")], env=env)
    installed_plugin_dirs = list((tmp_path / "plugs").glob("symlinkplug*"))
    assert installed_plugin_dirs, "Plugin directory not found after install"
    plug_dir = installed_plugin_dirs[0]
    json_file = plug_dir / "plugin.json"
    link_file = plug_dir / "meta.json"
    json_file.rename(link_file)
    json_file.symlink_to(link_file)
    res = run_cli(["plugins", "uninstall", "symlinkplug"], env=env)
    assert res.returncode == 0
    assert not plug_dir.exists()


def test_plugin_uninstall_plugin_dir_with_sub_dirs(tmp_path: Path) -> None:
    """Test that uninstall correctly removes a plugin with subdirectories."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "subdirplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "subdirplug")], env=env)
    installed_plugin_dirs = list((tmp_path / "plugs").glob("subdirplug*"))
    assert installed_plugin_dirs, "Plugin directory not found after install"
    plug_dir = installed_plugin_dirs[0]
    (plug_dir / "sub").mkdir()
    res = run_cli(["plugins", "uninstall", "subdirplug"], env=env)
    assert res.returncode == 0
    assert not plug_dir.exists()


def test_plugin_uninstall_debug_mode(tmp_path: Path) -> None:
    """Test the uninstall command with the --debug flag."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "dbgplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "dbgplug")], env=env)
    res = run_cli(["plugins", "uninstall", "dbgplug", "--debug"], env=env)
    assert res.returncode == 0


def test_plugin_uninstall_quiet_and_debug(tmp_path: Path) -> None:
    """Test that the --quiet flag overrides the --debug flag during uninstall."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "qdbg",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "qdbg")], env=env)
    res = run_cli(["plugins", "uninstall", "qdbg", "--quiet", "--debug"], env=env)
    assert res.returncode == 0
    assert res.stdout.strip() == ""
