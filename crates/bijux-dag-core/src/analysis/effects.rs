use crate::Effect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectContract {
    pub required: Vec<Effect>,
    pub optional: Vec<Effect>,
}
