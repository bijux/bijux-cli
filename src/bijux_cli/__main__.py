# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Module entrypoint for `python -m bijux_cli`."""

from __future__ import annotations

import sys

from bijux_cli.app.bootstrap import main


if __name__ == "__main__":
    sys.exit(main())
