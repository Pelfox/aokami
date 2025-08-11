mod registry;
mod blocks;

use std::path::{Path, PathBuf};
pub use registry::*;
pub use blocks::*;

pub fn get_output_path(root: &Path) -> PathBuf {
    root.join("transformed")
}
