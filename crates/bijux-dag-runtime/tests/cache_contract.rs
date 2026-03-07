use bijux_dag_runtime::cache::CacheKeyInput;

#[test]
fn cache_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<CacheKeyInput>();
}
