use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{cache_entry_valid, CacheValidationInput};

#[test]
fn adversarial_cache_entry_without_proof_is_rejected() {
    assert!(!cache_entry_valid(&CacheValidationInput {
        fingerprint_matches: true,
        schema_matches: true,
        proof_present: false,
    }));
}
