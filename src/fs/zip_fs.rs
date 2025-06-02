use std::{
    collections::{BTreeSet, HashMap},
    io::{Cursor, Read},
};

use zip::ZipArchive;

use crate::fs::{FileMetadata, FsError, SimpleFs, join_path, normalize_path};

#[derive(Clone, Debug)]
struct ZipEntry {
    index: usize,
    is_dir: bool,
    len: u64,
}

pub struct ZipFs {
    buf: Vec<u8>,
    /// Map of path (relative to root_prefix) -> zip entry (file or directory).
    entries: HashMap<String, ZipEntry>,
    /// Directory structure: key is a normalized directory path (e.g., "", "subdir"), value is set of names (file or dir names) under that dir.
    dir_map: HashMap<String, BTreeSet<String>>,
}

impl ZipFs {
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
        let mut entries: HashMap<String, ZipEntry> = HashMap::new();
        let mut dir_map: HashMap<String, BTreeSet<String>> = HashMap::new();

        // For normalization
        let prefix_len = root_prefix.as_ref().map(|s| s.len()).unwrap_or(0);

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = &file.name()[prefix_len..]; // strip root prefix, if any
            let norm_path = normalize_path(name); // always no leading/trailing
            let is_dir = file.name().ends_with('/');
            let norm: String = norm_path.to_string();
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
                (parent.to_string(), entry_name.to_string())
            } else {
                (String::new(), norm.clone())
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

impl SimpleFs for ZipFs {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let normalized = normalize_path(path);
        // "" is root.
        let entry = self.dir_map.get(&normalized);
        if let Some(set) = entry {
            let mut out = Vec::new();
            for name in set {
                let full = join_path(&normalized, name);
                out.push(full);
            }
            Ok(out)
        } else {
            Err(FsError::NotFound)
        }
    }

    fn open_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let path = normalize_path(path);
        match self.entries.get(&path) {
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
        match self.entries.get(&path) {
            Some(e) => Ok(FileMetadata {
                len: e.len,
                is_dir: e.is_dir,
            }),
            None => Err(FsError::NotFound),
        }
    }

    fn exists(&self, path: &str) -> bool {
        let path = normalize_path(path);
        self.entries.contains_key(&path)
    }
}
