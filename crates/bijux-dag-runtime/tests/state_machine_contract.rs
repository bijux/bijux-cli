use bijux_dag_runtime::RuntimeState;

#[test]
fn state_machine_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<RuntimeState>();
}
