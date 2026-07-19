use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::simulated_platform::{
    authorize, check_api_compatibility, filter_resources, paginate, ApiCompatibilityRule,
    ApiVersion, AuthContext, AuthenticationPrincipal, AuthorizationRule, DagResource, ListFilter,
    Pagination, VersionedResource,
};

#[test]
fn pagination_and_filter_contracts_are_stable() {
    let items = vec![
        DagResource {
            dag_id: "d1".to_string(),
            logical_name: "etl-main".to_string(),
            owner: "platform".to_string(),
            tags: vec!["prod".to_string()],
            version: VersionedResource { resource_version: 1, etag: "e1".to_string() },
        },
        DagResource {
            dag_id: "d2".to_string(),
            logical_name: "etl-dev".to_string(),
            owner: "analytics".to_string(),
            tags: vec!["dev".to_string()],
            version: VersionedResource { resource_version: 1, etag: "e2".to_string() },
        },
    ];
    let page = paginate(&items, &Pagination { limit: 1, cursor: None });
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.next_cursor, Some("1".to_string()));

    let filtered = filter_resources(
        items.clone(),
        &ListFilter { field: "owner".to_string(), value: "platform".to_string() },
        |item, field| match field {
            "owner" => Some(item.owner.clone()),
            "logical_name" => Some(item.logical_name.clone()),
            _ => None,
        },
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].dag_id, "d1");
}

#[test]
fn authorization_and_compatibility_contracts_are_stable() {
    let auth = AuthContext {
        principal: AuthenticationPrincipal::CliUser { subject: "bijan".to_string() },
        scopes: vec!["dag/etl".to_string()],
    };
    let rules = vec![AuthorizationRule {
        resource_prefix: "dag/".to_string(),
        allowed_actions: vec!["run.submit".to_string(), "run.cancel".to_string()],
    }];
    assert!(authorize(&auth, "run.submit", &rules));
    assert!(!authorize(&auth, "policy.update", &rules));

    let compat = ApiCompatibilityRule {
        min_supported_major: 1,
        max_supported_major: 2,
        supports_minor_additive_fields: true,
    };
    assert!(check_api_compatibility(&ApiVersion { major: 1, minor: 0 }, &compat));
    assert!(!check_api_compatibility(&ApiVersion { major: 3, minor: 0 }, &compat));
}
