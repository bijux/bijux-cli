use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCapabilityQuery {
    pub backend: String,
    pub status: String,
    pub supports_replay: bool,
    pub supports_stream_capture: bool,
}

pub fn local_backend_capability() -> BackendCapabilityQuery {
    BackendCapabilityQuery {
        backend: "local".to_string(),
        status: "implemented".to_string(),
        supports_replay: true,
        supports_stream_capture: true,
    }
}

pub fn kubernetes_backend_capability() -> BackendCapabilityQuery {
    BackendCapabilityQuery {
        backend: "kubernetes".to_string(),
        status: "implemented".to_string(),
        supports_replay: true,
        supports_stream_capture: true,
    }
}

pub fn slurm_backend_capability() -> BackendCapabilityQuery {
    BackendCapabilityQuery {
        backend: "slurm".to_string(),
        status: "implemented".to_string(),
        supports_replay: true,
        supports_stream_capture: true,
    }
}

pub fn hpc_backend_capability() -> BackendCapabilityQuery {
    BackendCapabilityQuery {
        backend: "hpc".to_string(),
        status: "simulated".to_string(),
        supports_replay: true,
        supports_stream_capture: true,
    }
}

pub fn remote_backend_capability() -> BackendCapabilityQuery {
    BackendCapabilityQuery {
        backend: "remote".to_string(),
        status: "simulated".to_string(),
        supports_replay: true,
        supports_stream_capture: true,
    }
}

pub fn query_backend_capability(name: &str) -> Option<BackendCapabilityQuery> {
    match name {
        "local" => Some(local_backend_capability()),
        "k8s" | "kubernetes" => Some(kubernetes_backend_capability()),
        "slurm" => Some(slurm_backend_capability()),
        "hpc" => Some(hpc_backend_capability()),
        "remote" | "distributed" => Some(remote_backend_capability()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        hpc_backend_capability, kubernetes_backend_capability, local_backend_capability,
        query_backend_capability, remote_backend_capability, slurm_backend_capability,
    };

    #[test]
    fn local_and_modeled_capability_queries_are_stable() {
        assert_eq!(local_backend_capability(), local_backend_capability());
        assert_eq!(kubernetes_backend_capability(), kubernetes_backend_capability());
        assert_eq!(slurm_backend_capability(), slurm_backend_capability());
        assert_eq!(hpc_backend_capability(), hpc_backend_capability());
        assert_eq!(remote_backend_capability(), remote_backend_capability());
    }

    #[test]
    fn unknown_backend_query_returns_none() {
        assert!(query_backend_capability("unknown").is_none());
    }
}
