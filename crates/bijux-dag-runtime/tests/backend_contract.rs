use bijux_dag_runtime::BackendKind;

#[test]
fn backend_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<BackendKind>();
}
