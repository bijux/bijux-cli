use bijux_dag_runtime::cache_entry_invalidated;

#[test]
fn cache_poisoning_inputs_force_invalidation() {
    assert!(cache_entry_invalidated(true, false, false));
    assert!(cache_entry_invalidated(false, true, false));
    assert!(cache_entry_invalidated(false, false, true));
}
