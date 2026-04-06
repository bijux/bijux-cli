use crate::{Edge, PortRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeDependencyKind {
    Data,
    Ordering,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedEdge {
    pub from: PortRef,
    pub to: PortRef,
    pub dependency: EdgeDependencyKind,
}

impl From<Edge> for TypedEdge {
    fn from(value: Edge) -> Self {
        Self {
            from: value.from,
            to: value.to,
            dependency: EdgeDependencyKind::Data,
        }
    }
}
