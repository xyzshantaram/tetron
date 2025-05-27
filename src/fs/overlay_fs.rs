use std::collections::HashSet;

use crate::fs::{FileMetadata, FsError, SimpleFs, normalize_path};

pub struct OverlayFs {
    layers: Vec<Box<dyn SimpleFs>>,
}

impl OverlayFs {
    pub fn from_layers(layers: Vec<Box<dyn SimpleFs>>) -> Self {
        let mut layers = layers;
        layers.reverse(); // Last is topmost
        OverlayFs { layers }
    }
}

impl SimpleFs for OverlayFs {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let path = normalize_path(path);
        let mut all: HashSet<String> = HashSet::new();
        let mut entries_found = false;

        for fs in &self.layers {
            if let Ok(entries) = fs.read_dir(&path) {
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
            if let Ok(file) = fs.open_file(&path) {
                return Ok(file);
            }
        }
        Err(FsError::NotFound)
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, FsError> {
        let path = normalize_path(path);
        for fs in &self.layers {
            if let Ok(meta) = fs.metadata(&path) {
                return Ok(meta);
            }
        }
        Err(FsError::NotFound)
    }

    fn exists(&self, path: &str) -> bool {
        let path = normalize_path(path);
        self.layers.iter().any(|fs| fs.exists(&path))
    }
}
