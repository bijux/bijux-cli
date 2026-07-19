# Fakes And Scenarios

Fakes exercise runtime integration without launching real external adapters or
backends. Scenario reports assemble cross-crate observations without claiming
that modeled behavior is production evidence.

## Fake Adapter Harness

`FakeAdapterScenario` names deterministic outcomes such as success, refusal,
failure, timeout, or malformed behavior supported by the harness.
`FakeAdapterHarness` records execution requests and returns
`FakeAdapterExecution` under the selected scenario.

A fake must preserve the production contract shape: adapter identity, inputs,
outputs, status, diagnostics, and invocation count. It may replace effects but
cannot bypass validation that the real boundary guarantees.

## Deterministic Behavior

For the same scenario and request, the fake returns the same result. Ordering
and identifiers are stable. Tests that need delay, cancellation, concurrency,
or retries supply explicit controls rather than wall-clock races.

## Product Scenarios

Typed reports cover representative cross-package stories:

- hello DAG;
- shell ETL;
- branch/join;
- reducer;
- failure and retry;
- cache-heavy execution;
- bundle portability;
- mounted-app and Python bridge parity;
- cross-app mock evidence.

Reports summarize supplied observations. They do not run production workflows
or certify release readiness by themselves.

## Appropriate Use

Use fakes for adapter result handling, lifecycle transitions, retry policy,
failure normalization, and orchestration. Use real integration lanes for
subprocess cleanup, container engines, Kubernetes, SLURM, filesystem
permissions, and packaging behavior.

Do not modify production code to detect a testkit fake. The fake implements or
models the same boundary used by production.

## Fault Design

Each fault has a named expected class. Malformed payload, launch failure,
timeout, cancellation, missing output, corruption, and policy refusal remain
separate. Unseeded random failure is unsuitable for shared scenarios.

## Verification

`fake_adapter_harness_contract.rs` protects deterministic fake behavior.
Runtime adapter, retry, cancellation, and failure contracts prove that
consuming behavior agrees with production contracts.
