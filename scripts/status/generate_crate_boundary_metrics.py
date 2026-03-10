#!/usr/bin/env python3
"""Generate crate boundary metrics and decision report artifacts."""

from __future__ import annotations

import json
import re
import subprocess
import time
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "crate_boundary_metrics.json"
REPORT_OUT = ROOT / "artifacts" / "status" / "crate_boundary_report.json"
WORKSPACE_TOML = ROOT / "Cargo.toml"


@dataclass
class CrateInfo:
    name: str
    rel: str


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""


def parse_toml(path: Path) -> dict:
    text = read_text(path)
    if not text:
        return {}
    if tomllib is not None:
        return tomllib.loads(text)
    try:
        import tomli  # type: ignore

        return tomli.loads(text)
    except Exception:
        return {}


def run_cmd(args: list[str]) -> tuple[int, str, str]:
    proc = subprocess.run(args, cwd=ROOT, capture_output=True, text=True)
    return proc.returncode, proc.stdout, proc.stderr


def workspace_crates() -> list[CrateInfo]:
    ws = parse_toml(WORKSPACE_TOML)
    members = ws.get("workspace", {}).get("members", [])
    crates: list[CrateInfo] = []
    for rel in members:
        cargo = ROOT / rel / "Cargo.toml"
        data = parse_toml(cargo)
        name = data.get("package", {}).get("name", Path(rel).name)
        crates.append(CrateInfo(name=name, rel=rel))
    return crates


def crate_internal_deps(crates: list[CrateInfo]) -> dict[str, set[str]]:
    crate_map = {c.name: c for c in crates}
    alt = {c.name.replace("-", "_"): c.name for c in crates}
    deps: dict[str, set[str]] = defaultdict(set)

    for c in crates:
        data = parse_toml(ROOT / c.rel / "Cargo.toml")
        for dep in (data.get("dependencies", {}) or {}).keys():
            target = None
            if dep in crate_map:
                target = dep
            elif dep in alt:
                target = alt[dep]
            if target and target != c.name:
                deps[c.name].add(target)
    return deps


def measure_compile_test_times(crates: list[CrateInfo]) -> dict[str, dict[str, float | bool]]:
    results: dict[str, dict[str, float | bool]] = {}
    for c in crates:
        compile_start = time.perf_counter()
        rc_compile, _, _ = run_cmd(["cargo", "check", "-q", "-p", c.name])
        compile_sec = round(time.perf_counter() - compile_start, 3)

        test_start = time.perf_counter()
        rc_test, _, _ = run_cmd(["cargo", "test", "-q", "-p", c.name, "--no-run"])
        test_sec = round(time.perf_counter() - test_start, 3)

        results[c.name] = {
            "compile_seconds": compile_sec,
            "test_build_seconds": test_sec,
            "compile_ok": rc_compile == 0,
            "test_build_ok": rc_test == 0,
        }
    return results


def count_public_api(crate_rel: str) -> int:
    crate_dir = ROOT / crate_rel
    count = 0
    for rs in crate_dir.rglob("src/**/*.rs"):
        txt = read_text(rs)
        count += len(re.findall(r"(?m)^pub\s+(?:fn|struct|enum|trait|mod|type|const|static|use)\b", txt))
    return count


def git_commits_touching_path(path_prefix: str, max_commits: int = 200) -> set[str]:
    rc, out, _ = run_cmd(["git", "log", f"-n{max_commits}", "--pretty=%H", "--", path_prefix])
    if rc != 0:
        return set()
    return {line.strip() for line in out.splitlines() if line.strip()}


def churn_metrics(crates: list[CrateInfo], max_commits: int = 200) -> dict[str, dict[str, int]]:
    metrics: dict[str, dict[str, int]] = {}
    for c in crates:
        commit_ids = git_commits_touching_path(c.rel, max_commits=max_commits)
        insertions = deletions = files_changed = 0
        if commit_ids:
            rc, stat_out, _ = run_cmd(
                ["git", "log", f"-n{max_commits}", "--numstat", "--pretty=tformat:", "--", c.rel]
            )
            if rc == 0:
                for line in stat_out.splitlines():
                    parts = line.split("\t")
                    if len(parts) == 3 and parts[0].isdigit() and parts[1].isdigit():
                        insertions += int(parts[0])
                        deletions += int(parts[1])
                        files_changed += 1
        metrics[c.name] = {
            "commit_count": len(commit_ids),
            "files_changed_entries": files_changed,
            "insertions": insertions,
            "deletions": deletions,
        }
    return metrics


def pair_change_frequency(a_rel: str, b_rel: str, max_commits: int = 200) -> int:
    a = git_commits_touching_path(a_rel, max_commits=max_commits)
    b = git_commits_touching_path(b_rel, max_commits=max_commits)
    return len(a & b)


def boundary_decisions() -> list[dict[str, str]]:
    return [
        {
            "boundary": "core <-> routing",
            "status": "watch",
            "decision": "keep separate for now",
            "reason": "high co-change expected during parity closure; separation still useful for parser test focus",
        },
        {
            "boundary": "core <-> output",
            "status": "watch",
            "decision": "keep separate for now",
            "reason": "output formatting contracts remain reusable and test-scoped",
        },
        {
            "boundary": "core <-> install",
            "status": "watch",
            "decision": "keep separate for now",
            "reason": "install concerns include path and packaging diagnostics outside core execution law",
        },
        {
            "boundary": "core <-> contracts",
            "status": "keep",
            "decision": "must stay separate",
            "reason": "machine contracts must remain independent from execution engine",
        },
        {
            "boundary": "core <-> python",
            "status": "keep",
            "decision": "must stay separate",
            "reason": "bridge packaging/runtime integration is language-boundary specific",
        },
        {
            "boundary": "core <-> plugin",
            "status": "keep",
            "decision": "must stay separate",
            "reason": "plugin lifecycle and registry law should not be merged into base execution core",
        },
        {
            "boundary": "core <-> repl",
            "status": "keep",
            "decision": "must stay separate",
            "reason": "interactive session model and transcript behavior are distinct runtime surfaces",
        },
    ]


def crate_decisions() -> list[dict[str, str]]:
    return [
        {
            "crate": "bijux-cli-contracts",
            "status": "keep",
            "review": "must stay separate",
            "reason": "schemas and envelope law are durable contracts shared by all runtimes",
        },
        {
            "crate": "bijux-cli-core",
            "status": "keep",
            "review": "must stay separate",
            "reason": "execution law center; merging outward increases coupling risk",
        },
        {
            "crate": "bijux-cli-routing",
            "status": "watch",
            "review": "paying rent with dedicated parser fixtures and namespace policy tests",
            "reason": "high co-change with core but still isolated by routing test surface",
        },
        {
            "crate": "bijux-cli-output",
            "status": "watch",
            "review": "paying rent with envelope and rendering parity checks",
            "reason": "candidate only if output contracts become static and trivial",
        },
        {
            "crate": "bijux-cli-core::install",
            "status": "watch",
            "review": "paying rent with path/install diagnostics and channel policy",
            "reason": "keep independent while runtime identity and install parity remain active",
        },
        {
            "crate": "bijux-cli-python",
            "status": "keep",
            "review": "must stay separate",
            "reason": "bridge packaging and binding boundary is language-specific",
        },
        {
            "crate": "bijux-cli-plugin",
            "status": "keep",
            "review": "must stay separate",
            "reason": "plugin registry/lifecycle semantics need explicit boundary protection",
        },
        {
            "crate": "bijux-cli-repl",
            "status": "keep",
            "review": "must stay separate",
            "reason": "interactive session and transcript behavior are distinct runtime concerns",
        },
        {
            "crate": "bijux-cli-core",
            "status": "candidate-to-merge-later",
            "review": "thin executable wrapper currently acceptable",
            "reason": "revisit only after parity and runtime identity reports converge",
        },
    ]


def main() -> None:
    crates = workspace_crates()
    deps = crate_internal_deps(crates)

    fan_out = {c.name: len(deps.get(c.name, set())) for c in crates}
    fan_in_count: dict[str, int] = {c.name: 0 for c in crates}
    for src, targets in deps.items():
        for dst in targets:
            fan_in_count[dst] += 1

    compile_test = measure_compile_test_times(crates)
    churn = churn_metrics(crates)

    public_api = {c.name: count_public_api(c.rel) for c in crates}

    pairwise_change_frequency: list[dict[str, object]] = []
    for i, left in enumerate(crates):
        for right in crates[i + 1 :]:
            shared = pair_change_frequency(left.rel, right.rel)
            pairwise_change_frequency.append(
                {"left": left.name, "right": right.name, "shared_commits": shared}
            )
    pairwise_change_frequency.sort(
        key=lambda item: int(item.get("shared_commits", 0)), reverse=True
    )

    crate_rows = []
    for c in sorted(crates, key=lambda x: x.name):
        crate_rows.append(
            {
                "crate": c.name,
                "compile_seconds": compile_test[c.name]["compile_seconds"],
                "test_build_seconds": compile_test[c.name]["test_build_seconds"],
                "dependency_fan_in": fan_in_count[c.name],
                "dependency_fan_out": fan_out[c.name],
                "public_api_count": public_api[c.name],
                "churn": churn[c.name],
                "compile_ok": compile_test[c.name]["compile_ok"],
                "test_build_ok": compile_test[c.name]["test_build_ok"],
            }
        )

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_crate_boundary_metrics.py",
        "metrics": {
            "per_crate": crate_rows,
            "cross_crate_change_frequency": pairwise_change_frequency,
        },
        "boundary_decisions": boundary_decisions(),
        "crate_decisions": crate_decisions(),
        "rules": {
            "no_large_merge_until_parity_stronger": True,
            "rule_text": "Large crate merges are frozen until parity coverage and mismatch trend show sustained improvement.",
        },
    }

    decision_summary = {
        "keep": sum(1 for row in crate_decisions() if row["status"] == "keep"),
        "watch": sum(1 for row in crate_decisions() if row["status"] == "watch"),
        "candidate_to_merge_later": sum(
            1 for row in crate_decisions() if row["status"] == "candidate-to-merge-later"
        ),
    }
    report_summary = {
        "generated_at": report["generated_at"],
        "generator": report["generator"],
        "evidence": {
            "metrics_artifact": str(OUT.relative_to(ROOT)),
            "top_cross_crate_pairs": pairwise_change_frequency[:10],
        },
        "crate_decision_summary": decision_summary,
        "crate_decisions": crate_decisions(),
        "boundary_decisions": boundary_decisions(),
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    REPORT_OUT.write_text(
        json.dumps(report_summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"wrote {OUT.relative_to(ROOT)}")
    print(f"wrote {REPORT_OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
