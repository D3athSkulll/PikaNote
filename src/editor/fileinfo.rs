use std::{
    fmt::{self, Display},
    path::PathBuf, //Internal Data structure to represent path
};

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(file_name: &str) -> Self {
        Self {
            path: Some(PathBuf::from(file_name)),
        }
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .path
            .as_ref() // ensuring we not take ownership of path , returns an Option or None
            .and_then(|path| path.file_name()) // pass path to closure if not None and return file name as option
            .and_then(|name| name.to_str()) // pass the none or filename and convert to string
            .unwrap_or("[No Name]"); // returning any of the none from above recieved as No name

        write!(formatter, "{name}") // write out the name
    }
}
