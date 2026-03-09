#!/usr/bin/env python3
"""Generate documentation inventory and overlap audit."""

from __future__ import annotations

import json
import os
import re
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "docs_audit.json"


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def list_markdown() -> list[Path]:
    return sorted(p for p in ROOT.rglob("*.md") if ".git" not in p.parts)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def heading_set(text: str) -> set[str]:
    out = set()
    for line in text.splitlines():
        m = re.match(r"^#{1,6}\s+(.+?)\s*$", line.strip())
        if m:
            out.add(m.group(1).strip().lower())
    return out


def overlap_candidates(files: list[Path]) -> list[dict[str, object]]:
    headings = {p: heading_set(read(p)) for p in files}
    out: list[dict[str, object]] = []
    for i, left in enumerate(files):
        for right in files[i + 1 :]:
            if left == right:
                continue
            shared = sorted(headings[left] & headings[right])
            if len(shared) >= 3:
                out.append(
                    {
                        "left": rel(left),
                        "right": rel(right),
                        "shared_heading_count": len(shared),
                        "shared_headings": shared[:12],
                    }
                )
    out.sort(key=lambda x: (-(x["shared_heading_count"]), x["left"], x["right"]))
    return out[:80]


def readme_docs_crate_dupes(files: list[Path]) -> list[dict[str, object]]:
    buckets: dict[str, list[str]] = defaultdict(list)
    for p in files:
        name = p.name.lower()
        if name in {"readme.md", "index.md", "installation.md", "quickstart.md", "repl.md", "plugins.md", "commands.md"}:
            buckets[name].append(rel(p))
    out = [{"name": k, "files": sorted(v), "count": len(v)} for k, v in buckets.items() if len(v) > 1]
    out.sort(key=lambda x: (-x["count"], x["name"]))
    return out


def docs_duplicate_crate_readmes(files: list[Path]) -> list[dict[str, object]]:
    crate_readmes = {
        rel(p): read(p).strip().lower()[:400]
        for p in ROOT.glob("crates/*/README.md")
        if p.exists()
    }
    docs_files = [p for p in files if "docs" in p.parts]
    duplicates: list[dict[str, object]] = []
    for doc in docs_files:
        text = read(doc).strip().lower()[:400]
        if not text:
            continue
        for crate_readme, crate_text in crate_readmes.items():
            if text == crate_text:
                duplicates.append({"doc": rel(doc), "crate_readme": crate_readme})
    return sorted(duplicates, key=lambda item: (item["doc"], item["crate_readme"]))


def docs_vs_tests_overlap(files: list[Path]) -> list[dict[str, object]]:
    docs = [p for p in files if "docs" in p.parts]
    tests = [p for p in files if "tests" in p.parts and p.suffix == ".md"]
    if not tests:
        return []

    docs_heads = {p: heading_set(read(p)) for p in docs}
    tests_heads = {p: heading_set(read(p)) for p in tests}
    out: list[dict[str, object]] = []
    for d in docs:
        for t in tests:
            shared = sorted(docs_heads[d] & tests_heads[t])
            if len(shared) >= 2:
                out.append(
                    {
                        "doc": rel(d),
                        "test_doc": rel(t),
                        "shared_heading_count": len(shared),
                        "shared_headings": shared[:10],
                    }
                )
    out.sort(key=lambda x: (-(x["shared_heading_count"]), x["doc"], x["test_doc"]))
    return out[:40]


def docs_duplicate_tests_or_snapshots(files: list[Path]) -> list[dict[str, object]]:
    docs_files = [p for p in files if "docs" in p.parts]
    out: list[dict[str, object]] = []
    for doc in docs_files:
        text = read(doc)
        hits = []
        if "tests/" in text:
            hits.append("tests")
        if "snapshots/" in text:
            hits.append("snapshots")
        if "include_str!(\"snapshots/" in text:
            hits.append("snapshot-inline")
        if hits:
            out.append({"doc": rel(doc), "signals": sorted(set(hits))})
    return sorted(out, key=lambda item: item["doc"])


def docs_duplicate_schemas_or_generated_artifacts(files: list[Path]) -> list[dict[str, object]]:
    docs_files = [p for p in files if "docs" in p.parts]
    out: list[dict[str, object]] = []
    for doc in docs_files:
        text = read(doc)
        signals = []
        if "artifacts/" in text:
            signals.append("generated-artifacts")
        if "schema" in text.lower():
            signals.append("schema-prose")
        if "command_parity_matrix.json" in text or "current_rust_state.json" in text:
            signals.append("status-reference")
        if signals:
            out.append({"doc": rel(doc), "signals": sorted(set(signals))})
    return sorted(out, key=lambda item: item["doc"])


def stable_generated_at() -> str:
    source_date_epoch = os.getenv("SOURCE_DATE_EPOCH", "").strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def classify_doc(path: str) -> str:
    if path in {
        "docs/NO_HYPE.md",
        "docs/WHAT_STILL_NEEDS_WORK.md",
        "docs/WHAT_WE_DO_NOT_DO.md",
    }:
        return "delete"
    if path in {
        "docs/guides/configuration.md",
        "docs/guides/plugins.md",
        "docs/getting-started/installation.md",
        "docs/rust-config-parity.md",
        "docs/architecture/known-remaining-parity-gaps.md",
        "docs/architecture/next-five-command-priorities.md",
    }:
        return "merge"
    if path.startswith("docs/architecture/") and "parity-report" in path:
        return "replace-with-generated"
    return "keep"


def classification_report(files: list[Path]) -> dict[str, object]:
    rows = []
    counts: dict[str, int] = {"keep": 0, "merge": 0, "replace-with-generated": 0, "delete": 0}
    for file in files:
        path = rel(file)
        decision = classify_doc(path)
        counts[decision] = counts.get(decision, 0) + 1
        rows.append({"path": path, "decision": decision})
    rows.sort(key=lambda item: item["path"])
    return {"counts": counts, "documents": rows}


def generated_artifact_count() -> int:
    artifacts = ROOT / "artifacts"
    if not artifacts.exists():
        return 0
    return sum(1 for p in artifacts.rglob("*") if p.is_file())


def main() -> int:
    files = list_markdown()
    overlap = overlap_candidates(files)
    readme_dupes = readme_docs_crate_dupes(files)
    tests_overlap = docs_vs_tests_overlap(files)
    crate_readme_dupes = docs_duplicate_crate_readmes(files)
    test_snapshot_dupes = docs_duplicate_tests_or_snapshots(files)
    schema_generated_dupes = docs_duplicate_schemas_or_generated_artifacts(files)
    classification = classification_report(files)

    report = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_docs_audit.py",
        "markdown_count": len(files),
        "markdown_files": [rel(p) for p in files],
        "generated_artifact_file_count": generated_artifact_count(),
        "docs_duplicate_crate_readmes": crate_readme_dupes,
        "docs_duplicate_tests_or_snapshots": test_snapshot_dupes,
        "docs_duplicate_schemas_or_generated_artifacts": schema_generated_dupes,
        "documentation_decisions": classification,
        "top_level_canonical_docs": {
            "index": "docs/index.md",
            "honest_status": "docs/HONEST_STATUS.md",
            "stability_breakage": "docs/STABILITY_AND_BREAKAGE.md",
            "contributor_engineering_rules": "docs/CONTRIBUTOR_ENGINEERING_RULES.md",
        },
        "merge_targets": {
            "installation": "docs/guides/installation-unified.md",
            "architecture": "docs/architecture/index.md",
            "plugin": "docs/guides/plugin-unified.md",
            "config": "docs/guides/config-unified.md",
            "parity": "artifacts/parity/command_parity_matrix.json",
        },
        "readme_docs_crate_duplicates": readme_dupes,
        "docs_heading_overlap_candidates": overlap,
        "docs_tests_overlap_candidates": tests_overlap,
        "target_long_form_docs_cap": 60,
        "docs_rule": "docs explain law or change; generated artifacts hold volatile detail",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
