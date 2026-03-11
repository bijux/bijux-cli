"""Python facade for Rust-backed bijux-cli runtime."""

from ._exceptions import (
    BijuxPythonError,
    InternalError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
    UsageError,
    ValidationError,
)
from ._facade import (
    check_python_runtime_supported,
    command_tree_introspection,
    config_resolution_helpers,
    ensure_native_extension,
    error_to_exception,
    execution_facade_with_status,
    execution_facade,
    install_path_helpers,
    migration_warnings,
    post_install_diagnostics,
    plugin_registry_inspection,
    version,
)
from .compat import get_command_tree, get_version, run_cli

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
]
