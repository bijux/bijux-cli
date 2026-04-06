from __future__ import annotations

import subprocess
from pathlib import Path


def on_post_build(config, **kwargs) -> None:
    workspace_root = Path(__file__).resolve().parents[2]
    site_dir = Path(config["site_dir"])
    if not site_dir.is_absolute():
        site_dir = (workspace_root / site_dir).resolve()

    result = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "bijux-dev",
            "--",
            "docs",
            "publish-contract-assets",
            "--site-dir",
            str(site_dir),
        ],
        cwd=workspace_root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode == 0:
        return

    stderr = result.stderr.strip() or result.stdout.strip() or "unknown error"
    raise RuntimeError(f"failed to publish contract assets: {stderr}")
