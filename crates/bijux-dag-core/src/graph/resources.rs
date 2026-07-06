use crate::{Node, Resources, RetryPolicy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphDefaults {
    pub retry: Option<RetryPolicy>,
    pub resources: Option<Resources>,
}

pub fn node_gpu_devices(node: &Node) -> u32 {
    node.resources
        .as_ref()
        .filter(|resources| resources.gpu_devices > 0)
        .map(|resources| resources.gpu_devices)
        .unwrap_or_else(|| gpu_devices_from_tags(&node.tags))
}

pub fn node_accelerator(node: &Node) -> Option<String> {
    node.tags
        .iter()
        .find_map(|tag| {
            if tag == "gpu" {
                Some("gpu".to_string())
            } else if let Some(accelerator) = tag.strip_prefix("accelerator:") {
                let accelerator = accelerator.trim();
                (!accelerator.is_empty()).then(|| accelerator.to_string())
            } else if tag.strip_prefix("gpu:").and_then(|value| value.parse::<u32>().ok()).is_some()
            {
                Some("gpu".to_string())
            } else {
                None
            }
        })
        .or_else(|| (node_gpu_devices(node) > 0).then(|| "gpu".to_string()))
}

pub fn node_named_resources(node: &Node) -> BTreeMap<String, u32> {
    node.resources
        .as_ref()
        .map(|resources| {
            resources
                .named_resources
                .iter()
                .filter(|(_, amount)| **amount > 0)
                .map(|(name, amount)| (name.clone(), *amount))
                .collect()
        })
        .unwrap_or_default()
}

fn gpu_devices_from_tags(tags: &[String]) -> u32 {
    tags.iter()
        .filter_map(|tag| {
            if tag == "gpu" || tag == "accelerator:gpu" {
                Some(1)
            } else {
                tag.strip_prefix("gpu:").and_then(|value| value.parse::<u32>().ok())
            }
        })
        .max()
        .unwrap_or(0)
}
