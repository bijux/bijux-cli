//! Cache command orchestration helpers.

mod service;

#[cfg(test)]
pub(crate) use service::unpack_cache_archive_bounded;
pub(crate) use service::{
    cache_diff, cache_prune_simulate, cache_stats, explain_cache_key, explain_run_node_cache_miss,
    pack_cache_entry, unpack_cache_entry, verify_cache_dirs, verify_cache_entry_cli,
};
