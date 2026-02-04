use crate::{ExecutionContext, NodeResult, RuntimeError};
use bijux_dag_core::{Effect, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterId {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectSet {
    pub filesystem: bool,
    pub env: bool,
    pub network: bool,
    pub clock: bool,
}

impl EffectSet {
    pub fn from_effects(effects: &[Effect]) -> Self {
        let mut set = EffectSet::default();
        for e in effects {
            match e {
                Effect::Filesystem => set.filesystem = true,
                Effect::Env => set.env = true,
                Effect::Network => set.network = true,
                Effect::Clock => set.clock = true,
            }
        }
        set
    }
}

pub struct NodeCtx<'a> {
    pub node: &'a Node,
    pub exec: &'a ExecutionContext,
    pub params: &'a serde_json::Value,
}

pub trait Adapter: Send + Sync {
    fn id(&self) -> AdapterId;
    fn required_effects(&self) -> EffectSet;
    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError>;
}
