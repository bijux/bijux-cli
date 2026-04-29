use crate::{Edge, EdgeKind, PortRef};
use serde::{Deserialize, Serialize};

pub type EdgeDependencyKind = EdgeKind;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedEdge {
    pub from: PortRef,
    pub to: PortRef,
    pub dependency: EdgeDependencyKind,
}

impl From<Edge> for TypedEdge {
    fn from(value: Edge) -> Self {
        Self { from: value.from, to: value.to, dependency: value.kind }
    }
}
