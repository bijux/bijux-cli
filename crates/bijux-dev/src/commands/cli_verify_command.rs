#[derive(Subcommand)]
pub(super) enum VerifyCommand {
    /// Validate complete evidence foundation integrity
    EvidenceFoundation,
    /// Validate canonical evidence registry integrity and drift
    EvidenceRegistry,
    /// Validate evidence metadata against strict schema contracts
    EvidenceSchema,
    /// Validate authoring evidence surfaces
    EvidenceAuthoring,
    /// Validate battle evidence surfaces and trust mapping
    EvidenceBattle,
    /// Validate cache evidence surfaces
    EvidenceCache,
    /// Validate replay evidence surfaces
    EvidenceReplay,
    /// Validate compatibility evidence surfaces
    EvidenceCompat,
    /// Validate fault evidence surfaces
    EvidenceFault,
    /// Validate performance evidence surfaces
    EvidencePerf,
    /// Validate comparison evidence surfaces
    EvidenceCompare,
    /// Validate evidence ownership metadata completeness
    EvidenceOwnership,
    /// Validate evidence drift and legacy scenario-root freeze
    EvidenceDrift,
    /// Validate that tests and governance consumers reference evidence-owned assets
    EvidenceConsumers,
    /// Validate release evidence set references and classification
    EvidenceReleaseSet,
}
