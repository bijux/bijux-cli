#![forbid(unsafe_code)]
//! Config parser and serializer stability checks.
//! test_type: config-parser-stability

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn temp_dir(label: &str) -> PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-config-parser-stability-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("execute")
}

fn run_json_ok(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
    assert!(out.stderr.is_empty(), "successful json command should keep stderr empty: {args:?}");
    assert!(!out.stdout.is_empty(), "successful json command should emit stdout payload: {args:?}");
    serde_json::from_slice(&out.stdout).expect("json")
}

fn seeded_pairs(seed: u64, n: usize) -> Vec<(String, String)> {
    let mut state = seed;
    let mut out = Vec::new();
    for i in 0..n {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let k = format!("k{}_{}", i, state % 1000);
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let v = format!("v{}_{}", i, state % 100000);
        out.push((k, v));
    }
    out
}

#[test]
fn fuzz_dotenv_style_config_parsing_is_stable() {
    let root = temp_dir("dotenv");
    let path = root.join("config.env");
    fs::write(&path, "# comment\nBIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=two\nBIJUXCLI_GAMMA=3\n")
        .expect("write");

    let json =
        run_json_ok(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(json["alpha"], "1");
    assert_eq!(json["beta"], "two");
    assert_eq!(json["gamma"], "3");
}

#[test]
fn fuzz_malformed_config_lines_fail_consistently() {
    let root = temp_dir("malformed");
    let path = root.join("bad.env");

    for sample in ["BROKEN\n", "=\n", "BIJUXCLI_OK=1\nBAD\n", "BIJUXCLI_A=1\nX\nY\n"] {
        fs::write(&path, sample).expect("write");
        let a = run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
        let b = run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
        assert_eq!(a.status.code(), b.status.code());
        assert_ne!(a.status.code(), Some(0));
        assert!(a.stdout.is_empty(), "malformed listing should not write stdout");
        assert!(b.stdout.is_empty(), "malformed listing should not write stdout");
        assert!(!a.stderr.is_empty(), "malformed listing should write stderr");
        assert!(!b.stderr.is_empty(), "malformed listing should write stderr");
        assert_eq!(a.stderr, b.stderr, "malformed listing diagnostics should be deterministic");
    }
}

#[test]
fn fuzz_duplicate_key_handling_rejects_ambiguous_state() {
    let root = temp_dir("dupes");
    let path = root.join("dupes.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=2\nBIJUXCLI_ALPHA=3\n").expect("write");

    let out = run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Duplicate key"));
}

#[test]
fn fuzz_weird_whitespace_handling_is_stable() {
    let root = temp_dir("whitespace");
    let path = root.join("w.env");
    fs::write(&path, "   BIJUXCLI_ALPHA   =value   \n\tBIJUXCLI_BETA=  two words  \n# c\n")
        .expect("write");

    let json =
        run_json_ok(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(json["alpha"], "value");
    assert_eq!(json["beta"], "two words");
}

#[test]
fn fuzz_quote_parsing_and_escape_parsing_are_stable() {
    let root = temp_dir("quotes");
    let path = root.join("q.env");
    fs::write(
        &path,
        "BIJUXCLI_A=\"quoted value\"\nBIJUXCLI_B='single quoted'\nBIJUXCLI_C=\"a\\\"b\"\nBIJUXCLI_D='a\\'b'\n",
    )
    .expect("write");

    let json =
        run_json_ok(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(json["a"], "\"quoted value\"");
    assert_eq!(json["b"], "'single quoted'");
    assert_eq!(json["c"], "\"a\"b\"");
    assert_eq!(json["d"], "'a'b'");
}

#[test]
fn fuzz_null_byte_and_control_characters_are_handled_deterministically() {
    let root = temp_dir("controls");
    let path = root.join("controls.env");

    fs::write(&path, b"BIJUXCLI_A=ok\nBIJUXCLI_B=bad\x00x\n").expect("write null bytes");
    let null_case_a =
        run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    let null_case_b =
        run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(null_case_a.status.code(), null_case_b.status.code());
    assert_eq!(null_case_a.stdout, null_case_b.stdout, "null-byte stdout should be deterministic");
    assert_eq!(null_case_a.stderr, null_case_b.stderr, "null-byte stderr should be deterministic");

    fs::write(&path, "BIJUXCLI_A=ok\nBIJUXCLI_B=bad\tvalue\n").expect("write tab");
    let tab_case = run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(tab_case.status.code(), Some(3));
    assert!(tab_case.stdout.is_empty(), "tab validation failures should not write stdout");
    assert!(!tab_case.stderr.is_empty(), "tab validation failures should write stderr");
}

#[test]
fn fuzz_mixed_valid_invalid_content_never_silently_succeeds() {
    let root = temp_dir("mixed");
    let path = root.join("mixed.env");
    fs::write(&path, "BIJUXCLI_OK=1\nBROKEN\nBIJUXCLI_STILL_OK=2\n").expect("write");

    let out = run(&["cli", "config", "list", "--config-path", path.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn fuzz_config_export_serialization_roundtrips_for_random_inputs() {
    let root = temp_dir("export");
    let active = root.join("active.env");
    let exported = root.join("exported.env");

    let mut expected = BTreeMap::new();
    for (k, v) in seeded_pairs(77, 32) {
        expected.insert(k.clone(), v.clone());
        let pair = format!("{k}={v}");
        let out =
            run(&["cli", "config", "set", &pair, "--config-path", active.to_str().expect("utf-8")]);
        assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
    }

    let export = run(&[
        "cli",
        "config",
        "export",
        exported.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(export.status.code(), Some(0));

    let loaded =
        run_json_ok(&["cli", "config", "list", "--config-path", exported.to_str().expect("utf-8")]);
    for (k, v) in expected {
        assert_eq!(loaded[k], v);
    }
}

#[test]
fn fuzz_config_load_import_parsing_is_deterministic() {
    let root = temp_dir("load");
    let active = root.join("active.env");
    let source = root.join("source.env");

    let mut lines = String::new();
    for (k, v) in seeded_pairs(202, 24) {
        lines.push_str(&format!("BIJUXCLI_{}={}\n", k.to_ascii_uppercase(), v));
    }
    fs::write(&source, lines).expect("write source");

    let a = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    let b = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(a.status.code(), b.status.code());
    assert_eq!(a.stdout, b.stdout, "load stdout should be deterministic");
    assert_eq!(a.stderr, b.stderr, "load stderr should be deterministic");

    let listed =
        run_json_ok(&["cli", "config", "list", "--config-path", active.to_str().expect("utf-8")]);
    assert!(
        !listed.as_object().expect("listed object").is_empty(),
        "loaded config should contain entries"
    );
}

#[test]
fn fuzz_roundtrip_parse_serialize_parse_is_semantically_stable() {
    let root = temp_dir("roundtrip");
    let active = root.join("active.env");
    let roundtrip = root.join("roundtrip.env");

    for (k, v) in seeded_pairs(909, 16) {
        let out = run(&[
            "cli",
            "config",
            "set",
            &format!("{k}={v}"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ]);
        assert_eq!(out.status.code(), Some(0));
    }

    let before =
        run_json_ok(&["cli", "config", "list", "--config-path", active.to_str().expect("utf-8")]);
    let export = run(&[
        "cli",
        "config",
        "export",
        roundtrip.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(export.status.code(), Some(0));
    let after = run_json_ok(&[
        "cli",
        "config",
        "list",
        "--config-path",
        roundtrip.to_str().expect("utf-8"),
    ]);
    assert_eq!(before, after);
}

#[test]
fn fuzz_key_normalization_and_value_validation_are_stable() {
    let root = temp_dir("norm");
    let path = root.join("norm.env");

    let ok = run(&[
        "cli",
        "config",
        "set",
        "BIJUXCLI_MixedKey=value",
        "--config-path",
        path.to_str().expect("utf-8"),
    ]);
    assert_eq!(ok.status.code(), Some(0));
    let got = run_json_ok(&[
        "cli",
        "config",
        "get",
        "mixedkey",
        "--config-path",
        path.to_str().expect("utf-8"),
    ]);
    assert_eq!(got["value"], "value");

    let bad_key = run(&[
        "cli",
        "config",
        "set",
        "bad-key=value",
        "--config-path",
        path.to_str().expect("utf-8"),
    ]);
    assert_eq!(bad_key.status.code(), Some(2));
    assert!(bad_key.stdout.is_empty());
    assert!(!bad_key.stderr.is_empty());

    let bad_value = run(&[
        "cli",
        "config",
        "set",
        "good=bad\tvalue",
        "--config-path",
        path.to_str().expect("utf-8"),
    ]);
    assert_eq!(bad_value.status.code(), Some(3));
    assert!(bad_value.stdout.is_empty());
    assert!(!bad_value.stderr.is_empty());
}

#[test]
fn fuzz_no_silent_key_loss_invariant_holds_under_repeated_exports() {
    let root = temp_dir("no-loss");
    let active = root.join("active.env");
    let exported = root.join("export.env");

    let mut keys = Vec::new();
    for (k, v) in seeded_pairs(333, 20) {
        keys.push(k.clone());
        let out = run(&[
            "cli",
            "config",
            "set",
            &format!("{k}={v}"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ]);
        assert_eq!(out.status.code(), Some(0));
    }

    for _ in 0..3 {
        let export = run(&[
            "cli",
            "config",
            "export",
            exported.to_str().expect("utf-8"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ]);
        assert_eq!(export.status.code(), Some(0));
        let load = run(&[
            "cli",
            "config",
            "load",
            exported.to_str().expect("utf-8"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ]);
        assert_eq!(load.status.code(), Some(0));
    }

    let listed =
        run_json_ok(&["cli", "config", "list", "--config-path", active.to_str().expect("utf-8")]);
    for k in keys {
        assert!(listed.get(&k).is_some(), "missing key after repeated export/load: {k}");
    }
}
