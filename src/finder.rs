use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::error::Result;
use crate::parser::Pattern;
use crate::types::{TagName};
use std::collections::HashMap;
use crate::parser::{Declaration, FileKey};

#[derive(Debug)]
pub struct FileCollection {
    pub tag: TagName,
    pattern: Pattern,
    // Map from key parts to file paths
    pub files: HashMap<FileKey, Vec<PathBuf>>,
}

impl FileCollection {
    pub fn new(tag: TagName, pattern: Pattern) -> Self {
        Self {
            tag,
            pattern,
            files: HashMap::new(),
        }
    }

    pub fn add_if_matches(&mut self, path: &Path) -> bool {
        if let Some(key_parts) = self.pattern.parse(path) {
            self.files
                .entry(key_parts)
                .or_insert_with(Vec::new)
                .push(path.to_path_buf());
            return true;
        }
        false
    }
}

pub fn find_files(root: &Path, declaration: Declaration) -> Result<Vec<FileCollection>> {
    let mut collections: Vec<FileCollection> = declaration
        .0
        .iter()
        .map(|(tag, pattern)| {
            FileCollection::new(tag.clone(), pattern.clone())
        })
        .collect();

    // Walk through all files in directory
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        // Try to match each file against each pattern
        for collection in &mut collections {
            let added = collection.add_if_matches(path);
            if added {
                break;
            }
        }
    }

    Ok(collections)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;
    use crate::parser::parse_declaration;

    #[test]
    fn test_find_matching_files() -> Result<()> {
        let root = tempdir()?;
        let root_path = root.path();

        fs::create_dir_all(root_path.join("images/2024"))?;
        fs::create_dir_all(root_path.join("images/2023"))?;

        File::create(root_path.join("images/2024/photo1.jpg"))?;
        File::create(root_path.join("images/2024/photo2.jpg"))?;
        File::create(root_path.join("images/2023/old.jpg"))?;

        let declaration = parse_declaration("images/{year}/{name}.jpg")?;

        let collections = find_files(root_path, declaration)?;
        assert_eq!(collections.len(), 1);

        let image_collection = &collections[0];
        assert_eq!(image_collection.tag.0, "main");

        // Check we found all the files
        let files = image_collection.files.values().flatten().count();
        assert_eq!(files, 3);

        Ok(())
    }
}
