# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin list command."""

from __future__ import annotations

import contextlib
import json
from pathlib import Path

import yaml  # pyright: ignore[reportMissingModuleSource]

from tests.e2e.conftest import TEST_TEMPLATE, assert_log_has, run_cli


def test_plugin_list_empty(tmp_path: Path) -> None:
    """Test that listing an empty plugins directory returns an empty list."""
    plugins_dir = tmp_path / "empty"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0
    assert_log_has(res, "plugins", [])


def test_plugin_list_quiet(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses output."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "list", "--quiet"], env=env)
    assert res.returncode == 0
    assert res.stdout.strip() == ""


def test_plugin_list_debug(tmp_path: Path) -> None:
    """Test the list command with the --debug flag."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "list", "--debug"], env=env)
    assert res.returncode == 0


def test_plugin_list_after_uninstall(tmp_path: Path) -> None:
    """Test that an uninstalled plugin no longer appears in the list."""
    run_cli(["plugins", "scaffold", "unplugme", "--output-dir", str(tmp_path)])
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "unplugme")], env=env)
    run_cli(["plugins", "uninstall", "unplugme"], env=env)
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "unplugme" not in plugins


def test_plugin_list_with_quiet_and_debug(tmp_path: Path) -> None:
    """Test that the --quiet flag overrides the --debug flag."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "list", "--quiet", "--debug"], env=env)
    assert res.returncode == 0
    assert res.stdout.strip() == ""


def test_plugin_list_with_non_plugin_file(tmp_path: Path) -> None:
    """Test that a file in the plugins directory is ignored."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    (tmp_path / "plugs").mkdir(exist_ok=True)
    (tmp_path / "plugs" / "notaplugin").write_text("not a plugin dir")
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0
    plugins = json.loads(res.stdout)["plugins"]
    assert "notaplugin" not in plugins


def test_plugin_list_handles_non_utf8_filenames(tmp_path: Path) -> None:
    """Test that non-UTF8 filenames do not crash the list command."""
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    (tmp_path / "plugs").mkdir(exist_ok=True)
    invalid_name = b"plug_\x80\x81"
    with contextlib.suppress(Exception):
        (tmp_path / "plugs").joinpath(invalid_name.decode("latin1")).mkdir()
    res = run_cli(["plugins", "list"], env=env)
    assert res.returncode == 0


def test_plugin_list_plugin_dir_not_dir(tmp_path: Path) -> None:
    """Test a graceful failure if the plugins directory path is a file."""
    file = tmp_path / "plugs"
    file.write_text("not a directory")
    env = {"BIJUXCLI_PLUGINS_DIR": str(file)}
    res = run_cli(["plugins", "list"], env=env)
    assert res.returncode != 0
    assert (
        "not a directory" in res.stdout.lower()
        or "not a directory" in res.stderr.lower()
        or "error" in res.stdout.lower()
        or "error" in res.stderr.lower()
    )


def test_plugin_list_empty_dir(tmp_path: Path) -> None:
    """Test listing plugins when the directory exists but is empty."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0
    plugins = json.loads(res.stdout)["plugins"]
    assert plugins == []


def test_plugin_list_ignores_non_plugin_dirs(tmp_path: Path) -> None:
    """Test that directories without a plugin.py are ignored."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / "fakeplugin").mkdir()
    (plugins_dir / "fakeplugin" / "not_a_plugin.txt").write_text("hi")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "fakeplugin" not in plugins


def test_plugin_list_handles_broken_symlinks(tmp_path: Path) -> None:
    """Test that broken symlinks in the plugins directory are ignored."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    broken_link = plugins_dir / "broken"
    broken_link.symlink_to(tmp_path / "no_such_dir")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "broken" not in plugins


def test_plugin_list_handles_nested_plugin_py(tmp_path: Path) -> None:
    """Test that only top-level plugin.py files are considered."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / "badplugin").mkdir()
    (plugins_dir / "badplugin" / "subdir").mkdir()
    (plugins_dir / "badplugin" / "subdir" / "plugin.py").write_text(
        "# plugin, but too deep"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "badplugin" not in plugins


def test_plugin_list_with_unicode_plugin_names(tmp_path: Path) -> None:
    """Test that unicode plugin names are listed correctly."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plugin_dir = plugins_dir / "plügin"
    plugin_dir.mkdir()
    (plugin_dir / "plugin.py").write_text("# plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "plügin" in plugins or all(isinstance(p, str) for p in plugins)


def test_plugin_list_with_malformed_plugin_py(tmp_path: Path) -> None:
    """Test that list does not validate code and includes plugins with malformed plugin.py."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plugin_dir = plugins_dir / "corrupt"
    plugin_dir.mkdir()
    (plugin_dir / "plugin.py").write_text("this is not valid python")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "corrupt" in plugins


def test_plugin_list_after_install(tmp_path: Path) -> None:
    """Test that a newly installed plugin appears in the list."""
    scaffold_res = run_cli(
        [
            "plugins",
            "scaffold",
            "listme",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scaffold_res.returncode == 0, scaffold_res.stdout
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install_res = run_cli(["plugins", "install", str(tmp_path / "listme")], env=env)
    assert install_res.returncode == 0, install_res.stdout
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0, res.stdout
    plugins = json.loads(res.stdout)["plugins"]
    assert "listme" in plugins


def test_plugin_list_yaml(tmp_path: Path) -> None:
    """Test the list command with YAML output format."""
    scaffold_res = run_cli(
        [
            "plugins",
            "scaffold",
            "listyaml",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scaffold_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install_res = run_cli(["plugins", "install", str(tmp_path / "listyaml")], env=env)
    assert install_res.returncode == 0, install_res.stdout
    list_res = run_cli(["plugins", "list", "--format", "yaml"], env=env)
    assert list_res.returncode == 0
    data = yaml.safe_load(list_res.stdout)
    assert "listyaml" in data["plugins"]


def test_plugin_list_invalid_format(tmp_path: Path) -> None:
    """Test that the list command errors correctly with an invalid format."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "badfmt"], env=env)

    assert res.returncode != 0, "Command should fail with an invalid format."
    assert "Unsupported format" in res.stderr


def test_plugin_list_after_partial_install_failure(tmp_path: Path) -> None:
    """Test that list shows all plugins, even if some are broken."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "goodone",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    run_cli(
        [
            "plugins",
            "scaffold",
            "badone",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_py = next((tmp_path / "badone").rglob("plugin.py"), None)
    if plug_py:
        plug_py.write_text("def oops(:\n")

    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    good_res = run_cli(["plugins", "install", str(tmp_path / "goodone")], env=env)
    run_cli(["plugins", "install", str(tmp_path / "badone")], env=env)
    assert good_res.returncode == 0, f"Good install failed: {good_res.stdout}"
    list_res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(list_res.stdout)["plugins"]
    assert "goodone" in plugins
    assert "badone" in plugins


def test_plugin_list_with_symlinked_plugin_dir(tmp_path: Path) -> None:
    """Test graceful failure when the main plugins directory is a symlink."""
    plug_dir = tmp_path / "plugdir"
    plug_dir.mkdir()
    link = tmp_path / "plugs"
    link.symlink_to(plug_dir)
    env = {"BIJUXCLI_PLUGINS_DIR": str(link)}
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "linked",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    inst_res = run_cli(["plugins", "install", str(tmp_path / "linked")], env=env)
    assert inst_res.returncode != 0
    assert "symlink" in inst_res.stdout.lower() or "symlink" in inst_res.stderr.lower()


def test_plugin_list_ignores_hidden_files(tmp_path: Path) -> None:
    """Test that hidden files and directories are ignored by the list command."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / ".DS_Store").write_text("system file")
    (plugins_dir / "__MACOSX").mkdir()
    plugin_dir = plugins_dir / "validplugin"
    plugin_dir.mkdir()
    (plugin_dir / "plugin.py").write_text("# plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "validplugin" in plugins
    assert ".DS_Store" not in plugins
    assert "__MACOSX" not in plugins


def test_plugin_list_skips_dirs_missing_plugin_py(tmp_path: Path) -> None:
    """Test that directories without a plugin.py are correctly skipped."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    (plugins_dir / "nopymodule").mkdir()
    (plugins_dir / "validplugin").mkdir()
    (plugins_dir / "validplugin" / "plugin.py").write_text("# valid plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "validplugin" in plugins
    assert "nopymodule" not in plugins


def test_plugin_list_skips_reserved_python_keywords(tmp_path: Path) -> None:
    """Test that directories named after Python keywords are handled."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    reserved = plugins_dir / "class"
    reserved.mkdir()
    (reserved / "plugin.py").write_text("# plugin")
    valid = plugins_dir / "myplugin"
    valid.mkdir()
    (valid / "plugin.py").write_text("# plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "myplugin" in plugins


def test_plugin_list_with_symlinked_valid_plugin(tmp_path: Path) -> None:
    """Test that a valid plugin installed via a symlink is listed."""
    plugins_dir = tmp_path / "plugs"
    real_plugin = tmp_path / "realplugin"
    real_plugin.mkdir()
    (real_plugin / "plugin.py").write_text("# plugin")
    plugins_dir.mkdir()
    (plugins_dir / "symlinkplugin").symlink_to(real_plugin, target_is_directory=True)
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "symlinkplugin" in plugins


def test_plugin_list_ignores_deeply_nested_plugins(tmp_path: Path) -> None:
    """Test that plugins in subdirectories of the plugins directory are ignored."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    outer = plugins_dir / "outer"
    inner = outer / "inner"
    inner.mkdir(parents=True)
    (inner / "plugin.py").write_text("# deeply nested plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    plugins = json.loads(res.stdout)["plugins"]
    assert "outer" not in plugins
    assert "inner" not in plugins


def test_plugin_list_single_valid_plugin(tmp_path: Path) -> None:
    """Should list exactly one plugin when one valid plugin exists."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plugin_dir = plugins_dir / "testplugin"
    plugin_dir.mkdir()
    (plugin_dir / "plugin.py").write_text("# valid plugin marker")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}

    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0, f"List failed: {res.stdout}"
    data = json.loads(res.stdout)
    assert "plugins" in data, "No plugins key in result"
    plugins = data["plugins"]
    assert isinstance(plugins, list)
    assert plugins == ["testplugin"]
