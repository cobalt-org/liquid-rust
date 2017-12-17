use std::fs::File;
use std::io::prelude::Read;
use std::path;

use error::{Error, Result};

pub trait Include: Send + Sync + IncludeClone {
    fn include(&self, path: &str) -> Result<String>;
}

pub trait IncludeClone {
    fn clone_box(&self) -> Box<Include>;
}

impl<T> IncludeClone for T
    where T: 'static + Include + Clone
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
        Err(Error::from(&*format!("{:?} does not exist", relative_path)))
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
        let root = self.root.canonicalize()?;
        let mut path = root.clone();
        path.extend(relative_path.split('/'));
        if !path.exists() {
            return Err(Error::from(&*format!("{:?} does not exist", path)));
        }
        let path = path.canonicalize()?;
        if !path.starts_with(&root) {
            return Err(Error::from(&*format!("{:?} is outside the include path", path)));
        }

        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
}
