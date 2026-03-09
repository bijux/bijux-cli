"""Python exception hierarchy for bijux-cli runtime facade."""


class BijuxPythonError(RuntimeError):
    """Base error for Python facade failures."""


class NativeExtensionUnavailable(BijuxPythonError):
    """Raised when the Rust extension module cannot be imported."""


class PlatformWheelUnavailable(BijuxPythonError):
    """Raised when no suitable wheel is available for the current platform."""
