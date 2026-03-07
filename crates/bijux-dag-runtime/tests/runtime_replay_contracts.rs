use bijux_dag_runtime::replay_equivalent;

#[test]
fn replay_mismatch_is_detected() {
    assert!(!replay_equivalent("fingerprint-a", "fingerprint-b"));
}
