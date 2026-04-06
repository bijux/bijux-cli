use crate::{Resources, RetryPolicy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDefaults {
    pub retry: Option<RetryPolicy>,
    pub resources: Option<Resources>,
}
