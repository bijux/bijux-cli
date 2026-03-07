use bijux_dag_runtime::PlannerExplainReport;

#[test]
fn replay_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<PlannerExplainReport>();
}
