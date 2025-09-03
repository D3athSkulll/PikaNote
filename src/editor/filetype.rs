use std::fmt::{Display,Result,Formatter};
use std::path::Path;

#[derive(Default,Eq,PartialEq,Debug,Copy,Clone)]
pub enum FileType{
    Rust,
    #[default]
    Text,
}

impl Display for FileType{
    fn fmt(&self, formatter: &mut Formatter<'_>)->Result{
        match self{
            Self::Rust=>write!(formatter,"Rust"),
            Self::Text=>write!(formatter,"Text"),
        }
    }
}//Display trait makes it easier to print out

impl From<&Path> for FileType {
    fn from(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => FileType::Rust,
            _ => FileType::Text,
        }
    }
}