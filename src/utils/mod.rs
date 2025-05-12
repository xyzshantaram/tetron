use std::{
    env,
    path::{Path, PathBuf},
};

pub fn resolve_physical_fs_path(path: &Path) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let full_path = cwd.join(path);
    Ok(full_path.canonicalize()?)
}
