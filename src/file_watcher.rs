use anyhow::Result;
use std::path::Path;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ms(pub u128);

pub fn file_modified_time(path: impl AsRef<Path>) -> Result<Ms> {
    Ok(Ms(std::fs::metadata(path.as_ref())?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub usize);

pub struct UpdatedFiles<'a> {
    files: &'a HashMap<FileId, PathBuf>,
}

pub struct FileData {
    path: PathBuf,
    id: FileId,
    last_modification_time: Ms,
}

impl UpdatedFiles<'_> {
    fn file_accessed(&self, file_id: FileId) -> bool {
        self.files.contains_key(&file_id)
    }
}

pub struct FileWatcher {
    updated_files: HashMap<FileId, PathBuf>,
    files_by_paths: HashMap<PathBuf, FileData>,
    file_id_counter: usize,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            files_by_paths: Default::default(),
            updated_files: Default::default(),
            file_id_counter: 0,
        }
    }

    pub fn watch_file(&mut self, path: &Path) -> Result<FileId> {
        let new_id = self.new_file_id();
        self.files_by_paths.insert(path.to_path_buf(), FileData {
            path: path.to_path_buf(),
            last_modification_time: file_modified_time(path)?,
            id: new_id,
        });
        Ok(new_id)
    }

    pub fn update(&mut self) -> UpdatedFiles {
        self.updated_files.clear();

        for (path, file_data) in &mut self.files_by_paths {
            if let Ok(new_modify_time) = file_modified_time(path) {
                if new_modify_time == file_data.last_modification_time {
                    continue;
                }

                file_data.last_modification_time = new_modify_time;
                self.updated_files.insert(file_data.id, path.to_path_buf());
            }
        }

        UpdatedFiles {
            files: &self.updated_files,
        }
    }

    fn new_file_id(&mut self) -> FileId {
        let file_id = FileId(self.file_id_counter);
        self.file_id_counter += 1;
        file_id
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::FileWatcher;

    #[test]
    fn file_watcher() {
        println!("watch started!");
        let mut watcher = FileWatcher::new();
        let mut access_count = 0;
        let id = watcher
            .watch_file(Path::new(
                "text.txt",
            ))
            .unwrap();

        loop {
            let files = watcher.update();
            if files.file_accessed(id) {
                println!("file was accessed!");
                access_count += 1;
                println!("access_count: {access_count}");
            }
        }
    }
}
