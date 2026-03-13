from __future__ import annotations

import shutil
from pathlib import Path


def on_post_build(config, **kwargs) -> None:
    workspace_root = Path(__file__).resolve().parents[2]
    source_dir = workspace_root / "contracts"
    site_dir = Path(config["site_dir"])
    destination_dir = site_dir / "contracts"

    if not source_dir.is_dir():
        raise RuntimeError(f"missing contract asset source directory: {source_dir}")

    if destination_dir.exists():
        shutil.rmtree(destination_dir)

    shutil.copytree(source_dir, destination_dir)
