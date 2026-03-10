#!/usr/bin/env python3
"""Generate state inventory, mutation, and guarantee artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def rg_lines(pattern: str) -> list[str]:
    cmd = ["rg", "-n", pattern, "crates", "-S"]
    result = subprocess.run(cmd, cwd=ROOT, check=False, capture_output=True, text=True)
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def main() -> None:
    generated_at = stable_generated_at()
    inventory = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "state_files": [
            {
                "id": "config_file",
                "classification": "core",
                "path_source": "discover_compatibility_paths",
                "reader": "FileConfigRepository::load",
                "writer": "FileConfigRepository::save",
            },
            {
                "id": "history_file",
                "classification": "core",
                "path_source": "discover_compatibility_paths",
                "reader": "read_history_entries",
                "writer": "repl::flush_history",
            },
            {
                "id": "plugin_registry_file",
                "classification": "core",
                "path_source": "registry_path_from_plugins_dir",
                "reader": "plugin::load_registry",
                "writer": "plugin::save_registry",
            },
            {
                "id": "memory_file",
                "classification": "optional",
                "path_source": "resolve_state_paths",
                "reader": "read_memory_map",
                "writer": "write_memory_map",
            },
            {
                "id": "compatibility_config_file",
                "classification": "optional",
                "path_source": "default_compatibility_paths",
                "reader": "load_compatibility_config",
                "writer": "write_compatibility_config",
            },
        ],
    }

    readers = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "matches": rg_lines(r"read_to_string|load_registry|load_history|read_history_entries|read_memory_map"),
    }
    writers = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "matches": rg_lines(r"atomic_write_text|save_registry|flush_history|write_compatibility_config|FileConfigRepository::save"),
    }
    mutations = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "matches": rg_lines(r"set_pair|unset_key|clear_all|install_plugin|uninstall_plugin|enable_plugin|disable_plugin"),
    }

    write_guarantees = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "guarantees": [
            {
                "name": "core config writes are atomic",
                "evidence": "crates/bijux-cli-core/src/config/storage.rs uses atomic_write_text",
            },
            {
                "name": "compatibility config writes are atomic",
                "evidence": "crates/bijux-cli-install/src/compatibility.rs uses atomic_write_text",
            },
            {
                "name": "plugin registry writes use temp+rename",
                "evidence": "crates/bijux-cli-plugin/src/registry.rs::save_registry",
            },
            {
                "name": "repl history writes are atomic",
                "evidence": "crates/bijux-cli-repl/src/history.rs::flush_history uses atomic_write_text",
            },
            {
                "name": "core history and memory writes are atomic",
                "evidence": "crates/bijux-cli-core/src/app.rs::write_json_document uses atomic_write_text",
            },
        ],
    }

    recovery_guarantees = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "guarantees": [
            {
                "name": "plugin registry rollback on mutation failure",
                "evidence": "crates/bijux-cli-plugin/src/registry.rs::update_registry",
            },
            {
                "name": "state doctor surfaces degraded state with issues",
                "evidence": "crates/bijux-cli-core/src/app.rs::state_diagnostics",
            },
            {
                "name": "history corruption is tolerated with fallback parser",
                "evidence": "crates/bijux-cli-core/src/app.rs::parse_history_entries",
            },
        ],
    }

    complexity = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_law_reports.py",
        "canonical_services": [
            "crates/bijux-cli-core/src/app.rs::resolve_state_paths",
            "crates/bijux-cli-install/src/io.rs::atomic_write_text",
        ],
        "hotspots": [
            "crates/bijux-cli-core/src/app.rs",
            "crates/bijux-cli-plugin/src/registry.rs",
            "crates/bijux-cli-repl/src/history.rs",
        ],
        "summary": {
            "inventory_count": len(inventory["state_files"]),
            "reader_matches": len(readers["matches"]),
            "writer_matches": len(writers["matches"]),
            "mutation_matches": len(mutations["matches"]),
        },
    }

    write_json(STATUS / "state_file_inventory.json", inventory)
    write_json(STATUS / "state_file_readers.json", readers)
    write_json(STATUS / "state_file_writers.json", writers)
    write_json(STATUS / "state_file_mutation_paths.json", mutations)
    write_json(STATUS / "state_write_guarantees.json", write_guarantees)
    write_json(STATUS / "state_recovery_guarantees.json", recovery_guarantees)
    write_json(STATUS / "state_complexity_report.json", complexity)

    print("wrote artifacts/status/state_file_inventory.json")
    print("wrote artifacts/status/state_file_readers.json")
    print("wrote artifacts/status/state_file_writers.json")
    print("wrote artifacts/status/state_file_mutation_paths.json")
    print("wrote artifacts/status/state_write_guarantees.json")
    print("wrote artifacts/status/state_recovery_guarantees.json")
    print("wrote artifacts/status/state_complexity_report.json")


if __name__ == "__main__":
    main()
