use std::fs;
use std::path::Path;

#[test]
fn runtime_has_no_cli_deps() {
    let path = Path::new("crates/bijux_dag_runtime/Cargo.toml");
    let content = fs::read_to_string(path).unwrap();
    assert!(
        !content.contains("bijux_dag_app"),
        "runtime must not depend on bijux_dag_app"
    );
    assert!(
        !content.contains("bijux_cli"),
        "runtime must not depend on bijux_cli"
    );
}
