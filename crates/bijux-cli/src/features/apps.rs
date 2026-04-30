#![forbid(unsafe_code)]
//! Official app discovery, resolution, and health reporting.

use std::env;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};

use crate::contracts::{
    known_bijux_tool_by_query, known_bijux_tools, product_mount_descriptor_schema,
    validate_product_mount_descriptor, KnownBijuxTool, Namespace, ProductEntrypoint,
    ProductEntrypointKind, ProductHelpMetadata, ProductMountDescriptor,
};
use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::plugins::list_plugins;
use crate::sdk::{FeatureCapabilityDeclaration, ProductMount};

const DEFAULT_SYSTEM_APPS_DIR: &str = "/etc/bijux/apps";
const BIJUX_APP_PATH: &str = "BIJUX_APP_PATH";
const BIJUX_SYSTEM_APP_PATH: &str = "BIJUX_SYSTEM_APP_PATH";
const BIJUX_PYTHON_BIN: &str = "BIJUX_PYTHON_BIN";
const APP_SCAFFOLD_VERSION: &str = "0.1.0";
const PYTHON_CALLABLE_RUNNER: &str = r#"import importlib, io, json, sys
from contextlib import redirect_stdout
module_name = sys.argv[1]
function_name = sys.argv[2]
argv = sys.argv[3:]
module = importlib.import_module(module_name)
target = getattr(module, function_name)
if not callable(target):
    raise TypeError(f"{module_name}.{function_name} is not callable")
log_buffer = io.StringIO()
with redirect_stdout(log_buffer):
    result = target(argv)
logs = log_buffer.getvalue()
if logs:
    sys.stderr.write(logs)
if hasattr(result, "emit") and callable(result.emit):
    raise SystemExit(int(result.emit()))
if result is None:
    raise SystemExit(0)
if isinstance(result, bool):
    raise SystemExit(0 if result else 1)
if isinstance(result, int):
    raise SystemExit(result)
if isinstance(result, (dict, list)):
    print(json.dumps(result, indent=2))
    raise SystemExit(0)
if isinstance(result, str):
    print(result)
    raise SystemExit(0)
raise SystemExit(int(result))
"#;
const PYTHON_DOCTOR_PROBE: &str = r#"import importlib, importlib.metadata, json, sys
module_name = sys.argv[1]
function_name = None if sys.argv[2] == "-" else sys.argv[2]
payload = {
    "module": module_name,
    "function": function_name,
    "import_ok": False,
    "import_error": None,
    "package_version": None,
    "callable_ok": None,
    "callable_error": None,
}
try:
    module = importlib.import_module(module_name)
    payload["import_ok"] = True
    for candidate in (module_name, module_name.split(".")[0]):
        try:
            payload["package_version"] = importlib.metadata.version(candidate)
            break
        except importlib.metadata.PackageNotFoundError:
            pass
    if function_name is not None:
        try:
            target = getattr(module, function_name)
            payload["callable_ok"] = callable(target)
            if not payload["callable_ok"]:
                payload["callable_error"] = f"{module_name}.{function_name} is not callable"
        except Exception as exc:
            payload["callable_ok"] = False
            payload["callable_error"] = f"{type(exc).__name__}: {exc}"
except Exception as exc:
    payload["import_error"] = f"{type(exc).__name__}: {exc}"
print(json.dumps(payload))
"#;
const RESERVED_APP_NAMESPACES: &[&str] = &[
    "apps",
    "cli",
    "completion",
    "config",
    "doctor",
    "help",
    "history",
    "inspect",
    "install",
    "memory",
    "plugins",
    "repl",
    "self",
    "status",
    "version",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppDiscoverySource {
    CompiledOfficialRegistry,
    ProjectLocal,
    EnvironmentPath,
    UserLocal,
    SystemLocal,
    PathFallback,
    PluginRegistry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppHealth {
    Ok,
    Missing,
    VersionMismatch,
    PermissionDenied,
    BadManifest,
    Conflict,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppQueryMatch {
    Namespace,
    Alias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PythonInterpreterSource {
    ActiveVenv,
    ProjectVenv,
    SystemPath,
    Configured,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppDiscoveryPath {
    pub source: AppDiscoverySource,
    pub location: String,
    pub precedence: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppProbe {
    pub source: AppDiscoverySource,
    pub location: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PythonInterpreterAttempt {
    pub source: PythonInterpreterSource,
    pub location: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PythonDoctorReport {
    pub module: String,
    pub function: Option<String>,
    pub interpreter_source: Option<PythonInterpreterSource>,
    pub interpreter: Option<String>,
    pub attempts: Vec<PythonInterpreterAttempt>,
    pub import_ok: bool,
    pub import_error: Option<String>,
    pub package_version: Option<String>,
    pub callable_ok: Option<bool>,
    pub callable_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppMountReport {
    pub namespace: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub source: AppDiscoverySource,
    pub entrypoint_kind: ProductEntrypointKind,
    pub entrypoint: String,
    pub status: String,
    pub version: Option<String>,
    pub version_source: Option<String>,
    pub health: AppHealth,
    pub resolved_entrypoint: Option<String>,
    pub resolution_policy: String,
    pub shadowed_plugins: Vec<String>,
    pub capabilities: Vec<String>,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub probes: Vec<AppProbe>,
    pub issues: Vec<String>,
    pub help_summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonDoctorReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppMountSummary {
    pub ok: usize,
    pub missing: usize,
    pub version_mismatch: usize,
    pub permission_denied: usize,
    pub bad_manifest: usize,
    pub conflict: usize,
    pub disabled: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppsListReport {
    pub status: String,
    pub precedence: Vec<String>,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub apps: Vec<AppMountReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppsDoctorReport {
    pub status: String,
    pub precedence: Vec<String>,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub summary: AppMountSummary,
    pub apps: Vec<AppMountReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppDoctorReport {
    pub status: String,
    pub query: String,
    pub matched_via: AppQueryMatch,
    pub namespace: String,
    pub app: AppMountReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppWhichReport {
    pub status: String,
    pub query: String,
    pub matched_via: AppQueryMatch,
    pub namespace: String,
    pub source: AppDiscoverySource,
    pub entrypoint_kind: ProductEntrypointKind,
    pub resolved_entrypoint: Option<String>,
    pub health: AppHealth,
    pub resolution_policy: String,
    pub shadowed_plugins: Vec<String>,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub probes: Vec<AppProbe>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppVersionReport {
    pub status: String,
    pub query: String,
    pub matched_via: AppQueryMatch,
    pub namespace: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub health: AppHealth,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppCapabilitiesReport {
    pub status: String,
    pub query: String,
    pub matched_via: AppQueryMatch,
    pub namespace: String,
    pub entrypoint_kind: ProductEntrypointKind,
    pub capabilities: Vec<String>,
    pub aliases: Vec<String>,
    pub source: AppDiscoverySource,
    pub health: AppHealth,
    pub help_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppManifestSchemaReport {
    pub status: String,
    pub schema: String,
    pub schema_json: Value,
    pub entrypoint_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppManifestValidationReport {
    pub status: String,
    pub path: String,
    pub valid: bool,
    pub namespace: Option<String>,
    pub entrypoint_kind: Option<ProductEntrypointKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<Value>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppScaffoldReport {
    pub status: String,
    pub kind: String,
    pub namespace: String,
    pub root: String,
    pub manifest_path: String,
    pub entrypoint_kind: ProductEntrypointKind,
    pub entrypoint: String,
    pub files: Vec<String>,
    pub guidance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAppCommand {
    pub command: String,
    pub args: Vec<String>,
    pub display_command: String,
    pub source: AppDiscoverySource,
    pub kind: ProductEntrypointKind,
    pub namespace: String,
    pub descriptor: ProductMountDescriptor,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ProductMountOverride {
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    aliases: Option<Vec<String>>,
    #[serde(default)]
    entrypoint: Option<ProductEntrypoint>,
    #[serde(default)]
    control_entrypoint: Option<ProductEntrypoint>,
    #[serde(default)]
    help: Option<ProductHelpMetadata>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    disabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DisabledProductsRegistry {
    Object { disabled: Vec<String> },
    Array(Vec<String>),
}

impl DisabledProductsRegistry {
    fn contains(&self, query: &str) -> bool {
        let normalized = crate::contracts::Namespace::normalize(query);
        match self {
            Self::Object { disabled } => disabled
                .iter()
                .any(|value| crate::contracts::Namespace::normalize(value) == normalized),
            Self::Array(disabled) => disabled
                .iter()
                .any(|value| crate::contracts::Namespace::normalize(value) == normalized),
        }
    }
}

#[derive(Debug, Clone)]
struct OverrideCandidate {
    source: AppDiscoverySource,
    location: PathBuf,
}

#[derive(Debug, Clone)]
struct DisabledRegistryCandidate {
    source: AppDiscoverySource,
    location: PathBuf,
}

#[derive(Debug, Clone)]
struct ResolutionConfig {
    probe_version: bool,
    probe_python: bool,
}

#[derive(Debug, Clone)]
struct AppResolution {
    descriptor: ProductMountDescriptor,
    descriptor_source: AppDiscoverySource,
    runtime_resolution: Option<ResolvedAppCommand>,
    health: AppHealth,
    shadowed_plugins: Vec<String>,
    issues: Vec<String>,
    discovery_paths: Vec<AppDiscoveryPath>,
    probes: Vec<AppProbe>,
    version: Option<String>,
    version_source: Option<String>,
    python: Option<PythonDoctorReport>,
}

fn precedence_names() -> Vec<String> {
    vec![
        "built_in_official".to_string(),
        "project_local".to_string(),
        "environment_path".to_string(),
        "user_installed".to_string(),
        "system_installed".to_string(),
        "path_fallback".to_string(),
    ]
}

fn user_apps_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".bijux/apps"))
}

fn system_apps_dir() -> PathBuf {
    env::var_os(BIJUX_SYSTEM_APP_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SYSTEM_APPS_DIR))
}

fn env_apps_dirs() -> Vec<PathBuf> {
    env::var_os(BIJUX_APP_PATH)
        .map(|raw| env::split_paths(&raw).collect::<Vec<_>>())
        .unwrap_or_default()
}

fn custom_descriptor_dirs() -> Vec<(AppDiscoverySource, PathBuf)> {
    let mut rows = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        rows.push((AppDiscoverySource::ProjectLocal, cwd.join(".bijux/apps")));
    }
    rows.extend(
        env_apps_dirs()
            .into_iter()
            .map(|directory| (AppDiscoverySource::EnvironmentPath, directory)),
    );
    if let Some(directory) = user_apps_dir() {
        rows.push((AppDiscoverySource::UserLocal, directory));
    }
    rows.push((AppDiscoverySource::SystemLocal, system_apps_dir()));
    rows
}

fn primary_descriptor_path(dir: &Path, namespace: &str) -> PathBuf {
    dir.join(format!("{namespace}.mount.json"))
}

fn legacy_descriptor_path(dir: &Path, namespace: &str) -> PathBuf {
    dir.join(format!("{namespace}.json"))
}

fn descriptor_paths(dir: &Path, namespace: &str) -> Vec<PathBuf> {
    vec![primary_descriptor_path(dir, namespace), legacy_descriptor_path(dir, namespace)]
}

fn disabled_registry_path(dir: &Path) -> PathBuf {
    dir.join("disabled.json")
}

fn is_safe_scaffold_path(path: &Path) -> bool {
    !path.components().any(|component| matches!(component, Component::ParentDir))
}

fn app_module_name(namespace: &str) -> String {
    format!("{}_app", namespace.replace('-', "_"))
}

fn rust_app_entrypoint_name(namespace: &str) -> String {
    format!("{namespace}-app")
}

fn discovery_paths_for(namespace: &str, paths: &ResolvedStatePaths) -> Vec<AppDiscoveryPath> {
    let mut rows = vec![AppDiscoveryPath {
        source: AppDiscoverySource::CompiledOfficialRegistry,
        location: format!("compiled:{}", namespace),
        precedence: 0,
    }];

    if let Ok(cwd) = env::current_dir() {
        rows.push(AppDiscoveryPath {
            source: AppDiscoverySource::ProjectLocal,
            location: primary_descriptor_path(&cwd.join(".bijux/apps"), namespace)
                .display()
                .to_string(),
            precedence: 1,
        });
    }

    for directory in env_apps_dirs() {
        rows.push(AppDiscoveryPath {
            source: AppDiscoverySource::EnvironmentPath,
            location: primary_descriptor_path(&directory, namespace).display().to_string(),
            precedence: 2,
        });
    }

    if let Some(directory) = user_apps_dir() {
        rows.push(AppDiscoveryPath {
            source: AppDiscoverySource::UserLocal,
            location: primary_descriptor_path(&directory, namespace).display().to_string(),
            precedence: 3,
        });
    }

    rows.push(AppDiscoveryPath {
        source: AppDiscoverySource::SystemLocal,
        location: primary_descriptor_path(&system_apps_dir(), namespace).display().to_string(),
        precedence: 4,
    });
    rows.push(AppDiscoveryPath {
        source: AppDiscoverySource::PathFallback,
        location: env::var("PATH").unwrap_or_default(),
        precedence: 5,
    });
    rows.push(AppDiscoveryPath {
        source: AppDiscoverySource::PluginRegistry,
        location: paths.plugin_registry_file.display().to_string(),
        precedence: 6,
    });
    rows
}

fn override_candidates(namespace: &str) -> Vec<OverrideCandidate> {
    let mut rows = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        rows.extend(descriptor_paths(&cwd.join(".bijux/apps"), namespace).into_iter().map(
            |location| OverrideCandidate { source: AppDiscoverySource::ProjectLocal, location },
        ));
    }
    for directory in env_apps_dirs() {
        rows.extend(descriptor_paths(&directory, namespace).into_iter().map(|location| {
            OverrideCandidate { source: AppDiscoverySource::EnvironmentPath, location }
        }));
    }
    if let Some(directory) = user_apps_dir() {
        rows.extend(
            descriptor_paths(&directory, namespace).into_iter().map(|location| OverrideCandidate {
                source: AppDiscoverySource::UserLocal,
                location,
            }),
        );
    }
    rows.extend(
        descriptor_paths(&system_apps_dir(), namespace).into_iter().map(|location| {
            OverrideCandidate { source: AppDiscoverySource::SystemLocal, location }
        }),
    );
    rows
}

fn disabled_registry_candidates() -> Vec<DisabledRegistryCandidate> {
    let mut rows = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        rows.push(DisabledRegistryCandidate {
            source: AppDiscoverySource::ProjectLocal,
            location: disabled_registry_path(&cwd.join(".bijux/apps")),
        });
    }
    rows.extend(env_apps_dirs().into_iter().map(|directory| DisabledRegistryCandidate {
        source: AppDiscoverySource::EnvironmentPath,
        location: disabled_registry_path(&directory),
    }));
    if let Some(directory) = user_apps_dir() {
        rows.push(DisabledRegistryCandidate {
            source: AppDiscoverySource::UserLocal,
            location: disabled_registry_path(&directory),
        });
    }
    rows.push(DisabledRegistryCandidate {
        source: AppDiscoverySource::SystemLocal,
        location: disabled_registry_path(&system_apps_dir()),
    });
    rows
}

fn which_in_path(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.exists().then(|| path.to_path_buf());
    }

    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|directory| directory.join(command))
        .find(|candidate| candidate.exists())
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn apply_override(
    descriptor: &mut ProductMountDescriptor,
    overlay: ProductMountOverride,
    candidate: &OverrideCandidate,
) -> Result<bool, String> {
    if let Some(namespace) = overlay.namespace {
        let normalized = crate::contracts::Namespace::normalize(&namespace);
        if normalized != descriptor.namespace.as_str() {
            return Err(format!(
                "override namespace `{normalized}` does not match requested namespace `{}`",
                descriptor.namespace.as_str()
            ));
        }
    }

    let mut changed = false;
    if let Some(display_name) = overlay.display_name {
        if !display_name.trim().is_empty() {
            descriptor.display_name = display_name;
            changed = true;
        }
    }
    if let Some(aliases) = overlay.aliases {
        descriptor.aliases =
            aliases.into_iter().map(|value| crate::contracts::Namespace(value)).collect();
        changed = true;
    }
    if let Some(entrypoint) = overlay.entrypoint {
        descriptor.entrypoint = absolutize_entrypoint(entrypoint, candidate.location.parent());
        changed = true;
    }
    if let Some(control_entrypoint) = overlay.control_entrypoint {
        descriptor.control_entrypoint =
            absolutize_entrypoint(control_entrypoint, candidate.location.parent());
        changed = true;
    }
    if let Some(help) = overlay.help {
        descriptor.help = help;
        changed = true;
    }
    if let Some(capabilities) = overlay.capabilities {
        descriptor.capabilities = capabilities;
        changed = true;
    }
    if let Some(version) = overlay.version {
        descriptor.version = Some(version);
        changed = true;
    }

    Ok(overlay.disabled.unwrap_or(false) || changed)
}

fn absolutize_entrypoint(
    mut entrypoint: ProductEntrypoint,
    base_dir: Option<&Path>,
) -> ProductEntrypoint {
    if matches!(
        entrypoint.kind,
        ProductEntrypointKind::Binary
            | ProductEntrypointKind::PythonConsoleScript
            | ProductEntrypointKind::PluginProcess
    ) {
        let path = Path::new(&entrypoint.command);
        if path.components().count() > 1 && path.is_relative() {
            if let Some(base) = base_dir {
                entrypoint.command = base.join(path).display().to_string();
            }
        }
    }
    entrypoint
}

fn load_full_mount_descriptor(raw: &str, path: &Path) -> Result<ProductMountDescriptor, String> {
    let mut descriptor = serde_json::from_str::<ProductMountDescriptor>(raw)
        .map_err(|error| format!("failed to parse descriptor JSON: {error}"))?;
    descriptor.entrypoint = absolutize_entrypoint(descriptor.entrypoint, path.parent());
    descriptor.control_entrypoint =
        absolutize_entrypoint(descriptor.control_entrypoint, path.parent());
    validate_product_mount_descriptor(&descriptor)?;
    Ok(descriptor)
}

fn read_mount_descriptor(path: &Path) -> Result<ProductMountDescriptor, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read descriptor file: {error}"))?;
    load_full_mount_descriptor(&raw, path)
}

fn custom_mount_is_disabled(descriptor: &ProductMountDescriptor) -> bool {
    for candidate in disabled_registry_candidates() {
        if !candidate.location.exists() {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&candidate.location) else {
            continue;
        };
        let Ok(registry) = serde_json::from_str::<DisabledProductsRegistry>(&raw) else {
            continue;
        };
        if registry.contains(descriptor.namespace.as_str())
            || descriptor.aliases.iter().any(|alias| registry.contains(alias.as_str()))
        {
            return true;
        }
    }
    false
}

fn descriptor_matches_query(descriptor: &ProductMountDescriptor, query: &str) -> bool {
    let normalized = Namespace::normalize(query);
    descriptor.namespace.as_str() == normalized
        || descriptor.aliases.iter().any(|alias| alias.as_str() == normalized)
}

fn resolve_descriptor_runtime_command(
    descriptor: ProductMountDescriptor,
    source: AppDiscoverySource,
) -> ResolvedAppCommand {
    match descriptor.entrypoint.kind {
        ProductEntrypointKind::Binary
        | ProductEntrypointKind::PythonConsoleScript
        | ProductEntrypointKind::PluginProcess => {
            let command = descriptor.entrypoint.command.clone();
            let resolved =
                which_in_path(&command).map(|path| path.display().to_string()).unwrap_or(command);
            ResolvedAppCommand {
                display_command: resolved.clone(),
                command: resolved,
                args: Vec::new(),
                source,
                kind: descriptor.entrypoint.kind.clone(),
                namespace: descriptor.namespace.as_str().to_string(),
                descriptor,
            }
        }
        ProductEntrypointKind::PythonModule => {
            let resolution = resolve_python_interpreter();
            let interpreter = resolution
                .selected
                .as_ref()
                .map(|(_, path)| path.display().to_string())
                .unwrap_or_else(|| {
                    env::var(BIJUX_PYTHON_BIN).unwrap_or_else(|_| "python3".to_string())
                });
            let args = python_runtime_args(&descriptor.entrypoint);
            ResolvedAppCommand {
                display_command: format_display_command(&interpreter, &args),
                command: interpreter,
                args,
                source,
                kind: ProductEntrypointKind::PythonModule,
                namespace: descriptor.namespace.as_str().to_string(),
                descriptor,
            }
        }
        ProductEntrypointKind::EmbeddedRust => ResolvedAppCommand {
            display_command: format!("embedded:{}", descriptor.entrypoint.command),
            command: descriptor.entrypoint.command.clone(),
            args: Vec::new(),
            source,
            kind: ProductEntrypointKind::EmbeddedRust,
            namespace: descriptor.namespace.as_str().to_string(),
            descriptor,
        },
    }
}

fn resolve_descriptor_control_command(
    descriptor: ProductMountDescriptor,
    source: AppDiscoverySource,
) -> ResolvedAppCommand {
    match descriptor.control_entrypoint.kind {
        ProductEntrypointKind::Binary
        | ProductEntrypointKind::PythonConsoleScript
        | ProductEntrypointKind::PluginProcess => {
            let command = descriptor.control_entrypoint.command.clone();
            let resolved =
                which_in_path(&command).map(|path| path.display().to_string()).unwrap_or(command);
            ResolvedAppCommand {
                display_command: resolved.clone(),
                command: resolved,
                args: Vec::new(),
                source,
                kind: descriptor.control_entrypoint.kind.clone(),
                namespace: descriptor.namespace.as_str().to_string(),
                descriptor,
            }
        }
        ProductEntrypointKind::PythonModule => {
            let resolution = resolve_python_interpreter();
            let interpreter = resolution
                .selected
                .as_ref()
                .map(|(_, path)| path.display().to_string())
                .unwrap_or_else(|| {
                    env::var(BIJUX_PYTHON_BIN).unwrap_or_else(|_| "python3".to_string())
                });
            let args = python_runtime_args(&descriptor.control_entrypoint);
            ResolvedAppCommand {
                display_command: format_display_command(&interpreter, &args),
                command: interpreter,
                args,
                source,
                kind: ProductEntrypointKind::PythonModule,
                namespace: descriptor.namespace.as_str().to_string(),
                descriptor,
            }
        }
        ProductEntrypointKind::EmbeddedRust => ResolvedAppCommand {
            display_command: format!("embedded:{}", descriptor.control_entrypoint.command),
            command: descriptor.control_entrypoint.command.clone(),
            args: Vec::new(),
            source,
            kind: ProductEntrypointKind::EmbeddedRust,
            namespace: descriptor.namespace.as_str().to_string(),
            descriptor,
        },
    }
}

fn discover_custom_mount(query: &str) -> Option<(ProductMountDescriptor, AppDiscoverySource)> {
    for (source, directory) in custom_descriptor_dirs() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        let mut files = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                path.file_name().and_then(|value| value.to_str()).is_some_and(|value| {
                    value.ends_with(".mount.json")
                        || (value.ends_with(".json") && value != "disabled.json")
                })
            })
            .collect::<Vec<_>>();
        files.sort();

        for path in files {
            let Ok(descriptor) = read_mount_descriptor(&path) else {
                continue;
            };
            if !descriptor_matches_query(&descriptor, query)
                || custom_mount_is_disabled(&descriptor)
            {
                continue;
            }
            return Some((descriptor, source));
        }
    }
    None
}

fn format_display_command(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        command.to_string()
    } else {
        format!("{command} {}", args.join(" "))
    }
}

#[derive(Debug, Clone)]
struct PythonInterpreterResolution {
    selected: Option<(PythonInterpreterSource, PathBuf)>,
    attempts: Vec<PythonInterpreterAttempt>,
}

fn python_runtime_args(entrypoint: &ProductEntrypoint) -> Vec<String> {
    let module = python_module_name(entrypoint);
    let function = entrypoint.function.as_deref().map(str::trim).filter(|value| !value.is_empty());
    match function {
        Some(function) => vec![
            "-c".to_string(),
            PYTHON_CALLABLE_RUNNER.to_string(),
            module.to_string(),
            function.to_string(),
        ],
        None => vec!["-m".to_string(), module.to_string()],
    }
}

fn python_module_name(entrypoint: &ProductEntrypoint) -> &str {
    entrypoint
        .module
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(entrypoint.command.as_str())
}

fn python_interpreter_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["python.exe", "python"]
    } else {
        &["python3", "python"]
    }
}

fn python_executable_in(root: &Path) -> Option<PathBuf> {
    let names = python_interpreter_names();
    let candidates = if cfg!(windows) {
        vec![root.join("Scripts").join(names[0]), root.join("Scripts").join(names[1])]
    } else {
        vec![root.join("bin").join(names[0]), root.join("bin").join(names[1])]
    };
    candidates.into_iter().find(|candidate| candidate.exists())
}

fn resolve_python_interpreter() -> PythonInterpreterResolution {
    let mut attempts = Vec::new();

    if let Some(explicit) = env::var_os(BIJUX_PYTHON_BIN) {
        let configured = explicit.to_string_lossy().trim().to_string();
        if configured.is_empty() {
            attempts.push(PythonInterpreterAttempt {
                source: PythonInterpreterSource::Configured,
                location: String::new(),
                status: "missing".to_string(),
                message: format!("{BIJUX_PYTHON_BIN} was provided but empty"),
            });
        } else {
            let configured_path = PathBuf::from(&configured);
            let resolved = which_in_path(&configured)
                .or_else(|| configured_path.exists().then_some(configured_path.clone()));
            if let Some(path) = resolved {
                attempts.push(PythonInterpreterAttempt {
                    source: PythonInterpreterSource::Configured,
                    location: path.display().to_string(),
                    status: "ok".to_string(),
                    message: format!("selected interpreter from {BIJUX_PYTHON_BIN}"),
                });
                return PythonInterpreterResolution {
                    selected: Some((PythonInterpreterSource::Configured, path)),
                    attempts,
                };
            }
            attempts.push(PythonInterpreterAttempt {
                source: PythonInterpreterSource::Configured,
                location: configured,
                status: "missing".to_string(),
                message: format!("{BIJUX_PYTHON_BIN} points to a missing interpreter"),
            });
        }
    }

    if let Some(active_venv) = env::var_os("VIRTUAL_ENV") {
        let root = PathBuf::from(active_venv);
        if let Some(candidate) = python_executable_in(&root) {
            attempts.push(PythonInterpreterAttempt {
                source: PythonInterpreterSource::ActiveVenv,
                location: candidate.display().to_string(),
                status: "ok".to_string(),
                message: "selected interpreter from active virtual environment".to_string(),
            });
            return PythonInterpreterResolution {
                selected: Some((PythonInterpreterSource::ActiveVenv, candidate)),
                attempts,
            };
        }
        attempts.push(PythonInterpreterAttempt {
            source: PythonInterpreterSource::ActiveVenv,
            location: root.display().to_string(),
            status: "missing".to_string(),
            message: "active virtual environment does not expose a usable python executable"
                .to_string(),
        });
    }

    if let Ok(cwd) = env::current_dir() {
        for candidate_root in cwd.ancestors().map(|value| value.join(".venv")) {
            if !candidate_root.exists() {
                continue;
            }
            if let Some(candidate) = python_executable_in(&candidate_root) {
                attempts.push(PythonInterpreterAttempt {
                    source: PythonInterpreterSource::ProjectVenv,
                    location: candidate.display().to_string(),
                    status: "ok".to_string(),
                    message: "selected interpreter from project .venv".to_string(),
                });
                return PythonInterpreterResolution {
                    selected: Some((PythonInterpreterSource::ProjectVenv, candidate)),
                    attempts,
                };
            }
            attempts.push(PythonInterpreterAttempt {
                source: PythonInterpreterSource::ProjectVenv,
                location: candidate_root.display().to_string(),
                status: "missing".to_string(),
                message: "project .venv exists but no usable python executable was found"
                    .to_string(),
            });
        }
    }

    for candidate_name in python_interpreter_names() {
        if let Some(candidate) = which_in_path(candidate_name) {
            attempts.push(PythonInterpreterAttempt {
                source: PythonInterpreterSource::SystemPath,
                location: candidate.display().to_string(),
                status: "ok".to_string(),
                message: format!("selected interpreter from PATH lookup for `{candidate_name}`"),
            });
            return PythonInterpreterResolution {
                selected: Some((PythonInterpreterSource::SystemPath, candidate)),
                attempts,
            };
        }
        attempts.push(PythonInterpreterAttempt {
            source: PythonInterpreterSource::SystemPath,
            location: candidate_name.to_string(),
            status: "missing".to_string(),
            message: "interpreter name was not found on PATH".to_string(),
        });
    }

    PythonInterpreterResolution { selected: None, attempts }
}

fn probe_version(invocation: &ResolvedAppCommand) -> Option<String> {
    let output =
        Command::new(&invocation.command).args(&invocation.args).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}

fn probe_python_doctor(
    entrypoint: &ProductEntrypoint,
    resolution: &PythonInterpreterResolution,
) -> PythonDoctorReport {
    let module = python_module_name(entrypoint).to_string();
    let function = entrypoint.function.clone();
    let mut report = PythonDoctorReport {
        module: module.clone(),
        function: function.clone(),
        interpreter_source: resolution.selected.as_ref().map(|(source, _)| *source),
        interpreter: resolution.selected.as_ref().map(|(_, path)| path.display().to_string()),
        attempts: resolution.attempts.clone(),
        import_ok: false,
        import_error: None,
        package_version: None,
        callable_ok: function.as_ref().map(|_| false),
        callable_error: None,
    };

    let Some((_, interpreter)) = &resolution.selected else {
        report.import_error = Some("no python interpreter resolved".to_string());
        return report;
    };

    let output = Command::new(interpreter)
        .args(["-c", PYTHON_DOCTOR_PROBE, module.as_str(), function.as_deref().unwrap_or("-")])
        .output();

    let Ok(output) = output else {
        report.import_error = Some("failed to execute python dependency probe".to_string());
        return report;
    };

    let payload =
        if output.stdout.is_empty() { output.stderr.clone() } else { output.stdout.clone() };
    let parsed = serde_json::from_slice::<Value>(&payload);
    let Ok(parsed) = parsed else {
        report.import_error = Some(format!(
            "python dependency probe returned non-json output: {}",
            String::from_utf8_lossy(&payload).trim()
        ));
        return report;
    };

    report.import_ok = parsed["import_ok"].as_bool().unwrap_or(false);
    report.import_error = parsed["import_error"].as_str().map(ToOwned::to_owned);
    report.package_version = parsed["package_version"].as_str().map(ToOwned::to_owned);
    report.callable_ok = parsed["callable_ok"].as_bool();
    report.callable_error = parsed["callable_error"].as_str().map(ToOwned::to_owned);
    report
}

fn compatibility_report_json(descriptor: &ProductMountDescriptor) -> Option<Value> {
    let window = descriptor.compatibility.as_ref()?;
    let host_cli_version = crate::shared::version::runtime_semver().to_string();
    let host = Version::parse(&host_cli_version).ok()?;
    let min = Version::parse(&window.min_cli_version).ok()?;
    let max =
        window.max_cli_version_exclusive.as_ref().and_then(|value| Version::parse(value).ok());
    let mut reasons = Vec::new();
    if host < min {
        reasons.push(format!(
            "host version `{host_cli_version}` is below required minimum `{}`",
            window.min_cli_version
        ));
    }
    if let Some(max_version) = &max {
        if host >= *max_version {
            reasons.push(format!(
                "host version `{host_cli_version}` is not below exclusive maximum `{}`",
                max_version
            ));
        }
    }
    Some(json!({
        "compatible": reasons.is_empty(),
        "host_cli_version": host_cli_version,
        "min_cli_version": window.min_cli_version,
        "max_cli_version_exclusive": window.max_cli_version_exclusive,
        "reasons": reasons,
    }))
}

fn plugin_conflicts(tool: &KnownBijuxTool, plugin_registry_path: &Path) -> Vec<String> {
    let Ok(installed) = list_plugins(plugin_registry_path) else {
        return Vec::new();
    };

    installed
        .into_iter()
        .filter_map(|plugin| {
            let namespace = plugin.manifest.namespace.0;
            let collides = namespace == tool.namespace
                || tool.aliases.iter().any(|alias| *alias == namespace)
                || plugin.manifest.aliases.iter().any(|alias| {
                    alias == tool.namespace || tool.aliases.iter().any(|value| *value == alias)
                });
            collides.then_some(namespace)
        })
        .collect()
}

fn query_match(tool: &KnownBijuxTool, query: &str) -> AppQueryMatch {
    if crate::contracts::Namespace::normalize(query) == tool.namespace {
        AppQueryMatch::Namespace
    } else {
        AppQueryMatch::Alias
    }
}

fn resolution_policy(shadowed_plugins: &[String]) -> String {
    if shadowed_plugins.is_empty() {
        "standard_precedence".to_string()
    } else {
        "official_wins".to_string()
    }
}

fn is_disabled_by_registry(
    tool: &KnownBijuxTool,
    probes: &mut Vec<AppProbe>,
) -> Result<bool, String> {
    for candidate in disabled_registry_candidates() {
        if !candidate.location.exists() {
            continue;
        }

        let raw = fs::read_to_string(&candidate.location)
            .map_err(|error| format!("failed to read disabled registry: {error}"))?;
        let registry = serde_json::from_str::<DisabledProductsRegistry>(&raw)
            .map_err(|error| format!("failed to parse disabled registry JSON: {error}"))?;
        if registry.contains(tool.namespace)
            || tool.aliases.iter().any(|alias| registry.contains(alias))
        {
            probes.push(AppProbe {
                source: candidate.source,
                location: candidate.location.display().to_string(),
                status: "disabled".to_string(),
                message: "product disabled by registry".to_string(),
            });
            return Ok(true);
        }
    }

    Ok(false)
}

fn resolve_tool(
    tool: &KnownBijuxTool,
    paths: &ResolvedStatePaths,
    config: ResolutionConfig,
) -> AppResolution {
    let mut descriptor = tool.descriptor();
    let mut descriptor_source = AppDiscoverySource::CompiledOfficialRegistry;
    let mut health = AppHealth::Missing;
    let mut shadowed_plugins = Vec::new();
    let mut issues = Vec::new();
    let mut probes = Vec::new();
    let discovery_paths = discovery_paths_for(tool.namespace, paths);
    let mut disabled = false;

    match is_disabled_by_registry(tool, &mut probes) {
        Ok(true) => {
            disabled = true;
        }
        Ok(false) => {}
        Err(error) => {
            health = AppHealth::BadManifest;
            issues.push(error.clone());
            probes.push(AppProbe {
                source: AppDiscoverySource::ProjectLocal,
                location: "disabled.json".to_string(),
                status: "bad_manifest".to_string(),
                message: error,
            });
        }
    }

    if !matches!(health, AppHealth::BadManifest) {
        for candidate in override_candidates(tool.namespace) {
            if !candidate.location.exists() {
                probes.push(AppProbe {
                    source: candidate.source,
                    location: candidate.location.display().to_string(),
                    status: "missing".to_string(),
                    message: "descriptor file not present".to_string(),
                });
                continue;
            }

            match fs::read_to_string(&candidate.location) {
                Ok(raw) => {
                    if let Ok(full_descriptor) =
                        load_full_mount_descriptor(&raw, &candidate.location)
                    {
                        if full_descriptor.namespace.as_str() != tool.namespace {
                            let message = format!(
                                "descriptor namespace `{}` does not match requested namespace `{}`",
                                full_descriptor.namespace.as_str(),
                                tool.namespace
                            );
                            health = AppHealth::BadManifest;
                            issues.push(message.clone());
                            probes.push(AppProbe {
                                source: candidate.source,
                                location: candidate.location.display().to_string(),
                                status: "bad_manifest".to_string(),
                                message,
                            });
                            break;
                        }
                        descriptor = full_descriptor;
                        descriptor_source = candidate.source;
                        probes.push(AppProbe {
                            source: candidate.source,
                            location: candidate.location.display().to_string(),
                            status: "ok".to_string(),
                            message: "descriptor loaded as full mount contract".to_string(),
                        });
                        break;
                    }

                    match serde_json::from_str::<ProductMountOverride>(&raw) {
                        Ok(overlay) => {
                            match apply_override(&mut descriptor, overlay.clone(), &candidate) {
                                Ok(applied_or_disabled) => {
                                    descriptor_source = candidate.source;
                                    disabled = disabled || overlay.disabled.unwrap_or(false);
                                    probes.push(AppProbe {
                                        source: candidate.source,
                                        location: candidate.location.display().to_string(),
                                        status: "ok".to_string(),
                                        message: if disabled {
                                            "descriptor loaded and product is disabled".to_string()
                                        } else if applied_or_disabled {
                                            "descriptor loaded and applied".to_string()
                                        } else {
                                            "descriptor loaded with no changes".to_string()
                                        },
                                    });
                                    break;
                                }
                                Err(error) => {
                                    health = AppHealth::BadManifest;
                                    issues.push(error.clone());
                                    probes.push(AppProbe {
                                        source: candidate.source,
                                        location: candidate.location.display().to_string(),
                                        status: "bad_manifest".to_string(),
                                        message: error,
                                    });
                                    break;
                                }
                            }
                        }
                        Err(error) => {
                            health = AppHealth::BadManifest;
                            let message = format!("failed to parse descriptor JSON: {error}");
                            issues.push(message.clone());
                            probes.push(AppProbe {
                                source: candidate.source,
                                location: candidate.location.display().to_string(),
                                status: "bad_manifest".to_string(),
                                message,
                            });
                            break;
                        }
                    }
                }
                Err(error) => {
                    health = AppHealth::BadManifest;
                    let message = format!("failed to read descriptor file: {error}");
                    issues.push(message.clone());
                    probes.push(AppProbe {
                        source: candidate.source,
                        location: candidate.location.display().to_string(),
                        status: "bad_manifest".to_string(),
                        message,
                    });
                    break;
                }
            }
        }
    }

    if disabled {
        health = AppHealth::Disabled;
        return AppResolution {
            descriptor,
            descriptor_source,
            runtime_resolution: None,
            health,
            shadowed_plugins,
            issues,
            discovery_paths,
            probes,
            version: None,
            version_source: None,
            python: None,
        };
    }

    shadowed_plugins = plugin_conflicts(tool, &paths.plugin_registry_file);
    if !shadowed_plugins.is_empty() {
        let conflict = format!(
            "plugin registry advertises conflicting plugin namespaces: {}",
            shadowed_plugins.join(", ")
        );
        probes.push(AppProbe {
            source: AppDiscoverySource::PluginRegistry,
            location: paths.plugin_registry_file.display().to_string(),
            status: "conflict".to_string(),
            message: conflict.clone(),
        });
        issues.push(conflict);
        health = AppHealth::Conflict;
    }

    let mut python_resolution = None;

    let runtime_resolution = match descriptor.entrypoint.kind {
        ProductEntrypointKind::Binary
        | ProductEntrypointKind::PythonConsoleScript
        | ProductEntrypointKind::PluginProcess => {
            let runtime_command = descriptor.entrypoint.command.clone();
            match which_in_path(&runtime_command) {
                Some(path) if !is_executable(&path) => {
                    health = AppHealth::PermissionDenied;
                    issues.push(format!(
                        "resolved entrypoint exists but is not executable: {}",
                        path.display()
                    ));
                    probes.push(AppProbe {
                        source: AppDiscoverySource::PathFallback,
                        location: path.display().to_string(),
                        status: "permission_denied".to_string(),
                        message: "resolved entrypoint is not executable".to_string(),
                    });
                    Some(ResolvedAppCommand {
                        command: path.display().to_string(),
                        args: Vec::new(),
                        display_command: path.display().to_string(),
                        source: AppDiscoverySource::PathFallback,
                        kind: descriptor.entrypoint.kind.clone(),
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
                Some(path) => {
                    probes.push(AppProbe {
                        source: AppDiscoverySource::PathFallback,
                        location: path.display().to_string(),
                        status: "ok".to_string(),
                        message: "runtime entrypoint resolved from PATH".to_string(),
                    });
                    if !matches!(health, AppHealth::Conflict) {
                        health = AppHealth::Ok;
                    }
                    Some(ResolvedAppCommand {
                        command: path.display().to_string(),
                        args: Vec::new(),
                        display_command: path.display().to_string(),
                        source: AppDiscoverySource::PathFallback,
                        kind: descriptor.entrypoint.kind.clone(),
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
                None => {
                    probes.push(AppProbe {
                        source: AppDiscoverySource::PathFallback,
                        location: runtime_command.clone(),
                        status: "missing".to_string(),
                        message: "runtime entrypoint was not found on PATH".to_string(),
                    });
                    if !matches!(health, AppHealth::Conflict | AppHealth::BadManifest) {
                        health = AppHealth::Missing;
                    }
                    Some(ResolvedAppCommand {
                        command: runtime_command.clone(),
                        args: Vec::new(),
                        display_command: runtime_command,
                        source: AppDiscoverySource::PathFallback,
                        kind: descriptor.entrypoint.kind.clone(),
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
            }
        }
        ProductEntrypointKind::PythonModule => {
            let resolution = resolve_python_interpreter();
            if config.probe_python {
                python_resolution = Some(probe_python_doctor(&descriptor.entrypoint, &resolution));
            }
            match resolution.selected.as_ref().map(|(_, path)| path.clone()) {
                Some(path) if !is_executable(&path) => {
                    health = AppHealth::PermissionDenied;
                    issues.push(format!(
                        "resolved python interpreter exists but is not executable: {}",
                        path.display()
                    ));
                    probes.push(AppProbe {
                        source: AppDiscoverySource::PathFallback,
                        location: path.display().to_string(),
                        status: "permission_denied".to_string(),
                        message: "python interpreter is not executable".to_string(),
                    });
                    let args = python_runtime_args(&descriptor.entrypoint);
                    Some(ResolvedAppCommand {
                        command: path.display().to_string(),
                        display_command: format_display_command(&path.display().to_string(), &args),
                        args,
                        source: AppDiscoverySource::PathFallback,
                        kind: ProductEntrypointKind::PythonModule,
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
                Some(path) => {
                    let selected_source = resolution
                        .selected
                        .as_ref()
                        .map(|(source, _)| *source)
                        .unwrap_or(PythonInterpreterSource::SystemPath);
                    probes.push(AppProbe {
                    source: AppDiscoverySource::PathFallback,
                    location: path.display().to_string(),
                    status: "ok".to_string(),
                    message: format!(
                        "python module will be executed through resolved interpreter ({selected_source:?})"
                    )
                    .to_ascii_lowercase(),
                });
                    if !matches!(health, AppHealth::Conflict) {
                        health = AppHealth::Ok;
                    }
                    let args = python_runtime_args(&descriptor.entrypoint);
                    Some(ResolvedAppCommand {
                        command: path.display().to_string(),
                        display_command: format_display_command(&path.display().to_string(), &args),
                        args,
                        source: AppDiscoverySource::PathFallback,
                        kind: ProductEntrypointKind::PythonModule,
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
                None => {
                    for attempt in &resolution.attempts {
                        probes.push(AppProbe {
                            source: AppDiscoverySource::PathFallback,
                            location: attempt.location.clone(),
                            status: attempt.status.clone(),
                            message: format!(
                                "{} ({})",
                                attempt.message,
                                serde_json::to_string(&attempt.source)
                                    .unwrap_or_else(|_| "\"unknown\"".to_string())
                                    .trim_matches('"')
                            ),
                        });
                    }
                    if !matches!(health, AppHealth::Conflict | AppHealth::BadManifest) {
                        health = AppHealth::Missing;
                    }
                    let command =
                        env::var(BIJUX_PYTHON_BIN).unwrap_or_else(|_| "python3".to_string());
                    let args = python_runtime_args(&descriptor.entrypoint);
                    Some(ResolvedAppCommand {
                        display_command: format_display_command(&command, &args),
                        command,
                        args,
                        source: AppDiscoverySource::PathFallback,
                        kind: ProductEntrypointKind::PythonModule,
                        namespace: tool.namespace.to_string(),
                        descriptor: descriptor.clone(),
                    })
                }
            }
        }
        ProductEntrypointKind::EmbeddedRust => {
            probes.push(AppProbe {
                source: descriptor_source,
                location: format!("embedded:{}", descriptor.entrypoint.command),
                status: "ok".to_string(),
                message: "embedded runtime handler is active".to_string(),
            });
            if !matches!(health, AppHealth::Conflict) {
                health = AppHealth::Ok;
            }
            Some(ResolvedAppCommand {
                command: descriptor.entrypoint.command.clone(),
                args: Vec::new(),
                display_command: format!("embedded:{}", descriptor.entrypoint.command),
                source: descriptor_source,
                kind: ProductEntrypointKind::EmbeddedRust,
                namespace: tool.namespace.to_string(),
                descriptor: descriptor.clone(),
            })
        }
    };

    let mut version = descriptor.version.clone();
    let mut version_source = descriptor.version.as_ref().map(|_| "manifest".to_string());

    if config.probe_version
        && version.is_none()
        && matches!(health, AppHealth::Ok | AppHealth::Conflict)
    {
        if let Some(invocation) = runtime_resolution.as_ref() {
            if !matches!(invocation.kind, ProductEntrypointKind::EmbeddedRust) {
                version = probe_version(invocation);
                if version.is_some() {
                    version_source = Some("binary_probe".to_string());
                }
            }
        }
    }

    if config.probe_version
        && descriptor.version.is_some()
        && matches!(health, AppHealth::Ok | AppHealth::Conflict)
    {
        if let Some(invocation) = runtime_resolution.as_ref() {
            if !matches!(invocation.kind, ProductEntrypointKind::EmbeddedRust) {
                if let Some(probed_version) = probe_version(invocation) {
                    if descriptor.version.as_deref() != Some(probed_version.as_str()) {
                        health = AppHealth::VersionMismatch;
                        issues.push(format!(
                            "manifest version `{}` does not match runtime probe `{}`",
                            descriptor.version.as_deref().unwrap_or_default(),
                            probed_version
                        ));
                    }
                }
            }
        }
    }

    AppResolution {
        descriptor,
        descriptor_source,
        runtime_resolution,
        health,
        shadowed_plugins,
        issues,
        discovery_paths,
        probes,
        version,
        version_source,
        python: python_resolution,
    }
}

fn status_from_healths(healths: &[AppHealth]) -> String {
    if healths.iter().any(|health| matches!(health, AppHealth::BadManifest | AppHealth::Conflict)) {
        "degraded".to_string()
    } else if healths.iter().any(|health| !matches!(health, AppHealth::Ok)) {
        "warning".to_string()
    } else {
        "ok".to_string()
    }
}

fn mount_report(tool: &KnownBijuxTool, resolution: AppResolution) -> AppMountReport {
    let resolution_policy = resolution_policy(&resolution.shadowed_plugins);
    AppMountReport {
        namespace: tool.namespace.to_string(),
        display_name: resolution.descriptor.display_name,
        aliases: resolution.descriptor.aliases.into_iter().map(|alias| alias.0).collect(),
        source: resolution.descriptor_source,
        entrypoint_kind: resolution.descriptor.entrypoint.kind.clone(),
        entrypoint: resolution.descriptor.entrypoint.command,
        status: tool.status.to_string(),
        version: resolution.version,
        version_source: resolution.version_source,
        health: resolution.health,
        resolved_entrypoint: resolution.runtime_resolution.map(|value| value.display_command),
        resolution_policy,
        shadowed_plugins: resolution.shadowed_plugins,
        capabilities: resolution.descriptor.capabilities,
        discovery_paths: resolution.discovery_paths,
        probes: resolution.probes,
        issues: resolution.issues,
        help_summary: resolution.descriptor.help.summary,
        python: resolution.python,
    }
}

fn app_summary(apps: &[AppMountReport]) -> AppMountSummary {
    let mut summary = AppMountSummary {
        ok: 0,
        missing: 0,
        version_mismatch: 0,
        permission_denied: 0,
        bad_manifest: 0,
        conflict: 0,
        disabled: 0,
    };
    for app in apps {
        match app.health {
            AppHealth::Ok => summary.ok += 1,
            AppHealth::Missing => summary.missing += 1,
            AppHealth::VersionMismatch => summary.version_mismatch += 1,
            AppHealth::PermissionDenied => summary.permission_denied += 1,
            AppHealth::BadManifest => summary.bad_manifest += 1,
            AppHealth::Conflict => summary.conflict += 1,
            AppHealth::Disabled => summary.disabled += 1,
        }
    }
    summary
}

/// Resolve the runtime command used for official app delegation.
pub fn resolve_runtime_command(query: &str) -> Option<ResolvedAppCommand> {
    if let Some(tool) = known_bijux_tool_by_query(query) {
        let paths = ResolvedStatePaths {
            config_file: PathBuf::new(),
            history_file: PathBuf::new(),
            plugins_dir: PathBuf::new(),
            plugin_registry_file: PathBuf::new(),
            memory_file: PathBuf::new(),
            compatibility_config_file: PathBuf::new(),
            compatibility_config_warning: None,
        };
        let resolution = resolve_tool(
            tool,
            &paths,
            ResolutionConfig { probe_version: false, probe_python: false },
        );
        return resolution.runtime_resolution.or_else(|| {
            Some(resolve_descriptor_runtime_command(
                resolution.descriptor,
                resolution.descriptor_source,
            ))
        });
    }

    discover_custom_mount(query)
        .map(|(descriptor, source)| resolve_descriptor_runtime_command(descriptor, source))
}

/// Resolve the control-plane command used for official app delegation.
pub fn resolve_control_command(query: &str) -> Option<ResolvedAppCommand> {
    if let Some(tool) = known_bijux_tool_by_query(query) {
        return Some(resolve_descriptor_control_command(
            tool.descriptor(),
            AppDiscoverySource::CompiledOfficialRegistry,
        ));
    }

    discover_custom_mount(query)
        .map(|(descriptor, source)| resolve_descriptor_control_command(descriptor, source))
}

/// Build root `apps list` output.
pub fn apps_list_report(
    paths: &ResolvedStatePaths,
    _plugin_registry_path: &Path,
) -> AppsListReport {
    let apps = known_bijux_tools()
        .iter()
        .map(|tool| {
            mount_report(
                tool,
                resolve_tool(
                    tool,
                    paths,
                    ResolutionConfig { probe_version: false, probe_python: false },
                ),
            )
        })
        .collect::<Vec<_>>();
    let healths = apps.iter().map(|item| item.health).collect::<Vec<_>>();
    AppsListReport {
        status: status_from_healths(&healths),
        precedence: precedence_names(),
        discovery_paths: known_bijux_tools()
            .first()
            .map(|tool| discovery_paths_for(tool.namespace, paths))
            .unwrap_or_default(),
        apps,
    }
}

/// Build root `apps doctor` output.
pub fn apps_doctor_report(
    paths: &ResolvedStatePaths,
    _plugin_registry_path: &Path,
) -> AppsDoctorReport {
    let apps = known_bijux_tools()
        .iter()
        .map(|tool| {
            mount_report(
                tool,
                resolve_tool(
                    tool,
                    paths,
                    ResolutionConfig { probe_version: true, probe_python: true },
                ),
            )
        })
        .collect::<Vec<_>>();
    let healths = apps.iter().map(|item| item.health).collect::<Vec<_>>();
    AppsDoctorReport {
        status: status_from_healths(&healths),
        precedence: precedence_names(),
        discovery_paths: known_bijux_tools()
            .first()
            .map(|tool| discovery_paths_for(tool.namespace, paths))
            .unwrap_or_default(),
        summary: app_summary(&apps),
        apps,
    }
}

/// Build root `apps doctor <query>` output.
pub fn app_doctor_report(
    query: &str,
    paths: &ResolvedStatePaths,
) -> Result<AppDoctorReport, String> {
    let tool =
        known_bijux_tool_by_query(query).ok_or_else(|| format!("unknown official app: {query}"))?;
    let resolution =
        resolve_tool(tool, paths, ResolutionConfig { probe_version: true, probe_python: true });
    Ok(AppDoctorReport {
        status: status_from_healths(&[resolution.health]),
        query: query.to_string(),
        matched_via: query_match(tool, query),
        namespace: tool.namespace.to_string(),
        app: mount_report(tool, resolution),
    })
}

/// Build root `apps which` output.
pub fn app_which_report(query: &str, paths: &ResolvedStatePaths) -> Result<AppWhichReport, String> {
    let tool =
        known_bijux_tool_by_query(query).ok_or_else(|| format!("unknown official app: {query}"))?;
    let resolution =
        resolve_tool(tool, paths, ResolutionConfig { probe_version: false, probe_python: false });
    let status = status_from_healths(&[resolution.health]);
    let resolution_policy = resolution_policy(&resolution.shadowed_plugins);
    Ok(AppWhichReport {
        status,
        query: query.to_string(),
        matched_via: query_match(tool, query),
        namespace: tool.namespace.to_string(),
        source: resolution.descriptor_source,
        entrypoint_kind: resolution.descriptor.entrypoint.kind.clone(),
        resolved_entrypoint: resolution.runtime_resolution.map(|value| value.display_command),
        health: resolution.health,
        resolution_policy,
        shadowed_plugins: resolution.shadowed_plugins,
        discovery_paths: resolution.discovery_paths,
        probes: resolution.probes,
        issues: resolution.issues,
    })
}

/// Build root `apps version` output.
pub fn app_version_report(
    query: &str,
    paths: &ResolvedStatePaths,
) -> Result<AppVersionReport, String> {
    let tool =
        known_bijux_tool_by_query(query).ok_or_else(|| format!("unknown official app: {query}"))?;
    let resolution =
        resolve_tool(tool, paths, ResolutionConfig { probe_version: true, probe_python: false });
    Ok(AppVersionReport {
        status: status_from_healths(&[resolution.health]),
        query: query.to_string(),
        matched_via: query_match(tool, query),
        namespace: tool.namespace.to_string(),
        version: resolution.version,
        source: resolution.version_source,
        health: resolution.health,
        issues: resolution.issues,
    })
}

/// Build root `apps capabilities` output.
pub fn app_capabilities_report(
    query: &str,
    paths: &ResolvedStatePaths,
) -> Result<AppCapabilitiesReport, String> {
    let tool =
        known_bijux_tool_by_query(query).ok_or_else(|| format!("unknown official app: {query}"))?;
    let resolution =
        resolve_tool(tool, paths, ResolutionConfig { probe_version: false, probe_python: false });
    Ok(AppCapabilitiesReport {
        status: status_from_healths(&[resolution.health]),
        query: query.to_string(),
        matched_via: query_match(tool, query),
        namespace: tool.namespace.to_string(),
        entrypoint_kind: resolution.descriptor.entrypoint.kind.clone(),
        capabilities: resolution.descriptor.capabilities,
        aliases: resolution.descriptor.aliases.into_iter().map(|alias| alias.0).collect(),
        source: resolution.descriptor_source,
        health: resolution.health,
        help_summary: resolution.descriptor.help.summary,
    })
}

/// Build root `apps schema` output.
pub fn app_manifest_schema_report() -> AppManifestSchemaReport {
    AppManifestSchemaReport {
        status: "ok".to_string(),
        schema: "product-mount-descriptor-v1".to_string(),
        schema_json: json!(product_mount_descriptor_schema()),
        entrypoint_kinds: vec![
            "binary".to_string(),
            "python_module".to_string(),
            "python_console_script".to_string(),
            "plugin_process".to_string(),
            "embedded_rust".to_string(),
        ],
    }
}

/// Validate a full app manifest contract on disk.
pub fn validate_app_manifest_report(path: &Path) -> AppManifestValidationReport {
    match read_mount_descriptor(path) {
        Ok(descriptor) => AppManifestValidationReport {
            status: "ok".to_string(),
            path: path.display().to_string(),
            valid: true,
            namespace: Some(descriptor.namespace.as_str().to_string()),
            entrypoint_kind: Some(descriptor.entrypoint.kind.clone()),
            python_module: descriptor.entrypoint.module.clone(),
            python_function: descriptor.entrypoint.function.clone(),
            compatibility: compatibility_report_json(&descriptor),
            issues: Vec::new(),
        },
        Err(error) => AppManifestValidationReport {
            status: "invalid".to_string(),
            path: path.display().to_string(),
            valid: false,
            namespace: None,
            entrypoint_kind: None,
            python_module: None,
            python_function: None,
            compatibility: None,
            issues: vec![error],
        },
    }
}

fn rust_app_entrypoint_script(binary_name: &str) -> String {
    format!(
        "#!/usr/bin/env sh\nset -eu\n\nSCRIPT_DIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\ncd \"$SCRIPT_DIR\"\n\nexport CARGO_TARGET_DIR=\"$SCRIPT_DIR/artifacts/rust-target\"\nBIN_PATH=\"$CARGO_TARGET_DIR/debug/{binary_name}\"\nif [ ! -x \"$BIN_PATH\" ] || [ \"$SCRIPT_DIR/Cargo.toml\" -nt \"$BIN_PATH\" ] || [ -d \"$SCRIPT_DIR/src\" ] && find \"$SCRIPT_DIR/src\" -type f -name '*.rs' -newer \"$BIN_PATH\" -print -quit | grep -q .; then\n  cargo build --quiet --locked\nfi\nexec \"$BIN_PATH\" \"$@\"\n",
    )
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("failed to set executable bit on `{}`: {error}", path.display()))
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

/// Generate a starter app mount and entrypoint surface.
pub fn scaffold_app_mount(
    kind: &str,
    namespace: &str,
    force: bool,
    target_root: &Path,
) -> Result<AppScaffoldReport, String> {
    let namespace = Namespace::new(namespace)?;
    if RESERVED_APP_NAMESPACES.contains(&namespace.as_str()) {
        return Err(format!("app namespace is reserved: {}", namespace.as_str()));
    }
    if known_bijux_tool_by_query(namespace.as_str()).is_some() {
        return Err(format!(
            "app namespace collides with an official product mount: {}",
            namespace.as_str()
        ));
    }
    if !is_safe_scaffold_path(target_root) {
        return Err("scaffold path is unsafe".to_string());
    }

    if target_root.exists() {
        if !force {
            return Err("scaffold path already exists; pass --force to overwrite".to_string());
        }
        if target_root.is_dir() {
            fs::remove_dir_all(target_root).map_err(|error| {
                format!(
                    "failed to remove existing scaffold directory `{}`: {error}",
                    target_root.display()
                )
            })?;
        } else {
            fs::remove_file(target_root).map_err(|error| {
                format!(
                    "failed to remove existing scaffold file `{}`: {error}",
                    target_root.display()
                )
            })?;
        }
    }

    fs::create_dir_all(target_root).map_err(|error| {
        format!("failed to create scaffold root `{}`: {error}", target_root.display())
    })?;
    let apps_dir = target_root.join(".bijux/apps");
    fs::create_dir_all(&apps_dir).map_err(|error| {
        format!("failed to create app manifest directory `{}`: {error}", apps_dir.display())
    })?;

    let descriptor = match kind {
        "python" => {
            let module_name = app_module_name(namespace.as_str());
            let module_dir = target_root.join(&module_name);
            fs::create_dir_all(&module_dir).map_err(|error| {
                format!(
                    "failed to create python module directory `{}`: {error}",
                    module_dir.display()
                )
            })?;
            fs::write(
                target_root.join("pyproject.toml"),
                format!(
                    "[project]\nname = \"bijux-{namespace}-app\"\nversion = \"{APP_SCAFFOLD_VERSION}\"\ndescription = \"Scaffolded Bijux Python app for {namespace}\"\nrequires-python = \">=3.11\"\ndependencies = [\"bijux-cli>=0.3\"]\n\n[project.scripts]\n{namespace} = \"{module_name}.__main__:entrypoint\"\n",
                    namespace = namespace.as_str(),
                    module_name = module_name,
                ),
            )
            .map_err(|error| format!("failed to write python pyproject.toml: {error}"))?;
            fs::write(
                module_dir.join("__init__.py"),
                "\"\"\"Scaffolded Bijux app package.\"\"\"\n",
            )
            .map_err(|error| format!("failed to write python module init: {error}"))?;
            fs::write(
                module_dir.join("cli.py"),
                format!(
                    "from bijux_cli_py.app_sdk import compatibility_report, success\n\n\ndef main(argv: list[str]):\n    if any(arg in ('--help', '-h', 'help') for arg in argv):\n        return success({{'usage': 'bijux {namespace} [ARGS]', 'summary': 'Scaffolded Python app mount for {namespace}.'}}, command=['{namespace}', 'help'])\n    if argv[:1] == ['version']:\n        return success({{'namespace': '{namespace}', 'version': '{APP_SCAFFOLD_VERSION}'}}, command=['{namespace}', 'version'])\n    if argv[:1] == ['compatibility']:\n        return success(compatibility_report('{min_cli_version}'), command=['{namespace}', 'compatibility'])\n    return success({{'status': 'ok', 'namespace': '{namespace}', 'argv': argv}}, command=['{namespace}'])\n",
                    namespace = namespace.as_str(),
                    min_cli_version = crate::shared::version::runtime_semver(),
                ),
            )
            .map_err(|error| format!("failed to write python module cli: {error}"))?;
            fs::write(
                module_dir.join("__main__.py"),
                "import sys\n\nfrom bijux_cli_py.app_sdk import run_json_app\n\nfrom .cli import main\n\n\ndef entrypoint() -> int:\n    return run_json_app(main, argv=sys.argv[1:])\n\n\nif __name__ == '__main__':\n    raise SystemExit(entrypoint())\n",
            )
            .map_err(|error| format!("failed to write python module main: {error}"))?;

            ProductMount::new(namespace.as_str())?
                .display_name(format!("{} App", namespace.as_str()))
                .python_callable(format!("{module_name}.cli"), "main")
                .control_python_callable(format!("{module_name}.cli"), "main")
                .summary(format!("Scaffolded Python app for {}", namespace.as_str()))
                .capability("json_output")
                .feature_capabilities(FeatureCapabilityDeclaration {
                    supports_completion: true,
                    ..FeatureCapabilityDeclaration::default()
                })
                .version(APP_SCAFFOLD_VERSION)
                .compatibility(crate::sdk::SdkCompatibilityWindow::new(
                    crate::shared::version::runtime_semver().to_string(),
                    None,
                )?)
                .build_descriptor()?
        }
        "rust" => {
            let binary_name = rust_app_entrypoint_name(namespace.as_str());
            let src_dir = target_root.join("src");
            fs::create_dir_all(&src_dir).map_err(|error| {
                format!("failed to create Rust source directory `{}`: {error}", src_dir.display())
            })?;
            fs::write(
                target_root.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"{binary_name}\"\nversion = \"{APP_SCAFFOLD_VERSION}\"\nedition = \"2021\"\nlicense = \"Apache-2.0\"\ndescription = \"Scaffolded Bijux app for {}\"\n\n[dependencies]\nserde_json = \"1\"\n",
                    namespace.as_str(),
                ),
            )
            .map_err(|error| format!("failed to write Cargo.toml: {error}"))?;
            fs::write(
                src_dir.join("main.rs"),
                format!(
                    "fn main() {{\n    let argv = std::env::args().skip(1).collect::<Vec<_>>();\n    if argv.iter().any(|arg| matches!(arg.as_str(), \"--help\" | \"-h\" | \"help\")) {{\n        println!(\"Usage: bijux {} [ARGS]\\n\\nScaffolded Rust app mount for {}.\");\n        return;\n    }}\n    if argv.first().is_some_and(|arg| arg == \"version\") {{\n        println!(\"{} {APP_SCAFFOLD_VERSION}\");\n        return;\n    }}\n    println!(\"{{}}\", serde_json::to_string_pretty(&serde_json::json!({{\"status\": \"ok\", \"namespace\": \"{}\", \"argv\": argv}})).expect(\"json\"));\n}}\n",
                    namespace.as_str(),
                    namespace.as_str(),
                    namespace.as_str(),
                    namespace.as_str(),
                ),
            )
            .map_err(|error| format!("failed to write Rust main.rs: {error}"))?;
            let wrapper = target_root.join(&binary_name);
            fs::write(&wrapper, rust_app_entrypoint_script(&binary_name))
                .map_err(|error| format!("failed to write Rust app entrypoint wrapper: {error}"))?;
            mark_executable(&wrapper)?;

            ProductMount::new(namespace.as_str())?
                .display_name(format!("{} App", namespace.as_str()))
                .plugin_process(format!("../../{binary_name}"))
                .control_plugin_process(format!("../../{binary_name}"))
                .summary(format!("Scaffolded Rust app for {}", namespace.as_str()))
                .capability("json_output")
                .feature_capabilities(FeatureCapabilityDeclaration {
                    supports_completion: true,
                    supports_repl: true,
                    ..FeatureCapabilityDeclaration::default()
                })
                .version(APP_SCAFFOLD_VERSION)
                .build_descriptor()?
        }
        other => {
            return Err(format!("app scaffold kind must be one of: python, rust; got `{other}`"))
        }
    };

    let manifest_path = apps_dir.join(format!("{}.mount.json", namespace.as_str()));
    let manifest_json = serde_json::to_string_pretty(&descriptor)
        .map_err(|error| format!("failed to serialize app manifest: {error}"))?;
    fs::write(&manifest_path, format!("{manifest_json}\n")).map_err(|error| {
        format!("failed to write app manifest `{}`: {error}", manifest_path.display())
    })?;

    let mut files = vec![manifest_path.display().to_string()];
    if kind == "python" {
        let module_name = app_module_name(namespace.as_str());
        files.push(target_root.join("pyproject.toml").display().to_string());
        files.push(target_root.join(&module_name).join("__init__.py").display().to_string());
        files.push(target_root.join(&module_name).join("cli.py").display().to_string());
        files.push(target_root.join(module_name).join("__main__.py").display().to_string());
    } else {
        files.push(target_root.join("Cargo.toml").display().to_string());
        files.push(target_root.join("src/main.rs").display().to_string());
        files.push(
            target_root.join(rust_app_entrypoint_name(namespace.as_str())).display().to_string(),
        );
    }
    files.sort();

    Ok(AppScaffoldReport {
        status: "scaffolded".to_string(),
        kind: kind.to_string(),
        namespace: namespace.as_str().to_string(),
        root: target_root.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        entrypoint_kind: descriptor.entrypoint.kind.clone(),
        entrypoint: descriptor.entrypoint.command.clone(),
        files,
        guidance: vec![
            format!("Run `cd {}` before invoking the scaffolded app mount.", target_root.display()),
            format!(
                "Invoke `bijux {} --help` to exercise the mounted surface.",
                namespace.as_str()
            ),
        ],
    })
}
