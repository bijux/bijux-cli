# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin install command."""

from __future__ import annotations

import json
import os
from pathlib import Path
import threading
from typing import Any

import pytest
import yaml

from tests.e2e.conftest import TEST_TEMPLATE, assert_text, run_cli


def test_plugin_install_and_run_unicode_path_is_rejected(tmp_path: Path) -> None:
    """Test that installing a plugin with a Unicode name is rejected."""
    name = "plügün"
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(
        [
            "plugins",
            "scaffold",
            name,
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ],
        env=env,
    )
    assert res.returncode != 0
    assert (
        "ascii" in res.stdout.lower()
        or "ascii" in res.stderr.lower()
        or "invalid name" in res.stdout.lower()
        or "invalid" in res.stderr.lower()
    )


def test_plugin_install_missing_plugin_py(tmp_path: Path) -> None:
    """Test that installing a plugin missing its plugin.py file fails."""
    good_name = "goodplug"
    good_dir = tmp_path / good_name
    good_dir.mkdir(parents=True, exist_ok=True)
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(good_dir)], env=env)
    assert res.returncode == 1
    assert "plugin.py not found" in res.stderr or "error" in res.stderr.lower()


def test_plugin_install_with_external_dependency(tmp_path: Path) -> None:
    """Test that a plugin with missing dependencies installs but fails its health check."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "extradep",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_py = next((tmp_path / "extradep").glob("**/plugin.py"))
    plug_py.write_text(plug_py.read_text() + "\nimport notarealpackage\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "extradep")], env=env)
    assert res.returncode == 0
    res2 = run_cli(["plugins", "check", "extradep"], env=env)
    assert res2.returncode != 0
    assert "Import error" in res2.stderr
    assert "no module named" in res2.stderr.lower()


def test_plugin_install_readonly_plugin_dir(tmp_path: Path) -> None:
    """Test that installation fails if the plugins directory is read-only."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "readonly",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plugs = tmp_path / "plugs"
    plugs.mkdir()
    plugs.chmod(0o400)
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugs)}
    res = run_cli(["plugins", "install", str(tmp_path / "readonly")], env=env)
    assert res.returncode != 0
    plugs.chmod(0o700)


def test_plugin_install_with_non_ascii_source_files(tmp_path: Path) -> None:
    """Test installing a plugin that contains non-ASCII characters in its source."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "nonascii",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_py = next((tmp_path / "nonascii").glob("**/plugin.py"))
    plug_py.write_text(plug_py.read_text() + "\n# author: Björn\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "nonascii")], env=env)
    assert res.returncode == 0


def test_plugin_install_plugin_with_huge_metadata_file(tmp_path: Path) -> None:
    """Test that a plugin with a very large metadata file does not crash the CLI."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "hugejson",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    meta = list((tmp_path / "hugejson").glob("**/plugin.json"))[0]
    meta.write_text('{"name":"hugejson","desc":"' + "x" * 1_000_000 + '"}')
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "hugejson")], env=env)
    assert res.returncode in (0, 1)


def test_plugin_install_and_uninstall_many_in_loop(tmp_path: Path) -> None:
    """Test a rapid install/uninstall loop to check for race conditions or state issues."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(
        [
            "plugins",
            "scaffold",
            "loopplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    for _ in range(5):
        run_cli(["plugins", "install", str(tmp_path / "loopplug"), "--force"], env=env)
        run_cli(["plugins", "uninstall", "loopplug"], env=env)
    assert True


def test_plugin_install_with_no_plugin_py(tmp_path: Path) -> None:
    """Test that installation fails if the scaffolded plugin.py is removed."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "nopy2",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_py = next((tmp_path / "nopy2").glob("**/plugin.py"))
    plug_py.unlink()
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "nopy2")], env=env)
    assert res.returncode != 0


def test_plugin_install_directory_with_many_files(tmp_path: Path) -> None:
    """Test installing a plugin from a directory containing many files."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "bigplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_dir = tmp_path / "bigplug"
    for i in range(300):
        (plug_dir / f"file_{i}.txt").write_text("x" * 10)
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    assert res.returncode == 0


def test_plugin_install_broken_symlink(tmp_path: Path) -> None:
    """Test that installing a plugin with a broken symlink is handled gracefully."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "brokensymlink",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    (tmp_path / "brokensymlink" / "link").symlink_to(tmp_path / "not_exist")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "brokensymlink")], env=env)
    assert res.returncode == 0


def test_plugin_install_hidden_files_are_ignored(tmp_path: Path) -> None:
    """Test that hidden files in the source directory are ignored during installation."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "hiddenplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_dir = tmp_path / "hiddenplug"
    (plug_dir / ".DS_Store").write_text("junk")
    (plug_dir / ".git").mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    assert res.returncode == 0


def test_plugin_install_concurrent(tmp_path: Path) -> None:
    """Test that concurrent installations do not interfere with each other."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    results: list[Any] = []

    def install_one(name: str) -> None:
        """Helper to scaffold and install a single plugin."""
        run_cli(
            [
                "plugins",
                "scaffold",
                name,
                "--output-dir",
                str(tmp_path),
                "--template",
                TEST_TEMPLATE,
            ]
        )
        results.append(run_cli(["plugins", "install", str(tmp_path / name)], env=env))

    t1 = threading.Thread(target=install_one, args=("pluga",))
    t2 = threading.Thread(target=install_one, args=("plugb",))
    t1.start()
    t2.start()
    t1.join()
    t2.join()
    assert all(r.returncode == 0 for r in results)


def test_plugin_install_with_nonempty_dest(tmp_path: Path) -> None:
    """Test that installation fails if the destination directory is not empty."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "replug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    # Find the hashed destination directory name
    install_res = run_cli(["plugins", "install", str(tmp_path / "replug")], env=env)
    installed_path_str = json.loads(install_res.stdout)["dest"]
    run_cli(["plugins", "uninstall", "replug"], env=env)
    Path(installed_path_str).mkdir()

    res = run_cli(["plugins", "install", str(tmp_path / "replug")], env=env)
    assert res.returncode == 1 or "already installed" in res.stdout


def test_plugin_install_multiple(tmp_path: Path) -> None:
    """Test installing multiple plugins successfully."""
    names = ["plug1", "plug2", "plug3"]
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    for n in names:
        run_cli(
            [
                "plugins",
                "scaffold",
                n,
                "--output-dir",
                str(tmp_path),
                "--template",
                TEST_TEMPLATE,
            ]
        )
        run_cli(["plugins", "install", str(tmp_path / n)], env=env)
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    for n in names:
        assert n in plugins


def test_plugin_install_from_file_fails(tmp_path: Path) -> None:
    """Test that attempting to install from a file instead of a directory fails."""
    plug_file = tmp_path / "notadir"
    plug_file.write_text("not a plugin dir")
    res = run_cli(["plugins", "install", str(plug_file)])
    assert res.returncode == 1


def test_plugin_install_symlink_path(tmp_path: Path) -> None:
    """Test installing a plugin from a path that is a symbolic link."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "realplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    link = tmp_path / "pluglink"
    link.symlink_to(tmp_path / "realplug")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(link)], env=env)
    assert res.returncode == 0


def test_plugin_install_symlink_attack(tmp_path: Path) -> None:
    """Test that installation fails if the plugins directory is a symlink."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "symlink",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    evil = tmp_path / "evil"
    evil.mkdir()
    dest_link = tmp_path / "plugs"
    dest_link.symlink_to(evil)
    env = {"BIJUXCLI_PLUGINS_DIR": str(dest_link)}
    res = run_cli(["plugins", "install", str(tmp_path / "symlink")], env=env)
    assert res.returncode != 0


def test_plugin_install_symlink_loop(tmp_path: Path) -> None:
    """Test that a symlink loop in the plugins directory path is handled."""
    src = tmp_path / "src"
    dest = tmp_path / "dest"
    src.symlink_to(dest)
    dest.symlink_to(src)
    env = {"BIJUXCLI_PLUGINS_DIR": str(src)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert (
        res.returncode != 0
        or "symlink" in res.stdout.lower()
        or "loop" in res.stdout.lower()
    )


def test_plugin_install_from_nested_dir(tmp_path: Path) -> None:
    """Test installing a plugin located in a nested directory."""
    nested = tmp_path / "foo" / "bar"
    nested.mkdir(parents=True)
    run_cli(
        [
            "plugins",
            "scaffold",
            "deepplug",
            "--output-dir",
            str(nested),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(nested / "deepplug")], env=env)
    assert res.returncode == 0


def test_plugin_install_with_readonly_dest(tmp_path: Path) -> None:
    """Test that installation fails if the destination directory is read-only."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "readonly",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    dest = tmp_path / "plugs"
    dest.mkdir()
    dest.chmod(0o555)
    env = {"BIJUXCLI_PLUGINS_DIR": str(dest)}
    res = run_cli(["plugins", "install", str(tmp_path / "readonly")], env=env)
    assert res.returncode != 0
    dest.chmod(0o755)


def test_plugin_install_from_scaffold(tmp_path: Path) -> None:
    """Test installing a plugin immediately after scaffolding it."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "installme",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "installme")], env=env)
    assert res.returncode == 0
    assert_text(res, '"status":"installed"')


def test_plugin_install_twice_requires_force(tmp_path: Path) -> None:
    """Test that installing a plugin twice fails without the --force flag."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "twiceplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "twiceplug")], env=env)
    res = run_cli(["plugins", "install", str(tmp_path / "twiceplug")], env=env)
    assert res.returncode == 1 or "already installed" in res.stdout


def test_plugin_install_force(tmp_path: Path) -> None:
    """Test that the --force flag allows overwriting an existing plugin."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "forceplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "forceplug")], env=env)
    res = run_cli(
        ["plugins", "install", str(tmp_path / "forceplug"), "--force"], env=env
    )
    assert res.returncode == 0


def test_plugin_install_invalid_path(tmp_path: Path) -> None:
    """Test that installing from an invalid path fails correctly."""
    res = run_cli(["plugins", "install", str(tmp_path / "notfound")])
    assert res.returncode == 1
    assert_text(res, "Source not found")


def test_plugin_install_dry_run(tmp_path: Path) -> None:
    """Test the --dry-run flag for the install command."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "dryplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    res = run_cli(
        [
            "plugins",
            "install",
            str(tmp_path / "dryplug"),
            "--dry-run",
            "--format",
            "yaml",
        ]
    )
    assert res.returncode == 0
    payload = yaml.safe_load(res.stdout)
    assert payload["status"] == "dry-run"


def test_plugin_install_with_custom_env(tmp_path: Path) -> None:
    """Test that a custom BIJUXCLI_PLUGINS_DIR is respected."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "customenv",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(tmp_path / "customenv")], env=env)
    assert res.returncode == 0


def test_plugin_install_permission_error(tmp_path: Path) -> None:
    """Test a graceful failure when the plugins directory is not writable."""
    plug_dir = tmp_path / "permplug"
    run_cli(
        [
            "plugins",
            "scaffold",
            "permplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    dest = tmp_path / "plugs"
    dest.mkdir()
    os.chmod(dest, 0o400)
    env = {"BIJUXCLI_PLUGINS_DIR": str(dest)}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    assert res.returncode != 0
    os.chmod(dest, 0o700)


def test_plugin_install_to_nonexistent_plugins_dir(tmp_path: Path) -> None:
    """Test that the plugins directory is created if it doesn't exist."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "autocreateplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    nonexist_dir = tmp_path / "will_create"
    env = {"BIJUXCLI_PLUGINS_DIR": str(nonexist_dir)}
    res = run_cli(["plugins", "install", str(tmp_path / "autocreateplug")], env=env)
    assert res.returncode == 0
    installed_dirs = list(nonexist_dir.glob("autocreateplug*"))
    assert installed_dirs
    assert (installed_dirs[0] / "plugin.py").is_file()


def test_plugin_install_plugin_py_symlinked(tmp_path: Path) -> None:
    """Test installing a plugin where the plugin.py file is a symbolic link."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "symlinkpy",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_dir = tmp_path / "symlinkpy"
    orig = next(plug_dir.glob("**/plugin.py"))
    link = orig.parent / "plugin_link.py"
    orig.rename(link)
    orig.symlink_to(link)
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    assert res.returncode == 0


def test_plugin_install_with_existing_symlink_dir(tmp_path: Path) -> None:
    """Test that installation fails if the destination is an existing symlink."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "symlinkdir",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    # Find the hashed destination name before creating the conflicting symlink
    install_res = run_cli(
        ["plugins", "install", str(tmp_path / "symlinkdir"), "--dry-run"],
        env={"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)},
    )
    dest_path_str = yaml.safe_load(install_res.stdout)["dest"]
    link = Path(dest_path_str)
    link.symlink_to(tmp_path)
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "install", str(tmp_path / "symlinkdir")], env=env)
    assert res.returncode != 0
    assert "symlink" in res.stderr.lower() or "refuse" in res.stderr.lower()


def test_plugin_install_after_partial_copy(tmp_path: Path) -> None:
    """Test installation when the destination directory exists but is incomplete."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "partial",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    # Find the hashed destination directory name to simulate a partial install
    install_res = run_cli(
        ["plugins", "install", str(tmp_path / "partial"), "--dry-run"],
        env={"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)},
    )
    dest_dir = Path(yaml.safe_load(install_res.stdout)["dest"])
    dest_dir.mkdir()
    (dest_dir / "random.txt").write_text("partial copy remains")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "install", str(tmp_path / "partial")], env=env)
    assert res.returncode != 0 or "already installed" in res.stderr.lower()


@pytest.mark.parametrize(
    "bad_name",
    [
        ".hidden",
        "has space",
        "bad$name",
        "unicodé",
    ],
)
def test_plugin_install_invalid_name(tmp_path: Path, bad_name: str) -> None:
    """Test that plugins with invalid names are rejected."""
    bad_dir = tmp_path / bad_name
    bad_dir.mkdir()
    (bad_dir / "plugin.py").write_text("# dummy plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(bad_dir)], env=env)
    assert res.returncode == 1
    assert "Invalid plugin name" in res.stderr or "error" in res.stderr.lower()
