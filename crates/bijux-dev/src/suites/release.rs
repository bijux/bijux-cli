pub const IDS: &[&str] = &[
    "verify",
    "readiness",
    "compatibility-matrix",
    "post-release-verify",
    "reproducibility-check",
    "evidence-bundle",
];

pub const VERIFY_FLOW: &[&str] = &[
    "checks.run",
    "tests.run",
    "contracts.run",
    "docs.run",
    "repo.run",
    "release.readiness",
    "release.compatibility-matrix",
];
