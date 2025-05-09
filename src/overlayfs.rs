use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use zip::ZipArchive;

#[derive(Debug)]
pub enum FsError {
    NotFound,
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
            FsError::NotFound => write!(f, "Resource not found"),
            FsError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}
impl std::error::Error for FsError {}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub len: u64,
    pub is_dir: bool,
}

pub trait SimpleFS: Send + Sync {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError>;
    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError>;
    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError>;
    fn exists(&self, path: &str) -> bool;
}

/// Normalize a path: always forward slash, no leading or trailing slash unless root.
/// Root is always normalized as an empty string, not "/".
fn normalize_path(path: &str) -> &str {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        "" // Root
    } else {
        trimmed
    }
}

/// Joins two forward-slash paths where lhs may be empty (root), ensuring single slash between.
/// `join_path("", "foo/bar") -> "foo/bar"`
/// `join_path("dir", "file") -> "dir/file"`
/// `join_path("foo", "bar/baz") -> "foo/bar/baz"`
fn join_path(lhs: &str, rhs: &str) -> String {
    if lhs.is_empty() {
        rhs.to_string()
    } else if rhs.is_empty() {
        lhs.to_string()
    } else {
        format!("{lhs}/{rhs}")
    }
}

#[derive(Clone, Debug)]
struct ZipEntry {
    index: usize,
    is_dir: bool,
    len: u64,
}

/// In-memory and file zip-backed fs.
pub struct ZipFS {
    buf: Vec<u8>,

    /// Map of path (relative to root_prefix) -> zip entry (file or directory).
    entries: HashMap<Cow<'static, str>, ZipEntry>,

    /// Directory structure: key is a normalized directory path (e.g., "", "subdir"), value is set of names (file or dir names) under that dir.
    dir_map: HashMap<Cow<'static, str>, BTreeSet<Cow<'static, str>>>,
}

impl ZipFS {
    pub fn new(buf: Vec<u8>) -> Result<Self, anyhow::Error> {
        let mut archive = ZipArchive::new(Cursor::new(&buf))?;

        // Gather all entry names. Find a root prefix if one exists.
        let mut names = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            names.push(file.name().to_string());
        }

        let root_prefix = Self::detect_and_strip_root_prefix(&names);

        // Core maps
        let mut entries: HashMap<Cow<'static, str>, ZipEntry> = HashMap::new();
        let mut dir_map: HashMap<Cow<'static, str>, BTreeSet<Cow<'static, str>>> = HashMap::new();

        // For normalization
        let prefix_len = root_prefix.as_ref().map(|s| s.len()).unwrap_or(0);

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = &file.name()[prefix_len..]; // strip root prefix, if any
            let norm_path = normalize_path(name); // always no leading/trailing
            let is_dir = file.name().ends_with('/');
            let norm: Cow<'static, str> = Cow::Owned(norm_path.to_string());
            if !norm.is_empty() {
                // skip synthetic root
                entries.insert(
                    norm.clone(),
                    ZipEntry {
                        index: i,
                        is_dir,
                        len: file.size(),
                    },
                );
            }
            // Now: populate dir_map for listing
            // Insert this entry's name in its parent dir.
            let (parent, entry_name) = if let Some(pos) = norm.rfind('/') {
                let parent = &norm[..pos];
                let entry_name = &norm[pos + 1..];
                (
                    Cow::Owned(parent.to_string()),
                    Cow::Owned(entry_name.to_string()),
                )
            } else {
                (Cow::Owned(String::new()), norm.clone())
            };
            dir_map.entry(parent).or_default().insert(entry_name);
        }

        // Now: also ensure that all ancestor directories exist as entries (as dirs).
        for dir in dir_map.keys() {
            if !entries.contains_key(dir) && !dir.is_empty() {
                entries.insert(
                    dir.clone(),
                    ZipEntry {
                        index: 0, // Index 0 is never read for directories.
                        is_dir: true,
                        len: 0,
                    },
                );
            }
        }

        Ok(Self {
            buf,
            entries,
            dir_map,
        })
    }

    /// If all names start with the same `foo/`, returns Some("foo"), else None.
    fn detect_and_strip_root_prefix(names: &[String]) -> Option<String> {
        if names.is_empty() {
            return None;
        }
        let first = &names[0];
        let prefix_pos = first.find('/')?;
        let prefix = &first[..prefix_pos];
        let prefix_slash = format!("{prefix}/");
        if names.iter().all(|n| n.starts_with(&prefix_slash)) {
            Some(prefix.to_string())
        } else {
            None
        }
    }

    /// Utility: open a new ZipArchive on self.buf for each op.
    fn open_archive(&self) -> Result<ZipArchive<Cursor<&[u8]>>, FsError> {
        ZipArchive::new(Cursor::new(&self.buf[..])).map_err(FsError::from)
    }
}

impl SimpleFS for ZipFS {
    /// Returns Vec<String> of paths under `path` (full normalized path, not just name).
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let path = normalize_path(path);
        // "" is root.
        let entry = self.dir_map.get(path);
        if let Some(set) = entry {
            let mut out = Vec::new();
            for name in set {
                let full = join_path(path, name);
                out.push(full);
            }
            Ok(out)
        } else {
            Err(FsError::NotFound)
        }
    }

    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let path = normalize_path(path);
        match self.entries.get(path) {
            Some(zip_entry) if !zip_entry.is_dir => {
                let mut archive = self.open_archive()?;
                let mut file = archive.by_index(zip_entry.index).map_err(FsError::from)?;
                let mut buf = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut buf)?;
                Ok(buf)
            }
            _ => Err(FsError::NotFound),
        }
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError> {
        let path = normalize_path(path);
        match self.entries.get(path) {
            Some(e) => Ok(FileMetadata {
                len: e.len,
                is_dir: e.is_dir,
            }),
            None => Err(FsError::NotFound),
        }
    }

    fn exists(&self, path: &str) -> bool {
        let path = normalize_path(path);
        self.entries.contains_key(path)
    }
}

/// OverlayFS allows you to stack several file systems. The last layer supplied is the first one queried.
/// OverlayFS searches through layers until a matching file is found.
pub struct OverlayFS {
    layers: Vec<Box<dyn SimpleFS>>,
}

impl OverlayFS {
    pub fn from_layers(layers: Vec<Box<dyn SimpleFS>>) -> Self {
        let mut layers = layers;
        layers.reverse(); // Last is topmost
        OverlayFS { layers }
    }
}

impl SimpleFS for OverlayFS {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let path = normalize_path(path);
        let mut all: HashSet<String> = HashSet::new();
        let mut entries_found = false;

        for fs in &self.layers {
            if let Ok(entries) = fs.read_dir(path) {
                for entry in entries {
                    entries_found = true;
                    all.insert(entry);
                }
            }
        }

        if !entries_found {
            Err(FsError::NotFound)
        } else {
            let mut out = all.into_iter().collect::<Vec<_>>();
            out.sort();
            Ok(out)
        }
    }

    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let path = normalize_path(path);
        for fs in &self.layers {
            if let Ok(file) = fs.open_file(path) {
                return Ok(file);
            }
        }
        Err(FsError::NotFound)
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError> {
        let path = normalize_path(path);
        for fs in &self.layers {
            if let Ok(meta) = fs.metadata(path) {
                return Ok(meta);
            }
        }
        Err(FsError::NotFound)
    }

    fn exists(&self, path: &str) -> bool {
        let path = normalize_path(path);
        self.layers.iter().any(|fs| fs.exists(path))
    }
}

/// DiskFs: local disk-backed SimpleFS
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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn to_vfs_layer(layer: &PathBuf) -> Result<Box<dyn SimpleFS>, anyhow::Error> {
    if layer.extension().is_some_and(|v| v == "zip") {
        let mut buf: Vec<u8> = Vec::new();
        File::open(layer)?.read_to_end(&mut buf)?;
        Ok(Box::new(ZipFS::new(buf)?))
    } else {
        fs::metadata(layer)?;
        Ok(Box::new(DiskFs::new(layer)))
    }
}
