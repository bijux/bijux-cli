use bijux_dag_runtime::cancellation_is_terminal;

#[test]
fn cancellation_requires_terminal_node_state() {
    assert!(cancellation_is_terminal(true, true));
    assert!(!cancellation_is_terminal(true, false));
}
