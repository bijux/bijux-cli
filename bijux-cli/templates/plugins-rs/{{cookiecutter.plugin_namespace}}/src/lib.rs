use serde_json::{json, Value};

pub fn run(argv: &[String]) -> Value {
    json!({
        "status": "ok",
        "namespace": "{{cookiecutter.plugin_namespace}}",
        "argv": argv
    })
}

pub fn help_text() -> &'static str {
    "Usage: {{cookiecutter.plugin_namespace}} [ARGS]\n\nRuns the {{cookiecutter.project_name}} Rust plugin."
}
