# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Property tests for LogLevel ordering and parsing."""

from __future__ import annotations

from hypothesis import given
from hypothesis import strategies as st

from bijux_cli.core.enums import LogLevel
from bijux_cli.core.precedence import _log_rank


@st.composite
def _level_triplets(draw: st.DrawFn) -> tuple[LogLevel, LogLevel, LogLevel]:
    levels = list(LogLevel)
    return (
        draw(st.sampled_from(levels)),
        draw(st.sampled_from(levels)),
        draw(st.sampled_from(levels)),
    )


@given(_level_triplets())
def test_log_level_rank_transitivity(
    levels: tuple[LogLevel, LogLevel, LogLevel],
) -> None:
    a, b, c = levels
    if _log_rank(a) <= _log_rank(b) and _log_rank(b) <= _log_rank(c):
        assert _log_rank(a) <= _log_rank(c)


@given(st.sampled_from(list(LogLevel)), st.sampled_from(list(LogLevel)))
def test_log_level_total_ordering(a: LogLevel, b: LogLevel) -> None:
    assert _log_rank(a) <= _log_rank(b) or _log_rank(b) <= _log_rank(a)


@given(st.sampled_from(list(LogLevel)))
def test_log_level_case_insensitive_parsing(level: LogLevel) -> None:
    value = level.value
    mixed = "".join(
        ch.upper() if idx % 2 else ch.lower() for idx, ch in enumerate(value)
    )
    assert LogLevel(mixed) is level


def test_log_level_strict_ordering() -> None:
    ordered = [
        LogLevel.DEBUG,
        LogLevel.INFO,
        LogLevel.WARNING,
        LogLevel.ERROR,
        LogLevel.CRITICAL,
    ]
    ranks = [_log_rank(level) for level in ordered]
    assert ranks == sorted(ranks)
    assert len(set(ranks)) == len(ranks)
