# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Plugin discovery, metadata validation, and caching."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any
import importlib.metadata as im
import json

from packaging.requirements import Requirement
from packaging.specifiers import SpecifierSet
from packaging.utils import canonicalize_name

from bijux_cli.__version__ import __version__ as cli_version
from bijux_cli.commands.plugins.utils import PLUGIN_NAME_RE
from bijux_cli.core.exceptions import BijuxError
from bijux_cli.services.plugins import get_plugins_dir


class PluginMetadataError(BijuxError):
    """Raised when plugin metadata is missing or incompatible."""


@dataclass(frozen=True)
class PluginMetadata:
    name: str
    version: str
    enabled: bool
    source: str
    requires_cli: str
    dist_name: str | None = None
    entrypoint: im.EntryPoint | None = None
    path: Path | None = None


_CACHE: list[PluginMetadata] | None = None


def invalidate_plugin_cache() -> None:
    """Invalidates the discovery cache."""
    global _CACHE
    _CACHE = None


def _require_cli_spec(spec: str, *, name: str) -> None:
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

    dist_name = dist.metadata.get("Name") or dist.name
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
        dist_name=dist_name,
        entrypoint=ep,
    )


def _plugin_meta_from_local(plug_dir: Path) -> PluginMetadata:
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

    if not name or not version or not requires:
        raise PluginMetadataError(
            f"Plugin {plug_dir.name!r} missing required metadata fields",
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
        path=plug_dir,
    )


def discover_plugins(*, strict: bool = True) -> list[PluginMetadata]:
    """Discover plugins without importing plugin bodies."""
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
            if meta.name in seen:
                raise PluginMetadataError(
                    f"Duplicate plugin name detected: {meta.name!r}", http_status=400
                )
            seen[meta.name] = meta

    _CACHE = sorted(seen.values(), key=lambda m: m.name)
    return list(_CACHE)


def get_plugin_metadata(name: str) -> PluginMetadata:
    for meta in discover_plugins():
        if meta.name == name:
            return meta
    raise PluginMetadataError(f"Plugin {name!r} not found", http_status=404)


def list_plugins() -> list[dict[str, Any]]:
    return [
        {
            "name": meta.name,
            "version": meta.version,
            "enabled": meta.enabled,
        }
        for meta in discover_plugins()
    ]


def plugins_for_package(package: str) -> list[PluginMetadata]:
    pkg = canonicalize_name(package)
    return [
        meta
        for meta in discover_plugins()
        if meta.dist_name and canonicalize_name(meta.dist_name) == pkg
    ]
