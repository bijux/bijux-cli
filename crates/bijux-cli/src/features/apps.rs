#![forbid(unsafe_code)]
//! Official app discovery, resolution, and health reporting.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::Serialize;

use crate::contracts::{
    known_bijux_tool_by_query, known_bijux_tools, KnownBijuxTool, ProductEntrypoint,
    ProductEntrypointKind, ProductHelpMetadata, ProductMountDescriptor,
};
use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::plugins::list_plugins;

const DEFAULT_SYSTEM_APPS_DIR: &str = "/etc/bijux/apps";
const BIJUX_APP_PATH: &str = "BIJUX_APP_PATH";
const BIJUX_SYSTEM_APP_PATH: &str = "BIJUX_SYSTEM_APP_PATH";

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
pub struct AppMountReport {
    pub namespace: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub source: AppDiscoverySource,
    pub entrypoint: String,
    pub status: String,
    pub version: Option<String>,
    pub version_source: Option<String>,
    pub health: AppHealth,
    pub resolved_entrypoint: Option<String>,
    pub capabilities: Vec<String>,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub probes: Vec<AppProbe>,
    pub issues: Vec<String>,
    pub help_summary: String,
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
pub struct AppWhichReport {
    pub status: String,
    pub query: String,
    pub namespace: String,
    pub source: AppDiscoverySource,
    pub resolved_entrypoint: Option<String>,
    pub health: AppHealth,
    pub discovery_paths: Vec<AppDiscoveryPath>,
    pub probes: Vec<AppProbe>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppVersionReport {
    pub status: String,
    pub query: String,
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
    pub namespace: String,
    pub capabilities: Vec<String>,
    pub aliases: Vec<String>,
    pub source: AppDiscoverySource,
    pub health: AppHealth,
    pub help_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAppCommand {
    pub command: String,
    pub source: AppDiscoverySource,
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

#[derive(Debug, Clone)]
struct OverrideCandidate {
    source: AppDiscoverySource,
    location: PathBuf,
}

#[derive(Debug, Clone)]
struct ResolutionConfig {
    probe_version: bool,
}

#[derive(Debug, Clone)]
struct AppResolution {
    descriptor: ProductMountDescriptor,
    descriptor_source: AppDiscoverySource,
    runtime_resolution: Option<ResolvedAppCommand>,
    health: AppHealth,
    issues: Vec<String>,
    discovery_paths: Vec<AppDiscoveryPath>,
    probes: Vec<AppProbe>,
    version: Option<String>,
    version_source: Option<String>,
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

fn descriptor_path(dir: &Path, namespace: &str) -> PathBuf {
    dir.join(format!("{namespace}.json"))
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
            location: descriptor_path(&cwd.join(".bijux/apps"), namespace).display().to_string(),
            precedence: 1,
        });
    }

    for directory in env_apps_dirs() {
        rows.push(AppDiscoveryPath {
            source: AppDiscoverySource::EnvironmentPath,
            location: descriptor_path(&directory, namespace).display().to_string(),
            precedence: 2,
        });
    }

    if let Some(directory) = user_apps_dir() {
        rows.push(AppDiscoveryPath {
            source: AppDiscoverySource::UserLocal,
            location: descriptor_path(&directory, namespace).display().to_string(),
            precedence: 3,
        });
    }

    rows.push(AppDiscoveryPath {
        source: AppDiscoverySource::SystemLocal,
        location: descriptor_path(&system_apps_dir(), namespace).display().to_string(),
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
        rows.push(OverrideCandidate {
            source: AppDiscoverySource::ProjectLocal,
            location: descriptor_path(&cwd.join(".bijux/apps"), namespace),
        });
    }
    rows.extend(env_apps_dirs().into_iter().map(|directory| OverrideCandidate {
        source: AppDiscoverySource::EnvironmentPath,
        location: descriptor_path(&directory, namespace),
    }));
    if let Some(directory) = user_apps_dir() {
        rows.push(OverrideCandidate {
            source: AppDiscoverySource::UserLocal,
            location: descriptor_path(&directory, namespace),
        });
    }
    rows.push(OverrideCandidate {
        source: AppDiscoverySource::SystemLocal,
        location: descriptor_path(&system_apps_dir(), namespace),
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
    if matches!(entrypoint.kind, ProductEntrypointKind::Binary) {
        let path = Path::new(&entrypoint.command);
        if path.components().count() > 1 && path.is_relative() {
            if let Some(base) = base_dir {
                entrypoint.command = base.join(path).display().to_string();
            }
        }
    }
    entrypoint
}

fn probe_version(command: &str) -> Option<String> {
    let output = std::process::Command::new(command).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}

fn plugin_conflict_issue(tool: &KnownBijuxTool, plugin_registry_path: &Path) -> Option<String> {
    let installed = list_plugins(plugin_registry_path).ok()?;
    for plugin in installed {
        let namespace = plugin.manifest.namespace.0;
        if namespace == tool.namespace
            || tool.aliases.iter().any(|alias| *alias == namespace)
            || plugin.manifest.aliases.iter().any(|alias| {
                alias == tool.namespace || tool.aliases.iter().any(|value| *value == alias)
            })
        {
            return Some(format!(
                "plugin registry advertises `{}` and collides with official app `{}`",
                namespace, tool.namespace
            ));
        }
    }
    None
}

fn resolve_tool(
    tool: &KnownBijuxTool,
    paths: &ResolvedStatePaths,
    config: ResolutionConfig,
) -> AppResolution {
    let mut descriptor = tool.descriptor();
    let mut descriptor_source = AppDiscoverySource::CompiledOfficialRegistry;
    let mut health = AppHealth::Missing;
    let mut issues = Vec::new();
    let mut probes = Vec::new();
    let discovery_paths = discovery_paths_for(tool.namespace, paths);
    let mut disabled = false;

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
            Ok(raw) => match serde_json::from_str::<ProductMountOverride>(&raw) {
                Ok(overlay) => match apply_override(&mut descriptor, overlay.clone(), &candidate) {
                    Ok(applied_or_disabled) => {
                        descriptor_source = candidate.source;
                        disabled = overlay.disabled.unwrap_or(false);
                        probes.push(AppProbe {
                            source: candidate.source,
                            location: candidate.location.display().to_string(),
                            status: "ok".to_string(),
                            message: if disabled {
                                "descriptor loaded and explicitly disabled".to_string()
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
                },
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
            },
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

    if disabled {
        health = AppHealth::Disabled;
        return AppResolution {
            descriptor,
            descriptor_source,
            runtime_resolution: None,
            health,
            issues,
            discovery_paths,
            probes,
            version: None,
            version_source: None,
        };
    }

    if let Some(conflict) = plugin_conflict_issue(tool, &paths.plugin_registry_file) {
        probes.push(AppProbe {
            source: AppDiscoverySource::PluginRegistry,
            location: paths.plugin_registry_file.display().to_string(),
            status: "conflict".to_string(),
            message: conflict.clone(),
        });
        issues.push(conflict);
        health = AppHealth::Conflict;
    }

    let runtime_command = descriptor.entrypoint.command.clone();
    let resolved_binary = which_in_path(&runtime_command);
    let runtime_resolution = resolved_binary.as_ref().map(|path| ResolvedAppCommand {
        command: path.display().to_string(),
        source: AppDiscoverySource::PathFallback,
    });

    let mut version = descriptor.version.clone();
    let mut version_source = descriptor.version.as_ref().map(|_| "manifest".to_string());

    match resolved_binary.as_ref() {
        Some(path) if !is_executable(path) => {
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
        }
        Some(path) => {
            probes.push(AppProbe {
                source: AppDiscoverySource::PathFallback,
                location: path.display().to_string(),
                status: "ok".to_string(),
                message: "runtime entrypoint resolved from PATH".to_string(),
            });
            if config.probe_version && version.is_none() {
                version = probe_version(&path.display().to_string());
                if version.is_some() {
                    version_source = Some("binary_probe".to_string());
                }
            }
            if !matches!(health, AppHealth::Conflict) {
                health = AppHealth::Ok;
            }
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
        }
    }

    if config.probe_version
        && descriptor.version.is_some()
        && resolved_binary.is_some()
        && matches!(health, AppHealth::Ok | AppHealth::Conflict)
    {
        if let Some(probed_version) = probe_version(&runtime_command) {
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

    AppResolution {
        descriptor,
        descriptor_source,
        runtime_resolution,
        health,
        issues,
        discovery_paths,
        probes,
        version,
        version_source,
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
    AppMountReport {
        namespace: tool.namespace.to_string(),
        display_name: resolution.descriptor.display_name,
        aliases: resolution.descriptor.aliases.into_iter().map(|alias| alias.0).collect(),
        source: resolution.descriptor_source,
        entrypoint: resolution.descriptor.entrypoint.command,
        status: tool.status.to_string(),
        version: resolution.version,
        version_source: resolution.version_source,
        health: resolution.health,
        resolved_entrypoint: resolution.runtime_resolution.map(|value| value.command),
        capabilities: resolution.descriptor.capabilities,
        discovery_paths: resolution.discovery_paths,
        probes: resolution.probes,
        issues: resolution.issues,
        help_summary: resolution.descriptor.help.summary,
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
    let tool = known_bijux_tool_by_query(query)?;
    let paths = ResolvedStatePaths {
        config_file: PathBuf::new(),
        history_file: PathBuf::new(),
        plugins_dir: PathBuf::new(),
        plugin_registry_file: PathBuf::new(),
        memory_file: PathBuf::new(),
        compatibility_config_file: PathBuf::new(),
        compatibility_config_warning: None,
    };
    let resolution = resolve_tool(tool, &paths, ResolutionConfig { probe_version: false });
    resolution.runtime_resolution.or_else(|| {
        Some(ResolvedAppCommand {
            command: resolution.descriptor.entrypoint.command,
            source: resolution.descriptor_source,
        })
    })
}

/// Resolve the control-plane command used for official app delegation.
pub fn resolve_control_command(query: &str) -> Option<ResolvedAppCommand> {
    let tool = known_bijux_tool_by_query(query)?;
    let command = tool.descriptor().control_entrypoint.command;
    let resolved =
        which_in_path(&command).map(|path| path.display().to_string()).unwrap_or(command);
    Some(ResolvedAppCommand { command: resolved, source: AppDiscoverySource::PathFallback })
}

/// Build root `apps list` output.
pub fn apps_list_report(
    paths: &ResolvedStatePaths,
    _plugin_registry_path: &Path,
) -> AppsListReport {
    let apps = known_bijux_tools()
        .iter()
        .map(|tool| {
            mount_report(tool, resolve_tool(tool, paths, ResolutionConfig { probe_version: false }))
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
            mount_report(tool, resolve_tool(tool, paths, ResolutionConfig { probe_version: true }))
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

/// Build root `apps which` output.
pub fn app_which_report(query: &str, paths: &ResolvedStatePaths) -> Result<AppWhichReport, String> {
    let tool =
        known_bijux_tool_by_query(query).ok_or_else(|| format!("unknown official app: {query}"))?;
    let resolution = resolve_tool(tool, paths, ResolutionConfig { probe_version: false });
    let status = status_from_healths(&[resolution.health]);
    Ok(AppWhichReport {
        status,
        query: query.to_string(),
        namespace: tool.namespace.to_string(),
        source: resolution.descriptor_source,
        resolved_entrypoint: resolution.runtime_resolution.map(|value| value.command),
        health: resolution.health,
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
    let resolution = resolve_tool(tool, paths, ResolutionConfig { probe_version: true });
    Ok(AppVersionReport {
        status: status_from_healths(&[resolution.health]),
        query: query.to_string(),
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
    let resolution = resolve_tool(tool, paths, ResolutionConfig { probe_version: false });
    Ok(AppCapabilitiesReport {
        status: status_from_healths(&[resolution.health]),
        query: query.to_string(),
        namespace: tool.namespace.to_string(),
        capabilities: resolution.descriptor.capabilities,
        aliases: resolution.descriptor.aliases.into_iter().map(|alias| alias.0).collect(),
        source: resolution.descriptor_source,
        health: resolution.health,
        help_summary: resolution.descriptor.help.summary,
    })
}
