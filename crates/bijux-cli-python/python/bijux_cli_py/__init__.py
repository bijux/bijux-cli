"""Python facade and app-author helpers for bijux-cli."""

from __future__ import annotations

from ._exceptions import (
    BijuxPythonError,
    InternalError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
    UsageError,
    ValidationError,
)
from .app_sdk import (
    CommandResult,
    CompatibilityWindow,
    build_python_mount_manifest,
    compatibility_report,
    failure,
    run_json_app,
    success,
)
from .dag_sdk import dag_post_install_diagnostics, dag_command_json, load_dag_graph

_FACADE_EXPORTS = {
    "check_python_runtime_supported",
    "command_tree_introspection",
    "config_resolution_helpers",
    "ensure_native_extension",
    "error_to_exception",
    "execution_facade",
    "execution_facade_with_status",
    "install_path_helpers",
    "migration_warnings",
    "plugin_registry_inspection",
    "post_install_diagnostics",
    "version",
}
_COMPAT_EXPORTS = {"get_command_tree", "get_version", "run_cli"}

__all__ = [
    "BijuxPythonError",
    "UsageError",
    "ValidationError",
    "InternalError",
    "NativeExtensionUnavailable",
    "PlatformWheelUnavailable",
    "version",
    "check_python_runtime_supported",
    "command_tree_introspection",
    "execution_facade",
    "execution_facade_with_status",
    "error_to_exception",
    "config_resolution_helpers",
    "plugin_registry_inspection",
    "install_path_helpers",
    "migration_warnings",
    "post_install_diagnostics",
    "get_version",
    "get_command_tree",
    "run_cli",
    "ensure_native_extension",
    "CommandResult",
    "CompatibilityWindow",
    "success",
    "failure",
    "run_json_app",
    "build_python_mount_manifest",
    "compatibility_report",
    "dag_command_json",
    "dag_post_install_diagnostics",
    "load_dag_graph",
]


def __getattr__(name: str):
    if name in _FACADE_EXPORTS:
        from . import _facade

        return getattr(_facade, name)
    if name in _COMPAT_EXPORTS:
        from . import compat

        return getattr(compat, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
