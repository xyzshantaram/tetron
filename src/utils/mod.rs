use std::{
    env,
    path::{Path, PathBuf},
};

pub fn resolve_physical_fs_path(path: &Path) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let full_path = cwd.join(path);
    Ok(full_path.canonicalize()?)
}

pub type RuneString = ::rune::alloc::String;

pub mod rune {
    use super::RuneString;
    use crate::error::TetronError;
    use rune::{
        alloc::{clone::TryClone, string::TryToString},
        runtime::Object,
    };

    pub(crate) fn clone_obj(obj: &Object) -> Result<Object, TetronError> {
        let mut copy = Object::new();
        for item in obj.iter() {
            copy.insert(item.0.try_to_string()?, item.1.try_clone()?)?;
        }

        Ok(copy)
    }

    pub(crate) fn obj_key(s: &str) -> Result<RuneString, rune::alloc::Error> {
        RuneString::try_from(s)
    }
}
