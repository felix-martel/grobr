use std::path::PathBuf;
use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct GroupKey(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyPart(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub struct TagName(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnMissing {
    Ignore,
    Skip,
    Fail,
}

// A group of files sharing the same key
#[derive(Debug, Serialize)]
pub struct Group {
    pub files: HashMap<TagName, Vec<PathBuf>>,
}
