#![forbid(unsafe_code)]
//! Property tests for global-flag placement normalization.

use bijux_cli_contracts as _;
use bijux_cli_routing::parser::parse_intent;
use clap as _;
use proptest::prelude::*;
use strsim as _;
use thiserror as _;
use serde as _;
use serde_json as _;

fn strategy_for_known_path() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        Just(vec!["status".to_string()]),
        Just(vec!["plugins".to_string(), "list".to_string()]),
        Just(vec!["config".to_string(), "get".to_string()]),
        Just(vec!["dev".to_string(), "routes".to_string()]),
    ]
}

proptest! {
    #[test]
    fn moving_global_flags_before_or_after_path_keeps_effective_intent(path in strategy_for_known_path()) {
        let mut left = vec!["bijux".to_string(), "--format".to_string(), "json".to_string(), "--quiet".to_string()];
        left.extend(path.clone());

        let mut right = vec!["bijux".to_string()];
        right.extend(path);
        right.extend(["--format".to_string(), "json".to_string(), "--quiet".to_string()]);

        let left_intent = parse_intent(&left).expect("left parse should succeed");
        let right_intent = parse_intent(&right).expect("right parse should succeed");

        prop_assert_eq!(left_intent.normalized_path, right_intent.normalized_path);
        prop_assert_eq!(left_intent.global_flags, right_intent.global_flags);
    }
}
