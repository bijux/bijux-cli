pub const IDS: &[&str] = &[
    "verify",
    "readiness",
    "compatibility-matrix",
    "post-release-verify",
    "reproducibility-check",
    "evidence-bundle",
];

pub const VERIFY_FLOW: &[&str] =
    &["release.validation-suite", "release.readiness", "release.compatibility-matrix"];
