#![forbid(unsafe_code)]
//! Guardrails for repository-owned plugin templates.

use std::fs;
use std::path::{Path, PathBuf};

use bijux_cli::contracts::{Namespace, PluginKind, PluginManifestV2};

const TEMPLATE_COMPATIBILITY_MIN_INCLUSIVE: &str = "0.2.1-dev";
const TEMPLATE_COMPATIBILITY_MAX_EXCLUSIVE: &str = "1.0.0";
const TEMPLATE_CONTRACT_VERSION: &str = "v2";

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

fn walk_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        dirs.push(dir.clone());
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    dirs.sort();
    dirs
}

fn render_template(text: &str) -> String {
    text.replace("{{cookiecutter.project_name}}", "testplug")
        .replace("{{cookiecutter.project_slug}}", "testplug")
        .replace("{{cookiecutter.plugin_namespace}}", "testplug")
        .replace("{{cookiecutter.plugin_version}}", "0.1.0")
        .replace("{{cookiecutter.cli_min}}", TEMPLATE_COMPATIBILITY_MIN_INCLUSIVE)
        .replace("{{cookiecutter.cli_max}}", TEMPLATE_COMPATIBILITY_MAX_EXCLUSIVE)
        .replace("{{cookiecutter.crate_name}}", "testplug_rs")
        .replace("{{cookiecutter.rust_edition}}", "2021")
}

fn assert_rendered_project_readme(path: &str) {
    let rendered = render_template(&read_repo_file(path));
    assert!(
        rendered.contains("bijux plugins install ./plugin.manifest.json"),
        "{path} should document local install with the current manifest contract"
    );
    assert!(
        rendered.contains("bijux plugins list"),
        "{path} should document list verification after install"
    );
    assert!(
        rendered.contains("bijux plugins inspect testplug"),
        "{path} should document targeted inspection for the rendered namespace"
    );
    assert!(
        rendered.contains("bijux plugins check testplug"),
        "{path} should document health checks for the rendered namespace"
    );
    assert!(
        rendered.contains("bijux plugins explain testplug"),
        "{path} should document diagnostic explanation for the rendered namespace"
    );
    assert!(
        rendered.contains("bijux plugins schema"),
        "{path} should document schema discovery for the manifest contract"
    );
    assert!(
        rendered.contains("compatibility range"),
        "{path} should explain compatibility maintenance"
    );
    assert!(
        rendered.to_ascii_lowercase().contains("reserved"),
        "{path} should warn about reserved Bijux namespaces"
    );
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
        for required in
            ["plugins inspect", "plugins explain", "plugins schema", "reserved", "compatibility"]
        {
            assert!(
                text.to_ascii_lowercase().contains(required),
                "{path} should document current plugin guidance: {required}"
            );
        }
        for forbidden in [
            "--template",
            "plugin.json",
            "list-plugins",
            "info <name|path>",
            "check <name|path>",
            "--source local",
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
fn rendered_project_readmes_describe_current_plugin_maintenance_flow() {
    assert_rendered_project_readme(
        "templates/plugins-py/{{cookiecutter.plugin_namespace}}/README.md",
    );
    assert_rendered_project_readme(
        "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/README.md",
    );
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
        let manifest: PluginManifestV2 =
            serde_json::from_str(&render_template(&read_repo_file(path)))
                .expect("valid manifest json");
        assert_eq!(manifest.schema_version, "v2");
        assert_eq!(manifest.manifest_version, "v2");
        assert_eq!(manifest.kind, expected_kind);
        assert_eq!(manifest.entrypoint, "plugin:main");
        assert!(manifest.aliases.is_empty());
        assert!(manifest.capabilities.is_empty());
        assert_eq!(manifest.namespace, Namespace::new("testplug").expect("valid namespace"));
        assert!(
            !manifest.compatibility.supports_host("0.2.0").expect("valid compatibility"),
            "{path} must not claim support for the published 0.2.0 host"
        );
        assert!(
            manifest
                .compatibility
                .supports_host(TEMPLATE_COMPATIBILITY_MIN_INCLUSIVE)
                .expect("valid compatibility"),
            "{path} should support the current post-0.2.0 host floor"
        );
    }
}

#[test]
fn template_default_release_window_matches_planned_plugin_publish_range() {
    for path in ["templates/plugins-py/cookiecutter.json", "templates/plugins-rs/cookiecutter.json"]
    {
        let payload: serde_json::Value =
            serde_json::from_str(&read_repo_file(path)).expect("valid cookiecutter json");
        assert_eq!(payload["plugin_version"], "0.1.0", "{path} must default new plugins to 0.1.0");
        assert_eq!(
            payload["cli_min"], TEMPLATE_COMPATIBILITY_MIN_INCLUSIVE,
            "{path} must require the first post-0.2.0 Bijux host line"
        );
        assert_eq!(
            payload["cli_max"], TEMPLATE_COMPATIBILITY_MAX_EXCLUSIVE,
            "{path} must keep future compatibility open until the 1.0.0 boundary"
        );
        assert_eq!(
            payload["_template_version"], TEMPLATE_CONTRACT_VERSION,
            "{path} must track the current template contract version"
        );

        let project_slug = payload["project_slug"].as_str().expect("project slug template");
        assert!(
            project_slug.contains("replace('--', '-')"),
            "{path} should collapse repeated hyphens in derived project_slug defaults"
        );
        assert!(
            project_slug.contains("strip('-')"),
            "{path} should trim unstable leading or trailing hyphens from project_slug defaults"
        );
        if path.ends_with("plugins-rs/cookiecutter.json") {
            let crate_name = payload["crate_name"].as_str().expect("crate name template");
            assert!(
                crate_name.contains("replace('__', '_')"),
                "{path} should collapse repeated underscores in crate_name defaults"
            );
            assert!(
                crate_name.contains("strip('_')"),
                "{path} should trim unstable leading or trailing underscores from crate_name defaults"
            );
        }
    }
}

#[test]
fn template_hooks_guard_namespace_and_crate_identifier_rules() {
    let py_hook = read_repo_file("templates/plugins-py/hooks/pre_gen_project.py");
    assert!(
        py_hook.contains("project_slug must be lowercase kebab-case"),
        "python template hook should reject unstable project slugs"
    );
    assert!(
        py_hook.contains("plugin_namespace must be lowercase kebab-case"),
        "python template hook should reject invalid plugin namespaces"
    );
    assert!(
        py_hook.contains("plugin_namespace is reserved by bijux-cli or an official Bijux tool"),
        "python template hook should reject reserved plugin namespaces"
    );
    for required in [
        "parse_semver(\"plugin_version\"",
        "parse_semver(\"cli_min\"",
        "parse_semver(\"cli_max\"",
        "cli_max must be greater than cli_min",
    ] {
        assert!(
            py_hook.contains(required),
            "python template hook should validate current release window inputs: {required}"
        );
    }

    let rs_hook = read_repo_file("templates/plugins-rs/hooks/pre_gen_project.py");
    assert!(
        rs_hook.contains("project_slug must be lowercase kebab-case"),
        "rust template hook should reject unstable project slugs"
    );
    assert!(
        rs_hook.contains("plugin_namespace must be lowercase kebab-case"),
        "rust template hook should reject invalid plugin namespaces"
    );
    assert!(
        rs_hook.contains("plugin_namespace is reserved by bijux-cli or an official Bijux tool"),
        "rust template hook should reject reserved plugin namespaces"
    );
    assert!(
        rs_hook.contains("crate_name must be lowercase snake_case"),
        "rust template hook should reject invalid crate identifiers"
    );
    assert!(
        rs_hook.contains("crate_name must not use a reserved Rust keyword"),
        "rust template hook should reject reserved Rust keywords"
    );
    for required in [
        "parse_semver(\"plugin_version\"",
        "parse_semver(\"cli_min\"",
        "parse_semver(\"cli_max\"",
        "cli_max must be greater than cli_min",
    ] {
        assert!(
            rs_hook.contains(required),
            "rust template hook should validate current release window inputs: {required}"
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

#[test]
fn template_tree_does_not_ship_empty_directories() {
    let template_root = repo_root().join("templates");
    for path in walk_dirs(&template_root) {
        if path == template_root {
            continue;
        }
        let mut entries = fs::read_dir(&path).expect("read dir");
        assert!(
            entries.next().is_some(),
            "template tree still contains empty directory: {}",
            path.display()
        );
    }
}
