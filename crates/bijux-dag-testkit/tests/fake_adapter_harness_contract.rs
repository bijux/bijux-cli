use bijux_dag_testkit::{FakeAdapterHarness, FakeAdapterScenario};

#[test]
fn fake_adapter_harness_covers_success_failure_timeout_and_panic() {
    let harness = FakeAdapterHarness::new("artifact.bin");
    let temp = tempfile::tempdir().expect("tempdir");

    let success = harness
        .execute(FakeAdapterScenario::Success, &temp.path().join("success"))
        .expect("success");
    assert_eq!(success.exit_code, Some(0));
    assert_eq!(success.produced_outputs.get("artifact.bin"), Some(&3usize));

    let failure = harness
        .execute(FakeAdapterScenario::Failure, &temp.path().join("failure"))
        .expect("failure scenario still reports");
    assert_eq!(failure.exit_code, Some(17));
    assert_eq!(failure.failure_kind.as_deref(), Some("ExecutionFailed"));

    let timeout = harness
        .execute(FakeAdapterScenario::Timeout, &temp.path().join("timeout"))
        .expect("timeout scenario still reports");
    assert_eq!(timeout.exit_code, Some(124));
    assert_eq!(timeout.failure_kind.as_deref(), Some("TimeoutExceeded"));

    let panic = harness.execute(FakeAdapterScenario::Panic, &temp.path().join("panic"));
    assert!(panic.is_err());
}

#[test]
fn fake_adapter_harness_covers_missing_corrupt_and_large_outputs() {
    let harness = FakeAdapterHarness::new("artifact.bin");
    let temp = tempfile::tempdir().expect("tempdir");

    let missing = harness
        .execute(FakeAdapterScenario::MissingOutput, &temp.path().join("missing"))
        .expect("missing output scenario");
    assert!(missing.produced_outputs.is_empty());
    assert_eq!(missing.failure_kind.as_deref(), Some("MissingRequiredOutput"));

    let corrupt = harness
        .execute(FakeAdapterScenario::CorruptOutput, &temp.path().join("corrupt"))
        .expect("corrupt output scenario");
    assert_eq!(corrupt.produced_outputs.get("artifact.bin"), Some(&5usize));
    assert_eq!(corrupt.failure_kind.as_deref(), Some("ArtifactCorrupt"));

    let large = harness
        .execute(FakeAdapterScenario::LargeOutput, &temp.path().join("large"))
        .expect("large output scenario");
    assert_eq!(large.exit_code, Some(0));
    assert_eq!(large.produced_outputs.get("artifact.bin"), Some(&(256 * 1024usize)));
    assert!(large.output_hashes.contains_key("artifact.bin"));
}
