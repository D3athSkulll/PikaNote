use super::super::super::FileType;
use std::{
    fmt::{self, Display},
    path::{Path, PathBuf}, //Internal Data structure to represent path
};

#[derive(Debug, Default)]
pub struct FileInfo {
    path: Option<PathBuf>,
    file_type: FileType,
}

impl FileInfo {
    pub fn from(file_name: &str) -> Self {
        let path = PathBuf::from(file_name);
        let file_type = if path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("rs"))
        {
            FileType::Rust
        } else {
            FileType::Text
        };
        eprintln!("DEBUG: loading {file_name}, detected type = {file_type:?}");
        Self {
            path: Some(path),
            file_type,
        }
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub const fn has_path(&self) -> bool {
        self.path.is_some()
    }
    pub const fn get_file_type(&self) -> FileType {
        self.file_type
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name()) // pass path to closure if not None and return file name as option
            .and_then(|name| name.to_str()) // pass the none or filename and convert to string
            .unwrap_or("[No Name]"); // returning any of the none from above recieved as No name

        write!(formatter, "{name}") // write out the name
    }
}
