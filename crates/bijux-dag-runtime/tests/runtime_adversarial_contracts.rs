use bijux_dag_runtime::{cache_entry_valid, CacheValidationInput};

#[test]
fn adversarial_cache_entry_without_proof_is_rejected() {
    assert!(!cache_entry_valid(&CacheValidationInput {
        fingerprint_matches: true,
        schema_matches: true,
        proof_present: false,
    }));
}
