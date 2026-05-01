use bijux_cli::contracts::version_compatibility_lanes_query;
use bijux_dag_artifacts::RunDirSchemaIndex;
use bijux_dag_core::parse_graph_strict;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct VersionCompatibilityLanesContract {
    schema_version: String,
    surfaces: Vec<VersionCompatibilitySurface>,
}

#[derive(Debug, Clone, Deserialize)]
struct VersionCompatibilitySurface {
    surface: String,
    current_versions: Vec<String>,
    accepted_previous_versions: Vec<String>,
    refused_versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct VersionCompatibilityFixtureSet {
    fixtures: Vec<VersionCompatibilityFixture>,
}

#[derive(Debug, Deserialize)]
struct VersionCompatibilityFixture {
    surface: String,
    version: String,
    expected_lane: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_contract() -> VersionCompatibilityLanesContract {
    let path = repo_root().join("contracts/foundation/version_compatibility_lanes.v1.json");
    read_json(&path)
}

fn read_fixtures() -> VersionCompatibilityFixtureSet {
    let path = repo_root()
        .join("crates/bijux-dev/tests/data/foundation/version_compatibility_lanes_fixtures.json");
    read_json(&path)
}

fn classify_lane(surface: &VersionCompatibilitySurface, version: &str) -> &'static str {
    if surface.current_versions.iter().any(|value| value == version) {
        return "current";
    }
    if surface.accepted_previous_versions.iter().any(|value| value == version) {
        return "previous";
    }
    "refused"
}

#[test]
fn version_compatibility_lane_contract_schema_is_current() {
    let contract = read_contract();
    assert_eq!(contract.schema_version, "foundation-version-compatibility-lanes/v1");
}

#[test]
fn version_compatibility_lane_query_matches_foundation_contract() {
    let contract = read_contract();
    let query = version_compatibility_lanes_query();

    assert_eq!(query.schema_version, "v1");
    assert_eq!(query.surfaces.len(), contract.surfaces.len());

    for (query_surface, contract_surface) in query.surfaces.iter().zip(contract.surfaces.iter()) {
        assert_eq!(query_surface.surface, contract_surface.surface);
        assert_eq!(query_surface.current_versions, contract_surface.current_versions);
        assert_eq!(
            query_surface.accepted_previous_versions,
            contract_surface.accepted_previous_versions
        );
        assert_eq!(query_surface.refused_versions, contract_surface.refused_versions);
    }
}

#[test]
fn version_compatibility_fixtures_cover_current_previous_and_refused_lanes() {
    let contract = read_contract();
    let fixtures = read_fixtures();
    let by_surface = contract
        .surfaces
        .into_iter()
        .map(|surface| (surface.surface.clone(), surface))
        .collect::<BTreeMap<_, _>>();

    let mut saw_previous = false;
    let mut saw_current = false;
    let mut saw_refused = false;

    for fixture in fixtures.fixtures {
        let surface = by_surface
            .get(&fixture.surface)
            .unwrap_or_else(|| panic!("unknown surface in fixture: {}", fixture.surface));
        let lane = classify_lane(surface, &fixture.version);
        assert_eq!(
            lane, fixture.expected_lane,
            "fixture lane mismatch for surface={} version={}",
            fixture.surface, fixture.version
        );
        saw_previous |= lane == "previous";
        saw_current |= lane == "current";
        saw_refused |= lane == "refused";

        if fixture.expected_lane == "refused" {
            assert!(
                surface.refused_versions.iter().any(|value| value == &fixture.version),
                "refused fixture version is missing from refused_versions list"
            );
        }
    }

    assert!(saw_current, "fixtures must include current-lane examples");
    assert!(saw_previous, "fixtures must include previous-lane examples");
    assert!(saw_refused, "fixtures must include refused-lane examples");
}

#[test]
fn graph_spec_current_and_previous_versions_parse_while_refused_versions_fail() {
    let contract = read_contract();
    let graph_surface = contract
        .surfaces
        .into_iter()
        .find(|surface| surface.surface == "graph-spec")
        .expect("graph-spec lane must exist");

    for version in
        graph_surface.current_versions.iter().chain(graph_surface.accepted_previous_versions.iter())
    {
        let payload = format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, version);
        let parsed = parse_graph_strict(&payload).unwrap_or_else(|err| {
            panic!("expected graph spec to parse for version {}: {err}", version)
        });
        assert_eq!(parsed.spec, bijux_dag_core::SPEC_VERSION);
    }

    for version in &graph_surface.refused_versions {
        let payload = format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, version);
        assert!(
            parse_graph_strict(&payload).is_err(),
            "expected graph spec refusal for {}",
            version
        );
    }
}

#[test]
fn artifact_index_current_version_matches_runtime_default() {
    let contract = read_contract();
    let artifact_surface = contract
        .surfaces
        .into_iter()
        .find(|surface| surface.surface == "artifact-index")
        .expect("artifact-index lane must exist");
    let index = RunDirSchemaIndex::default();
    assert!(
        artifact_surface.current_versions.iter().any(|version| version == &index.schema_version),
        "artifact index default schema version drifted from compatibility lanes"
    );
}
