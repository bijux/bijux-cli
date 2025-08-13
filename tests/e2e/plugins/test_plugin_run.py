# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for the plugin run command."""

from __future__ import annotations

import contextlib
from pathlib import Path

from tests.e2e.conftest import TEST_TEMPLATE, run_cli


def test_plugin_run_custom_command(tmp_path: Path) -> None:
    """Test running a custom command from an installed plugin."""
    name = "cmdplug"
    plug_dir = tmp_path / name
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
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
    plug_py = next(plug_dir.glob("**/plugin.py"))
    plug_py.write_text(
        "import typer\n"
        "app = typer.Typer()\n\n"
        "@app.command('run')\n"
        "def run(input: str):\n"
        "    print(f'Hello from {input}')\n"
    )
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    res = run_cli([name, "run", "hello"], env=env)
    assert res.returncode == 0, res.stderr
    assert "Hello from hello" in res.stdout or "Hello from hello" in res.stderr


def test_plugin_run_after_reinstall(tmp_path: Path) -> None:
    """Test that a plugin can be uninstalled, reinstalled, and still run."""
    name = "replug"
    plug_dir = tmp_path / name
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
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
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    run_cli(["plugins", "uninstall", name], env=env)
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    res = run_cli([name, "run", "again"], env=env)
    assert res.returncode == 0, res.stderr
    assert "again" in res.stdout or "again" in res.stderr


def test_plugin_run_invalid(tmp_path: Path) -> None:
    """Test that running a non-existent subcommand from a plugin returns an error."""
    name = "failcmd"
    plug_dir = tmp_path / name
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
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
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    res = run_cli([name, "notacommand"], env=env)

    assert res.returncode != 0
    assert "No such command" in res.stdout or "notacommand" in res.stdout


def test_plugin_run_after_uninstall(tmp_path: Path) -> None:
    """Test that a plugin is not callable after being uninstalled."""
    name = "goneplug"
    plug_dir = tmp_path / name
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
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
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    run_cli(["plugins", "uninstall", name], env=env)
    res = run_cli([name, "run", "test"], env=env)

    assert res.returncode != 0
    assert "No such command" in res.stdout or name in res.stdout


def test_plugin_run_with_env_var(tmp_path: Path) -> None:
    """Test that a plugin can access custom environment variables at runtime."""
    name = "envplug"
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
    plug_py = next((tmp_path / name).glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text() + "\nimport os\n"
        "@app.command('envtest')\n"
        "def envtest():\n"
        "    print('MYENVVAR=' + os.environ.get('MYENVVAR', 'unset'))\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs"), "MYENVVAR": "xyz"}
    run_cli(["plugins", "install", str(tmp_path / name)], env=env)
    res = run_cli([name, "envtest"], env=env)
    output = res.stdout or res.stderr
    assert "MYENVVAR=xyz" in output


def test_plugin_run_crashes_should_not_crash_cli(tmp_path: Path) -> None:
    """Test that a plugin that raises an exception does not crash the main CLI process."""
    run_cli(
        [
            "plugins",
            "scaffold",
            "crashplug",
            "--output-dir",
            str(tmp_path),
            "--template",
            TEST_TEMPLATE,
        ]
    )
    plug_py = next((tmp_path / "crashplug").glob("**/plugin.py"))
    plug_py.write_text(
        plug_py.read_text()
        + "\n@app.command('explode')\ndef explode():\n    raise RuntimeError('boom')\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(tmp_path / "crashplug")], env=env)
    res = run_cli(["crashplug", "explode"], env=env)
    assert res.returncode != 0
    error_out = res.stdout + res.stderr
    assert "boom" in error_out or "RuntimeError" in error_out


def test_plugin_run_version_compatible(tmp_path: Path) -> None:
    """Test that a plugin with a compatible version requirement installs successfully."""
    name = "versplug"
    plug_dir = tmp_path / name
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
    plugin_py = next(plug_dir.glob("**/plugin.py"))
    with plugin_py.open("a") as fh:
        fh.write("\nrequires_cli_version = '>=0.1.0,<0.2.0'\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    assert res.returncode == 0, res.stderr


def test_plugin_run_version_incompatible(tmp_path: Path) -> None:
    """Test that a plugin with an incompatible version requirement fails to install."""
    name = "badvers"
    plug_dir = tmp_path / name
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
    plugin_py = next(plug_dir.glob("**/plugin.py"))
    with plugin_py.open("a") as fh:
        fh.write("\nrequires_cli_version = '>=9.9.9'\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plug_dir)], env=env)
    error_out = res.stdout + res.stderr
    assert (
        res.returncode != 0
        or "Failed to load plugin" in error_out
        or "incompatible" in error_out
    )


def test_plugin_run_with_subcommand(tmp_path: Path) -> None:
    """Test that a plugin can expose and run its own subcommands."""
    name = "subcmdplug"
    plug_dir = tmp_path / name
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
    plugin_py = next(plug_dir.glob("**/plugin.py"))
    plugin_py.write_text(
        "import typer\n"
        "app=typer.Typer()\n"
        "@app.command('echo')\n"
        "def echo(text: str):\n"
        "    print(f'ECHO: {text}')\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(plug_dir)], env=env)
    res = run_cli([name, "echo", "test"], env=env)
    assert res.returncode == 0, res.stderr
    assert "ECHO: test" in res.stdout or "ECHO: test" in res.stderr


def test_plugin_run_symlinked_plugin_dir(tmp_path: Path) -> None:
    """Test that installing from a symlinked directory is handled correctly."""
    plugdir = tmp_path / "realplugin"
    plugdir.mkdir()
    (plugdir / "plugin.py").write_text(
        "import typer\n"
        "app = typer.Typer()\n"
        "@app.command('run')\n"
        "def run(): print('ran')\n"
    )
    symlink_dir = tmp_path / "symlinkplug"
    symlink_dir.symlink_to(plugdir, target_is_directory=True)
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    install_res = run_cli(["plugins", "install", str(symlink_dir)], env=env)
    assert install_res.returncode == 0


def test_plugin_run_plugin_py_missing(tmp_path: Path) -> None:
    """Test that installing a plugin without a plugin.py file fails."""
    plugdir = tmp_path / "nopymodule"
    plugdir.mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    res = run_cli(["plugins", "install", str(plugdir)], env=env)
    assert res.returncode != 0
    assert "plugin.py" in (res.stdout + res.stderr)


def test_plugin_run_broken_symlink(tmp_path: Path) -> None:
    """Test that a broken symlink in the plugins directory is ignored."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    broken = plugins_dir / "broken"
    broken.symlink_to(tmp_path / "doesnotexist")
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert "broken" not in (res.stdout + res.stderr)


def test_plugin_run_with_non_utf8_filename(tmp_path: Path) -> None:
    """Test that a non-UTF8 filename in the plugins directory does not crash the CLI."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    invalid_name = b"bad_\x80"
    with contextlib.suppress(Exception):
        (plugins_dir / invalid_name.decode("latin1")).mkdir()
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0


def test_plugin_run_with_reserved_python_keyword(tmp_path: Path) -> None:
    """Test that a plugin named after a Python keyword is handled."""
    plugins_dir = tmp_path / "plugs"
    plugins_dir.mkdir()
    reserved = plugins_dir / "class"
    reserved.mkdir()
    (reserved / "plugin.py").write_text(
        "import typer\n"
        "app = typer.Typer()\n"
        "@app.command('run')\n"
        "def run(): print('should work')\n"
    )
    env = {"BIJUXCLI_PLUGINS_DIR": str(plugins_dir)}
    res = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert res.returncode == 0


def test_plugin_run_plugin_py_is_invalid_python(tmp_path: Path) -> None:
    """Test that a plugin with a syntax error fails but does not crash the CLI."""
    plugdir = tmp_path / "corrupt"
    plugdir.mkdir()
    (plugdir / "plugin.py").write_text("def oops(:\n")
    env = {"BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugs")}
    run_cli(["plugins", "install", str(plugdir)], env=env)
    res = run_cli(["corrupt", "run"], env=env)
    assert res.returncode != 0
    assert "SyntaxError" in (res.stdout + res.stderr) or "corrupt" in (
        res.stdout + res.stderr
    )
