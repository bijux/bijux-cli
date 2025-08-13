# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services doctor module."""

from __future__ import annotations

import pytest

from bijux_cli.contracts import DoctorProtocol
from bijux_cli.services.doctor import Doctor


def test_doctor_implements_protocol() -> None:
    """Test that the Doctor class implements the DoctorProtocol."""
    doctor = Doctor()
    assert isinstance(doctor, DoctorProtocol)


def test_check_health_returns_healthy() -> None:
    """Test that the check_health method returns the string 'healthy'."""
    doctor = Doctor()
    assert doctor.check_health() == "healthy"


def test_doctor_init_no_args() -> None:
    """Test that the Doctor class can be initialized without arguments."""
    doctor = Doctor()
    assert doctor is not None


@pytest.mark.parametrize("call_count", range(10))
def test_check_health_multiple_calls(call_count: int) -> None:
    """Test that check_health is consistent over multiple calls."""
    doctor = Doctor()
    for _ in range(call_count + 1):
        assert doctor.check_health() == "healthy"


def test_doctor_in_all() -> None:
    """Test that 'Doctor' is in the module's __all__ export list."""
    from bijux_cli.services.doctor import __all__

    assert "Doctor" in __all__


def test_doctor_no_side_effects() -> None:
    """Test that check_health has no side effects between calls."""
    doctor = Doctor()
    result1 = doctor.check_health()
    result2 = doctor.check_health()
    assert result1 == result2 == "healthy"


def test_doctor_subclass() -> None:
    """Test that a subclass of Doctor inherits the check_health behavior."""

    class SubDoctor(Doctor):
        """A test subclass of Doctor."""

    sub = SubDoctor()
    assert sub.check_health() == "healthy"


def test_check_health_type() -> None:
    """Test that the check_health method returns a string."""
    doctor = Doctor()
    assert isinstance(doctor.check_health(), str)


@pytest.mark.parametrize("instance", [Doctor() for _ in range(5)])
def test_multiple_instances(instance: Doctor) -> None:
    """Test that multiple instances of Doctor behave correctly."""
    assert instance.check_health() == "healthy"


def test_doctor_docstring() -> None:
    """Test that the Doctor class has a docstring."""
    assert Doctor.__doc__ is not None


def test_check_health_docstring() -> None:
    """Test that the check_health method has a docstring."""
    assert Doctor.check_health.__doc__ is not None
