"""Python facade for Rust-backed bijux-cli runtime."""

from ._exceptions import BijuxPythonError, NativeExtensionUnavailable, PlatformWheelUnavailable
from ._facade import (
    command_tree_introspection,
    config_resolution_helpers,
    ensure_native_extension,
    error_to_exception,
    execution_facade,
    install_path_helpers,
    output_envelope_model,
    plugin_registry_inspection,
    version,
)

__all__ = [
    "BijuxPythonError",
    "NativeExtensionUnavailable",
    "PlatformWheelUnavailable",
    "version",
    "command_tree_introspection",
    "execution_facade",
    "output_envelope_model",
    "error_to_exception",
    "config_resolution_helpers",
    "plugin_registry_inspection",
    "install_path_helpers",
    "ensure_native_extension",
]
