#!/usr/bin/env python3
import argparse
import json
import re
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path
from typing import Dict, List, Set, Tuple


REPO_ROOT = Path(__file__).resolve().parent.parent
DOCS_ROOT = REPO_ROOT / "docs"
POLICY_PATH = REPO_ROOT / "configs/policy/docs_lint_policy.json"
ALLOWED_STATUS = {"stable", "generated", "historical", "internal"}
LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def load_policy() -> Dict:
    if POLICY_PATH.exists():
        return json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    return {
        "exclude_prefixes": [],
        "metadata_required_prefixes": ["docs/"],
        "metadata_required_exact": [],
        "standalone_allowlist": [],
    }


def all_docs() -> List[Path]:
    return sorted(DOCS_ROOT.rglob("*.md"))


def rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def should_exclude(path_rel: str, policy: Dict) -> bool:
    return any(path_rel.startswith(prefix) for prefix in policy.get("exclude_prefixes", []))


def metadata_required(path_rel: str, policy: Dict) -> bool:
    if path_rel in set(policy.get("metadata_required_exact", [])):
        return True
    return any(path_rel.startswith(prefix) for prefix in policy.get("metadata_required_prefixes", []))


def read_lines(path: Path) -> List[str]:
    return path.read_text(encoding="utf-8").splitlines()


def parse_metadata(lines: List[str]) -> Dict[str, str]:
    meta: Dict[str, str] = {}
    head = lines[:50]
    if len(head) >= 3 and head[0].strip() == "---":
        for line in head[1:]:
            if line.strip() == "---":
                break
            if ":" in line:
                k, v = line.split(":", 1)
                meta[k.strip().lower()] = v.strip()
    for line in head:
        if ":" not in line:
            continue
        k, v = line.split(":", 1)
        key = k.strip().lower()
        if key in {"audience", "owner", "status"} and key not in meta:
            meta[key] = v.strip()
    return meta


def title_of(lines: List[str]) -> str:
    for line in lines:
        if line.startswith("# "):
            return line[2:].strip()
    return ""


def resolve_link(source: Path, target: str) -> Path:
    clean = target.split("#", 1)[0].strip()
    if clean.startswith("/"):
        return REPO_ROOT / clean.lstrip("/")
    return (source.parent / clean).resolve()


def collect_inbound(policy: Dict) -> Tuple[Counter, Set[str]]:
    inbound: Counter = Counter()
    existing: Set[str] = set()
    for path in all_docs():
        path_rel = rel(path)
        if should_exclude(path_rel, policy):
            continue
        existing.add(path_rel)
    for path in all_docs():
        path_rel = rel(path)
        if should_exclude(path_rel, policy):
            continue
        content = path.read_text(encoding="utf-8")
        for target in LINK_RE.findall(content):
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            resolved = resolve_link(path, target)
            if resolved.suffix.lower() != ".md":
                continue
            if not resolved.exists():
                continue
            target_rel = rel(resolved)
            if should_exclude(target_rel, policy):
                continue
            inbound[target_rel] += 1
    return inbound, existing


def lint() -> int:
    policy = load_policy()
    inbound, existing = collect_inbound(policy)
    missing_metadata: List[str] = []
    bad_status: List[str] = []
    duplicate_titles: Dict[str, List[str]] = defaultdict(list)
    orphan_docs: List[str] = []
    duplicate_topics: List[str] = []

    for path in all_docs():
        path_rel = rel(path)
        if should_exclude(path_rel, policy):
            continue
        lines = read_lines(path)
        meta = parse_metadata(lines)
        if metadata_required(path_rel, policy):
            for key in ("audience", "owner", "status"):
                if key not in meta or not meta[key]:
                    missing_metadata.append(f"{path_rel}: missing `{key}`")
            status = meta.get("status", "").lower()
            if status and status not in ALLOWED_STATUS:
                bad_status.append(f"{path_rel}: invalid status `{meta.get('status')}`")
        title = title_of(lines)
        if title:
            normalized = re.sub(r"[^a-z0-9]+", " ", title.lower()).strip()
            duplicate_titles[title].append(path_rel)
            duplicate_topics.append(f"{normalized}\t{path_rel}")

        standalone = any(line.strip().lower() == "standalone: yes" for line in lines[:80])
        allowlist = set(policy.get("standalone_allowlist", []))
        if (
            path_rel in existing
            and inbound[path_rel] == 0
            and not path.name.lower().startswith(("readme", "index"))
            and not standalone
            and path_rel not in allowlist
        ):
            orphan_docs.append(path_rel)

    duplicate_title_errors = []
    for title, paths in duplicate_titles.items():
        if len(paths) > 1:
            duplicate_title_errors.append(f"duplicate title `{title}`: {', '.join(sorted(paths))}")

    normalized_map: Dict[str, List[str]] = defaultdict(list)
    for item in duplicate_topics:
        normalized, path_rel = item.split("\t", 1)
        normalized_map[normalized].append(path_rel)
    duplicate_topic_errors = []
    for topic, paths in normalized_map.items():
        if topic and len(paths) > 1:
            duplicate_topic_errors.append(f"duplicate topic `{topic}`: {', '.join(sorted(paths))}")

    errors = (
        missing_metadata
        + bad_status
        + duplicate_title_errors
        + duplicate_topic_errors
        + [f"orphan: {p}" for p in sorted(orphan_docs)]
    )
    if errors:
        print("docs-governance-lint: violations detected")
        for err in errors:
            print(f"- {err}")
        return 1
    print("docs-governance-lint: ok")
    return 0


def generate() -> int:
    policy = load_policy()
    inbound, existing = collect_inbound(policy)
    status_counter: Counter = Counter()
    section_counter: Counter = Counter()
    missing_metadata: List[str] = []

    docs_files = [p for p in all_docs() if not should_exclude(rel(p), policy)]
    for path in docs_files:
        path_rel = rel(path)
        lines = read_lines(path)
        meta = parse_metadata(lines)
        section = path_rel.split("/")[1] if "/" in path_rel else "root"
        section_counter[section] += 1
        status = meta.get("status", "").lower()
        if status in ALLOWED_STATUS:
            status_counter[status] += 1
        else:
            status_counter["missing_or_invalid"] += 1
        if metadata_required(path_rel, policy):
            for key in ("audience", "owner", "status"):
                if key not in meta or not meta[key]:
                    missing_metadata.append(f"{path_rel}: missing `{key}`")

    inventory_path = REPO_ROOT / "docs/generated/DOCS_INVENTORY.md"
    consolidation_path = REPO_ROOT / "docs/generated/DOCS_CONSOLIDATION_CANDIDATES.md"

    inventory_lines = [
        "# Documentation inventory",
        "",
        f"Generated: {date.today().isoformat()}",
        "",
        "## Counts by section",
        "",
    ]
    for section, count in sorted(section_counter.items()):
        inventory_lines.append(f"- `{section}`: {count}")
    inventory_lines.extend(["", "## Counts by status", ""])
    for status, count in sorted(status_counter.items()):
        inventory_lines.append(f"- `{status}`: {count}")
    inventory_lines.extend(["", "## Metadata gaps", ""])
    if missing_metadata:
        inventory_lines.extend([f"- {item}" for item in sorted(missing_metadata)[:200]])
    else:
        inventory_lines.append("- none")
    inventory_path.write_text("\n".join(inventory_lines) + "\n", encoding="utf-8")

    orphan_candidates = sorted(
        path_rel
        for path_rel in existing
        if inbound[path_rel] == 0
        and not Path(path_rel).name.lower().startswith(("readme", "index"))
        and path_rel not in set(policy.get("standalone_allowlist", []))
    )
    consolidate_lines = [
        "# Documentation consolidation candidates",
        "",
        f"Generated: {date.today().isoformat()}",
        "",
        "These files have no inbound links from other non-archived docs and are candidates for merge, move, or deletion.",
        "",
    ]
    if orphan_candidates:
        for candidate in orphan_candidates[:300]:
            consolidate_lines.append(f"- `{candidate}`")
    else:
        consolidate_lines.append("- none")
    consolidation_path.write_text("\n".join(consolidate_lines) + "\n", encoding="utf-8")

    print(f"wrote {inventory_path.relative_to(REPO_ROOT)}")
    print(f"wrote {consolidation_path.relative_to(REPO_ROOT)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="docs governance lint and report generator")
    parser.add_argument("command", choices=["lint", "generate"])
    args = parser.parse_args()
    if args.command == "lint":
        return lint()
    return generate()


if __name__ == "__main__":
    raise SystemExit(main())
