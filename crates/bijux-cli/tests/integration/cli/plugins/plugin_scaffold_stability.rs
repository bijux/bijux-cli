#![forbid(unsafe_code)]
//! Plugin scaffold stability checks for scaffolded surfaces and diagnostics rendering.
//! test_type: plugin-scaffold-stability

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

use super::{current_plugin_host_ceiling, current_plugin_host_floor};

fn temp_dir(label: &str) -> PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-plugin-scaffold-stability-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn run(args: &[&str], plugins_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .env("BIJUXCLI_PLUGINS_DIR", plugins_dir)
        .output()
        .expect("execute")
}

fn run_ok_json(args: &[&str], plugins_dir: &Path) -> Value {
    let out = run(args, plugins_dir);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).expect("json")
}

#[test]
fn fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable() {
    let root = temp_dir("option-template");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");

    let cases = [("python", "alpha-fuzz"), ("rust", "beta-fuzz"), ("python", "gamma-fuzz")];

    for (kind, namespace) in cases {
        let target = root.join(format!("{namespace}-{kind}"));
        let out = run_ok_json(
            &[
                "cli",
                "plugins",
                "scaffold",
                kind,
                namespace,
                "--path",
                target.to_str().expect("utf-8"),
            ],
            &plugins,
        );
        assert_eq!(out["status"], "scaffolded");
        let manifest_path = target.join("plugin.manifest.json");
        let text = fs::read_to_string(&manifest_path).expect("manifest");
        assert!(text.contains(&format!("\"namespace\": \"{namespace}\"")));
    }
}

#[test]
fn fuzz_python_and_rust_scaffold_manifest_generation_are_correct() {
    let root = temp_dir("kind-manifest");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");

    let py_dir = root.join("python-plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "py-fuzz",
            "--path",
            py_dir.to_str().expect("utf-8"),
        ],
        &plugins,
    );
    let py_manifest: Value = serde_json::from_str(
        &fs::read_to_string(py_dir.join("plugin.manifest.json")).expect("py manifest"),
    )
    .expect("parse py manifest");
    assert_eq!(py_manifest["kind"], "python");
    assert_eq!(py_manifest["entrypoint"], "plugin:main");
    assert_eq!(py_manifest["version"], "0.1.0");
    assert_eq!(py_manifest["compatibility"]["min_inclusive"], current_plugin_host_floor());
    assert_eq!(py_manifest["compatibility"]["max_exclusive"], current_plugin_host_ceiling());

    let rs_dir = root.join("rust-plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "rust",
            "rs-fuzz",
            "--path",
            rs_dir.to_str().expect("utf-8"),
        ],
        &plugins,
    );
    let rs_manifest: Value = serde_json::from_str(
        &fs::read_to_string(rs_dir.join("plugin.manifest.json")).expect("rs manifest"),
    )
    .expect("parse rs manifest");
    assert_eq!(rs_manifest["kind"], "external-exec");
    assert_eq!(rs_manifest["entrypoint"], "plugin-entrypoint");
    assert_eq!(rs_manifest["version"], "0.1.0");
    assert_eq!(rs_manifest["compatibility"]["min_inclusive"], current_plugin_host_floor());
    assert_eq!(rs_manifest["compatibility"]["max_exclusive"], current_plugin_host_ceiling());
    let cargo_manifest =
        fs::read_to_string(rs_dir.join("Cargo.toml")).expect("read rust Cargo manifest");
    assert!(
        cargo_manifest.contains("\n[workspace]\n"),
        "rust scaffold must remain standalone when generated below another Cargo workspace"
    );
    assert!(rs_dir.join("plugin-entrypoint").exists(), "rust scaffold must emit an entrypoint");
    assert!(rs_dir.join("src/main.rs").exists(), "rust scaffold must emit a Rust binary");
    let entrypoint =
        fs::read_to_string(rs_dir.join("plugin-entrypoint")).expect("read rust entrypoint");
    assert!(
        entrypoint.contains("cargo build --quiet --locked"),
        "rust scaffold entrypoint should build before execution when needed"
    );
    assert!(
        entrypoint.contains("cargo generate-lockfile"),
        "rust scaffold entrypoint should create Cargo.lock before the first locked build"
    );
    assert!(
        !entrypoint.contains("cargo run --quiet"),
        "rust scaffold entrypoint should not route every execution through cargo run"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(rs_dir.join("plugin-entrypoint"))
            .expect("entrypoint metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "rust scaffold entrypoint must be executable");
    }
}

#[test]
fn fuzz_scaffold_path_sanitization_rejects_parent_segments() {
    let root = temp_dir("path-sanitize");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");

    let bad_path = root.join("ok").join("..").join("escape");
    let out = run(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "safe-fuzz",
            "--path",
            bad_path.to_str().expect("utf-8"),
        ],
        &plugins,
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unsafe"));
}

#[test]
fn fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable() {
    let root = temp_dir("inspect-check");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");

    let scaffold_dir = root.join("inspector");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "inspector-fuzz",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins,
    );

    let install = run(
        &[
            "cli",
            "plugins",
            "install",
            scaffold_dir.join("plugin.manifest.json").to_str().expect("utf-8"),
        ],
        &plugins,
    );
    assert_eq!(install.status.code(), Some(0));

    let inspect_a = run(&["cli", "plugins", "inspect"], &plugins);
    let inspect_b = run(&["cli", "plugins", "inspect"], &plugins);
    assert_eq!(inspect_a.status.code(), Some(0));
    assert_eq!(inspect_a.stdout, inspect_b.stdout);

    let ext_entrypoint = root.join("extcheck.sh");
    fs::write(&ext_entrypoint, "#!/bin/sh\necho ok\n").expect("write entrypoint");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&ext_entrypoint, fs::Permissions::from_mode(0o755))
            .expect("set executable");
    }

    let ext_manifest = root.join("external.manifest.json");
    fs::write(
        &ext_manifest,
        format!(
            r#"{{
  "name": "extcheck",
  "version": "1.0.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "extcheck",
  "kind": "external-exec",
  "trust_class": "community",
  "aliases": ["extcheck-run"],
  "entrypoint": "{}",
  "capabilities": [{{"name":"exec","version":"1"}}]
}}"#,
            ext_entrypoint.to_string_lossy()
        ),
    )
    .expect("write manifest");
    let install_ext =
        run(&["cli", "plugins", "install", ext_manifest.to_str().expect("utf-8")], &plugins);
    assert_eq!(install_ext.status.code(), Some(0));

    fs::remove_file(&ext_entrypoint).expect("remove entrypoint");

    let check = run(&["cli", "plugins", "check", "extcheck"], &plugins);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("entrypoint"));
}

#[test]
fn fuzz_plugin_reserved_name_error_rendering_is_stable() {
    let root = temp_dir("reserved-name");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");

    for reserved in ["cli", "help", "plugins", "atlas"] {
        let target = root.join(format!("reserved-{reserved}"));
        let out = run(
            &[
                "cli",
                "plugins",
                "scaffold",
                "python",
                reserved,
                "--path",
                target.to_str().expect("utf-8"),
            ],
            &plugins,
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("reserved"));
    }
}
