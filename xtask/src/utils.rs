use std::path::{Path, PathBuf};

use crate::PROFILE;

pub fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

pub fn format_arg() -> String {
    format!("--{}", if PROFILE == "debug" { "" } else { PROFILE })
}
