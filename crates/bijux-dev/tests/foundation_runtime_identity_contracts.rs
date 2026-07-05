use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn runtime_build_stamp_boundary_stays_build_time_only() {
    let build_script = read_repo_file("crates/bijux-dag-runtime/build.rs");
    let runtime_lib = read_repo_file("crates/bijux-dag-runtime/src/lib.rs");
    let release_make = read_repo_file("makes/rust.mk");
    let runtime_readme = read_repo_file("crates/bijux-dag-runtime/README.md");
    let release_ops = read_repo_file("docs/bijux-dag/operations/release-and-versioning.md");

    assert!(
        build_script.contains("println!(\"cargo:rerun-if-env-changed={BUILD_GIT_SHA_ENV}\");"),
        "runtime build script must honor explicit build-sha injection"
    );
    assert!(
        build_script.contains("BUILD_GIT_SHA_ENV"),
        "runtime build script must share a durable build-sha env name"
    );
    assert!(
        build_script.contains("Command::new(\"git\")"),
        "runtime build script may capture git only at build time"
    );
    assert!(
        runtime_lib.contains("option_env!(\"BIJUX_DAG_BUILD_GIT_SHA\")"),
        "runtime library must read build-stamped git sha from compile-time env only"
    );
    assert!(
        !runtime_lib.contains("Command::new(\"git\")"),
        "runtime library must not shell out to git at execution time"
    );

    for required_snippet in [
        "RS_BUILD_GIT_SHA ?=",
        "RS_BUILD_GIT_SHA_ENV ?=",
        "$(RS_BUILD_GIT_SHA_ENV) CARGO_TARGET_DIR=\"$(RS_TARGET_DIR)\" cargo build --release",
        "cargo publish -p \"$${package}\" --dry-run --locked",
    ] {
        assert!(
            release_make.contains(required_snippet),
            "release makeflow must preserve build-stamped runtime identity: missing `{required_snippet}`"
        );
    }
    assert!(
        release_make.contains("$(RS_BUILD_GIT_SHA_ENV) \\"),
        "release-tree cargo flows must pass the explicit runtime build stamp"
    );

    for text in [runtime_readme, release_ops] {
        assert!(
            text.contains("BIJUX_DAG_BUILD_GIT_SHA"),
            "runtime identity docs must mention the release-tree build stamp"
        );
    }
}
