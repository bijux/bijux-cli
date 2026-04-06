"""Python exception hierarchy for bijux-cli runtime facade."""

from __future__ import annotations


class BijuxPythonError(RuntimeError):
    """Base error for Python facade failures."""


class UsageError(BijuxPythonError):
    """Raised for command usage failures."""


class ValidationError(BijuxPythonError):
    """Raised for validation failures."""


class InternalError(BijuxPythonError):
    """Raised for runtime/internal failures."""


class NativeExtensionUnavailable(BijuxPythonError):
    """Raised when the Rust extension module cannot be imported."""


class PlatformWheelUnavailable(BijuxPythonError):
    """Raised when no suitable wheel is available for the current platform."""
