"""Console script entrypoint for `bijux`."""

from __future__ import annotations

import sys

from ._facade import execution_facade_with_status


def main() -> int:
    result = execution_facade_with_status(sys.argv[1:])
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    return result.exit_code
