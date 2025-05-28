use std::io;

#[derive(Debug)]
pub enum FsError {
    NotFound,
    ReadError(String),
    Io(io::Error),
}

impl From<io::Error> for FsError {
    fn from(e: io::Error) -> Self {
        FsError::Io(e)
    }
}

impl From<zip::result::ZipError> for FsError {
    fn from(e: zip::result::ZipError) -> Self {
        FsError::Io(io::Error::other(e))
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::NotFound => write!(f, "SimpleFs: Resource not found"),
            FsError::Io(e) => write!(f, "SimpleFs: I/O error: {e}"),
            FsError::ReadError(s) => write!(f, "SimpleFs: Error reading file: {s}"),
        }
    }
}

impl std::error::Error for FsError {}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub len: u64,
    pub is_dir: bool,
}

pub trait SimpleFs: Send + Sync {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError>;
    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError>;
    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError>;
    fn exists(&self, path: &str) -> bool;

    fn read_text_file(&self, path: &str) -> Result<String, FsError> {
        let bytes = self.open_file(path)?;
        String::from_utf8(bytes)
            .map_err(|_| FsError::ReadError(format!("Error converting {path} as UTF-8")))
    }
}

/// Normalize a path: always forward slash, no leading or trailing slash unless root.
/// Root is always normalized as an empty string, not "/".
pub fn normalize_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

/// Joins two forward-slash paths where lhs may be empty (root), ensuring single slash between.
/// `join_path("", "foo/bar") -> "foo/bar"`
/// `join_path("dir", "file") -> "dir/file"`
/// `join_path("foo", "bar/baz") -> "foo/bar/baz"`
pub(crate) fn join_path(lhs: &str, rhs: &str) -> String {
    if lhs.is_empty() {
        rhs.to_string()
    } else if rhs.is_empty() {
        lhs.to_string()
    } else {
        format!("{lhs}/{rhs}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::fs::{disk_fs::DiskFs, zip_fs::ZipFs};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn to_vfs_layer(layer: &PathBuf) -> Result<Box<dyn SimpleFs>, anyhow::Error> {
    if layer.extension().is_some_and(|v| v == "zip") {
        let mut buf: Vec<u8> = Vec::new();
        File::open(layer)?.read_to_end(&mut buf)?;
        Ok(Box::new(ZipFs::new(buf)?))
    } else {
        fs::metadata(layer)?;
        Ok(Box::new(DiskFs::new(layer)))
    }
}

pub mod disk_fs;
pub mod noop_fs;
pub mod overlay_fs;
pub mod zip_fs;
