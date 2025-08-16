pub mod error;
pub mod types;

pub use error::{TwinError, TwinResult};
pub use types::{Config, FileMapping, HookCommand, HookConfig, MappingType, SymlinkInfo};
