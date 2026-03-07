use bijux_dag_runtime::ExecutionPlan;

#[test]
fn engine_flow_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<ExecutionPlan>();
}
