use std::rc::Rc;

use rune::alloc::borrow::TryToOwned;
use rune::alloc::path::Path;
use rune::ast::Spanned;
use rune::compile;
use rune::{Item, Source};

use crate::fs::SimpleFs;
use rune::compile::SourceLoader; // Replace with your actual module

pub struct SimpleFsSourceLoader {
    fs: Rc<dyn SimpleFs>,
}

impl SimpleFsSourceLoader {
    pub fn new(fs: Rc<dyn SimpleFs>) -> Self {
        Self { fs }
    }
}

impl SourceLoader for SimpleFsSourceLoader {
    fn load(&mut self, root: &Path, item: &Item, span: &dyn Spanned) -> compile::Result<Source> {
        // Build the base directory from root by popping the file name, similar to FileSourceLoader
        let mut base = root.try_to_owned()?;
        if !base.pop() {
            return Err(compile::Error::msg(span, "Invalid module root"));
        }

        for c in item {
            if let Some(segment) = c.as_str() {
                base.push(segment);
            } else {
                return Err(compile::Error::msg(span, "Invalid module item"));
            }
        }

        // Consider both mod.rn inside the dir and dir.rn as file candidates
        let candidate1 = base.join("mod.rn").to_string_lossy().to_string();
        let candidate2 = base.with_extension("rn").to_string_lossy().to_string();

        let path = if self.fs.exists(&candidate1) {
            candidate1
        } else if self.fs.exists(&candidate2) {
            candidate2
        } else {
            return Err(compile::Error::msg(
                span,
                format!("Module not found: {}, {}", candidate1, candidate2),
            ));
        };

        // Read file contents as string (using your method)
        let src = self
            .fs
            .read_text_file(&path)
            .map_err(|e| compile::Error::msg(span, format!("Error reading file: {path}, {e:?}")))?;

        // Build a Source with the file path and contents
        Ok(Source::new(path, src)?)
    }
}
