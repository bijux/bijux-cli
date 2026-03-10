//! Shared CLI context and state helper routines.

mod argv;
mod persistence;
mod scaffold;
mod state;

pub(crate) use argv::{command_has_flag, command_option_value};
pub(crate) use persistence::{
    read_history_entries, read_memory_map, write_history_entries, write_memory_map,
};
pub(crate) use scaffold::scaffold_plugin_layout;
pub(crate) use state::{
    env_map, resolve_state_paths, state_diagnostics, state_path_status_value, ResolvedStatePaths,
};
