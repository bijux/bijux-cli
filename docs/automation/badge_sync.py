"""Synchronize shared badge blocks into repository documentation surfaces."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import re
import sys

REPO_ROOT = Path(__file__).resolve().parents[2]
BADGE_SOURCE_PATH = REPO_ROOT / "docs" / "badges.md"
START_MARKER = "<!-- bijux-core-badges:generated:start -->"
END_MARKER = "<!-- bijux-core-badges:generated:end -->"
PACKAGE_MAP_START_MARKER = "<!-- bijux-core-package-map:generated:start -->"
PACKAGE_MAP_END_MARKER = "<!-- bijux-core-package-map:generated:end -->"
BADGE_BLOCK_RE = re.compile(
    r"<!-- bijux-core-badges:(?P<name>[a-z0-9-]+):start -->\n"
    r"(?P<body>.*?)\n"
    r"<!-- bijux-core-badges:(?P=name):end -->",
    re.DOTALL,
)
TOKEN_RE = re.compile(r"{{\s*(?P<name>[a-z0-9_]+)\s*}}")


@dataclass(frozen=True)
class PackageRecord:
    key: str
    family_key: str
    kind: str
    published: bool
    display_name: str
    badge_label: str
    purpose: str
    docs_url: str
    source_url: str
    crate_name: str | None = None
    crates_url: str | None = None
    docsrs_url: str | None = None
    pypi_name: str | None = None
    pypi_url: str | None = None
    ghcr_url: str | None = None


@dataclass(frozen=True)
class BadgeTarget:
    path: Path
    kind: str
    package_key: str | None = None


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _read_section_value(path: Path, section: str, key: str) -> str:
    text = _read_text(path)
    section_match = re.search(
        rf"(?ms)^\[{re.escape(section)}\]\n(?P<body>.*?)(?=^\[|\Z)",
        text,
    )
    if section_match is None:
        raise ValueError(f"Missing TOML section [{section}] in {path}")
    value_match = re.search(
        rf'(?m)^{re.escape(key)}\s*=\s*"(?P<value>[^"]+)"',
        section_match.group("body"),
    )
    if value_match is None:
        raise ValueError(f"Missing TOML key {key!r} in section [{section}] of {path}")
    return value_match.group("value")


def _shield_text(value: str) -> str:
    return value.replace("-", "--").replace(" ", "%20")


def package_records() -> dict[str, PackageRecord]:
    python_manifest_path = REPO_ROOT / "crates" / "bijux-cli-python" / "pyproject.toml"
    python_package_name = _read_section_value(python_manifest_path, "project", "name")

    def rust_record(
        *,
        key: str,
        family_key: str,
        badge_label: str,
        purpose: str,
        docs_section: str,
        source_dir: str,
        ghcr_url: str | None = None,
    ) -> PackageRecord:
        manifest_path = REPO_ROOT / "crates" / source_dir / "Cargo.toml"
        crate_name = _read_section_value(manifest_path, "package", "name")
        return PackageRecord(
            key=key,
            family_key=family_key,
            kind="rust",
            published=True,
            display_name=crate_name,
            badge_label=badge_label,
            purpose=purpose,
            docs_url=f"https://bijux.io/bijux-core/bijux-dag/packages/{docs_section}/"
            if key.startswith("bijux-dag-")
            else f"https://bijux.io/bijux-core/bijux-cli/packages/{docs_section}/",
            source_url=f"https://github.com/bijux/bijux-core/tree/main/crates/{source_dir}",
            crate_name=crate_name,
            crates_url=f"https://crates.io/crates/{crate_name}",
            docsrs_url=f"https://docs.rs/{crate_name}",
            ghcr_url=ghcr_url,
        )

    return {
        "bijux-cli": rust_record(
            key="bijux-cli",
            family_key="bijux-cli",
            badge_label="bijux-cli",
            purpose="Public Rust runtime for the `bijux` command surface, including routing, runtime behavior, and deterministic output contracts.",
            docs_section="bijux-cli",
            source_dir="bijux-cli",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli",
        ),
        "bijux-dag-artifacts": rust_record(
            key="bijux-dag-artifacts",
            family_key="bijux-dag",
            badge_label="artifacts",
            purpose="Artifact identity, storage layout, retention, integrity, and lineage helpers for retained DAG run evidence.",
            docs_section="bijux-dag-artifacts",
            source_dir="bijux-dag-artifacts",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag-artifacts",
        ),
        "bijux-dag-core": rust_record(
            key="bijux-dag-core",
            family_key="bijux-dag",
            badge_label="core",
            purpose="Deterministic graph kernel for parsing, validation, canonicalization, planning, and semantic identity.",
            docs_section="bijux-dag-core",
            source_dir="bijux-dag-core",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag-core",
        ),
        "bijux-dag-runtime": rust_record(
            key="bijux-dag-runtime",
            family_key="bijux-dag",
            badge_label="runtime",
            purpose="Execution engine and replay policy layer for DAG runs, cache decisions, and retained runtime diagnostics.",
            docs_section="bijux-dag-runtime",
            source_dir="bijux-dag-runtime",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag-runtime",
        ),
        "bijux-dag-app": rust_record(
            key="bijux-dag-app",
            family_key="bijux-dag",
            badge_label="app",
            purpose="Application orchestration and response-shaping layer that turns DAG runtime behavior into user-facing workflows.",
            docs_section="bijux-dag-app",
            source_dir="bijux-dag-app",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag-app",
        ),
        "bijux-dag-cli": rust_record(
            key="bijux-dag-cli",
            family_key="bijux-dag",
            badge_label="bijux-dag",
            purpose="Installable `bijux-dag` command package for validating, running, replaying, and inspecting DAG workflows.",
            docs_section="bijux-dag-cli",
            source_dir="bijux-dag-cli",
            ghcr_url="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag-cli",
        ),
        "bijux-cli-python": PackageRecord(
            key="bijux-cli-python",
            family_key="bijux-cli",
            kind="python",
            published=False,
            display_name="bijux-cli-python",
            badge_label="bijux-cli-python",
            purpose="Python distribution and native bridge for installing and launching `bijux`.",
            docs_url="https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli-python/",
            source_url="https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python",
            pypi_name=python_package_name,
            pypi_url=f"https://pypi.org/project/{python_package_name}/",
        ),
    }


def load_badge_catalog() -> dict[str, str]:
    text = BADGE_SOURCE_PATH.read_text(encoding="utf-8")
    catalog = {
        match.group("name"): match.group("body").strip()
        for match in BADGE_BLOCK_RE.finditer(text)
    }
    if not catalog:
        raise ValueError(f"No badge blocks found in {BADGE_SOURCE_PATH}")
    return catalog


def _render_template(template: str, context: dict[str, str]) -> str:
    def replace(match: re.Match[str]) -> str:
        key = match.group("name")
        if key not in context:
            raise KeyError(f"Missing badge token {key!r}")
        return context[key]

    return TOKEN_RE.sub(replace, template)


def _record_context(record: PackageRecord) -> dict[str, str]:
    badge_title = record.pypi_name if record.kind == "python" else record.display_name
    return {
        "badge_title": badge_title,
        "crate_badge_label": _shield_text(record.badge_label),
        "crate_name": record.crate_name or "",
        "crates_url": record.crates_url or "",
        "docs_badge_alt": f"{record.display_name} docs",
        "docs_badge_label": _shield_text(record.badge_label),
        "docs_url": record.docs_url,
        "docsrs_badge_alt": f"{record.display_name} docs.rs",
        "docsrs_badge_label": _shield_text(f"{record.badge_label} docs.rs"),
        "docsrs_url": record.docsrs_url or "",
        "ghcr_badge_label": _shield_text(record.badge_label),
        "ghcr_url": record.ghcr_url or "",
        "pypi_badge_label": _shield_text(record.pypi_name or record.badge_label),
        "pypi_name": record.pypi_name or "",
        "pypi_url": record.pypi_url or "",
    }


def render_repository_badges(
    catalog: dict[str, str], records: tuple[PackageRecord, ...]
) -> str:
    public_rust_records = tuple(
        record for record in records if record.published and record.kind == "rust"
    )
    pypi_records = tuple(record for record in records if record.pypi_url)
    ghcr_records = tuple(record for record in records if record.ghcr_url)
    docs_records = tuple(record for record in records if record.docs_url)
    summary = _render_template(
        catalog["repository-summary"],
        {
            "ghcr_package_count": str(len(ghcr_records)),
            "public_crate_count": str(len(public_rust_records)),
        },
    )
    registry_badges = [
        _render_template(catalog["family-crates-badge"], _record_context(record))
        for record in public_rust_records
    ]
    registry_badges.extend(
        _render_template(catalog["family-pypi-badge"], _record_context(record))
        for record in pypi_records
    )
    ghcr_badges = [
        _render_template(catalog["family-ghcr-badge"], _record_context(record))
        for record in ghcr_records
    ]
    docs_badges = [catalog["repository-docs-badge"]]
    docs_badges.extend(
        _render_template(catalog["family-docs-badge"], _record_context(record))
        for record in docs_records
    )
    return "\n\n".join(
        (
            summary,
            "\n".join(registry_badges),
            "\n".join(ghcr_badges),
            "\n".join(docs_badges),
        )
    )


def render_package_badges(
    catalog: dict[str, str],
    current: PackageRecord,
    records: tuple[PackageRecord, ...],
) -> str:
    del records
    summary_name = (
        "rust-package-summary" if current.kind == "rust" else "python-package-summary"
    )
    sections = [_render_template(catalog[summary_name], _record_context(current))]
    docs_parts = [
        _render_template(catalog["family-docs-badge"], _record_context(current))
    ]
    repository_docs_badge = catalog.get("repository-docs-badge", "").strip()
    if repository_docs_badge:
        docs_parts.insert(0, repository_docs_badge)
    sections.append(" ".join(docs_parts))
    return "\n\n".join(sections)


def _replace_managed_block(
    text: str, start_marker: str, end_marker: str, body: str, path: Path
) -> str:
    if start_marker not in text or end_marker not in text:
        raise ValueError(f"{path} is missing managed block markers")
    before, remainder = text.split(start_marker, 1)
    _, after = remainder.split(end_marker, 1)
    return f"{before}{start_marker}\n{body}\n{end_marker}{after}"


def _link_badge(href: str, alt: str, src: str) -> str:
    return f'<a href="{href}"><img alt="{alt}" src="{src}" height="18"></a>'


def _package_map_links(
    record: PackageRecord, family_records: tuple[PackageRecord, ...]
) -> str:
    links: list[str] = []
    if record.crates_url and record.crate_name:
        links.append(
            _link_badge(
                record.crates_url,
                "Crates.io",
                f"https://img.shields.io/crates/v/{record.crate_name}?label=crates.io&logo=rust",
            )
        )
    if record.docsrs_url and record.crate_name:
        links.append(
            _link_badge(
                record.docsrs_url,
                "Rust docs",
                f"https://img.shields.io/badge/rust--docs-{_shield_text(record.badge_label)}-DEA584?logo=rust&logoColor=white",
            )
        )
    python_record = next(
        (candidate for candidate in family_records if candidate.pypi_url), None
    )
    if python_record and python_record.pypi_url and python_record.pypi_name:
        links.append(
            _link_badge(
                python_record.pypi_url,
                "PyPI",
                f"https://img.shields.io/pypi/v/{python_record.pypi_name}?label=PyPI&logo=pypi",
            )
        )
    links.append(
        _link_badge(
            record.docs_url,
            "Docs",
            f"https://img.shields.io/badge/docs-{_shield_text(record.badge_label)}-2563EB?logo=materialformkdocs&logoColor=white",
        )
    )
    if record.ghcr_url:
        links.append(
            _link_badge(
                record.ghcr_url,
                "GHCR",
                f"https://img.shields.io/badge/{_shield_text(record.badge_label)}-ghcr-181717?logo=github&logoColor=white",
            )
        )
    links.append(
        _link_badge(
            record.source_url,
            "Source",
            "https://img.shields.io/badge/source-181717?logo=github&logoColor=white",
        )
    )
    return " ".join(links)


def render_package_map(records: dict[str, PackageRecord]) -> str:
    ordered = [record for record in records.values() if record.published]
    lines = [
        "The public package families in this repository are:",
        "",
        "| Package | Purpose | Links |",
        "| --- | --- | --- |",
    ]
    for record in ordered:
        family_records = tuple(
            candidate
            for candidate in records.values()
            if candidate.family_key == record.family_key
        )
        lines.append(
            f"| `{record.display_name}` | {record.purpose} | {_package_map_links(record, family_records)} |"
        )
    return "\n".join(lines)


def iter_targets() -> tuple[BadgeTarget, ...]:
    return (
        BadgeTarget(REPO_ROOT / "README.md", "repository"),
        BadgeTarget(REPO_ROOT / "docs" / "index.md", "repository"),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-cli" / "README.md", "package", "bijux-cli"
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-cli" / "packages" / "bijux-cli.md",
            "package",
            "bijux-cli",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-cli-python" / "README.md",
            "package",
            "bijux-cli-python",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-cli" / "packages" / "bijux-cli-python.md",
            "package",
            "bijux-cli-python",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-dag-artifacts" / "README.md",
            "package",
            "bijux-dag-artifacts",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-dag" / "packages" / "bijux-dag-artifacts.md",
            "package",
            "bijux-dag-artifacts",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-dag-core" / "README.md",
            "package",
            "bijux-dag-core",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-dag" / "packages" / "bijux-dag-core.md",
            "package",
            "bijux-dag-core",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-dag-runtime" / "README.md",
            "package",
            "bijux-dag-runtime",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-dag" / "packages" / "bijux-dag-runtime.md",
            "package",
            "bijux-dag-runtime",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-dag-app" / "README.md",
            "package",
            "bijux-dag-app",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-dag" / "packages" / "bijux-dag-app.md",
            "package",
            "bijux-dag-app",
        ),
        BadgeTarget(
            REPO_ROOT / "crates" / "bijux-dag-cli" / "README.md",
            "package",
            "bijux-dag-cli",
        ),
        BadgeTarget(
            REPO_ROOT / "docs" / "bijux-dag" / "packages" / "bijux-dag-cli.md",
            "package",
            "bijux-dag-cli",
        ),
    )


def render_target(
    target: BadgeTarget,
    catalog: dict[str, str],
    record_map: dict[str, PackageRecord],
) -> str:
    records = tuple(record_map.values())
    text = target.path.read_text(encoding="utf-8")
    if target.kind == "repository":
        body = render_repository_badges(catalog, records)
        updated = _replace_managed_block(
            text, START_MARKER, END_MARKER, body, target.path
        )
        if target.path == REPO_ROOT / "README.md":
            updated = _replace_managed_block(
                updated,
                PACKAGE_MAP_START_MARKER,
                PACKAGE_MAP_END_MARKER,
                render_package_map(record_map),
                target.path,
            )
        return updated

    if target.package_key is None:
        raise ValueError(f"{target.path} is missing package key metadata")
    body = render_package_badges(catalog, record_map[target.package_key], records)
    return _replace_managed_block(text, START_MARKER, END_MARKER, body, target.path)


def sync() -> int:
    catalog = load_badge_catalog()
    record_map = package_records()
    for target in iter_targets():
        rendered = render_target(target, catalog, record_map)
        target.path.write_text(rendered, encoding="utf-8")
    return 0


def check() -> int:
    catalog = load_badge_catalog()
    record_map = package_records()
    failures: list[str] = []
    for target in iter_targets():
        expected = render_target(target, catalog, record_map)
        actual = target.path.read_text(encoding="utf-8")
        if actual != expected:
            failures.append(str(target.path.relative_to(REPO_ROOT)))
    if failures:
        print("Badge blocks are out of sync with docs/badges.md:", file=sys.stderr)
        for path in failures:
            print(f"- {path}", file=sys.stderr)
        print("Run `make sync-badges` to refresh generated sections.", file=sys.stderr)
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("sync", "check"))
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if args.command == "sync":
        return sync()
    return check()


if __name__ == "__main__":
    raise SystemExit(main())
