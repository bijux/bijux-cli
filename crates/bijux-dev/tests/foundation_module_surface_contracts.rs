use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct ModuleSurfaceContract {
    schema_version: String,
    binary_only_crates: Vec<String>,
    crates: Vec<ModuleSurfaceCrate>,
}

#[derive(Debug, Deserialize)]
struct ModuleSurfaceCrate {
    #[serde(rename = "crate")]
    crate_name: String,
    default_internal_lane: String,
    stable_public_modules: Vec<String>,
    experimental_public_modules: Vec<String>,
    simulated_public_modules: Vec<String>,
}

#[derive(Debug)]
struct ModuleDecl {
    name: String,
    is_public: bool,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_contract() -> ModuleSurfaceContract {
    let path = repo_root().join("contracts/foundation/module_surface_lanes.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw).expect("module surface contract must be valid JSON")
}

fn parse_module_decl(line: &str) -> Option<ModuleDecl> {
    let trimmed = line.trim_start();
    let (is_public, rest) = if let Some(rest) = trimmed.strip_prefix("pub mod ") {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix("mod ") {
        (false, rest)
    } else {
        return None;
    };

    let name =
        rest.chars().take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_').collect::<String>();
    if name.is_empty() {
        return None;
    }

    let delimiter = rest.chars().nth(name.len());
    if !matches!(delimiter, Some(';' | '{' | ' ' | '\t')) {
        return None;
    }

    Some(ModuleDecl { name, is_public })
}

fn parse_top_level_module_decls(crate_name: &str) -> Vec<ModuleDecl> {
    let path = repo_root().join(format!("crates/{crate_name}/src/lib.rs"));
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut depth = 0usize;
    let mut decls = Vec::new();

    for line in source.lines() {
        if depth == 0 {
            if let Some(decl) = parse_module_decl(line) {
                decls.push(decl);
            }
        }

        let opens = line.matches('{').count();
        let closes = line.matches('}').count();
        depth = depth.saturating_add(opens).saturating_sub(closes);
    }

    decls
}

#[test]
fn module_surface_contract_schema_is_current() {
    let contract = read_contract();
    assert_eq!(contract.schema_version, "foundation-module-surface-lanes/v1");
}

#[test]
fn binary_only_crates_do_not_expose_library_modules() {
    let contract = read_contract();
    for crate_name in contract.binary_only_crates {
        let lib_path = repo_root().join(format!("crates/{crate_name}/src/lib.rs"));
        let main_path = repo_root().join(format!("crates/{crate_name}/src/main.rs"));
        assert!(
            !lib_path.exists(),
            "binary-only crate unexpectedly has library entrypoint: {}",
            lib_path.display()
        );
        assert!(
            main_path.is_file(),
            "binary-only crate must expose src/main.rs: {}",
            main_path.display()
        );
    }
}

#[test]
fn public_module_lanes_match_contract_and_internals_stay_private() {
    let contract = read_contract();
    for crate_contract in contract.crates {
        assert_eq!(
            crate_contract.default_internal_lane, "private",
            "{} must declare private default internal lane",
            crate_contract.crate_name
        );

        let decls = parse_top_level_module_decls(&crate_contract.crate_name);
        let observed_public = decls
            .iter()
            .filter(|decl| decl.is_public)
            .map(|decl| decl.name.clone())
            .collect::<BTreeSet<_>>();

        let stable = crate_contract.stable_public_modules.into_iter().collect::<BTreeSet<_>>();
        let experimental =
            crate_contract.experimental_public_modules.into_iter().collect::<BTreeSet<_>>();
        let simulated =
            crate_contract.simulated_public_modules.into_iter().collect::<BTreeSet<_>>();

        assert!(
            stable.is_disjoint(&experimental),
            "{} has modules in both stable and experimental lanes",
            crate_contract.crate_name
        );
        assert!(
            stable.is_disjoint(&simulated),
            "{} has modules in both stable and simulated lanes",
            crate_contract.crate_name
        );
        assert!(
            experimental.is_disjoint(&simulated),
            "{} has modules in both experimental and simulated lanes",
            crate_contract.crate_name
        );

        let expected_public = stable
            .union(&experimental)
            .cloned()
            .collect::<BTreeSet<_>>()
            .union(&simulated)
            .cloned()
            .collect::<BTreeSet<_>>();

        assert_eq!(
            observed_public, expected_public,
            "public module exports drifted for {}",
            crate_contract.crate_name
        );

        for decl in decls.iter().filter(|decl| !decl.is_public) {
            assert!(
                !expected_public.contains(&decl.name),
                "internal module in {} must not appear in public lanes: {}",
                crate_contract.crate_name,
                decl.name
            );
        }
    }
}
