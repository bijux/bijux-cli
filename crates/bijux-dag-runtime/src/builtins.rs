//! Built-in adapter surface.

pub mod const_adapter;
pub mod container_adapter;
pub mod shell_adapter;

pub use const_adapter::ConstAdapter;
pub use container_adapter::ContainerAdapter;
pub use shell_adapter::ShellAdapter;
