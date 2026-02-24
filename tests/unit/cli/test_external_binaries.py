# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

from pathlib import Path
import stat

import pytest

from bijux_cli.cli.external_binaries import ExternalBinaryCommand, run_external


def test_run_external_rejects_non_basename(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BIJUXCLI_ALLOWED_PRODUCT_BINS", "bijux-atlas")
    with pytest.raises(SystemExit) as exc:
        run_external(
            ExternalBinaryCommand(
                bin_name="./bijux-atlas",
                description="runtime",
                allowlist_key="atlas",
            ),
            [],
        )
    assert exc.value.code == 2


def test_run_external_rejects_when_not_allowed(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("BIJUXCLI_ALLOWED_PRODUCT_BINS", raising=False)
    monkeypatch.delenv("BIJUX_DEV_MODE", raising=False)

    with pytest.raises(SystemExit) as exc:
        run_external(
            ExternalBinaryCommand(
                bin_name="bijux-atlas",
                description="runtime",
                allowlist_key="atlas",
            ),
            [],
        )
    assert exc.value.code == 2


def test_run_external_passes_args_unchanged(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    bin_path = tmp_path / "bijux-atlas"
    bin_path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    bin_path.chmod(bin_path.stat().st_mode | stat.S_IXUSR)

    monkeypatch.setenv("BIJUXCLI_ALLOWED_PRODUCT_BINS", "bijux-atlas")
    monkeypatch.setattr("bijux_cli.cli.external_binaries.shutil.which", lambda _n: str(bin_path))

    captured: dict[str, object] = {}

    class _Proc:
        returncode = 0

    def _fake_run(argv: list[str], check: bool, env: dict[str, str]) -> _Proc:
        captured["argv"] = argv
        captured["check"] = check
        captured["env"] = env
        return _Proc()

    monkeypatch.setattr("bijux_cli.cli.external_binaries.subprocess.run", _fake_run)

    code = run_external(
        ExternalBinaryCommand(
            bin_name="bijux-atlas",
            description="runtime",
            allowlist_key="atlas",
        ),
        ["check", "run", "--group", "ops"],
    )

    assert code == 0
    assert captured["argv"] == [str(bin_path), "check", "run", "--group", "ops"]
    assert captured["check"] is False
