use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
pub enum LocalPathError {
    #[error("path was not in specified directory")]
    PathNotInDir,
}

pub trait LocalPath {
    fn local_path(&self, directory: &Path) -> Result<PathBuf, LocalPathError>;
}

impl LocalPath for Path {
    fn local_path(&self, directory: &Path) -> Result<PathBuf, LocalPathError> {
        let is_in_dir =
            directory == Path::new("") || self.ancestors().any(|path| directory == path);
        if !is_in_dir {
            return Err(LocalPathError::PathNotInDir);
        }

        let mut path_parts = vec![];
        for (component, ancestor) in self.components().rev().zip(self.ancestors().skip(1)) {
            path_parts.push(component.as_os_str().to_owned());
            if ancestor == directory {
                break;
            }
        }

        Ok(path_parts
            .iter()
            .rev()
            .fold(PathBuf::new(), |mut path, part| {
                path.push(part);
                path
            }))
    }
}

#[cfg(test)]
mod test {
    use crate::path_utils::LocalPathError;

    use super::LocalPath;
    use std::path::Path;

    #[test]
    fn local_path() {
        let path = Path::new("/foo/bar/stuff.txt");
        assert_eq!(
            path.local_path(Path::new("/foo/bar")),
            Ok(Path::new("stuff.txt").into())
        );
        assert_eq!(
            path.local_path(Path::new("/foo")),
            Ok(Path::new("bar/stuff.txt").into())
        );
        assert_eq!(
            path.local_path(Path::new("")),
            Ok(Path::new("foo/bar/stuff.txt").into())
        );
        assert_eq!(
            path.local_path(Path::new("something/else")),
            Err(LocalPathError::PathNotInDir)
        );
    }
}
