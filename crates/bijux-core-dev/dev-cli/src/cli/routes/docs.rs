use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use crate::cli::args::{command_option_value, command_positionals_with_options};
use crate::cli::workspace::workspace_root;
use crate::infra::artifacts::replace_directory_tree;

pub(super) fn try_handle(normalized_path: &[String], argv: &[String]) -> Result<Option<Value>> {
    let payload = match normalized_path {
        [group, command] if group == "docs" && command == "publish-contract-assets" => {
            let extra_positionals = command_positionals_with_options(
                argv,
                &["docs", "publish-contract-assets"],
                &["--site-dir"],
            );
            if !extra_positionals.is_empty() {
                return Err(anyhow!(
                    "Invalid argument: docs publish-contract-assets does not accept positional arguments"
                ));
            }

            let site_dir =
                command_option_value(argv, &["docs", "publish-contract-assets"], "--site-dir")
                    .ok_or_else(|| anyhow!("Missing argument: --site-dir required"))?;
            if site_dir.trim().is_empty() {
                return Err(anyhow!("Invalid argument: --site-dir cannot be empty"));
            }

            let workspace_root = workspace_root();
            let source_dir = workspace_root.join("contracts");
            let destination_dir =
                resolve_site_dir(&workspace_root, Path::new(&site_dir)).join("contracts");
            let copied_file_count = replace_directory_tree(&source_dir, &destination_dir)
                .with_context(|| {
                    format!(
                        "failed to publish contract assets from {} to {}",
                        source_dir.display(),
                        destination_dir.display(),
                    )
                })?;

            json!({
                "status": "ok",
                "command": "bijux-dev-cli docs publish-contract-assets",
                "source_dir": render_path(&source_dir, &workspace_root),
                "destination_dir": render_path(&destination_dir, &workspace_root),
                "copied_file_count": copied_file_count,
            })
        }
        _ => return Ok(None),
    };

    Ok(Some(payload))
}

fn resolve_site_dir(workspace_root: &Path, site_dir: &Path) -> PathBuf {
    if site_dir.is_absolute() {
        site_dir.to_path_buf()
    } else {
        workspace_root.join(site_dir)
    }
}

fn render_path(path: &Path, workspace_root: &Path) -> String {
    path.strip_prefix(workspace_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{render_path, resolve_site_dir};
    use std::path::Path;

    #[test]
    fn resolve_site_dir_keeps_absolute_paths() {
        let workspace_root = Path::new("/workspace");
        let site_dir = Path::new("/tmp/site");

        assert_eq!(resolve_site_dir(workspace_root, site_dir), site_dir);
    }

    #[test]
    fn resolve_site_dir_anchors_relative_paths_to_workspace_root() {
        let workspace_root = Path::new("/workspace");
        let site_dir = Path::new("artifacts/docs/site");

        assert_eq!(
            resolve_site_dir(workspace_root, site_dir),
            Path::new("/workspace/artifacts/docs/site")
        );
    }

    #[test]
    fn render_path_prefers_workspace_relative_paths() {
        let workspace_root = Path::new("/workspace");
        let path = Path::new("/workspace/contracts");

        assert_eq!(render_path(path, workspace_root), "contracts");
    }
}
