use std::fs::File;
use std::io::prelude::Read;
use std::path;

use super::{Error, Result, ResultLiquidChainExt, ResultLiquidExt};

pub trait Include: Send + Sync + IncludeClone {
    fn include(&self, path: &str) -> Result<String>;
}

pub trait IncludeClone {
    fn clone_box(&self) -> Box<Include>;
}

impl<T> IncludeClone for T
where
    T: 'static + Include + Clone,
{
    fn clone_box(&self) -> Box<Include> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Include> {
    fn clone(&self) -> Box<Include> {
        self.clone_box()
    }
}

/// `Include` no files
#[derive(Clone, Debug, Default)]
pub struct NullInclude {}

impl NullInclude {
    pub fn new() -> Self {
        Self {}
    }
}

impl Include for NullInclude {
    fn include(&self, relative_path: &str) -> Result<String> {
        Err(Error::with_msg("File does not exist").context("path", &relative_path.to_owned()))
    }
}

/// `Include` files relative to the root.
#[derive(Clone, Debug)]
pub struct FilesystemInclude {
    root: path::PathBuf,
}

impl FilesystemInclude {
    pub fn new<P: Into<path::PathBuf>>(root: P) -> Self {
        let root: path::PathBuf = root.into();
        Self { root }
    }
}

impl Include for FilesystemInclude {
    fn include(&self, relative_path: &str) -> Result<String> {
        let root = self.root
            .canonicalize()
            .chain("Snippet does not exist")
            .context_with(|| {
                let key = "non-existent source".into();
                let value = self.root.to_string_lossy().into();
                (key, value)
            })?;
        let mut path = root.clone();
        path.extend(relative_path.split('/'));
        let path = path.canonicalize()
            .chain("Snippet does not exist")
            .context_with(|| ("non-existent path".into(), path.to_string_lossy().into()))?;
        if !path.starts_with(&root) {
            return Err(Error::with_msg("Snippet is outside of source")
                .context("source", &root.to_string_lossy())
                .context("full path", &path.to_string_lossy()));
        }

        let mut file = File::open(&path)
            .chain("Failed to open snippet")
            .context_with(|| ("full path".into(), path.to_string_lossy().into()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .chain("Failed to read snippet")
            .context_with(|| ("full path".into(), path.to_string_lossy().into()))?;
        Ok(content)
    }
}
