//! Routing registry, conflict handling, and introspection APIs.

use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::{CommandPath, Namespace, NamespaceMetadata, OFFICIAL_PRODUCT_NAMESPACES};

/// Route target categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteTarget {
    /// Built-in route target.
    BuiltIn,
    /// Plugin route target by namespace.
    Plugin(String),
}

/// Route resolution error categories.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RouteError {
    /// Namespace is reserved.
    #[error("namespace is reserved: {0}")]
    Reserved(String),
    /// Namespace collides with existing route owner.
    #[error("namespace conflict: {0}")]
    Conflict(String),
    /// Route is unknown.
    #[error("unknown route: {0}")]
    Unknown(String),
    /// Ambiguous route due to multiple owners.
    #[error("ambiguous route: {0}")]
    Ambiguous(String),
}

/// Deterministic routing registry for built-ins and plugins.
#[derive(Debug, Clone)]
pub struct RouteRegistry {
    built_ins: BTreeSet<String>,
    plugin_namespaces: BTreeSet<String>,
    aliases: BTreeMap<String, String>,
    reserved: BTreeSet<String>,
}

impl Default for RouteRegistry {
    fn default() -> Self {
        let built_ins = super::model::built_in_route_paths()
            .iter()
            .map(|path| (*path).to_string())
            .collect::<BTreeSet<_>>();

        let aliases = super::model::alias_rewrites()
            .iter()
            .map(|(alias, canonical)| ((*alias).to_string(), (*canonical).to_string()))
            .collect::<BTreeMap<_, _>>();

        let mut reserved = BTreeSet::from([
            "cli".to_string(),
            "dev".to_string(),
            "help".to_string(),
            "version".to_string(),
            "doctor".to_string(),
            "repl".to_string(),
            "plugins".to_string(),
            "completion".to_string(),
            "inspect".to_string(),
        ]);
        reserved.extend(OFFICIAL_PRODUCT_NAMESPACES.iter().map(std::string::ToString::to_string));

        Self { built_ins, plugin_namespaces: BTreeSet::new(), aliases, reserved }
    }
}

impl RouteRegistry {
    fn blocked_namespace_roots(&self) -> BTreeSet<String> {
        let mut blocked = BTreeSet::new();
        for route in &self.built_ins {
            if let Some(head) = route.split(' ').next() {
                blocked.insert(head.to_string());
            }
        }
        for alias in self.aliases.keys() {
            if let Some(head) = alias.split(' ').next() {
                blocked.insert(head.to_string());
            }
        }
        blocked
    }

    /// Register a plugin namespace with deterministic rejection rules.
    pub fn register_plugin_namespace(&mut self, raw_namespace: &str) -> Result<(), RouteError> {
        let ns = normalize_namespace(raw_namespace);
        if self.reserved.contains(&ns) {
            return Err(RouteError::Reserved(ns));
        }

        if self.blocked_namespace_roots().contains(&ns) {
            return Err(RouteError::Conflict(ns));
        }

        if self.plugin_namespaces.contains(&ns) {
            return Err(RouteError::Conflict(ns));
        }

        self.plugin_namespaces.insert(ns);
        Ok(())
    }

    /// Resolve normalized command path to a route target.
    pub fn resolve(&self, normalized_path: &[String]) -> Result<RouteTarget, RouteError> {
        if normalized_path.is_empty() {
            return Err(RouteError::Unknown(String::new()));
        }

        let key = normalized_path.join(" ");
        let rewritten = self.aliases.get(&key).map_or(key.as_str(), String::as_str);

        if self.built_ins.contains(rewritten) {
            return Ok(RouteTarget::BuiltIn);
        }

        let root = rewritten.split(' ').next().unwrap_or_default();
        if self.plugin_namespaces.contains(root) {
            if self.built_ins.iter().any(|x| x.split(' ').next() == Some(root)) {
                return Err(RouteError::Ambiguous(root.to_string()));
            }
            return Ok(RouteTarget::Plugin(root.to_string()));
        }

        Err(RouteError::Unknown(rewritten.to_string()))
    }

    /// Suggest nearest namespace for unknown routes.
    #[must_use]
    pub fn suggest_namespace(&self, raw: &str) -> Option<String> {
        let query = normalize_namespace(raw);
        let mut universe = BTreeSet::new();

        for route in &self.built_ins {
            if let Some(head) = route.split(' ').next() {
                universe.insert(head.to_string());
            }
        }
        for ns in &self.plugin_namespaces {
            universe.insert(ns.clone());
        }
        for reserved in &self.reserved {
            universe.insert(reserved.clone());
        }

        universe.into_iter().max_by_key(|candidate| similarity_score(&query, candidate))
    }

    /// Build route-tree introspection payload.
    #[must_use]
    pub fn route_tree(&self) -> Vec<NamespaceMetadata> {
        let mut rows = Vec::new();

        for ns in &self.reserved {
            rows.push(NamespaceMetadata {
                name: Namespace(ns.clone()),
                reserved: true,
                owner: "bijux-cli".to_string(),
            });
        }

        for ns in &self.plugin_namespaces {
            rows.push(NamespaceMetadata {
                name: Namespace(ns.clone()),
                reserved: false,
                owner: "plugin".to_string(),
            });
        }

        rows.sort_by(|a, b| a.name.0.cmp(&b.name.0));
        rows
    }

    /// Render namespace tree lines for snapshot testing and diagnostics.
    #[must_use]
    pub fn render_command_tree(&self) -> String {
        let mut roots = BTreeSet::new();
        for route in &self.built_ins {
            if let Some(head) = route.split(' ').next() {
                roots.insert(head.to_string());
            }
        }
        for reserved in &self.reserved {
            roots.insert(reserved.clone());
        }
        let mut out = String::new();
        for root in roots {
            out.push_str(&root);
            out.push('\n');
        }
        out
    }

    /// Render built-in route paths for introspection.
    #[must_use]
    pub fn built_in_paths(&self) -> Vec<CommandPath> {
        self.built_ins
            .iter()
            .map(|raw| CommandPath {
                segments: raw.split(' ').map(|segment| Namespace(segment.to_string())).collect(),
            })
            .collect()
    }

    /// Render alias route rewrites for diagnostics introspection.
    #[must_use]
    pub fn alias_rewrites(&self) -> Vec<(CommandPath, CommandPath)> {
        self.aliases
            .iter()
            .map(|(alias, canonical)| {
                let to_path = |raw: &str| CommandPath {
                    segments: raw
                        .split(' ')
                        .map(|segment| Namespace(segment.to_string()))
                        .collect(),
                };
                (to_path(alias), to_path(canonical))
            })
            .collect()
    }
}

fn similarity_score(left: &str, right: &str) -> usize {
    let prefix = common_prefix_len(left, right);
    // Bias toward shared prefix and low edit distance while keeping deterministic ordering.
    let distance = levenshtein_distance(left, right);
    let normalized = max(left.chars().count(), right.chars().count());
    (prefix * 1000) + normalized.saturating_sub(distance)
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars().zip(right.chars()).take_while(|(a, b)| a == b).count()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let l: Vec<char> = left.chars().collect();
    let r: Vec<char> = right.chars().collect();
    if l.is_empty() {
        return r.len();
    }
    if r.is_empty() {
        return l.len();
    }

    let mut prev: Vec<usize> = (0..=r.len()).collect();
    let mut curr = vec![0; r.len() + 1];

    for (i, lc) in l.iter().enumerate() {
        curr[0] = i + 1;
        for (j, rc) in r.iter().enumerate() {
            let cost = usize::from(lc != rc);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        prev.clone_from(&curr);
    }
    prev[r.len()]
}

fn normalize_namespace(input: &str) -> String {
    Namespace::normalize(input)
}
