
use crate::Graph;
use bijux_dag_core::ParamValue;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub(crate) fn docker_available() -> bool {
    Command::new("docker").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

pub(crate) fn sample_graph() -> Graph {
    bijux_dag_testkit::graph_diamond()
}

pub(crate) fn param_object(items: Vec<(&str, Value)>) -> ParamValue {
    let mut map = BTreeMap::new();
    for (k, v) in items {
        map.insert(k.to_string(), ParamValue::Literal(v));
    }
    ParamValue::Object(map)
}
