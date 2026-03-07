use bijux_dag_runtime::{artifact_lineage_complete, classify_failure, RuntimeFailureClass};
use std::collections::BTreeMap;

#[test]
fn artifact_corruption_is_classified_and_lineage_missing_is_detected() {
    assert_eq!(
        classify_failure(false, false, false, false, false, true),
        RuntimeFailureClass::ArtifactCorruption
    );
    let lineage = BTreeMap::from([("a/out".to_string(), "extract".to_string())]);
    assert!(!artifact_lineage_complete(&["a/out".to_string(), "b/out".to_string()], &lineage));
}
