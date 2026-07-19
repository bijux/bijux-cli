//! Built-in adapter surface.
#![allow(unused_imports)]

pub mod const_adapter;
pub mod container_adapter;
pub mod file_transform_adapter;
pub mod shell_adapter;

pub use const_adapter::ConstAdapter;
pub use container_adapter::ContainerAdapter;
pub use file_transform_adapter::FileTransformAdapter;
pub use shell_adapter::ShellAdapter;
