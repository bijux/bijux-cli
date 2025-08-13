# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services docs module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import os
from pathlib import Path
from typing import Any, cast
from unittest.mock import ANY, MagicMock, Mock, patch

import pytest

from bijux_cli.contracts import DocsProtocol, ObservabilityProtocol, TelemetryProtocol
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import ServiceError
from bijux_cli.services.docs import Docs


@pytest.fixture
def mock_observability() -> ObservabilityProtocol:
    """Provide a mock ObservabilityProtocol instance."""
    mock_instance = Mock(spec=ObservabilityProtocol)
    return cast(ObservabilityProtocol, mock_instance)


@pytest.fixture
def mock_telemetry() -> TelemetryProtocol:
    """Provide a mock TelemetryProtocol instance."""
    mock_instance = Mock(spec=TelemetryProtocol)
    return cast(TelemetryProtocol, mock_instance)


@pytest.fixture
def temp_root(tmp_path: Path) -> Path:
    """Provide a temporary root directory for docs."""
    return tmp_path / "docs"


@pytest.fixture
def docs(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    temp_root: Path,
) -> Docs:
    """Provide a Docs instance with mocked dependencies."""
    return Docs(
        observability=mock_observability, telemetry=mock_telemetry, root=temp_root
    )


def test_docs_implements_protocol(docs: Docs) -> None:
    """Test that the Docs class implements the DocsProtocol."""
    assert isinstance(docs, DocsProtocol)


def test_init_creates_root(docs: Docs, temp_root: Path) -> None:
    """Test that the Docs constructor creates the root directory."""
    assert temp_root.exists()


def test_init_env_root(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    tmp_path: Path,
) -> None:
    """Test that the Docs constructor uses the root directory from the environment."""
    env_root = tmp_path / "env_docs"
    with patch.dict(os.environ, {"BIJUXCLI_DOCS_DIR": str(env_root)}):
        d = Docs(mock_observability, mock_telemetry)
    assert d._root == env_root


def test_init_default_root(
    mock_observability: ObservabilityProtocol, mock_telemetry: TelemetryProtocol
) -> None:
    """Test that the Docs constructor uses a default root directory."""
    with patch.object(Path, "mkdir"):
        d = Docs(mock_observability, mock_telemetry)
        assert d._root.name == "docs"


def test_init_none_root(
    mock_observability: ObservabilityProtocol, mock_telemetry: TelemetryProtocol
) -> None:
    """Test that providing a None root to the constructor uses the default."""
    with patch.object(Path, "mkdir"):
        d = Docs(mock_observability, mock_telemetry, root=None)
        assert d._root.name == "docs"


def test_serializer_cache(docs: Docs, mock_telemetry: TelemetryProtocol) -> None:
    """Test that the serializer cache is correctly initialized."""
    assert mock_telemetry in Docs._serializers
    assert isinstance(Docs._serializers[mock_telemetry], dict)


@patch("bijux_cli.services.docs.serializer_for")
def test_render_caches_serializer(mock_serializer_for: MagicMock, docs: Docs) -> None:
    """Test that the render method caches the serializer instance."""
    mock_serializer = Mock()
    mock_serializer.dumps.return_value = "serialized"
    mock_serializer_for.return_value = mock_serializer
    spec = {"key": "value"}
    result = docs.render(spec, fmt=OutputFormat.JSON)
    assert result == "serialized"
    mock_serializer_for.assert_called_once_with(OutputFormat.JSON, docs._telemetry)
    mock_serializer.dumps.assert_called_once_with(
        spec, fmt=OutputFormat.JSON, pretty=False
    )


def test_render_non_string(docs: Docs) -> None:
    """Test that the render method raises a TypeError if the serializer returns bytes."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_serializer_for:
        mock_serializer = Mock()
        mock_serializer.dumps.return_value = b"bytes"
        mock_serializer_for.return_value = mock_serializer
        with pytest.raises(TypeError):
            docs.render({}, fmt=OutputFormat.JSON)


def test_write_returns_str(docs: Docs, tmp_path: Path) -> None:
    """Test that the write method returns the path as a string."""
    spec = {"key": "value"}
    file_path = tmp_path / "test.json"
    path_str = docs.write(
        spec,
        name=file_path,  # type: ignore[arg-type]
    )
    assert isinstance(path_str, str)
    assert Path(path_str).exists()


def test_write_sync_returns_path(docs: Docs, tmp_path: Path) -> None:
    """Test that the write_sync method returns a Path object."""
    spec = {"key": "value"}
    file_path = tmp_path / "test.json"
    path = docs.write_sync(spec, OutputFormat.JSON, file_path)
    assert isinstance(path, Path)
    assert path.exists()


def test_write_sync_dir_name(docs: Docs, temp_root: Path) -> None:
    """Test that write_sync creates a default filename when a directory is provided."""
    spec: dict[str, Any] = {}
    path = docs.write_sync(spec, OutputFormat.JSON, temp_root)
    assert path == temp_root / "spec.json"
    assert path.exists()


def test_write_sync_expanduser(docs: Docs, tmp_path: Path) -> None:
    """Test that write_sync correctly expands the user home directory symbol."""
    expand_path = tmp_path / "expanded" / "test.json"
    with (
        patch.object(Path, "expanduser", return_value=expand_path),
        patch.object(Path, "write_text"),
        patch.object(Path, "mkdir"),
    ):
        docs.write_sync({}, OutputFormat.JSON, "~/test.json")
        Path.expanduser.assert_called_once()  # type: ignore[attr-defined]


def test_write_sync_mkdir(docs: Docs, tmp_path: Path) -> None:
    """Test that write_sync creates parent directories as needed."""
    subdir = tmp_path / "sub"
    file_path = subdir / "spec.json"
    docs.write_sync({}, OutputFormat.JSON, file_path)
    assert file_path.exists()


def test_write_sync_logs(
    docs: Docs,
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    tmp_path: Path,
) -> None:
    """Test that write_sync logs and emits telemetry on success."""
    file_path = tmp_path / "test.json"
    path = docs.write_sync({}, OutputFormat.JSON, file_path)
    mock_observability.log.assert_called_with(  # type: ignore[attr-defined]
        "info", f"Wrote docs to {path}"
    )
    mock_telemetry.event.assert_called_with(  # type:ignore[attr-defined]
        "docs_written", {"path": str(path), "format": "json"}
    )


def test_write_sync_oserror(
    docs: Docs, mock_telemetry: TelemetryProtocol, tmp_path: Path
) -> None:
    """Test that write_sync raises a ServiceError on OSError."""
    file_path = tmp_path / "test"
    with (
        patch.object(Path, "write_text", side_effect=OSError("io")),
        pytest.raises(ServiceError) as exc,
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)
    assert exc.value.http_status == 403
    mock_telemetry.event.assert_called_with(  # type:ignore[attr-defined]
        "docs_write_failed", {"path": ANY, "error": "io"}
    )


def test_write_sync_permission(docs: Docs, tmp_path: Path) -> None:
    """Test that a PermissionError during directory creation is handled."""
    file_path = tmp_path / "no_perm" / "test.json"
    with (
        patch.object(Path, "mkdir", side_effect=PermissionError("perm")),
        pytest.raises(ServiceError),
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)


def test_close_noop(docs: Docs) -> None:
    """Test that the close method is a no-op."""
    docs.close()


@pytest.mark.parametrize("fmt", [OutputFormat.JSON, OutputFormat.YAML])
def test_multiple_formats(docs: Docs, fmt: OutputFormat, tmp_path: Path) -> None:
    """Test writing docs in multiple different formats."""
    spec = {"format": fmt.value}
    file_path = tmp_path / f"test.{fmt.value}"
    path = docs.write_sync(spec, fmt, file_path)
    assert path.exists()


def test_serializer_cache_multiple_telemetry(
    mock_observability: ObservabilityProtocol,
) -> None:
    """Test that the serializer cache is unique per telemetry instance."""
    tel1 = cast(TelemetryProtocol, Mock())
    tel2 = cast(TelemetryProtocol, Mock())
    Docs(mock_observability, tel1)
    Docs(mock_observability, tel2)
    assert tel1 in Docs._serializers
    assert tel2 in Docs._serializers
    assert Docs._serializers[tel1] is not Docs._serializers[tel2]


def test_render_pretty_false(docs: Docs) -> None:
    """Test that the render method calls the serializer with pretty=False."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:
        mock_serializer = Mock()
        mock_ser.return_value = mock_serializer
        mock_serializer.dumps.return_value = "{}"
        docs.render({}, fmt=OutputFormat.JSON)
        mock_serializer.dumps.assert_called_with(ANY, fmt=ANY, pretty=False)


def test_write_name_path(docs: Docs, tmp_path: Path) -> None:
    """Test that the written path object has the correct name."""
    file_path = tmp_path / "test.json"
    path = docs.write_sync({}, OutputFormat.JSON, file_path)
    assert path.name == "test.json"


def test_write_sync_unresolved_path(docs: Docs, tmp_path: Path) -> None:
    """Test that an error is raised for an unresolved path."""
    file_path = tmp_path / "test"
    with (
        patch.object(Path, "resolve", side_effect=OSError("resolve fail")),
        pytest.raises(ServiceError),
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)


def test_docs_in_all() -> None:
    """Test that 'Docs' is in the module's __all__ export list."""
    from bijux_cli.services.docs import __all__

    assert "Docs" in __all__


def test_docs_docstring() -> None:
    """Test that the Docs class has a docstring."""
    assert Docs.__doc__ is not None


def test_render_docstring() -> None:
    """Test that the render method has a docstring."""
    assert Docs.render.__doc__ is not None


def test_write_docstring() -> None:
    """Test that the write method has a docstring."""
    assert Docs.write.__doc__ is not None


def test_write_sync_docstring() -> None:
    """Test that the write_sync method has a docstring."""
    assert Docs.write_sync.__doc__ is not None


def test_close_docstring() -> None:
    """Test that the close method has a docstring."""
    assert Docs.close.__doc__ is not None


@pytest.mark.parametrize(
    "root",
    [str(Path("/tmp/root1")), Path("/tmp/root2")],  # noqa: S108
)
def test_init_root_type(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    root: str | Path,
    tmp_path: Path,
) -> None:
    """Test that the root can be initialized with a str or Path."""
    actual_root = tmp_path / "root"
    with patch("bijux_cli.services.docs.Path", return_value=actual_root):
        d = Docs(mock_observability, mock_telemetry, root=root)
    assert isinstance(d._root, Path)


def test_init_mkdir_exists(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    temp_root: Path,
) -> None:
    """Test that initialization does not fail if the root directory already exists."""
    temp_root.mkdir(exist_ok=True)
    Docs(mock_observability, mock_telemetry, root=temp_root)


def test_render_empty_spec(docs: Docs) -> None:
    """Test that rendering an empty spec produces an empty JSON object."""
    result = docs.render({}, fmt=OutputFormat.JSON)
    assert result == "{}"


def test_write_empty_spec(docs: Docs, temp_root: Path) -> None:
    """Test that writing an empty spec creates a file with an empty JSON object."""
    path = docs.write({}, name=temp_root / "empty")  # type:ignore[arg-type]
    assert Path(path).read_text() == "{}"


def test_service_error_message(docs: Docs, tmp_path: Path) -> None:
    """Test the error message content of a ServiceError raised during write."""
    file_path = tmp_path / "test"
    with (
        patch.object(Path, "write_text", side_effect=OSError("fail")),
        pytest.raises(ServiceError) as exc,
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)
    assert "Unable to write spec" in str(exc.value)


def test_write_name_dir_append_fmt(docs: Docs, temp_root: Path) -> None:
    """Test that the correct file extension is appended when writing to a directory."""
    path = docs.write_sync({}, OutputFormat.YAML, temp_root)
    assert path.suffix == ".yaml"


def test_export_large_spec(docs: Docs, temp_root: Path) -> None:
    """Test writing a large specification to a file."""
    large = {str(i): i for i in range(1000)}
    path = docs.write_sync(large, OutputFormat.JSON, temp_root / "large")
    assert len(path.read_text()) > 1000


def test_close_called(docs: Docs) -> None:
    """Test that the close method can be called multiple times without error."""
    docs.close()
    docs.close()


@pytest.mark.parametrize("name", ["spec", Path("spec"), "dir/spec"])
def test_various_names(docs: Docs, name: str | Path, temp_root: Path) -> None:
    """Test writing docs with various types of names (str, Path)."""
    path = docs.write_sync({}, OutputFormat.JSON, temp_root / name)
    assert path.exists()


def test_render_yaml(docs: Docs) -> None:
    """Test rendering a spec to YAML format."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:
        mock_serializer = Mock()
        mock_ser.return_value = mock_serializer
        mock_serializer.dumps.return_value = "---\nkey: value\n"
        result = docs.render({"key": "value"}, fmt=OutputFormat.YAML)
        assert "key: value" in result


def test_write_yaml(docs: Docs, temp_root: Path) -> None:
    """Test writing a spec to a YAML file."""
    path_str = docs.write(
        {"key": "value"},
        fmt=OutputFormat.YAML,
        name=temp_root / "spec.yaml",  # type:ignore[arg-type]
    )
    text = Path(path_str).read_text()
    assert "key: value" in text


def test_weakdict_multiple(mock_observability: ObservabilityProtocol) -> None:
    """Test that the serializer cache handles multiple telemetry instances."""
    tel1 = cast(TelemetryProtocol, Mock())
    tel2 = cast(TelemetryProtocol, Mock())
    Docs(mock_observability, tel1)
    Docs(mock_observability, tel2)
    assert len(Docs._serializers) >= 2


def test_telemetry_failed_path_unresolved(docs: Docs, tmp_path: Path) -> None:
    """Test that a failure telemetry event is sent for an unresolved path."""
    file_path = tmp_path / "test"
    with (
        patch.object(Path, "write_text", side_effect=OSError("io")),
        pytest.raises(ServiceError),
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)
    docs._telemetry.event.assert_called_with(  # type:ignore[attr-defined]
        "docs_write_failed", {"path": ANY, "error": "io"}
    )


def test_no_telemetry_on_success(
    docs: Docs, mock_telemetry: TelemetryProtocol, tmp_path: Path
) -> None:
    """Test that a 'docs_written' event is sent on a successful write."""
    file_path = tmp_path / "test"
    with patch("bijux_cli.services.docs.serializer_for") as mock_serializer_factory:
        mock_serializer_instance = MagicMock()
        mock_serializer_instance.dumps.return_value = "{}"
        mock_serializer_factory.return_value = mock_serializer_instance

        docs.write_sync({}, OutputFormat.JSON, file_path)

    mock_telemetry.event.assert_called_with(  # type:ignore[attr-defined]
        "docs_written", ANY
    )


def test_init_mkdir_fail(
    mock_observability: ObservabilityProtocol, mock_telemetry: TelemetryProtocol
) -> None:
    """Propagate OSError if directory creation fails during init."""
    with (
        patch.object(Path, "mkdir", side_effect=OSError("mkdir fail")),
        pytest.raises(OSError, match="mkdir fail"),
    ):
        Docs(mock_observability, mock_telemetry, root="/fail")


def test_write_sync_is_dir_true(docs: Docs, temp_root: Path) -> None:
    """Test that write_sync correctly handles being given a directory path."""
    temp_root.mkdir(exist_ok=True)
    path = docs.write_sync({}, OutputFormat.JSON, temp_root)
    assert path == temp_root / "spec.json"


def test_write_sync_mkdir_fail(docs: Docs, tmp_path: Path) -> None:
    """Test that a failure to create a directory during write_sync is handled."""
    invalid_path = tmp_path / "no" / "parent" / "test.json"
    with (
        patch.object(Path, "mkdir", side_effect=OSError("mkdir fail")),
        pytest.raises(ServiceError),
    ):
        docs.write_sync({}, OutputFormat.JSON, invalid_path)


def test_render_pretty_param_not_used(docs: Docs) -> None:
    """Test that render's pretty parameter defaults to False."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:
        mock_serializer = Mock()
        mock_ser.return_value = mock_serializer
        mock_serializer.dumps.return_value = "{}"
        docs.render({}, fmt=OutputFormat.JSON)
        mock_serializer.dumps.assert_called_with(ANY, fmt=ANY, pretty=False)


def test_service_error_from_oserror(docs: Docs, tmp_path: Path) -> None:
    """Test that an OSError during write is wrapped in a ServiceError."""
    file_path = tmp_path / "test"
    with (
        patch.object(Path, "write_text", side_effect=OSError("fail")),
        pytest.raises(ServiceError) as exc,
    ):
        docs.write_sync({}, OutputFormat.JSON, file_path)
    assert "fail" in str(exc.value)


def test_init_no_root_env(
    mock_observability: ObservabilityProtocol, mock_telemetry: TelemetryProtocol
) -> None:
    """Test that the default root is used if the environment variable is empty."""
    with patch.dict(os.environ, {"BIJUXCLI_DOCS_DIR": ""}), patch.object(Path, "mkdir"):
        d = Docs(mock_observability, mock_telemetry)
        assert d._root.name == "docs"


def test_serializer_cache_hit(docs: Docs) -> None:
    """Test that the serializer is cached and not created on every render call."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = "data"
        mock_ser.return_value = mock_serializer
        docs.render({}, fmt=OutputFormat.JSON)
        docs.render({}, fmt=OutputFormat.JSON)
    mock_ser.assert_called_once()


def test_different_fmt_new_serializer(docs: Docs) -> None:
    """Test that different formats use different serializer instances."""
    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:
        mock_json = MagicMock()
        mock_json.dumps.return_value = "json"
        mock_yaml = MagicMock()
        mock_yaml.dumps.return_value = "yaml"
        mock_ser.side_effect = [mock_json, mock_yaml]
        docs.render({}, fmt=OutputFormat.JSON)
        docs.render({}, fmt=OutputFormat.YAML)
    assert mock_ser.call_count == 2


def test_docs_serializer_cache_miss(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    tmp_path: Path,
) -> None:
    """Test the render method when the telemetry instance is not in the cache."""
    Docs._serializers.clear()
    assert not Docs._serializers

    d = Docs(mock_observability, mock_telemetry, root=tmp_path)
    assert mock_telemetry in Docs._serializers
    Docs._serializers.pop(mock_telemetry)

    with patch("bijux_cli.services.docs.serializer_for") as mock_ser:

        class DummySerializer:
            """A mock serializer for tests that always returns a fixed JSON string."""

            @staticmethod
            def dumps(*a: Any, **k: Any) -> str:
                """Ignores all input and returns a hardcoded success JSON string."""
                return '{"ok":true}'

        mock_ser.return_value = DummySerializer()
        result = d.render({}, fmt=OutputFormat.JSON)
        assert result == '{"ok":true}'
        assert mock_telemetry in Docs._serializers


def test_docs_init_telemetry_already_in_serializers(
    mock_observability: ObservabilityProtocol,
    mock_telemetry: TelemetryProtocol,
    tmp_path: Path,
) -> None:
    """Test the Docs constructor when the telemetry instance is already cached."""
    Docs._serializers[mock_telemetry] = {}
    Docs(mock_observability, mock_telemetry, root=tmp_path)
    assert Docs._serializers[mock_telemetry] == {}
