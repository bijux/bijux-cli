from __future__ import annotations


def main(argv: list[str]) -> dict[str, object]:
    return {
        "status": "ok",
        "argv": argv,
        "namespace": "{{cookiecutter.plugin_namespace}}",
        "bridge": "replace plugin.py with your Rust bridge entrypoint",
    }
