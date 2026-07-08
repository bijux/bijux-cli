use crate::backend::capability::{query_backend_capability, BackendCapabilityQuery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendContractDeclaration {
    pub backend: String,
    pub required_status: String,
    pub require_replay: bool,
}

pub fn validate_backend_contract(declaration: &BackendContractDeclaration) -> Result<(), String> {
    let found = query_backend_capability(&declaration.backend)
        .ok_or_else(|| format!("unsupported backend: {}", declaration.backend))?;
    validate_capability_match(&found, declaration)
}

fn validate_capability_match(
    found: &BackendCapabilityQuery,
    declaration: &BackendContractDeclaration,
) -> Result<(), String> {
    if found.status != declaration.required_status {
        return Err(format!(
            "backend `{}` status mismatch: expected `{}`, got `{}`",
            declaration.backend, declaration.required_status, found.status
        ));
    }
    if declaration.require_replay && !found.supports_replay {
        return Err(format!("backend `{}` must support replay", declaration.backend));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_backend_contract, BackendContractDeclaration};

    #[test]
    fn backend_contract_accepts_implemented_and_simulated_surfaces() {
        validate_backend_contract(&BackendContractDeclaration {
            backend: "local".to_string(),
            required_status: "implemented".to_string(),
            require_replay: true,
        })
        .expect("local contract");

        validate_backend_contract(&BackendContractDeclaration {
            backend: "kubernetes".to_string(),
            required_status: "implemented".to_string(),
            require_replay: true,
        })
        .expect("kubernetes contract");

        validate_backend_contract(&BackendContractDeclaration {
            backend: "hpc".to_string(),
            required_status: "simulated".to_string(),
            require_replay: true,
        })
        .expect("modeled contract");
    }

    #[test]
    fn backend_contract_rejects_unknown_backend() {
        let err = validate_backend_contract(&BackendContractDeclaration {
            backend: "unknown".to_string(),
            required_status: "simulated".to_string(),
            require_replay: false,
        })
        .expect_err("unknown backend must fail");
        assert!(err.contains("unsupported backend"));
    }
}
