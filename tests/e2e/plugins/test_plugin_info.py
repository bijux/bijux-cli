# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin info command."""

from __future__ import annotations

import json
from pathlib import Path
import stat

import yaml

from tests.e2e.conftest import TEST_TEMPLATE, run_cli


def test_plugin_info_ok(tmp_path: Path) -> None:
    """Test a successful info command on a valid plugin."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "infoplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "infoplug")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    info = run_cli(["plugins", "info", "infoplug", "--format", "json"], env=env)
    assert info.returncode == 0, info.stdout
    meta = json.loads(info.stdout)
    assert meta.get("name") == "infoplug", f"Metadata: {meta}"


def test_plugin_info_nonexistent() -> None:
    """Test that getting info for a non-existent plugin fails."""
    res = run_cli(["plugins", "info", "notfound", "--format", "json"])
    assert res.returncode != 0 or "not found" in res.stdout.lower()


def test_plugin_info_yaml(tmp_path: Path) -> None:
    """Test the info command with YAML output format."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "yamlinfo",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "yamlinfo")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    info = run_cli(["plugins", "info", "yamlinfo", "--format", "yaml"], env=env)
    assert info.returncode == 0, info.stdout
    meta = yaml.safe_load(info.stdout)
    assert meta.get("name") == "yamlinfo", f"Metadata: {meta}"


def test_plugin_info_quiet(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses output."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "infoplugq",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "infoplugq")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    info = run_cli(["plugins", "info", "infoplugq", "--quiet"], env=env)
    assert info.returncode == 0, info.stdout
    assert info.stdout.strip() == ""


def test_plugin_info_invalid_format(tmp_path: Path) -> None:
    """Test that providing an invalid format errors correctly."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "fmtinfo",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"

    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}

    install = run_cli(["plugins", "install", str(tmp_path / "fmtinfo")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stderr}"

    info = run_cli(["plugins", "info", "fmtinfo", "--format", "invalid"], env=env)

    assert info.returncode != 0, "Command should fail when given an invalid format."
    assert "Unsupported format" in info.stderr, (
        "Expected error message not found in output."
    )


def test_plugin_info_after_uninstall(tmp_path: Path) -> None:
    """Test getting info for a plugin after it has been uninstalled."""
    scf_res = run_cli(
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
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "goneplug")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    uninstall = run_cli(["plugins", "uninstall", "goneplug"], env=env)
    assert uninstall.returncode == 0, f"Uninstall failed: {uninstall.stdout}"
    info = run_cli(["plugins", "info", "goneplug"], env=env)
    assert info.returncode != 0 or "not found" in info.stdout.lower()


def test_plugin_info_yaml_vs_json(tmp_path: Path) -> None:
    """Test that YAML and JSON outputs are consistent."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "multiformat",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "multiformat")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    json_out = run_cli(["plugins", "info", "multiformat", "--format", "json"], env=env)
    yaml_out = run_cli(["plugins", "info", "multiformat", "--format", "yaml"], env=env)
    assert json_out.returncode == 0, json_out.stdout
    assert yaml_out.returncode == 0, yaml_out.stdout
    data_json = json.loads(json_out.stdout)
    data_yaml = yaml.safe_load(yaml_out.stdout)
    assert data_json == data_yaml, f"JSON: {data_json} vs YAML: {data_yaml}"


def test_plugin_info_broken_metadata(tmp_path: Path) -> None:
    """Test that getting info for a plugin with a corrupt metadata file errors correctly."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "meta",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"

    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "meta")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stderr}"

    meta_file = plugins_dir / "meta" / "plugin.json"
    if meta_file.exists():
        meta_file.write_text("{not json}")

    info = run_cli(["plugins", "info", "meta"], env=env)

    assert info.returncode != 0, "Command should fail with corrupt metadata."
    assert "metadata is corrupt" in info.stderr


def test_plugin_info_quiet_and_debug(tmp_path: Path) -> None:
    """Test that the --quiet flag overrides the --debug flag."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "infoqd",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "infoqd")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"
    info = run_cli(["plugins", "info", "infoqd", "--quiet", "--debug"], env=env)
    assert info.returncode == 0, info.stdout
    assert info.stdout.strip() == ""


def test_plugin_info_with_missing_plugin_py(tmp_path: Path) -> None:
    """Test getting info for a plugin that is missing its plugin.py file."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "nopy",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stderr}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "nopy")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stderr}"

    # Corrupt the installed plugin by removing plugin.py
    installed_plugin_dirs = list((plugins_dir).glob("nopy*"))
    if installed_plugin_dirs:
        plug_py = next(installed_plugin_dirs[0].glob("**/plugin.py"), None)
        if plug_py:
            plug_py.unlink()

    info = run_cli(["plugins", "info", "nopy"], env=env)
    assert info.returncode != 0
    assert "not found" in info.stderr.lower()


def test_plugin_info_fails_on_corrupt_symlink(tmp_path: Path) -> None:
    """Test a graceful failure when the plugin path is a broken symlink."""
    scf_res = run_cli(
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
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    link = plugins_dir / "symlinkplug"
    link.symlink_to(tmp_path / "does_not_exist")
    info = run_cli(["plugins", "info", "symlinkplug"], env=env)
    assert (
        info.returncode != 0
        or "error" in info.stdout.lower()
        or "not found" in info.stdout.lower()
    )


def test_plugin_info_missing_json_file(tmp_path: Path) -> None:
    """Test getting info for a plugin with a missing plugin.json file."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "noj",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0, f"Scaffold failed: {scf_res.stdout}"
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(tmp_path / "noj")], env=env)
    assert install.returncode == 0, f"Install failed: {install.stdout}"

    # Corrupt by removing plugin.json from the installed plugin
    installed_plugin_dirs = list((plugins_dir).glob("noj*"))
    if installed_plugin_dirs:
        pj = installed_plugin_dirs[0] / "plugin.json"
        if pj.exists():
            pj.unlink()

    info = run_cli(["plugins", "info", "noj"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "noj"


def test_plugin_info_handles_extra_files(tmp_path: Path) -> None:
    """Test that extra files in a plugin directory do not break the info command."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "extrafiles",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "extrafiles"
    (plug_path / "notes.txt").write_text("Some irrelevant note")
    (plug_path / ".DS_Store").write_text("Junk")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    info = run_cli(["plugins", "info", "extrafiles", "--format", "json"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "extrafiles"


def test_plugin_info_does_not_read_subdirectories(tmp_path: Path) -> None:
    """Test that the info command ignores subdirectories within a plugin."""
    scf_res = run_cli(
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
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "subdirplug"
    (plug_path / "ignored_subdir").mkdir()
    ((plug_path / "ignored_subdir") / "plugin.py").write_text("# Not the main plugin")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    info = run_cli(["plugins", "info", "subdirplug", "--format", "json"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "subdirplug"


def test_plugin_info_symlinked_plugin_json(tmp_path: Path) -> None:
    """Test getting info for a plugin where plugin.json is a symlink."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "symlinkmeta",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "symlinkmeta"
    # Find the installed plugin path, which may have a hash suffix
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("symlinkmeta*"))
    assert installed_plugin_dirs
    installed_plugin_path = installed_plugin_dirs[0]

    orig_meta = installed_plugin_path / "plugin.json"
    backup_meta = installed_plugin_path / "meta.json"
    orig_meta.rename(backup_meta)
    orig_meta.symlink_to(backup_meta)

    info = run_cli(["plugins", "info", "symlinkmeta", "--format", "json"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "symlinkmeta"


def test_plugin_info_large_metadata(tmp_path: Path) -> None:
    """Test getting info for a plugin with a very large metadata file."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "bigmeta",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "bigmeta"
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("bigmeta*"))
    assert installed_plugin_dirs
    meta_file = installed_plugin_dirs[0] / "plugin.json"
    big_meta = {"name": "bigmeta", "description": "x" * 100_000}
    meta_file.write_text(json.dumps(big_meta))
    info = run_cli(["plugins", "info", "bigmeta", "--format", "json"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "bigmeta"
    assert "description" in meta


def test_plugin_info_non_utf8_metadata(tmp_path: Path) -> None:
    """Test a graceful failure when metadata is not UTF-8 encoded."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "latinmeta",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "latinmeta"
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("latinmeta*"))
    assert installed_plugin_dirs
    meta_file = installed_plugin_dirs[0] / "plugin.json"
    meta_file.write_bytes(b'{"name": "latinmeta", "desc": "\xe9xample"}')
    info = run_cli(["plugins", "info", "latinmeta"], env=env)
    assert info.returncode != 0 or "error" in info.stdout.lower()


def test_plugin_info_unexpected_metadata_fields(tmp_path: Path) -> None:
    """Test that unexpected fields in metadata are handled correctly."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "extrafields",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "extrafields"
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("extrafields*"))
    assert installed_plugin_dirs
    meta_file = installed_plugin_dirs[0] / "plugin.json"
    meta = {"name": "extrafields", "nonsense": 12345, "listfield": [1, 2, 3]}
    meta_file.write_text(json.dumps(meta))
    info = run_cli(["plugins", "info", "extrafields"], env=env)
    assert info.returncode == 0
    meta_out = json.loads(info.stdout)
    assert meta_out.get("name") == "extrafields"
    assert meta_out.get("nonsense") == 12345
    assert meta_out.get("listfield") == [1, 2, 3]


def test_plugin_info_permission_denied(tmp_path: Path) -> None:
    """Test a graceful failure when plugin files are not readable."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "denyinfo",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "denyinfo"
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("denyinfo*"))
    assert installed_plugin_dirs
    plugin_dir = installed_plugin_dirs[0]
    plugin_dir.chmod(0o000)
    info = run_cli(["plugins", "info", "denyinfo"], env=env)
    assert info.returncode != 0 or "error" in info.stdout.lower()
    plugin_dir.chmod(stat.S_IRWXU)


def test_plugin_info_many_files(tmp_path: Path) -> None:
    """Test getting info for a plugin that contains many extra files."""
    scf_res = run_cli(
        [
            "plugins",
            "scaffold",
            "manyfiles",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    assert scf_res.returncode == 0
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    plug_path = tmp_path / "manyfiles"
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    install = run_cli(["plugins", "install", str(plug_path)], env=env)
    assert install.returncode == 0
    installed_plugin_dirs = list(plugins_dir.glob("manyfiles*"))
    assert installed_plugin_dirs
    plug_dir = installed_plugin_dirs[0]
    for i in range(200):
        (plug_dir / f"garbage_{i}.txt").write_text("data")
    info = run_cli(["plugins", "info", "manyfiles"], env=env)
    assert info.returncode == 0
    meta = json.loads(info.stdout)
    assert meta.get("name") == "manyfiles"
