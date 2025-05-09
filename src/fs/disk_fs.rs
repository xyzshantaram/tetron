#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

#[cfg(not(target_arch = "wasm32"))]
use crate::fs::{FileMetadata, FsError, SimpleFS, join_path, normalize_path};

#[cfg(not(target_arch = "wasm32"))]
pub struct DiskFs {
    base: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl DiskFs {
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        DiskFs {
            base: p.as_ref().to_owned(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SimpleFS for DiskFs {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let path = normalize_path(path);
        let real = self.base.join(path);
        let mut entries = Vec::new();
        for entry in fs::read_dir(&real).map_err(FsError::Io)? {
            let e = entry.map_err(FsError::Io)?;
            if let Some(name) = e.file_name().to_str() {
                let sub = join_path(path, name);
                entries.push(sub);
            }
        }
        Ok(entries)
    }

    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let path = normalize_path(path);
        let real = self.base.join(path);
        let mut buf: Vec<u8> = Vec::new();
        File::open(real)?.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError> {
        let path = normalize_path(path);
        let real = self.base.join(path);
        let meta = fs::metadata(real).map_err(FsError::Io)?;
        Ok(FileMetadata {
            len: meta.len(),
            is_dir: meta.is_dir(),
        })
    }

    fn exists(&self, path: &str) -> bool {
        let path = normalize_path(path);
        let real = self.base.join(path);
        real.exists()
    }
}
