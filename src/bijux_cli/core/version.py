# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides version metadata for the Bijux CLI package.

This module dynamically retrieves version information from the installed
package's metadata and the project's pyproject.toml file. This ensures that
versions are managed from a single source of truth.

It exposes two primary versions:
    * `version`: The main application version (`packaging.version.Version`).
    * `api_version`: A separate version for the plugin API, used for
        compatibility checks.
"""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _get_version
from pathlib import Path
import tomllib

from packaging.version import Version

try:
    __version__: str = _get_version("bijux-cli")
except PackageNotFoundError:
    __version__ = "0.1.0"


try:
    pyproject_path = Path(__file__).parent.parent.parent / "pyproject.toml"
    with pyproject_path.open("rb") as f:
        pyproject_data = tomllib.load(f)
    __api_version__: str = pyproject_data["tool"]["bijux"]["api_version"]
except (FileNotFoundError, KeyError):
    __api_version__ = "0.1.0"

version = Version(__version__)
api_version = Version(__api_version__)

__all__ = ["version", "api_version", "__version__", "__api_version__"]
