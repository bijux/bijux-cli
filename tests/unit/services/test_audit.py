# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services audit module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

from unittest.mock import Mock, call, patch

import pytest

from bijux_cli.core.exceptions import BijuxError
from bijux_cli.services.audit import (
    DryRunAudit,
    RealAudit,
    _BaseAudit,
    get_audit_service,
)


@pytest.fixture
def mock_log() -> Mock:
    """Provide a mock logging object."""
    return Mock()


@pytest.fixture
def mock_tel() -> Mock:
    """Provide a mock telemetry object."""
    return Mock()


def test_base_shutdown(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the base audit shutdown calls flush and close."""
    audit = _BaseAudit(mock_log, mock_tel)
    audit.shutdown()
    mock_tel.flush.assert_called_once()
    mock_log.close.assert_called_once()


def test_base_shutdown_exceptions(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that exceptions during shutdown are suppressed."""
    with (
        patch.object(mock_tel, "flush", side_effect=Exception("fail")),
        patch.object(mock_log, "close", side_effect=Exception("fail")),
    ):
        audit = _BaseAudit(mock_log, mock_tel)
        audit.shutdown()


def test_base_get_commands(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the get_commands method of the base audit class."""
    audit = _BaseAudit(mock_log, mock_tel)
    assert not audit.get_commands()
    audit._commands = [{"a": 1}]
    assert audit.get_commands() == [{"a": 1}]


def test_base_get_status(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the get_status method of the base audit class."""
    audit = _BaseAudit(mock_log, mock_tel)
    assert audit.get_status() == {"commands_processed": 0}
    audit._commands = [1]  # type: ignore[list-item]
    assert audit.get_status() == {"commands_processed": 1}


def test_base_cli_audit(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the base cli_audit method is a no-op."""
    audit = _BaseAudit(mock_log, mock_tel)
    audit.cli_audit()


def test_dryrun_log(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the log method of the DryRunAudit class."""
    audit = DryRunAudit(mock_log, mock_tel)
    cmd = ["cmd", "arg"]
    audit.log(cmd, executor="exec")
    assert audit._commands == [{"cmd": cmd, "executor": "exec"}]
    mock_log.log.assert_called_with(
        "info", "Dry-run", extra={"cmd": cmd, "executor": "exec"}
    )
    mock_tel.event.assert_called_with("audit_dry_run", {"cmd": cmd, "executor": "exec"})


def test_dryrun_run(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the run method of the DryRunAudit class."""
    audit = DryRunAudit(mock_log, mock_tel)
    cmd = ["cmd"]
    rc, out, err = audit.run(cmd, executor="exec")
    assert (rc, out, err) == (0, b"", b"")


def test_dryrun_cli_audit(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the cli_audit method of the DryRunAudit class."""
    audit = DryRunAudit(mock_log, mock_tel)
    audit.cli_audit()
    mock_log.log.assert_called_with("info", "CLI audit (dry-run)", extra={})
    mock_tel.event.assert_called_with("audit_cli_dry_run", {})


def test_real_log(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the log method of the RealAudit class."""
    audit = RealAudit(mock_log, mock_tel)
    cmd = ["cmd", "arg"]
    audit.log(cmd, executor="exec")
    assert audit._commands == [{"cmd": cmd, "executor": "exec"}]
    mock_log.log.assert_called_with(
        "debug", "Executing exec", extra={"cmd": cmd, "executor": "exec"}
    )
    mock_tel.event.assert_called_with("audit_execute", {"cmd": cmd, "executor": "exec"})


def test_real_run_success(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the successful execution path of the RealAudit run method."""
    audit = RealAudit(mock_log, mock_tel)
    cmd = ["echo", "test"]
    safe_cmd = cmd
    mock_proc = Mock()
    mock_proc.returncode = 0
    mock_proc.stdout = b"test\n"
    mock_proc.stderr = b""
    with (
        patch("subprocess.run", return_value=mock_proc) as mock_run,
        patch("bijux_cli.services.audit.validate_command", return_value=safe_cmd),
    ):
        rc, out, err = audit.run(cmd, executor="exec")
        assert rc == 0
        assert out == b"test\n"
        assert err == b""
        mock_run.assert_called_once_with(
            safe_cmd, capture_output=True, check=False, shell=False
        )
        mock_tel.event.assert_has_calls(
            [
                call("audit_execute", {"cmd": safe_cmd, "executor": "exec"}),
                call(
                    "audit_executed",
                    {"cmd": safe_cmd, "executor": "exec", "returncode": 0},
                ),
            ]
        )


def test_real_run_validate_fail(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that a command validation failure is handled correctly."""
    audit = RealAudit(mock_log, mock_tel)
    cmd = ["bad"]
    with (
        patch(
            "bijux_cli.services.audit.validate_command",
            side_effect=BijuxError("invalid"),
        ),
        pytest.raises(BijuxError, match="invalid"),
    ):
        audit.run(cmd, executor="exec")
    mock_tel.event.assert_called_with(
        "audit_execution_failed", {"cmd": cmd, "executor": "exec", "error": "invalid"}
    )


def test_real_run_exec_fail(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that a subprocess execution failure is handled correctly."""
    audit = RealAudit(mock_log, mock_tel)
    cmd = ["cmd"]
    safe_cmd = cmd
    with (
        patch("subprocess.run", side_effect=Exception("exec fail")),
        patch("bijux_cli.services.audit.validate_command", return_value=safe_cmd),
        pytest.raises(BijuxError, match="exec fail"),
    ):
        audit.run(cmd, executor="exec")
    mock_tel.event.assert_called_with(
        "audit_execution_failed", {"cmd": cmd, "executor": "exec", "error": "exec fail"}
    )


def test_real_cli_audit(mock_log: Mock, mock_tel: Mock) -> None:
    """Test the cli_audit method of the RealAudit class."""
    audit = RealAudit(mock_log, mock_tel)
    audit._commands = [1, 2]  # type: ignore[list-item]
    audit.cli_audit()
    mock_log.log.assert_called_with("info", "CLI audit (real)", extra={"commands": 2})
    mock_tel.event.assert_called_with("audit_cli_real", {"commands": 2})


def test_get_audit_service_dry(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the audit service factory returns a DryRunAudit instance."""
    audit = get_audit_service(mock_log, mock_tel, dry_run=True)
    assert isinstance(audit, DryRunAudit)


def test_get_audit_service_real(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the audit service factory returns a RealAudit instance."""
    audit = get_audit_service(mock_log, mock_tel, dry_run=False)
    assert isinstance(audit, RealAudit)


def test_base_audit_run_notimplemented(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the base audit class's run method is not implemented."""
    audit = _BaseAudit(mock_log, mock_tel)
    with pytest.raises(
        NotImplementedError, match="Subclasses must implement 'run' method."
    ):
        audit.run(["foo"], executor="bar")


def test_base_audit_log_nop(mock_log: Mock, mock_tel: Mock) -> None:
    """Test that the base audit class's log method is a no-op."""
    audit = _BaseAudit(mock_log, mock_tel)
    audit.log(["foo"], executor="bar")
