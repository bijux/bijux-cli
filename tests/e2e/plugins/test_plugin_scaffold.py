# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin scaffold command."""

from __future__ import annotations

import json
from pathlib import Path
import shutil

import yaml

from tests.e2e.conftest import TEST_TEMPLATE, run_cli


def test_plugin_scaffold_creates_directory(tmp_path: Path) -> None:
    """Test that the scaffold command successfully creates a plugin directory."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "plugA",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    assert (tmp_path / "plugA").is_dir()


def test_plugin_scaffold_overwrites_if_exists(tmp_path: Path) -> None:
    """Test that the --force flag allows overwriting an existing directory."""
    plug = tmp_path / "dupPlug"
    plug.mkdir()
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "dupPlug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
            "--force",
        ]
    )
    assert res.returncode == 0
    assert (tmp_path / "dupPlug").is_dir()


def test_plugin_scaffold_yaml_output(tmp_path: Path) -> None:
    """Test that the scaffold command supports YAML output."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "plugYaml",
            "--output-dir",
            str(tmp_path),
            "--format",
            "yaml",
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    data = yaml.safe_load(res.stdout)
    assert data["status"] == "created"
    assert data["plugin"] == "plugYaml"


def test_plugin_scaffold_debug(tmp_path: Path) -> None:
    """Test the scaffold command with the --debug flag."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "dbg",
            "--output-dir",
            str(tmp_path),
            "--debug",
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0


def test_plugin_scaffold_quiet(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses output."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "quietplug",
            "--output-dir",
            str(tmp_path),
            "--quiet",
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    assert res.stdout.strip() == ""


def test_plugin_scaffold_rejects_invalid_name(tmp_path: Path) -> None:
    """Test that scaffolding a plugin with an invalid name fails."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "bad name",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0
    assert (
        "invalid plugin name" in res.stdout.lower()
        or "invalid plugin name" in res.stderr.lower()
    )


def test_plugin_scaffold_existing_file(tmp_path: Path) -> None:
    """Test that scaffolding can overwrite an existing file with the --force flag."""
    plug_file = tmp_path / "fileplug"
    plug_file.write_text("not a directory")
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "fileplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
            "--force",
        ]
    )
    assert res.returncode == 0
    assert (tmp_path / "fileplug").is_dir()


def test_plugin_scaffold_invalid_format(tmp_path: Path) -> None:
    """Test that the scaffold command fails with an invalid format."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "invfmt",
            "--output-dir",
            str(tmp_path),
            "--format",
            "invalid",
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0


def test_plugin_scaffold_unicode_name(tmp_path: Path) -> None:
    """Test that scaffolding a plugin with a unicode name is rejected."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "插件",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0
    assert (
        "invalid plugin name" in res.stdout.lower() or "invalid" in res.stderr.lower()
    )


def test_plugin_scaffold_in_non_writable_dir(tmp_path: Path) -> None:
    """Test that scaffolding fails when the output directory is not writable."""
    unwritable = tmp_path / "nowrite"
    unwritable.mkdir()
    unwritable.chmod(0o400)
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "failplug",
            "--output-dir",
            str(unwritable),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0
    unwritable.chmod(0o700)


def test_plugin_scaffold_with_existing_nonempty_dir(tmp_path: Path) -> None:
    """Test that scaffolding fails if the target directory exists and is not empty."""
    plug_dir = tmp_path / "already"
    plug_dir.mkdir()
    (plug_dir / "foo.txt").write_text("not empty")
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "already",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0


def test_plugin_scaffold_creates_valid_plugin_py(tmp_path: Path) -> None:
    """Test that the scaffolded plugin.py is a valid Python file."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "validplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    plug_py = next((tmp_path / "validplug").glob("**/plugin.py"))
    assert plug_py.is_file()
    code = plug_py.read_text("utf-8")
    assert "def" in code or "class" in code
    compile(code, str(plug_py), "exec")


def test_plugin_scaffold_creates_plugin_json(tmp_path: Path) -> None:
    """Test that the scaffolded plugin.json is a valid JSON file."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "jsonplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    plug_json = next((tmp_path / "jsonplug").glob("**/plugin.json"))
    assert plug_json.is_file()
    with plug_json.open(encoding="utf-8") as f:
        data = json.load(f)
    assert isinstance(data, dict)


def test_plugin_scaffold_plugin_py_utf8(tmp_path: Path) -> None:
    """Test that the scaffolded plugin.py is UTF-8 encoded."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "utfplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    plug_py = next((tmp_path / "utfplug").glob("**/plugin.py"))
    text = plug_py.read_bytes()
    s = text.decode("utf-8")
    assert any(c.isalpha() for c in s)


def test_plugin_scaffold_does_not_copy_temp_files(tmp_path: Path) -> None:
    """Test that temporary or system files are not copied into the scaffold."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "cleanplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    plug_dir = tmp_path / "cleanplug"
    forbidden = [".DS_Store", "__pycache__"]
    for f in forbidden:
        assert not (plug_dir / f).exists()


def test_plugin_scaffold_force_overwrites_nonempty(tmp_path: Path) -> None:
    """Test that --force allows overwriting a non-empty directory."""
    plug_dir = tmp_path / "nonempty"
    plug_dir.mkdir()
    (plug_dir / "file.txt").write_text("keep me")
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "nonempty",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
            "--force",
        ]
    )
    assert res.returncode == 0
    assert not (plug_dir / "file.txt").exists()
    assert (plug_dir / "plugin.py").exists()


def test_plugin_scaffold_subdir(tmp_path: Path) -> None:
    """Test scaffolding a plugin into a nested subdirectory."""
    sub = tmp_path / "subdir" / "subsub"
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "deepplug",
            "--output-dir",
            str(sub),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    assert (sub / "deepplug" / "plugin.py").exists()


def test_plugin_scaffold_clears_symlink(tmp_path: Path) -> None:
    """Test that --force correctly replaces a symlink with the new directory."""
    plug_dir = tmp_path / "symlinkplug"
    alt = tmp_path / "target"
    alt.mkdir()
    plug_dir.symlink_to(alt, target_is_directory=True)
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "symlinkplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
            "--force",
        ]
    )
    assert res.returncode == 0
    assert plug_dir.is_dir()
    assert not plug_dir.is_symlink()
    assert (plug_dir / "plugin.py").exists()


def test_plugin_scaffold_ignores_extra_template_files(tmp_path: Path) -> None:
    """Test that extra, non-essential files are not created in the scaffold."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "extraplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    plug_dir = tmp_path / "extraplug"
    forbidden = ["README.md", ".gitkeep"]
    for f in forbidden:
        assert not (plug_dir / f).exists()


def test_plugin_scaffold_idempotent(tmp_path: Path) -> None:
    """Test that repeated scaffolding (with cleanup) produces the same result."""
    name = "repeatplug"
    res1 = run_cli(
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
    assert res1.returncode == 0
    assert (tmp_path / name).is_dir()
    shutil.rmtree(tmp_path / name)
    res2 = run_cli(
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
    assert res2.returncode == 0
    assert (tmp_path / name).is_dir()
    files = {p.name for p in (tmp_path / name).rglob("*") if p.is_file()}
    assert "plugin.py" in files
    assert "plugin.json" in files


def test_plugin_scaffold_handles_long_plugin_name(tmp_path: Path) -> None:
    """Test that the scaffold command handles very long plugin names."""
    long_name = "a" * 50
    res = run_cli(
        [
            "plugins",
            "scaffold",
            long_name,
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode == 0
    assert (tmp_path / long_name / "plugin.py").exists()


def test_plugin_scaffold_fails_with_reserved_name(tmp_path: Path) -> None:
    """Test that scaffolding a plugin with a Python reserved keyword as a name fails."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "class",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert res.returncode != 0
    assert "invalid" in res.stdout.lower() or "reserved" in res.stderr.lower()


def test_plugin_scaffold_removes_broken_symlink(tmp_path: Path) -> None:
    """Test that --force allows scaffolding over a broken symlink."""
    plug_dir = tmp_path / "deadplug"
    plug_dir.symlink_to(tmp_path / "nonexistent_target")
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "deadplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
            "--force",
        ]
    )
    assert res.returncode == 0
    assert plug_dir.is_dir()
    assert (plug_dir / "plugin.py").exists()


def test_plugin_scaffold_fails_if_template_missing(tmp_path: Path) -> None:
    """Test that scaffolding fails if the specified template does not exist."""
    res = run_cli(
        [
            "plugins",
            "scaffold",
            "notempl",
            "--output-dir",
            str(tmp_path),
            "--template",
            str(tmp_path / "doesnotexist"),
        ]
    )
    assert res.returncode != 0
    assert "template" in res.stdout.lower() or "not found" in res.stderr.lower()


def test_plugin_scaffold_name_case_insensitive_duplicate(tmp_path: Path) -> None:
    """Test that case-insensitive duplicate plugin names are rejected."""
    scf1 = run_cli(
        [
            "plugins",
            "scaffold",
            "Upper",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf1.returncode == 0, scf1.stdout

    scf2 = run_cli(
        [
            "plugins",
            "scaffold",
            "upper",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf2.returncode != 0, scf2.stdout
    assert "conflict" in scf2.stderr.lower() or "exists" in scf2.stderr.lower()

    expected_dir = tmp_path / "Upper"
    assert expected_dir.is_dir()

    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    ins1 = run_cli(["plugins", "install", str(expected_dir)], env=env)
    assert ins1.returncode == 0, ins1.stdout
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0, res.stdout
    plugins = json.loads(res.stdout)["plugins"]
    assert "Upper" in plugins
    lowered = [name.lower() for name in plugins]
    assert lowered.count("upper") == 1, plugins
