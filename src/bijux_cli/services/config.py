# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a robust, file-based configuration management service.

This module defines the `Config` class, a concrete implementation of the
`ConfigProtocol`. It is responsible for loading, accessing, and persisting
key-value configuration settings from `.env` files.

Key features include:
    * **Atomic Writes:** Changes are written to a temporary file before being
        atomically moved into place to prevent data corruption.
    * **Cross-Process Safety:** On POSIX systems, `fcntl.flock` is used with
        retries to handle concurrent access from multiple CLI processes.
    * **Key Normalization:** Configuration keys are handled case-insensitively
        and the `BIJUXCLI_` prefix is optional.
    * **Security Checks:** Includes validation to prevent operating on device
        files or traversing symbolic link loops.
"""

from __future__ import annotations

import codecs
import fcntl
import json
import os
from pathlib import Path
from tempfile import NamedTemporaryFile
import time
from typing import Any

from injector import inject

from bijux_cli.contracts import ConfigProtocol, ObservabilityProtocol
from bijux_cli.core.exceptions import CommandError
from bijux_cli.core.paths import CONFIG_FILE

yaml: Any
try:
    import yaml
except ImportError:  # pragma: no cover
    yaml = None


def _escape(raw: str) -> str:
    """Escapes special characters in a string for safe storage in a .env file.

    Args:
        raw (str): The raw string to escape.

    Returns:
        str: The escaped string.
    """
    return (
        raw.replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace('"', '\\"')
    )


def _unescape(raw: str) -> str:
    """Reverses the escaping process for strings read from a .env file.

    Args:
        raw (str): The escaped string to unescape.

    Returns:
        str: The unescaped, original string.

    Raises:
        ValueError: If the input string contains an invalid escape sequence.
    """
    try:
        return codecs.decode(raw, "unicode_escape")
    except UnicodeDecodeError as err:
        raise ValueError(f"Invalid escaped string: {raw}") from err


def _detect_symlink_loop(path: Path, max_depth: int = 10) -> None:
    """Detects symbolic link loops in a given path to prevent infinite recursion.

    Args:
        path (Path): The path to check.
        max_depth (int): The maximum number of symbolic links to follow.

    Raises:
        CommandError: If a loop is detected or the traversal depth exceeds `max_depth`.
    """
    seen: set[Path] = set()
    curr = path
    for _ in range(max_depth):
        if not curr.is_symlink():
            return
        try:
            raw = os.readlink(curr)
        except OSError as exc:
            raise CommandError(
                f"Symlink loop detected: {curr}", http_status=400
            ) from exc
        target = Path(raw)
        if not target.is_absolute():
            target = curr.parent / target
        if target in seen:
            raise CommandError(f"Symlink loop detected: {curr}", http_status=400)
        seen.add(target)
        curr = target
    raise CommandError(f"Symlink depth exceeded: {path}", http_status=400)


class Config(ConfigProtocol):
    """A robust configuration handler for `.env` files.

    This service manages loading, saving, and persisting configuration values,
    featuring atomic writes and key normalization. Keys are stored internally
    in lowercase and without the `BIJUXCLI_` prefix.

    Attributes:
        _di (Any): The dependency injection container.
        _log (ObservabilityProtocol): The logging service.
        _data (dict[str, str]): The in-memory dictionary of configuration data.
        _path (Path | None): The path to the configuration file being managed.
    """

    @inject
    def __init__(self, dependency_injector: Any) -> None:
        """Initializes the Config service and attempts to autoload configuration.

        Args:
            dependency_injector (Any): The DI container for resolving dependencies.
        """
        self._di = dependency_injector
        self._log: ObservabilityProtocol = dependency_injector.resolve(
            ObservabilityProtocol
        )
        self._data: dict[str, str] = {}
        self._path: Path | None = None
        try:
            self.load()
        except FileNotFoundError:
            pass
        except CommandError as e:
            self._log.log(
                "error", f"Auto-load of config failed during init: {e}", extra={}
            )

    def load(self, path: str | Path | None = None) -> None:
        """Loads configuration from a `.env` file.

        This method reads a specified `.env` file, parsing `KEY=VALUE` pairs.
        It handles comments, validates syntax, and normalizes keys. If no path
        is given, it uses the default path from `.env` or environment.

        Args:
            path (str | Path | None): Path to the `.env` file. If None, uses
                the default path from the environment or project structure.

        Raises:
            FileNotFoundError: If a specified config file does not exist.
            ValueError: If a line is malformed or contains non-ASCII characters.
            CommandError: If the file is binary or another parsing error occurs.
        """
        import_path = Path(path) if path is not None else None
        current_path = Path(os.getenv("BIJUXCLI_CONFIG", str(CONFIG_FILE)))
        self._validate_config_path(current_path)
        if import_path:
            self._validate_config_path(import_path)
        read_path = import_path or current_path
        _detect_symlink_loop(read_path)
        if not read_path.exists():
            if import_path is not None:
                raise FileNotFoundError(f"Config file not found: {read_path}")
            self._data = {}
            return
        new_data = {}
        try:
            content = read_path.read_text(encoding="utf-8")
            for i, line in enumerate(content.splitlines()):
                stripped = line.strip()
                if not stripped or stripped.startswith("#"):
                    continue
                if "=" not in line:
                    raise ValueError(f"Malformed line {i + 1}: {line}")
                key_part, val_part = line.split("=", 1)
                key = key_part.strip()
                value = _unescape(val_part)
                if len(value) >= 2 and value[0] == '"' and value[-1] == '"':
                    value = value[1:-1]
                if not all(ord(c) < 128 for c in key + value):
                    raise ValueError(f"Non-ASCII characters in line {i + 1}: {line}")
                normalized_key = key.strip().removeprefix("BIJUXCLI_").lower()
                new_data[normalized_key] = value
        except UnicodeDecodeError as exc:
            self._log.log(
                "error",
                f"Failed to parse config file {read_path}: Binary or non-text content",
                extra={"path": str(read_path)},
            )
            raise CommandError(
                f"Failed to parse config file {read_path}: Binary or non-text content",
                http_status=400,
            ) from exc
        except Exception as exc:
            self._log.log(
                "error",
                f"Failed to parse config file {read_path}: {exc}",
                extra={"path": str(read_path)},
            )
            raise CommandError(
                f"Failed to parse config file {read_path}: {exc}", http_status=400
            ) from exc
        self._data = new_data
        if import_path is not None and import_path != current_path:
            self._path = current_path
            self.set_many(new_data)
        else:
            self._path = read_path
        self._log.log(
            "info",
            f"Loaded config from {read_path} (active: {self._path})",
            extra={"src": str(read_path), "active": str(self._path)},
        )

    def set_many(self, items: dict[str, Any]) -> None:
        """Sets multiple key-value pairs and persists them to the config file.

        Args:
            items (dict[str, Any]): A dictionary of key-value pairs to set.
        """
        self._data = {k: str(v) for k, v in items.items()}
        if not self._path:
            self._path = Path(os.getenv("BIJUXCLI_CONFIG", str(CONFIG_FILE)))
            self._validate_config_path(self._path)
        _detect_symlink_loop(self._path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = self._path.with_suffix(".tmp")
        retry = 40
        while retry > 0:
            try:
                with open(tmp_path, "w", encoding="utf-8", newline="") as temp_file:
                    fd = temp_file.fileno()
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    for k, v in self._data.items():
                        safe_v = _escape(str(v))
                        temp_file.write(f"BIJUXCLI_{k.upper()}={safe_v}\n")
                    temp_file.flush()
                    os.fsync(fd)
                    fcntl.flock(fd, fcntl.LOCK_UN)
                tmp_path.replace(self._path)
                self._log.log(
                    "info",
                    f"Persisted config to {self._path}",
                    extra={"path": str(self._path)},
                )
                return
            except BlockingIOError:
                retry -= 1
                time.sleep(0.05)
            except Exception as exc:
                if tmp_path.exists():
                    tmp_path.unlink()
                self._log.log(
                    "error",
                    f"Failed to persist config to {self._path}: {exc}",
                    extra={"path": str(self._path)},
                )
                raise CommandError(
                    f"Failed to persist config to {self._path}: {exc}", http_status=500
                ) from exc
        if tmp_path.exists():
            tmp_path.unlink()
        raise CommandError(
            f"Failed to persist config to {self._path}: File locked after retries",
            http_status=400,
        )

    def all(self) -> dict[str, str]:
        """Returns all configuration key-value pairs.

        Returns:
            dict[str, str]: A dictionary of all configuration data.
        """
        return dict(self._data)

    def list_keys(self) -> list[str]:
        """Returns a list of all configuration keys.

        Returns:
            list[str]: A list of all keys in the configuration.
        """
        return list(self._data.keys())

    def clear(self) -> None:
        """Deletes all configuration entries and removes the config file.

        Raises:
            CommandError: If the config file cannot be deleted due to a lock
                or other filesystem error.
        """
        self._data = {}
        if self._path and self._path.exists():
            try:
                retry = 40
                while True:
                    try:
                        with open(self._path, "a+") as real:
                            fcntl.flock(real.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                            self._path.unlink()
                            fcntl.flock(real.fileno(), fcntl.LOCK_UN)
                        break
                    except BlockingIOError as err:
                        retry -= 1
                        if retry == 0:
                            raise CommandError(
                                f"Failed to clear config file {self._path}: File locked",
                                http_status=400,
                            ) from err
                        time.sleep(0.05)
            except Exception as exc:
                self._log.log(
                    "error",
                    f"Failed to clear config file {self._path}: {exc}",
                    extra={"path": str(self._path)},
                )
                raise CommandError(
                    f"Failed to clear config file {self._path}: {exc}", http_status=500
                ) from exc
        self._log.log(
            "info",
            "Cleared config data",
            extra={"path": str(self._path) if self._path else "None"},
        )

    def get(self, key: str, default: Any = None) -> Any:
        """Retrieves a configuration value by key.

        The key is normalized (lowercase, `BIJUXCLI_` prefix removed), and the
        environment is checked first before consulting the in-memory store.

        Args:
            key (str): The key to retrieve.
            default (Any): The value to return if the key is not found.

        Returns:
            Any: The value associated with the key, or the default value.

        Raises:
            CommandError: If the key is not found and no default is provided.
        """
        normalized_key = key.strip().removeprefix("BIJUXCLI_").lower()
        env_key = f"BIJUXCLI_{normalized_key.upper()}"
        if env_key in os.environ:
            return os.environ[env_key]
        value = self._data.get(normalized_key, default)
        if isinstance(value, str):
            val_lower = value.lower()
            if val_lower in {"true", "false"}:
                return val_lower == "true"
        if value is default and default is None:
            self._log.log(
                "error", f"Config key not found: {key}", extra={"key": normalized_key}
            )
            raise CommandError(f"Config key not found: {key}", http_status=400)
        self._log.log(
            "debug",
            f"Retrieved config key: {normalized_key}",
            extra={"key": normalized_key, "value": str(value)},
        )
        return value

    def set(self, key: str, value: Any) -> None:
        """Sets a single configuration key-value pair and persists it.

        Args:
            key (str): The key to set (case-insensitive, `BIJUXCLI_` prefix optional).
            value (Any): The value to associate with the key.

        Returns:
            None:

        Raises:
            CommandError: If the configuration cannot be persisted.
        """
        normalized_key = key.strip().removeprefix("BIJUXCLI_").lower()
        self._data[normalized_key] = str(value)
        if not self._path:
            self._path = Path(os.getenv("BIJUXCLI_CONFIG", str(CONFIG_FILE)))
            self._validate_config_path(self._path)
        _detect_symlink_loop(self._path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = self._path.with_suffix(".tmp")
        retry = 40
        while retry > 0:
            try:
                with open(tmp_path, "w", encoding="utf-8", newline="") as temp_file:
                    fd = temp_file.fileno()
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    for k, v in self._data.items():
                        safe_v = _escape(str(v))
                        temp_file.write(f"BIJUXCLI_{k.upper()}={safe_v}\n")
                    temp_file.flush()
                    os.fsync(fd)
                    fcntl.flock(fd, fcntl.LOCK_UN)
                tmp_path.replace(self._path)
                self._log.log(
                    "info",
                    f"Persisted config to {self._path}",
                    extra={"path": str(self._path), "key": normalized_key},
                )
                return
            except BlockingIOError:
                retry -= 1
                time.sleep(0.05)
            except Exception as exc:
                if tmp_path.exists():
                    tmp_path.unlink()
                self._log.log(
                    "error",
                    f"Failed to persist config to {self._path}: {exc}",
                    extra={"path": str(self._path)},
                )
                raise CommandError(
                    f"Failed to persist config to {self._path}: {exc}", http_status=500
                ) from exc
        if tmp_path.exists():
            tmp_path.unlink()
        raise CommandError(
            f"Failed to persist config to {self._path}: File locked after retries",
            http_status=400,
        )

    def reload(self) -> None:
        """Reloads configuration from the last-loaded file path.

        Raises:
            CommandError: If no file path has been previously loaded.
        """
        if self._path is None:
            self._log.log("error", "Config.reload() called before load()", extra={})
            raise CommandError("Config.reload() called before load()", http_status=400)
        if not self._path.exists():
            self._log.log(
                "error", f"Config file missing for reload: {self._path}", extra={}
            )
            raise CommandError(
                f"Config file missing for reload: {self._path}", http_status=400
            )
        self.load(self._path)

    def export(self, path: str | Path, out_format: str | None = None) -> None:
        """Exports the configuration to a file or standard output.

        Args:
            path (str | Path): The destination file path, or "-" for stdout.
            out_format (str | None): The output format ('env', 'json', 'yaml').
                If None, the format is auto-detected from the file extension.

        Raises:
            CommandError: If the format is unsupported or the export fails.
        """
        export_path = Path(path) if path != "-" else path
        output_fmt = (
            out_format.lower()
            if out_format
            else (
                "env"
                if path == "-" or str(path).endswith(".env")
                else "yaml"
                if str(path).endswith((".yaml", ".yml"))
                else "json"
            )
        )
        try:
            if output_fmt == "env":
                lines = [f"BIJUXCLI_{k.upper()}={v}" for k, v in self._data.items()]
                text = "\n".join(lines) + ("\n" if lines else "")
            elif output_fmt == "json":
                text = (
                    json.dumps({k.upper(): v for k, v in self._data.items()}, indent=2)
                    + "\n"
                )
            elif output_fmt == "yaml":
                if yaml is None:
                    raise CommandError(
                        "PyYAML not installed for YAML support", http_status=400
                    )
                text = yaml.safe_dump(
                    {k.upper(): v for k, v in self._data.items()}, sort_keys=False
                )
            else:
                raise CommandError(f"Unsupported format: {output_fmt}", http_status=400)
            if path == "-":
                print(text, end="")
                self._log.log(
                    "info",
                    "Exported config to stdout",
                    extra={"format": output_fmt},
                )
                return
            export_path = Path(path)
            export_path.resolve(strict=False)
            if not export_path.parent.exists():
                raise CommandError(
                    f"No such file or directory: {export_path.parent}", http_status=400
                )
            if export_path.exists() and not os.access(export_path, os.W_OK):
                raise PermissionError(f"Permission denied: '{export_path}'")
            if not os.access(export_path.parent, os.W_OK):
                raise PermissionError(f"Permission denied: '{export_path.parent}'")
            with NamedTemporaryFile(
                "w", delete=False, dir=export_path.parent, encoding="utf-8", newline=""
            ) as temp_file:
                fd = temp_file.fileno()
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                temp_file.write(text)
                temp_file.flush()
                os.fsync(fd)
                fcntl.flock(fd, fcntl.LOCK_UN)
                Path(temp_file.name).replace(export_path)
            self._log.log(
                "info",
                f"Exported config to {export_path}",
                extra={"path": str(export_path), "format": output_fmt},
            )
        except BlockingIOError as exc:
            self._log.log(
                "error",
                f"Failed to export config to {export_path}: File locked",
                extra={"path": str(export_path), "format": output_fmt},
            )
            raise CommandError(
                f"Failed to export config to {export_path}: File locked",
                http_status=400,
            ) from exc
        except (OSError, PermissionError, ValueError) as exc:
            self._log.log(
                "error",
                f"Failed to export config to {export_path}: {exc}",
                extra={"path": str(export_path), "format": output_fmt},
            )
            raise CommandError(
                f"Failed to export config to {export_path}: {exc}", http_status=400
            ) from exc

    def delete(self, key: str) -> None:
        """Deletes a configuration key and persists the change.

        Args:
            key (str): The key to delete (case-insensitive, `BIJUXCLI_` prefix optional).

        Raises:
            CommandError: If the key does not exist or the change cannot be persisted.
        """
        normalized_key = key.strip().removeprefix("BIJUXCLI_").lower()
        if normalized_key not in self._data:
            self._log.log(
                "error", f"Config key not found: {key}", extra={"key": normalized_key}
            )
            raise CommandError(f"Config key not found: {key}", http_status=400)
        del self._data[normalized_key]
        if not self._path:
            self._path = Path(os.getenv("BIJUXCLI_CONFIG", str(CONFIG_FILE)))
            self._validate_config_path(self._path)
        _detect_symlink_loop(self._path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = self._path.with_suffix(".tmp")
        retry = 40
        while retry > 0:
            try:
                with open(tmp_path, "w", encoding="utf-8", newline="") as temp_file:
                    fd = temp_file.fileno()
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    for k, v in self._data.items():
                        safe_v = _escape(str(v))
                        temp_file.write(f"BIJUXCLI_{k.upper()}={safe_v}\n")
                    temp_file.flush()
                    os.fsync(fd)
                    fcntl.flock(fd, fcntl.LOCK_UN)
                tmp_path.replace(self._path)
                self._log.log(
                    "info",
                    f"Deleted config key and persisted to {self._path}",
                    extra={"path": str(self._path), "key": normalized_key},
                )
                return
            except BlockingIOError:
                retry -= 1
                time.sleep(0.05)
            except Exception as exc:
                if tmp_path.exists():
                    tmp_path.unlink()
                self._log.log(
                    "error",
                    f"Failed to persist config after deleting {normalized_key}: {exc}",
                    extra={"path": str(self._path), "key": normalized_key},
                )
                raise CommandError(
                    f"Failed to persist config after deleting {normalized_key}: {exc}",
                    http_status=500,
                ) from exc
        if tmp_path.exists():
            tmp_path.unlink()
        raise CommandError(
            f"Failed to persist config to {self._path}: File locked after retries",
            http_status=400,
        )

    def unset(self, key: str) -> None:
        """Removes a configuration key (alias for `delete`).

        Args:
            key (str): The key to remove.
        """
        self.delete(key)

    def save(self) -> None:
        """Persists the current in-memory configuration to its source file."""
        if not self._path:
            self._path = Path(os.getenv("BIJUXCLI_CONFIG", str(CONFIG_FILE)))
            self._validate_config_path(self._path)
        try:
            self.set_many(self._data)
        except Exception as exc:
            self._log.log(
                "error",
                f"Failed to save config to {self._path}: {exc}",
                extra={"path": str(self._path)},
            )
            raise CommandError(
                f"Failed to save config to {self._path}: {exc}", http_status=500
            ) from exc

    @staticmethod
    def _validate_config_path(path: Path) -> None:
        """Prevents using device files or other unsafe paths as a config file.

        Args:
            path (Path): The path to validate.

        Raises:
            CommandError: If the path is determined to be unsafe.
        """
        pstr = path.as_posix()
        if (
            pstr.startswith("/dev/")
            or pstr == "/dev/null"
            or pstr.startswith("\\\\.\\")
        ):
            raise CommandError(
                f"Invalid config path: {path} is a device file or not allowed"
            )

    @staticmethod
    def _preflight_write(path: Path) -> None:
        """Performs pre-flight checks before a write operation.

        Fails fast if the path has a symlink loop or the file is locked.

        Args:
            path (Path): The path to check.

        Raises:
            CommandError: If the path is invalid or the file is locked.
        """
        _detect_symlink_loop(path)
        if path.exists():
            try:
                with open(path, "a+") as f:
                    fcntl.flock(f.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    fcntl.flock(f.fileno(), fcntl.LOCK_UN)
            except BlockingIOError as exc:
                raise CommandError(
                    f"Failed to persist config to {path}: File locked", http_status=400
                ) from exc


__all__ = ["Config"]
