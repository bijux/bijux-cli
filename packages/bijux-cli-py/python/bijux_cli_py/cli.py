"""Console script entrypoint for `bijux`."""

from __future__ import annotations

import sys

from ._facade import execution_facade


def main() -> int:
    output = execution_facade(sys.argv[1:])
    sys.stdout.write(output)
    return 0
