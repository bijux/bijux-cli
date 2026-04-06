from __future__ import annotations

import shutil
from pathlib import Path

CONTRACT_ASSETS = [
    "schemas/output-envelope-v1.schema.json",
    "schemas/error-envelope-v1.schema.json",
    "schemas/plugin-manifest-v2.schema.json",
    "official_product_namespace_registry.json",
    "product_mount_metadata_contract.json",
]


def on_post_build(config, **kwargs) -> None:
    workspace_root = Path(__file__).resolve().parents[2]
    source_root = workspace_root / "contracts"

    site_dir = Path(config["site_dir"])
    if not site_dir.is_absolute():
        site_dir = (workspace_root / site_dir).resolve()
    target_root = site_dir / "contracts"

    for rel_path in CONTRACT_ASSETS:
        source = source_root / rel_path
        target = target_root / rel_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
