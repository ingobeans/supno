use serde::{ Deserialize, Serialize };
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FileOrDirectory {
    File(String),
    Directory(HashMap<String, FileOrDirectory>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileSystem {
    #[serde(flatten)]
    pub entries: HashMap<String, FileOrDirectory>,
}
