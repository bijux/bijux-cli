# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin discovery, metadata validation, and caching."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import importlib.metadata as im
import json
from pathlib import Path
from typing import Any, cast

from packaging.requirements import Requirement
from packaging.specifiers import SpecifierSet
from packaging.utils import canonicalize_name

from bijux_cli.core.errors import BijuxError
from bijux_cli.core.version import __version__ as cli_version
from bijux_cli.plugins import get_plugins_dir
from bijux_cli.plugins.catalog import PLUGIN_NAME_RE


class PluginMetadataError(BijuxError):
    """Raised when plugin metadata is missing or incompatible."""


@dataclass(frozen=True)
class PluginMetadata:
    """Holds metadata for a discovered plugin."""

    name: str
    version: str
    enabled: bool
    source: str
    requires_cli: str
    schema_version: str = "1"
    dist_name: str | None = None
    entrypoint: im.EntryPoint | None = None
    path: Path | None = None


_CACHE: list[PluginMetadata] | None = None


def invalidate_plugin_cache() -> None:
    """Invalidates the discovery cache."""
    global _CACHE
    _CACHE = None


def _require_cli_spec(spec: str, *, name: str) -> None:
    """Validate that the current CLI version satisfies the plugin spec."""
    try:
        SpecifierSet(spec).contains(cli_version, prereleases=True)
    except Exception as exc:
        raise PluginMetadataError(
            f"Plugin {name!r} has invalid version spec {spec!r}: {exc}",
            http_status=400,
        ) from exc

    if not SpecifierSet(spec).contains(cli_version, prereleases=True):
        raise PluginMetadataError(
            f"Plugin {name!r} requires bijux-cli {spec}, host is {cli_version}",
            http_status=400,
        )


def _plugin_meta_from_dist(ep: im.EntryPoint) -> PluginMetadata:
    """Build plugin metadata from an entry point distribution."""
    if not PLUGIN_NAME_RE.fullmatch(ep.name) or not ep.name.isascii():
        raise PluginMetadataError(
            f"Plugin name {ep.name!r} is invalid",
            http_status=400,
        )
    dist = getattr(ep, "dist", None)
    if dist is None:
        try:
            dist = im.distribution(ep.module.split(".")[0])
        except Exception as exc:
            raise PluginMetadataError(
                f"Entry point {ep.name!r} has no distribution metadata: {exc}",
                http_status=400,
            ) from exc

    meta = dist.metadata
    if hasattr(meta, "get"):
        dist_name = cast(Mapping[str, str], meta).get("Name") or dist.name
    else:
        dist_name = dist.name
    requires = dist.metadata.get_all("Requires-Dist") or []
    spec = None
    for req_line in requires:
        req = Requirement(req_line)
        if canonicalize_name(req.name) == canonicalize_name("bijux-cli"):
            spec = str(req.specifier) or None
            break
    if not spec:
        raise PluginMetadataError(
            f"Plugin {ep.name!r} missing bijux-cli requirement",
            http_status=400,
        )

    _require_cli_spec(spec, name=ep.name)

    return PluginMetadata(
        name=ep.name,
        version=dist.version or "unknown",
        enabled=True,
        source="entrypoint",
        requires_cli=spec,
        schema_version="1",
        dist_name=dist_name,
        entrypoint=ep,
    )


def _plugin_meta_from_local(plug_dir: Path) -> PluginMetadata:
    """Build plugin metadata from a local plugin directory."""
    meta_file = plug_dir / "plugin.json"
    if not meta_file.is_file():
        raise PluginMetadataError(
            f"Plugin {plug_dir.name!r} missing plugin.json",
            http_status=400,
        )

    try:
        meta = json.loads(meta_file.read_text("utf-8"))
    except Exception as exc:
        raise PluginMetadataError(
            f"Plugin {plug_dir.name!r} has invalid plugin.json: {exc}",
            http_status=400,
        ) from exc

    name = meta.get("name")
    version = meta.get("version")
    requires = meta.get("bijux_cli_version")
    enabled = bool(meta.get("enabled", True))
    schema_version = meta.get("schema_version")

    if not name or not version or not requires or not schema_version:
        raise PluginMetadataError(
            f"Plugin {plug_dir.name!r} missing required metadata fields",
            http_status=400,
        )

    if str(schema_version) != "1":
        raise PluginMetadataError(
            f"Plugin {plug_dir.name!r} has unsupported schema version {schema_version!r}",
            http_status=400,
        )

    if not PLUGIN_NAME_RE.fullmatch(name) or not name.isascii():
        raise PluginMetadataError(
            f"Plugin name {name!r} is invalid",
            http_status=400,
        )

    if name != plug_dir.name:
        raise PluginMetadataError(
            f"Plugin dir {plug_dir.name!r} does not match metadata name {name!r}",
            http_status=400,
        )

    _require_cli_spec(str(requires), name=name)

    return PluginMetadata(
        name=name,
        version=str(version),
        enabled=enabled,
        source="local",
        requires_cli=str(requires),
        schema_version=str(schema_version),
        path=plug_dir,
    )


def validate_plugin_metadata(meta: PluginMetadata) -> None:
    """Validate core metadata fields for a plugin."""
    if not PLUGIN_NAME_RE.fullmatch(meta.name) or not meta.name.isascii():
        raise PluginMetadataError(
            f"Plugin name {meta.name!r} is invalid",
            http_status=400,
        )
    if not meta.version or str(meta.version).strip() == "":
        raise PluginMetadataError(
            f"Plugin {meta.name!r} missing version",
            http_status=400,
        )
    if not meta.requires_cli:
        raise PluginMetadataError(
            f"Plugin {meta.name!r} missing bijux-cli requirement",
            http_status=400,
        )
    if str(meta.schema_version or "").strip() == "":
        raise PluginMetadataError(
            f"Plugin {meta.name!r} missing schema version",
            http_status=400,
        )
    if str(meta.schema_version) != "1":
        raise PluginMetadataError(
            f"Plugin {meta.name!r} has unsupported schema version {meta.schema_version!r}",
            http_status=400,
        )
    _require_cli_spec(meta.requires_cli, name=meta.name)


def discover_plugins(*, strict: bool = True) -> list[PluginMetadata]:
    """Discover plugins without importing plugin bodies.

    Stages: discover → validate metadata → register → activate (lazy) → unload.
    This function performs discovery + metadata validation only.
    """
    global _CACHE
    if _CACHE is not None:
        return list(_CACHE)

    seen: dict[str, PluginMetadata] = {}

    for ep in im.entry_points().select(group="bijux_cli.plugins"):
        try:
            meta = _plugin_meta_from_dist(ep)
        except PluginMetadataError:
            if strict:
                raise
            continue
        validate_plugin_metadata(meta)
        if meta.name in seen:
            raise PluginMetadataError(
                f"Duplicate plugin name detected: {meta.name!r}", http_status=400
            )
        seen[meta.name] = meta

    plugins_dir = get_plugins_dir()
    if plugins_dir.exists():
        for pdir in plugins_dir.iterdir():
            plug_py = pdir / "plugin.py"
            if not plug_py.is_file():
                continue
            try:
                meta = _plugin_meta_from_local(pdir)
            except PluginMetadataError:
                if strict:
                    raise
                continue
            validate_plugin_metadata(meta)
            if meta.name in seen:
                raise PluginMetadataError(
                    f"Duplicate plugin name detected: {meta.name!r}", http_status=400
                )
            seen[meta.name] = meta

    _CACHE = sorted(seen.values(), key=lambda m: m.name)
    return list(_CACHE)


def get_plugin_metadata(name: str) -> PluginMetadata:
    """Return metadata for a plugin by name."""
    for meta in discover_plugins():
        if meta.name == name:
            return meta
    raise PluginMetadataError(f"Plugin {name!r} not found", http_status=404)


def list_plugins() -> list[dict[str, Any]]:
    """List plugin metadata as dictionaries."""
    return [
        {
            "name": meta.name,
            "version": meta.version,
            "enabled": meta.enabled,
        }
        for meta in discover_plugins(strict=False)
    ]


def plugins_for_package(package: str) -> list[PluginMetadata]:
    """Return plugins belonging to a package name."""
    pkg = canonicalize_name(package)
    return [
        meta
        for meta in discover_plugins()
        if meta.dist_name and canonicalize_name(meta.dist_name) == pkg
    ]
