from __future__ import annotations


def main(argv: list[str]) -> dict[str, object]:
    return {
        "status": "ok",
        "argv": argv,
        "namespace": "{{cookiecutter.plugin_namespace}}",
    }
