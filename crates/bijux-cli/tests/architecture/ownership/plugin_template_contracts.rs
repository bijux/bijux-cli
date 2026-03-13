#![forbid(unsafe_code)]
//! Guardrails for repository-owned plugin templates.

use std::fs;
use std::path::{Path, PathBuf};

use bijux_cli::contracts::{Namespace, PluginKind, PluginManifestV1};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("template file should exist")
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn render_template(text: &str) -> String {
    text.replace("{{cookiecutter.project_name}}", "testplug")
        .replace("{{cookiecutter.plugin_namespace}}", "testplug")
        .replace("{{cookiecutter.plugin_version}}", "0.3.0")
        .replace("{{cookiecutter.cli_min}}", "0.3.0")
        .replace("{{cookiecutter.cli_max}}", "1.0.0")
        .replace("{{cookiecutter.crate_name}}", "testplug_rs")
        .replace("{{cookiecutter.rust_edition}}", "2021")
}

#[test]
fn template_docs_reference_current_rendering_and_install_flow() {
    for path in
        ["templates/README.md", "templates/plugins-py/README.md", "templates/plugins-rs/README.md"]
    {
        let text = read_repo_file(path);
        assert!(
            text.to_ascii_lowercase().contains("cookiecutter"),
            "{path} should describe cookiecutter rendering"
        );
        assert!(
            text.contains("plugin.manifest.json"),
            "{path} should reference plugin.manifest.json"
        );
        for forbidden in [
            "--template",
            "plugin.json",
            "list-plugins",
            "info <name|path>",
            "check <name|path>",
            "bijux_cli.plugins",
            "bijux_cli_version",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} still references stale surface: {forbidden}"
            );
        }
    }
}

#[test]
fn template_manifests_match_current_plugin_contract() {
    for (path, expected_kind) in [
        (
            "templates/plugins-py/{{cookiecutter.plugin_namespace}}/plugin.manifest.json",
            PluginKind::Python,
        ),
        (
            "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/plugin.manifest.json",
            PluginKind::Delegated,
        ),
    ] {
        let manifest: PluginManifestV1 =
            serde_json::from_str(&render_template(&read_repo_file(path)))
                .expect("valid manifest json");
        assert_eq!(manifest.schema_version, "v1");
        assert_eq!(manifest.manifest_version, "v1");
        assert_eq!(manifest.kind, expected_kind);
        assert_eq!(manifest.entrypoint, "plugin:main");
        assert!(manifest.aliases.is_empty());
        assert!(manifest.capabilities.is_empty());
        assert_eq!(manifest.namespace, Namespace::new("testplug").expect("valid namespace"));
        assert!(
            manifest.compatibility.supports_host("0.3.0").expect("valid compatibility"),
            "{path} should support the planned 0.3.0 release floor"
        );
    }
}

#[test]
fn template_default_release_window_matches_planned_plugin_publish_range() {
    for path in ["templates/plugins-py/cookiecutter.json", "templates/plugins-rs/cookiecutter.json"]
    {
        let payload: serde_json::Value =
            serde_json::from_str(&read_repo_file(path)).expect("valid cookiecutter json");
        assert_eq!(payload["plugin_version"], "0.3.0", "{path} must default new plugins to 0.3.0");
        assert_eq!(payload["cli_min"], "0.3.0", "{path} must require bijux-cli >=0.3.0");
        assert_eq!(
            payload["cli_max"], "1.0.0",
            "{path} must keep future compatibility open until the 1.0.0 boundary"
        );
    }
}

#[test]
fn template_tree_does_not_ship_legacy_plugin_json_files() {
    let template_root = repo_root().join("templates");
    for path in walk_files(&template_root) {
        assert_ne!(
            path.file_name().and_then(|name| name.to_str()),
            Some("plugin.json"),
            "template tree still contains legacy plugin.json: {}",
            path.display()
        );
    }
}
