#![forbid(unsafe_code)]
//! Guardrails for repository-owned plugin templates.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_cli::contracts::{known_bijux_tool_namespaces, Namespace, PluginKind, PluginManifestV2};
use semver::{Prerelease, Version};

const SCAFFOLD_CONTRACT_VERSION: &str = "v2";

fn scaffold_compatibility_min_inclusive() -> String {
    let runtime = Version::parse(env!("CARGO_PKG_VERSION")).expect("package semver");
    let mut min = Version::new(runtime.major, runtime.minor, runtime.patch);
    if !runtime.pre.is_empty() {
        let channel = runtime.pre.as_str().split('.').next().expect("runtime prerelease channel");
        min.pre = Prerelease::new(channel).expect("prerelease channel");
    }
    min.to_string()
}

fn scaffold_compatibility_max_exclusive() -> String {
    let runtime = Version::parse(env!("CARGO_PKG_VERSION")).expect("package semver");
    if runtime.major == 0 {
        Version::new(0, runtime.minor + 1, 0).to_string()
    } else {
        Version::new(runtime.major + 1, 0, 0).to_string()
    }
}

fn previous_release_host_boundary() -> String {
    let runtime = Version::parse(env!("CARGO_PKG_VERSION")).expect("package semver");
    if runtime.major == 0 && runtime.minor > 0 {
        Version::new(0, runtime.minor - 1, 0).to_string()
    } else if runtime.major > 0 {
        Version::new(runtime.major - 1, 0, 0).to_string()
    } else {
        "0.0.0".to_string()
    }
}

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

fn render_scaffold_template(text: &str) -> String {
    let min = scaffold_compatibility_min_inclusive();
    let max = scaffold_compatibility_max_exclusive();
    text.replace("{{cookiecutter.project_name}}", "testplug")
        .replace("{{cookiecutter.project_slug}}", "testplug")
        .replace("{{cookiecutter.plugin_namespace}}", "testplug")
        .replace("{{cookiecutter.plugin_version}}", "0.1.0")
        .replace("{{cookiecutter.cli_min}}", &min)
        .replace("{{cookiecutter.cli_max}}", &max)
        .replace("{{cookiecutter.crate_name}}", "testplug_rs")
        .replace("{{cookiecutter.rust_edition}}", "2021")
}

fn expected_scaffold_reserved_namespaces() -> BTreeSet<String> {
    ["apps", "cli", "completion", "dev", "doctor", "help", "inspect", "plugins", "repl", "version"]
        .into_iter()
        .map(str::to_string)
        .chain(known_bijux_tool_namespaces().iter().map(|value| (*value).to_string()))
        .collect()
}

fn reserved_namespaces_from_hook(path: &str) -> BTreeSet<String> {
    let text = read_repo_file(path);
    let block = text
        .split("RESERVED_NAMESPACES = {")
        .nth(1)
        .and_then(|tail| tail.split("\n}\n").next())
        .expect("reserved namespace block");
    block
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_end_matches(',');
            trimmed.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')).map(str::to_string)
        })
        .collect()
}

fn assert_rendered_project_readme(path: &str) {
    let rendered = render_scaffold_template(&read_repo_file(path));
    assert!(
        rendered.contains("bijux plugins install ."),
        "{path} should document local install from the rendered project root"
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
        rendered.contains("bijux testplug --help"),
        "{path} should document routed execution for the rendered namespace"
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
fn scaffold_docs_reference_current_rendering_and_install_flow() {
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
        assert!(
            text.contains("bijux plugins install ./my-plugin"),
            "{path} should document directory-root install flow"
        );
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
    let rust_readme = read_repo_file("templates/plugins-rs/README.md");
    assert!(
        rust_readme.contains("plugin-entrypoint"),
        "rust template docs must describe the executable entrypoint"
    );
    assert!(
        rust_readme.to_ascii_lowercase().contains("cargo"),
        "rust template docs must describe the cargo-backed execution model"
    );
}

#[test]
fn rendered_project_readmes_describe_current_plugin_maintenance_flow() {
    assert_rendered_project_readme(
        "templates/plugins-py/{{cookiecutter.plugin_namespace}}/README.md",
    );
    assert_rendered_project_readme(
        "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/README.md",
    );
    let rust_rendered = render_scaffold_template(&read_repo_file(
        "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/README.md",
    ));
    assert!(
        rust_rendered.contains("./plugin-entrypoint --help"),
        "rendered rust project README must document the executable entrypoint"
    );
    assert!(
        rust_rendered.to_ascii_lowercase().contains("cargo"),
        "rendered rust project README must describe the cargo-backed runtime"
    );
    assert!(
        rust_rendered.to_ascii_lowercase().contains("rebuild"),
        "rendered rust project README must explain the local rebuild contract"
    );
    let python_rendered = render_scaffold_template(&read_repo_file(
        "templates/plugins-py/{{cookiecutter.plugin_namespace}}/README.md",
    ));
    assert!(
        python_rendered.contains("Python 3.11 or newer"),
        "rendered python project README must document the Python runtime floor"
    );
}

#[test]
fn scaffold_manifests_match_current_plugin_contract() {
    for (path, expected_kind) in [
        (
            "templates/plugins-py/{{cookiecutter.plugin_namespace}}/plugin.manifest.json",
            PluginKind::Python,
        ),
        (
            "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/plugin.manifest.json",
            PluginKind::ExternalExec,
        ),
    ] {
        let manifest: PluginManifestV2 =
            serde_json::from_str(&render_scaffold_template(&read_repo_file(path)))
                .expect("valid manifest json");
        assert_eq!(manifest.schema_version, "v2");
        assert_eq!(manifest.manifest_version, "v2");
        assert_eq!(manifest.kind, expected_kind);
        let expected_entrypoint = if expected_kind == PluginKind::ExternalExec {
            "plugin-entrypoint"
        } else {
            "plugin:main"
        };
        assert_eq!(manifest.entrypoint, expected_entrypoint);
        assert!(manifest.aliases.is_empty());
        assert!(manifest.capabilities.is_empty());
        assert_eq!(manifest.namespace, Namespace::new("testplug").expect("valid namespace"));
        let previous_host = previous_release_host_boundary();
        assert!(
            !manifest.compatibility.supports_host(&previous_host).expect("valid compatibility"),
            "{path} must not claim support for the previous stable host line"
        );
        assert!(
            manifest
                .compatibility
                .supports_host(&scaffold_compatibility_min_inclusive())
                .expect("valid compatibility"),
            "{path} should support the current repository host floor"
        );
    }

    let rust_entrypoint = render_scaffold_template(&read_repo_file(
        "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/plugin-entrypoint",
    ));
    assert!(
        rust_entrypoint.contains("cargo build --quiet --locked"),
        "rust template entrypoint should build the binary when needed"
    );
    assert!(
        rust_entrypoint.contains("cargo generate-lockfile"),
        "rust template entrypoint should materialize Cargo.lock before the first locked build"
    );
    assert!(
        !rust_entrypoint.contains("cargo run --quiet"),
        "rust template entrypoint should not route every execution through cargo run"
    );

    let rust_cargo_toml = render_scaffold_template(&read_repo_file(
        "templates/plugins-rs/{{cookiecutter.plugin_namespace}}/Cargo.toml",
    ));
    assert!(
        rust_cargo_toml.contains("name = \"testplug\""),
        "rust template Cargo.toml should align package and binary names with plugin_namespace"
    );
    assert!(
        rust_cargo_toml.contains("[lib]\nname = \"testplug_rs\""),
        "rust template Cargo.toml should keep crate_name for the Rust library identifier"
    );
    assert!(
        rust_cargo_toml.contains("\n[workspace]\n"),
        "rust template must remain standalone when rendered below another Cargo workspace"
    );
}

#[test]
fn scaffold_defaults_preserve_plugin_semver_and_host_compatibility_window() {
    for path in ["templates/plugins-py/cookiecutter.json", "templates/plugins-rs/cookiecutter.json"]
    {
        let payload: serde_json::Value =
            serde_json::from_str(&read_repo_file(path)).expect("valid cookiecutter json");
        assert_eq!(
            payload["plugin_version"], "0.1.0",
            "{path} must keep new plugin semver independent from the Bijux host release line"
        );
        assert_eq!(
            payload["cli_min"],
            scaffold_compatibility_min_inclusive(),
            "{path} must require the current repository host line"
        );
        assert_eq!(
            payload["cli_max"],
            scaffold_compatibility_max_exclusive(),
            "{path} must keep host compatibility open through the next supported host boundary"
        );
        assert_eq!(
            payload["_template_version"], SCAFFOLD_CONTRACT_VERSION,
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
fn scaffold_hooks_guard_namespace_and_crate_identifier_rules() {
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
fn scaffold_hook_reserved_namespaces_match_runtime_contracts() {
    let expected = expected_scaffold_reserved_namespaces();
    for path in [
        "templates/plugins-py/hooks/pre_gen_project.py",
        "templates/plugins-rs/hooks/pre_gen_project.py",
    ] {
        assert_eq!(
            reserved_namespaces_from_hook(path),
            expected,
            "{path} must stay aligned with reserved runtime and official-product namespaces"
        );
    }
}

#[test]
fn scaffold_projects_ship_local_ignore_rules() {
    let py_ignore =
        read_repo_file("templates/plugins-py/{{cookiecutter.plugin_namespace}}/.gitignore");
    assert!(
        py_ignore.contains("__pycache__/"),
        "python template should ignore interpreter cache directories"
    );
    assert!(
        py_ignore.contains(".venv/"),
        "python template should ignore local virtual environments"
    );

    let rust_ignore =
        read_repo_file("templates/plugins-rs/{{cookiecutter.plugin_namespace}}/.gitignore");
    assert!(
        rust_ignore.contains("/target/"),
        "rust template should ignore the local Cargo target directory"
    );
}

#[test]
fn scaffold_tree_does_not_ship_legacy_plugin_json_files() {
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
fn scaffold_tree_does_not_ship_empty_directories() {
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
