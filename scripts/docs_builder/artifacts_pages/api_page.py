# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Builds the documentation page for API toolchain artifacts.

This module defines `APIArtifactPage`, which gathers and presents logs and
files related to the sandboxed Node.js toolchain, OpenAPI schema linting,
API testing with Schemathesis, and the local development server.

Module Constants:
    API_DIR: The directory where API artifacts are stored.
    CORE: A list of key artifacts to display first in a stable order.
    BLURBS: A dictionary mapping artifact filenames to their descriptions
        and a summary of a "good" result.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.artifacts_pages.base import StandardArtifactPage
from scripts.docs_builder.artifacts_pages.base import Bullet
from scripts.docs_builder.helpers import anchor_for
from scripts.docs_builder.helpers import indent

API_DIR = Path("artifacts/api")

CORE = [
    "server.log",
    "test/schemathesis.log",
    "test/schemathesis.xml",
    "node/tool-versions.txt",
    "node/npm-install.log",
    "node/package.json",
    "node/package-lock.json",
]

BLURBS = {
    "server.log": ("API mock/dev server log.", "Starts cleanly; no stack traces or repeated errors."),
    "test/schemathesis.log": ("Schemathesis run output (property/fuzz tests).", "Run completes; no falsifying examples."),
    "test/schemathesis.xml": ("Schemathesis JUnit report (if supported).", "0 failures / 0 errors."),
    "node/tool-versions.txt": ("Pinned CLI versions in the sandbox.", "Exact versions recorded for reproducibility."),
    "node/npm-install.log": ("Log of sandboxed `npm install`.", "Install succeeds; no hard errors."),
    "node/package.json": ("Node workspace manifest (API tooling).", "Scripts usable; deps consistent."),
    "node/package-lock.json": ("NPM lockfile for the sandbox.", "Deterministic installs."),
}


def _parse_junit(fp: Path) -> dict:
    """Performs a minimal, safe parse of a JUnit XML file.

    This function is designed to be self-contained and fault-tolerant,
    extracting basic test suite statistics without external dependencies. It
    reads the file from the provided path.

    Args:
        fp: The path to the JUnit XML file.

    Returns:
        A dictionary containing test statistics: 'tests', 'failures', 'errors',
        'skipped', 'time', and a boolean 'ok' status. Returns a dictionary
        with zero values if parsing fails.
    """
    out = {"tests": 0, "failures": 0, "errors": 0, "skipped": 0, "time": 0.0, "ok": False}
    try:
        import xml.etree.ElementTree as ET

        root = ET.parse(fp).getroot()
        suites = root.findall(".//testsuite") if root.tag.endswith("testsuites") else [root]
        for s in suites:
            out["tests"] += int(s.attrib.get("tests", "0"))
            out["failures"] += int(s.attrib.get("failures", "0"))
            out["errors"] += int(s.attrib.get("errors", "0"))
            out["skipped"] += int(s.attrib.get("skipped", "0"))
            try:
                out["time"] += float(s.attrib.get("time", "0"))
            except ValueError:
                pass
        out["ok"] = out["failures"] == 0 and out["errors"] == 0 and out["tests"] > 0
    except Exception:
        pass
    return out


def _pkg_json_stats(fp: Path) -> tuple[int, int, int]:
    """Parses a package.json file to count dependencies and scripts.

    Args:
        fp: The path to the package.json file.

    Returns:
        A tuple containing counts of (dependencies, devDependencies, scripts).
        Returns (0, 0, 0) if the file cannot be read or parsed.
    """
    try:
        d = json.loads(fp.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return 0, 0, 0
    deps = len(d.get("dependencies") or {})
    dev = len(d.get("devDependencies") or {})
    scripts = len(d.get("scripts") or {})
    return deps, dev, scripts


def _tool_versions(fp: Path) -> dict[str, str]:
    """Parses a tool-versions.txt file into a dictionary.

    Args:
        fp: The path to the tool-versions.txt file.

    Returns:
        A dictionary mapping tool names to their pinned versions. Returns
        an empty dictionary if the file cannot be read.
    """
    out: dict[str, str] = {}
    try:
        for ln in fp.read_text(encoding="utf-8").splitlines():
            m = re.match(r"\s*([^=\s]+)\s*=\s*(\S+)\s*$", ln)
            if m:
                out[m.group(1)] = m.group(2)
    except OSError:
        pass
    return out


class APIArtifactPage(StandardArtifactPage):
    """Generates the documentation page for API toolchain artifacts."""

    out_md = Path("artifacts/api.md")

    def title(self) -> str:
        """Returns the main title for the API artifacts page."""
        return "API Artifacts"

    def intro(self) -> str:
        """Returns the introductory text for the API artifacts page."""
        return (
            "Artifacts produced by the **API toolchain** (see `make api`, `api-install`, `api-lint`, `api-test`).\n\n"
            "- **lint/** — OpenAPI schema validation via: `prance`, `openapi-spec-validator`, `redocly lint`,"
            " and `openapi-generator-cli validate`\n"
            "- **test/** — Schemathesis property tests (optional JUnit)\n"
            "- **node/** — sandboxed Node workspace for CLI tooling (no root pollution)\n"
            "- **server.log** — dev server telemetry\n"
        )

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields API artifacts to be documented in a structured order.

        The ordering is determined in two stages to ensure a predictable layout:
        1.  Core, stable items defined in the `CORE` constant.
        2.  All schema lint logs from the `lint/` subdirectory, sorted by name.

        Yields:
            An iterable of (label, path) tuples for each artifact, where the label
            is the relative path from the API artifact root.
        """
        items: list[tuple[str, Path]] = []
        for rel in CORE:
            p = API_DIR / rel
            if p.exists():
                items.append((rel, p))
        lint_dir = API_DIR / "lint"
        if lint_dir.is_dir():
            for p in sorted(lint_dir.glob("*.log")):
                items.append((f"lint/{p.name}", p))
        return items

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Builds a detailed summary bullet for a specific API artifact.

        This method has custom logic to dynamically generate descriptions for
        lint logs and to add parsed statistics to the summaries of other files.

        Args:
            label: The relative path label of the artifact.
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A `Bullet` object populated with a title, summary, and usage guidance.
        """
        if label.startswith("lint/") and label.endswith(".log"):
            schema_name = label.split("/", 1)[1].removesuffix(".log")
            desc = f"OpenAPI schema lint — results for `{schema_name}`."
            good = "All validators pass (no errors)."
            usage = (
                "Run `make api-lint`; fix $ref/type/path issues flagged by prance, "
                "openapi-spec-validator, redocly, and openapi-generator."
            )
            title = f"[{label}](#{anchor_for(label)}) — {desc}"
            return Bullet(title=title, good=good, usage=usage)

        desc, good = BLURBS.get(label, ("API artifact.", "Present and well-formed."))
        extra = ""
        if label == "node/package.json":
            deps, dev, scripts = _pkg_json_stats(path)
            extra = f" — deps: {deps} • devDeps: {dev} • scripts: {scripts}"
        elif label == "node/tool-versions.txt":
            tv = _tool_versions(path)
            if tv:
                picks = [k for k in ("openapi-generator-cli", "redocly-cli") if k in tv]
                rest = [k for k in tv if k not in picks]
                ordered = picks + rest
                shown = ", ".join(f"{k}={tv[k]}" for k in ordered[:3])
                extra = f" — {shown}"
        elif label == "test/schemathesis.xml":
            s = _parse_junit(path)
            extra = f" — {s['tests']} tests; {s['failures']} failures; {s['errors']} errors; {s['skipped']} skipped"

        title = f"[{label}](#{anchor_for(label)}) — {desc}"
        good_line = good + extra
        usage = {
            "server.log": "Start with `make api-test` or `make api-serve[-bg]`; check for stack traces/timeouts.",
            "test/schemathesis.log": "Investigate falsifying examples; tighten schema or handlers; re-run `make api-test`.",
            "test/schemathesis.xml": "Publish as CI artifact; wire up test summaries from JUnit.",
            "node/tool-versions.txt": "Commit this file to pin CLI versions; regenerate via `make node_deps`.",
            "node/npm-install.log": "If install fails, open this log; re-run `make node_deps` after fixes.",
            "node/package.json": "Edit/inspect scripts; run inside sandbox dir (`artifacts/api/node`).",
            "node/package-lock.json": "Commit to keep installs deterministic in CI.",
        }.get(label, "Kept for traceability; usually produced by `make api`.")
        return Bullet(title=title, good=good_line, usage=usage)

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates a detailed, context-aware description for an API artifact.

        This method generates a rich 'info' block with custom logic for
        different artifact types, such as summarizing validation tools for lint
        logs or showing dependency counts for Node.js files.

        Args:
            label: The relative path label of the artifact.
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A formatted Markdown string for the artifact's detail section.
        """
        if label.startswith("lint/") and label.endswith(".log"):
            schema = label.split("/", 1)[1].removesuffix(".log")
            lines = [
                f"**What it is:** Lint/validate results for `{schema}`.",
                "**Validators:** `prance validate`, `openapi-spec-validator`, `redocly lint`, "
                "`openapi-generator-cli validate` (all run in sequence).",
                "**What good looks like:** No errors; only informational/warn-level notes at most.",
                "**Re-run:** `make api-lint`.",
            ]
            return '!!! info "About this artifact"\n\n' + indent("\n".join(lines) + "\n")

        desc, good = BLURBS.get(label, ("API artifact.", "Present and well-formed."))
        lines = [f"**What it is:** {desc}", f"**What good looks like:** {good}"]

        if label == "test/schemathesis.log":
            lines += [
                "**How it’s produced:** `make api-test` starts the server, waits for `/health`, then runs "
                "`schemathesis run` against each schema (adds `--stateful=links` and `--junit-xml` if supported).",
                "**Hypothesis DB:** stored under `artifacts/api/test/hypothesis/` for reproducible shrinking.",
            ]
        elif label == "test/schemathesis.xml":
            s = _parse_junit(path)
            lines.append(
                f"**Summary:** total {s['tests']}, failures {s['failures']}, errors {s['errors']}, "
                f"skipped {s['skipped']}, time ~{s['time']:.2f}s"
            )
        elif label == "server.log":
            lines += [
                "**Tip:** Search for `ERROR`, `Traceback`, or repeated 5xx responses. "
                "Logs are from the foreground/background Uvicorn server launched by the make targets.",
            ]
        elif label == "node/package.json":
            deps, dev, scripts = _pkg_json_stats(path)
            lines.append(f"**Counts:** deps={deps}, devDeps={dev}, scripts={scripts}")
            lines.append("**Sandbox dir:** `artifacts/api/node/` (no repository root pollution).")
        elif label == "node/tool-versions.txt":
            tv = _tool_versions(path)
            if tv:
                lines.append("**Resolved versions:** " + ", ".join(f"`{k}={v}`" for k, v in tv.items()))

        return '!!! info "About this artifact"\n\n' + indent("\n".join(lines) + "\n")
