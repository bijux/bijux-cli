#![forbid(unsafe_code)]
//! Evidence schema and validation for maintainer control-plane proofs.

use serde::{Deserialize, Serialize};

/// Evidence status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    /// Fully backed claim with valid artifacts.
    Proven,
    /// Partially backed claim.
    Partial,
    /// Backing evidence is out of date.
    Stale,
    /// Claim is blocked due to missing or invalid evidence.
    Blocked,
}

/// Optional evidence strength model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStrength {
    /// Strong direct proof from executable artifact.
    Strong,
    /// Medium confidence proof.
    Medium,
    /// Weak hint; not release-blocker quality.
    Weak,
}

/// Canonical evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    /// Stable evidence identifier: `EVIDENCE-1001-...`.
    pub id: String,
    /// Short claim description.
    pub claim: String,
    /// Evidence owner (team/crate).
    pub ownership: String,
    /// Source producer command/report.
    pub source: String,
    /// Proof kind (test/report/fuzz/integration).
    pub proof_kind: String,
    /// Artifact links backing this record.
    pub artifact_links: Vec<String>,
    /// Freshness marker (`fresh`/`stale`/`unknown`).
    pub freshness: String,
    /// Lifecycle status.
    pub status: EvidenceStatus,
    /// Optional strength.
    pub strength: EvidenceStrength,
}

/// Validate evidence id format: `EVIDENCE-<4+ digits>-<slug>`.
#[must_use]
pub fn valid_evidence_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("EVIDENCE-") else {
        return false;
    };
    let mut parts = rest.splitn(2, '-');
    let Some(number) = parts.next() else {
        return false;
    };
    let Some(slug) = parts.next() else {
        return false;
    };
    number.len() >= 4
        && number.chars().all(|ch| ch.is_ascii_digit())
        && !slug.is_empty()
        && slug.chars().all(|ch| {
            ch.is_ascii_uppercase() || ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'
        })
}

#[cfg(test)]
mod tests {
    use super::valid_evidence_id;

    #[test]
    fn evidence_id_format_is_enforced() {
        assert!(valid_evidence_id("EVIDENCE-1001-PARITY"));
        assert!(valid_evidence_id("EVIDENCE-2026-rustdoc-health"));
        assert!(!valid_evidence_id("EVIDENCE-123-PARITY"));
        assert!(!valid_evidence_id("EVIDENCE-AAAA-PARITY"));
        assert!(!valid_evidence_id("PARITY-1001"));
    }
}
