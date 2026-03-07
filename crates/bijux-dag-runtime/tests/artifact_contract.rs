use bijux_dag_runtime::ArtifactStore;

#[test]
fn artifact_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<Option<ArtifactStore>>();
}
