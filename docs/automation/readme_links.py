"""Canonicalize and validate public links in repository and crate READMEs."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit

REPO_ROOT = Path(__file__).resolve().parents[2]
PUBLIC_DOCS_ROOT = REPO_ROOT / "docs"
INTERNAL_DOCS_ROOTS = (
    PUBLIC_DOCS_ROOT / "reports",
    PUBLIC_DOCS_ROOT / "spec",
)
GITHUB_ROOT = "https://github.com/bijux/bijux-core"
DOCS_ROOT = "https://bijux.io/bijux-core"

INLINE_LINK_RE = re.compile(
    r"(?P<prefix>!?\[[^\]\n]*\]\()(?P<destination>[^)\s]+)"
)
REFERENCE_LINK_RE = re.compile(
    r"(?m)^(?P<prefix>\[[^\]\n]+\]:\s*)(?P<destination>\S+)"
)
HTML_LINK_RE = re.compile(
    r'(?P<prefix>\b(?:href|src)=")(?P<destination>[^"]+)'
)
LINK_PATTERNS = (INLINE_LINK_RE, REFERENCE_LINK_RE, HTML_LINK_RE)


def readme_paths() -> tuple[Path, ...]:
    return (
        REPO_ROOT / "README.md",
        *sorted((REPO_ROOT / "crates").glob("*/README.md")),
    )


def _public_docs_url(path: Path) -> str:
    relative = path.relative_to(PUBLIC_DOCS_ROOT)
    if relative.suffix == ".md":
        relative = relative.with_suffix("")
        if relative.name == "index":
            relative = relative.parent
        route = relative.as_posix().strip("/")
        return f"{DOCS_ROOT}/{route}/" if route else f"{DOCS_ROOT}/"
    return f"{DOCS_ROOT}/{relative.as_posix()}"


def _repository_url(path: Path) -> str:
    relative = path.relative_to(REPO_ROOT).as_posix()
    object_kind = "tree" if path.is_dir() else "blob"
    return f"{GITHUB_ROOT}/{object_kind}/main/{relative}"


def _is_internal_docs_source(path: Path) -> bool:
    return any(
        path == internal_root or internal_root in path.parents
        for internal_root in INTERNAL_DOCS_ROOTS
    )


def canonical_destination(source: Path, destination: str) -> str:
    wrapped = destination.startswith("<") and destination.endswith(">")
    raw = destination[1:-1] if wrapped else destination
    if raw.startswith(("https://", "mailto:", "#")):
        return destination
    if "://" in raw:
        raise ValueError(f"unsupported URL scheme: {raw}")

    path_text, separator, fragment = raw.partition("#")
    resolved = (source.parent / unquote(path_text)).resolve()
    try:
        resolved.relative_to(REPO_ROOT)
    except ValueError as error:
        raise ValueError(f"destination escapes repository: {raw}") from error
    if not resolved.exists():
        raise ValueError(f"destination does not exist: {raw}")

    is_public_docs_source = (
        resolved == PUBLIC_DOCS_ROOT or PUBLIC_DOCS_ROOT in resolved.parents
    ) and not _is_internal_docs_source(resolved)
    if is_public_docs_source:
        canonical = _public_docs_url(resolved)
    else:
        canonical = _repository_url(resolved)
    if separator:
        canonical = f"{canonical}#{fragment}"
    return f"<{canonical}>" if wrapped else canonical


def _rewrite_text(path: Path, text: str) -> tuple[str, list[str]]:
    failures: list[str] = []
    updated = text
    for pattern in LINK_PATTERNS:
        def replace(match: re.Match[str]) -> str:
            destination = match.group("destination")
            try:
                canonical = canonical_destination(path, destination)
            except ValueError as error:
                line = updated.count("\n", 0, match.start()) + 1
                failures.append(f"{path.relative_to(REPO_ROOT)}:{line}: {error}")
                return match.group(0)
            return f"{match.group('prefix')}{canonical}"

        updated = pattern.sub(replace, updated)
    return updated, failures


def _validate_public_destination(path: Path, destination: str) -> str | None:
    raw = destination[1:-1] if destination.startswith("<") else destination
    if raw.startswith(("mailto:", "#")):
        return None
    parsed = urlsplit(raw)
    if parsed.scheme != "https" or not parsed.netloc:
        return f"link is not an absolute HTTPS destination: {raw}"

    if parsed.netloc == "github.com":
        prefix = "/bijux/bijux-core/"
        if not parsed.path.startswith(prefix):
            return None
        repository_path = parsed.path.removeprefix(prefix)
        match = re.fullmatch(r"(blob|tree)/main/(.+)", repository_path)
        if match is None:
            return None
        expected = (REPO_ROOT / unquote(match.group(2))).resolve()
        if not expected.exists():
            return f"GitHub destination has no repository source: {raw}"
        if match.group(1) == "blob" and not expected.is_file():
            return f"GitHub blob destination is not a file: {raw}"
        if match.group(1) == "tree" and not expected.is_dir():
            return f"GitHub tree destination is not a directory: {raw}"

    if parsed.netloc == "bijux.io":
        prefix = "/bijux-core/"
        if parsed.path == "/bijux-core":
            route = ""
        elif parsed.path.startswith(prefix):
            route = parsed.path.removeprefix(prefix).strip("/")
        else:
            return None
        candidates = (
            PUBLIC_DOCS_ROOT / route / "index.md",
            (PUBLIC_DOCS_ROOT / route).with_suffix(".md"),
        )
        source = next((candidate for candidate in candidates if candidate.is_file()), None)
        if source is None:
            return f"documentation destination has no source page: {raw}"
        if _is_internal_docs_source(source):
            return f"internal documentation must link to its GitHub source: {raw}"

    return None


def sync() -> int:
    failures: list[str] = []
    for path in readme_paths():
        text = path.read_text(encoding="utf-8")
        updated, path_failures = _rewrite_text(path, text)
        failures.extend(path_failures)
        if not path_failures and updated != text:
            path.write_text(updated, encoding="utf-8")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


def check() -> int:
    failures: list[str] = []
    for path in readme_paths():
        text = path.read_text(encoding="utf-8")
        _, canonicalization_failures = _rewrite_text(path, text)
        failures.extend(canonicalization_failures)
        for pattern in LINK_PATTERNS:
            for match in pattern.finditer(text):
                destination = match.group("destination")
                error = _validate_public_destination(path, destination)
                if error is not None:
                    line = text.count("\n", 0, match.start()) + 1
                    failures.append(
                        f"{path.relative_to(REPO_ROOT)}:{line}: {error}"
                    )
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("sync", "check"))
    return parser


def main() -> int:
    args = build_parser().parse_args()
    return sync() if args.command == "sync" else check()


if __name__ == "__main__":
    raise SystemExit(main())
