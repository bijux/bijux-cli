#![cfg(test)]

use crate::Graph;
use std::process::Command;

pub(crate) fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(crate) fn sample_graph() -> Graph {
    bijux_dag_testkit::graph_diamond()
}
