#!/usr/bin/env python3
"""Generate public API inventory, internal-only candidates, and cross-crate usage reports."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRATES_DIR = ROOT / "crates"
OUT_DIR = ROOT / "artifacts" / "status"
TARGET_CRATES = {
    "bijux-cli-core",
    "bijux-cli-plugin",
    "bijux-cli-repl",
    "bijux-cli-routing",
    "bijux-cli-install",
    "bijux-cli-output",
}

PUBLIC_ITEM_RE = re.compile(
    r"^pub\s+(?:\([^\)]*\)\s+)?(?P<kind>fn|struct|enum|trait|type|const|static)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b"
)


@dataclass
class PublicItem:
    crate: str
    path: str
    line: int
    kind: str
    name: str


def workspace_crates() -> list[tuple[str, Path]]:
    crates: list[tuple[str, Path]] = []
    for cargo_toml in sorted(CRATES_DIR.glob("*/Cargo.toml")):
        text = cargo_toml.read_text(encoding="utf-8")
        match = re.search(r'^name\s*=\s*"([^"]+)"', text, flags=re.MULTILINE)
        if not match:
            continue
        name = match.group(1)
        if name in TARGET_CRATES:
            crates.append((name, cargo_toml.parent))
    return crates


def collect_public_items(crate: str, crate_dir: Path) -> list[PublicItem]:
    items: list[PublicItem] = []
    for rs in sorted((crate_dir / "src").rglob("*.rs")):
        rel = rs.relative_to(ROOT).as_posix()
        for idx, line in enumerate(rs.read_text(encoding="utf-8").splitlines(), start=1):
            m = PUBLIC_ITEM_RE.match(line.strip())
            if not m:
                continue
            items.append(
                PublicItem(
                    crate=crate,
                    path=rel,
                    line=idx,
                    kind=m.group("kind"),
                    name=m.group("name"),
                )
            )
    return items


def crate_text(crate_dir: Path) -> str:
    chunks: list[str] = []
    for rs in sorted((crate_dir / "src").rglob("*.rs")):
        chunks.append(rs.read_text(encoding="utf-8"))
    return "\n".join(chunks)


def usage_count(symbol: str, all_crates: dict[str, str], owner: str) -> tuple[int, list[str]]:
    hits = 0
    users: list[str] = []
    pattern = re.compile(rf"\b{re.escape(symbol)}\b")
    for crate, text in all_crates.items():
        if crate == owner:
            continue
        if pattern.search(text):
            hits += 1
            users.append(crate)
    return hits, users


def classify(item: PublicItem, cross_crate_hits: int) -> str:
    if item.kind in {"struct", "enum", "trait"}:
        return "necessary"
    if cross_crate_hits > 0:
        return "necessary"
    if item.name.startswith("new") or item.name.endswith("_marker"):
        return "accidental"
    return "convenience"


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    generated_at = datetime.now(timezone.utc).isoformat()

    crates = workspace_crates()
    all_items: list[PublicItem] = []
    crate_texts = {crate: crate_text(crate_dir) for crate, crate_dir in crates}

    for crate, crate_dir in crates:
        all_items.extend(collect_public_items(crate, crate_dir))

    by_crate: dict[str, dict] = {}
    internal_candidates: dict[str, list[dict]] = {}
    cross_usage: list[dict] = []

    for crate, _ in crates:
        by_crate[crate] = {"necessary": [], "convenience": [], "accidental": []}
        internal_candidates[crate] = []

    for item in all_items:
        hits, users = usage_count(item.name, crate_texts, item.crate)
        classification = classify(item, hits)
        row = {
            "name": item.name,
            "kind": item.kind,
            "path": item.path,
            "line": item.line,
            "classification": classification,
            "cross_crate_users": users,
            "cross_crate_user_count": hits,
        }
        by_crate[item.crate][classification].append(row)
        cross_usage.append(
            {
                "crate": item.crate,
                "symbol": item.name,
                "kind": item.kind,
                "cross_crate_users": users,
                "cross_crate_user_count": hits,
                "defined_at": f"{item.path}:{item.line}",
            }
        )
        if hits == 0 and item.kind == "fn":
            internal_candidates[item.crate].append(
                {
                    "symbol": item.name,
                    "defined_at": f"{item.path}:{item.line}",
                    "reason": "no cross-crate consumer detected",
                }
            )

    with (OUT_DIR / "public_api_by_crate.json").open("w", encoding="utf-8") as fh:
        json.dump(
            {
                "generated_at": generated_at,
                "crates": by_crate,
            },
            fh,
            indent=2,
            sort_keys=True,
        )
        fh.write("\n")

    with (OUT_DIR / "internal_only_candidates_by_crate.json").open("w", encoding="utf-8") as fh:
        json.dump(
            {
                "generated_at": generated_at,
                "crates": internal_candidates,
            },
            fh,
            indent=2,
            sort_keys=True,
        )
        fh.write("\n")

    with (OUT_DIR / "cross_crate_api_usage.json").open("w", encoding="utf-8") as fh:
        json.dump(
            {
                "generated_at": generated_at,
                "items": sorted(cross_usage, key=lambda x: (x["crate"], x["symbol"])),
            },
            fh,
            indent=2,
            sort_keys=True,
        )
        fh.write("\n")

    print("wrote artifacts/status/public_api_by_crate.json")
    print("wrote artifacts/status/internal_only_candidates_by_crate.json")
    print("wrote artifacts/status/cross_crate_api_usage.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
