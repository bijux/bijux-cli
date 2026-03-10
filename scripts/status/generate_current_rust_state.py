#!/usr/bin/env python3
"""Generate current Rust status artifact for parity and ownership visibility."""

from __future__ import annotations

import json
import os
import re
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "current_rust_state.json"


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


def run_cmd(args: list[str], cwd: Path | None = None) -> str:
    proc = subprocess.run(args, cwd=cwd or ROOT, capture_output=True, text=True)
    if proc.returncode != 0:
        return ""
    return proc.stdout


def extract_quoted_list_block(source: str, marker: str) -> list[str]:
    idx = source.find(marker)
    if idx < 0:
        return []
    chunk = source[idx:]
    start = chunk.find("[")
    end = chunk.find("])")
    if start < 0 or end < 0:
        return []
    block = chunk[start : end + 1]
    return re.findall(r'"([^"]+)"\.to_string\(\)', block)


def parse_rust_routed_commands() -> dict[str, list[str]]:
    registry = read_text(ROOT / "crates/bijux-cli-routing/src/registry.rs")
    built_ins = extract_quoted_list_block(registry, "let built_ins")
    aliases_block = registry.split("let aliases =", 1)[1] if "let aliases =" in registry else ""
    alias_pairs = re.findall(r'\("([^"]+)"\.to_string\(\),\s*"([^"]+)"\.to_string\(\)\)', aliases_block)
    aliases = [a for a, _ in alias_pairs]
    canonical = sorted(set(built_ins))
    surface = sorted(set(canonical + aliases))
    return {
        "canonical": canonical,
        "surface": surface,
        "aliases": sorted(set(aliases)),
    }


def parse_python_command_tree() -> list[str]:
    exe = ROOT / "bin" / "bijux"
    if not exe.exists():
        return []

    def subcommands_for(path_tokens: tuple[str, ...]) -> list[str]:
        args = [str(exe), *path_tokens, "--help"]
        out = run_cmd(args)
        if not out:
            return []
        lines = out.splitlines()
        in_commands = False
        subs: list[str] = []
        for line in lines:
            if line.strip() == "Commands:":
                in_commands = True
                continue
            if in_commands and line.strip().startswith("Options:"):
                break
            if not in_commands:
                continue
            stripped = line.strip()
            if not stripped:
                continue
            token = stripped.split()[0]
            if token == "help":
                continue
            if re.match(r"^[a-z][a-z0-9_-]*$", token):
                subs.append(token)
        return subs

    discovered: set[tuple[str, ...]] = set()
    queue: list[tuple[str, ...]] = [tuple()]
    max_depth = 4

    while queue:
        path = queue.pop(0)
        if path in discovered:
            continue
        discovered.add(path)
        if len(path) >= max_depth:
            continue
        for sub in subcommands_for(path):
            child = (*path, sub)
            if child not in discovered:
                queue.append(child)

    commands = sorted(" ".join(p) for p in discovered if p)
    return commands


def parse_parity_report() -> tuple[list[str], list[dict[str, object]]]:
    report_path = ROOT / "artifacts" / "parity" / "rust_python_parity_report.json"
    if not report_path.exists():
        return [], []
    data = json.loads(report_path.read_text(encoding="utf-8"))
    commands = data.get("commands", []) if isinstance(data, dict) else []
    covered = sorted({c.get("name", "") for c in commands if c.get("name")})

    mismatches = []
    for c in commands:
        if c.get("status") != "rust-complete" or not (c.get("exit_match") and c.get("stdout_match") and c.get("stderr_match")):
            mismatches.append(
                {
                    "command": c.get("name"),
                    "status": c.get("status"),
                    "exit_match": bool(c.get("exit_match")),
                    "stdout_match": bool(c.get("stdout_match")),
                    "stderr_match": bool(c.get("stderr_match")),
                }
            )
    return covered, mismatches


def parity_assertions() -> dict[str, object]:
    report_path = ROOT / "artifacts" / "parity" / "rust_python_parity_report.json"
    if not report_path.exists():
        return {
            "source": str(report_path.relative_to(ROOT)),
            "same_command_tree_where_parity_exists": False,
            "same_exit_codes_where_parity_exists": False,
            "same_output_envelopes_where_parity_exists": False,
            "checked_commands": [],
            "violations": ["missing parity report artifact"],
        }

    data = json.loads(report_path.read_text(encoding="utf-8"))
    commands = data.get("commands", []) if isinstance(data, dict) else []

    checked_commands: list[str] = []
    tree_violations: list[str] = []
    exit_violations: list[str] = []
    envelope_violations: list[str] = []

    for row in commands:
        name = str(row.get("name", "")).strip()
        status = str(row.get("status", "")).strip()
        if not name or status == "python-only":
            continue
        checked_commands.append(name)
        if status not in {"rust-complete", "rust-partial"}:
            tree_violations.append(name)
        if not bool(row.get("exit_match")):
            exit_violations.append(name)
        if not bool(row.get("stdout_match")) or not bool(row.get("stderr_match")):
            envelope_violations.append(name)

    violations: list[str] = []
    if tree_violations:
        violations.append(f"command_tree={','.join(sorted(tree_violations))}")
    if exit_violations:
        violations.append(f"exit_code={','.join(sorted(exit_violations))}")
    if envelope_violations:
        violations.append(f"output_envelope={','.join(sorted(envelope_violations))}")

    return {
        "source": str(report_path.relative_to(ROOT)),
        "same_command_tree_where_parity_exists": not tree_violations,
        "same_exit_codes_where_parity_exists": not exit_violations,
        "same_output_envelopes_where_parity_exists": not envelope_violations,
        "checked_commands": sorted(set(checked_commands)),
        "violations": violations,
    }


def parse_snapshot_coverage() -> list[str]:
    snap_root = ROOT / "crates" / "bijux-cli" / "tests" / "snapshots"
    cmds: set[str] = set()
    if not snap_root.exists():
        return []
    for p in snap_root.rglob("*.txt"):
        name = p.stem
        if name.startswith("help_"):
            cmd = name.removeprefix("help_").replace("_", " ")
            cmds.add(cmd)
        elif name.startswith("config_"):
            chunks = name.split("_")
            if len(chunks) >= 2:
                if chunks[1] in {"root"}:
                    cmds.add("config")
                else:
                    cmds.add(f"cli config {chunks[1]}")
        elif name.startswith("history_"):
            cmds.add("history")
        elif name.startswith("memory_"):
            cmds.add("memory list")
        elif name.startswith("inspect"):
            cmds.add("cli inspect")
        elif name.startswith("dev_cli_"):
            parts = name.split("_")
            if len(parts) >= 3:
                cmds.add(f"dev cli {parts[2]}")
    return sorted(cmds)


def collect_test_coverages() -> tuple[list[str], list[str]]:
    test_files = sorted((ROOT / "crates").rglob("tests/*.rs"))
    exit_cmds: set[str] = set()
    stream_cmds: set[str] = set()

    for tf in test_files:
        txt = read_text(tf)
        file_cmds: set[str] = set()
        for m in re.finditer(r'\[(.*?)\]', txt, flags=re.S):
            raw = m.group(1)
            tokens = re.findall(r'"([a-z][a-z0-9_-]*)"', raw)
            if not tokens:
                continue
            if tokens[0] in {"bijux", "bijux-rs"}:
                tokens = tokens[1:]
            if not tokens:
                continue
            if tokens[0] in {"cli", "dev", "config", "status", "history", "memory", "plugins", "doctor", "version", "inspect"}:
                file_cmds.add(" ".join(tokens[:4]).strip())

        if "status.code()" in txt or "exit_code" in txt:
            exit_cmds.update(file_cmds)
        if ".stderr" in txt or ".stdout" in txt:
            stream_cmds.update(file_cmds)

    norm = lambda s: " ".join(s.split())
    return sorted({norm(c) for c in exit_cmds if c}), sorted({norm(c) for c in stream_cmds if c})


def parse_still_shimmed() -> list[str]:
    out: set[str] = set()
    for p in [ROOT / "crates/bijux-cli/src/kernel.rs", ROOT / "docs/architecture/core-kernel-parity-audit.md"]:
        txt = read_text(p)
        for line in txt.splitlines():
            if "PARITY-PARTIAL" in line or "shim" in line.lower():
                if "help|version|completion" in line or "help/version/completion" in line:
                    out.update(["cli help", "cli version", "cli completion"])
                m = re.findall(r'`([^`]+)`', line)
                out.update(x for x in m if " " in x)
    return sorted(out)


def count_public_api(crate_dir: Path) -> int:
    count = 0
    for rs in crate_dir.rglob("src/**/*.rs"):
        txt = read_text(rs)
        count += len(re.findall(r"(?m)^pub\s+(?:fn|struct|enum|trait|mod|type|const|static|use)\b", txt))
    return count


def crate_dependency_edges(members: list[str]) -> list[dict[str, str]]:
    member_names: dict[str, str] = {}
    for rel in members:
        cpath = ROOT / rel / "Cargo.toml"
        data = parse_toml(cpath) if cpath.exists() else {}
        folder_name = Path(rel).name
        package_name = data.get("package", {}).get("name", folder_name)
        member_names[folder_name] = folder_name
        member_names[folder_name.replace("-", "_")] = folder_name
        member_names[package_name] = folder_name
        member_names[package_name.replace("-", "_")] = folder_name

    edges = []
    for rel in members:
        cpath = ROOT / rel / "Cargo.toml"
        data = parse_toml(cpath) if cpath.exists() else {}
        deps = data.get("dependencies", {})
        for dep_name in deps.keys():
            if dep_name in member_names:
                edges.append({"from": Path(rel).name, "to": member_names[dep_name]})
    edges.sort(key=lambda x: (x["from"], x["to"]))
    return edges


def list_docs_files() -> list[str]:
    return sorted(str(p.relative_to(ROOT)) for p in ROOT.rglob("*.md") if ".git" not in p.parts)


def list_scripts_outside_dev_cli() -> list[str]:
    script_dir = ROOT / "scripts"
    if not script_dir.exists():
        return []
    return sorted(str(p.relative_to(ROOT)) for p in script_dir.rglob("*") if p.is_file())


def list_contracts_and_schemas() -> list[str]:
    out = set()
    for pat in ["api/**/*.yaml", "api/**/*.yml", "api/**/*.json", "**/*schema*.json", "**/*schema*.yaml", "**/*schema*.yml"]:
        for p in ROOT.glob(pat):
            if p.is_file() and ".git" not in p.parts and "target" not in p.parts:
                out.add(str(p.relative_to(ROOT)))
    for p in (ROOT / "crates" / "bijux-cli-contracts" / "src" / "contracts").glob("*.rs"):
        out.add(str(p.relative_to(ROOT)))
    return sorted(out)


def list_parity_fixtures() -> list[str]:
    out = set()
    for p in ROOT.rglob("*parity*"):
        if p.is_file() and ("tests" in p.parts or "artifacts" in p.parts or "docs" in p.parts):
            out.add(str(p.relative_to(ROOT)))
    return sorted(out)


def list_snapshot_fixtures() -> list[str]:
    out = []
    for p in ROOT.rglob("tests/snapshots/*"):
        if p.is_file():
            out.append(str(p.relative_to(ROOT)))
    return sorted(out)


def list_platform_assumptions() -> list[dict[str, str]]:
    assumptions = []
    for p in ROOT.rglob("*.rs"):
        if "target" in p.parts:
            continue
        txt = read_text(p)
        for i, line in enumerate(txt.splitlines(), start=1):
            if "cfg(unix)" in line or "cfg(windows)" in line or "HOME" in line or "XDG" in line:
                assumptions.append({"file": str(p.relative_to(ROOT)), "line": i, "assumption": line.strip()})
    for p in (ROOT / "docs").rglob("*.md"):
        txt = read_text(p)
        for i, line in enumerate(txt.splitlines(), start=1):
            l = line.lower()
            if any(k in l for k in ["linux", "macos", "windows", "home", "xdg"]):
                assumptions.append({"file": str(p.relative_to(ROOT)), "line": i, "assumption": line.strip()})
    return assumptions[:200]


def package_entrypoints() -> list[dict[str, str]]:
    out = []
    pyproject = parse_toml(ROOT / "pyproject.toml")
    for name, target in pyproject.get("project", {}).get("scripts", {}).items():
        out.append({"package": "python", "entrypoint": name, "target": target})

    for cargo in ROOT.glob("crates/*/Cargo.toml"):
        data = parse_toml(cargo)
        pkg = data.get("package", {}).get("name")
        for b in data.get("bin", []):
            out.append({"package": pkg or cargo.parent.name, "entrypoint": b.get("name", ""), "target": b.get("path", "")})

    package_json = json.loads(read_text(ROOT / "package.json") or "{}")
    for name in package_json.get("scripts", {}).keys():
        out.append({"package": "node", "entrypoint": name, "target": "package.json#scripts"})

    return sorted(out, key=lambda x: (x["package"], x["entrypoint"]))


def runtime_identity_rules() -> dict[str, object]:
    pyproject = parse_toml(ROOT / "pyproject.toml")
    project_scripts = pyproject.get("project", {}).get("scripts", {})
    python_entrypoints = sorted(project_scripts.keys())
    canonical_python_entrypoint = "bijux" in project_scripts
    forbidden_public_runtime_names = {"bijux-rs", "bijux-cli-rs", "bijux-cli-py"}
    forbidden_present = sorted(name for name in python_entrypoints if name in forbidden_public_runtime_names)

    cargo_bin = parse_toml(ROOT / "crates" / "bijux-cli" / "Cargo.toml")
    cargo_bins = [row.get("name", "") for row in cargo_bin.get("bin", []) if isinstance(row, dict)]

    return {
        "canonical_user_binary": "bijux",
        "python_package_entrypoints": python_entrypoints,
        "python_package_points_users_to_bijux": canonical_python_entrypoint and not forbidden_present,
        "forbidden_public_runtime_names_present": forbidden_present,
        "internal_runtime_labels": {
            "rust_package_concept": "bijux-cli-rs",
            "python_package_concept": "bijux-cli-py",
        },
        "cargo_bin_names": sorted(cargo_bins),
    }


def plugin_reserved_namespaces() -> list[str]:
    registry = read_text(ROOT / "crates/bijux-cli-routing/src/registry.rs")
    idx = registry.find("let reserved")
    if idx < 0:
        return []
    block = registry[idx:]
    start = block.find("[")
    end = block.find("])")
    if start < 0 or end < 0:
        return []
    values = re.findall(r'"([^"]+)"\.to_string\(\)', block[start : end + 1])
    return sorted(set(values))


def workspace_members() -> list[str]:
    ws = parse_toml(ROOT / "Cargo.toml")
    return ws.get("workspace", {}).get("members", [])


def main() -> None:
    members = workspace_members()
    routed = parse_rust_routed_commands()
    py_commands = parse_python_command_tree()
    parity_covered, mismatches = parse_parity_report()
    snapshot_covered = parse_snapshot_coverage()
    exit_covered, stream_covered = collect_test_coverages()

    crate_apis = []
    for rel in members:
        crate_apis.append({
            "crate": Path(rel).name,
            "public_api_count": count_public_api(ROOT / rel),
        })

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_current_rust_state.py",
        "rust_routed_commands": routed,
        "python_only_commands_not_routed_by_rust": sorted(set(py_commands) - set(routed["surface"])),
        "parity_covered_commands": parity_covered,
        "snapshot_covered_commands": snapshot_covered,
        "exit_code_covered_commands": exit_covered,
        "stderr_stdout_covered_commands": stream_covered,
        "still_shimmed_commands": parse_still_shimmed(),
        "crates_public_api_counts": sorted(crate_apis, key=lambda x: x["crate"]),
        "crate_dependency_edges": crate_dependency_edges(members),
        "docs_files": list_docs_files(),
        "scripts_outside_dev_cli": list_scripts_outside_dev_cli(),
        "machine_readable_contracts_and_schemas": list_contracts_and_schemas(),
        "parity_fixtures": list_parity_fixtures(),
        "snapshot_fixtures": list_snapshot_fixtures(),
        "rust_vs_python_mismatches": mismatches,
        "runtime_parity_assertions": parity_assertions(),
        "runtime_identity_rules": runtime_identity_rules(),
        "known_platform_assumptions": list_platform_assumptions(),
        "package_entrypoints": package_entrypoints(),
        "plugin_reserved_namespaces": plugin_reserved_namespaces(),
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
