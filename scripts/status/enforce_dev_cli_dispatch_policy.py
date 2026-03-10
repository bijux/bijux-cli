#!/usr/bin/env python3
"""Enforce dev-cli dispatch ownership and bin entrypoint-only policy."""

from __future__ import annotations

import json
from pathlib import Path


REPORT_PATH = Path("artifacts/status/dev_cli_dispatch_ownership_report.json")
BIN_DIFF_PATH = Path("artifacts/status/bin_entrypoint_responsibility_diff.json")


def _load(path: Path) -> dict:
    if not path.exists():
        raise SystemExit(f"missing artifact: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    dispatch = _load(REPORT_PATH)
    bin_diff = _load(BIN_DIFF_PATH)
    failures: list[str] = []

    checks = dispatch.get("checks", {})
    if checks.get("bin_mentions_dev_cli_literals") is not False:
        failures.append("bin entrypoint must not contain dev cli command literals")
    if checks.get("bin_has_direct_dispatch_match_arms") is not False:
        failures.append("bin entrypoint must not contain direct dispatch match arms")
    if int(checks.get("core_dev_cli_dispatch_arm_count", 0)) < 10:
        failures.append("core must expose explicit dev cli dispatch arm coverage")
    if int(checks.get("core_dev_cli_builder_call_count", 0)) < 10:
        failures.append("core must delegate dev cli reports to bijux-dev-cli builders")

    current = bin_diff.get("current", {})
    if int(current.get("dev_cli_literal_mentions", 0)) != 0:
        failures.append("bin responsibility diff reports dev cli literals in main.rs")
    if int(current.get("core_run_app_calls", 0)) == 0:
        failures.append("bin entrypoint must call core run_app")
    if int(current.get("direct_dispatch_match_mentions", 0)) != 0:
        failures.append("bin entrypoint must not have route dispatch matches")
    if int(current.get("parser_dependency_mentions", 0)) != 0:
        failures.append("bin entrypoint must not consume routing parser directly")

    routing = bin_diff.get("routing_identity_checks", {})
    if int(routing.get("parser_build_report_mentions", 0)) != 0:
        failures.append("routing parser must not assemble maintainer reports")
    if int(routing.get("registry_build_report_mentions", 0)) != 0:
        failures.append("routing registry must not assemble maintainer reports")
    if int(routing.get("parser_json_assembly_mentions", 0)) != 0:
        failures.append("routing parser must not shape maintainer json payloads")
    if int(routing.get("registry_json_assembly_mentions", 0)) != 0:
        failures.append("routing registry must not shape maintainer json payloads")

    if failures:
        for failure in failures:
            print(f"dispatch-policy-failure: {failure}")
        return 1

    print("dev cli dispatch ownership policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

