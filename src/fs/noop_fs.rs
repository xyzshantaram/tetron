use crate::fs::{FileMetadata, FsError, SimpleFs};

pub struct NoOpFs {}

impl NoOpFs {
    pub fn new() -> Self {
        NoOpFs {}
    }
}

impl SimpleFs for NoOpFs {
    fn read_dir(&self, _: &str) -> Result<Vec<String>, FsError> {
        unimplemented!("no-op filesystem")
    }

    fn open_file(&self, _: &str) -> Result<Vec<u8>, FsError> {
        unimplemented!("no-op filesystem")
    }

    fn metadata(&self, _: &str) -> Result<FileMetadata, FsError> {
        unimplemented!("no-op filesystem")
    }

    fn exists(&self, _: &str) -> bool {
        unimplemented!("no-op filesystem")
    }
}
