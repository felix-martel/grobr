use std::collections::HashMap;
use crate::finder::FileCollection;
use crate::types::Group;

pub fn group_files(collections: Vec<FileCollection>) -> HashMap<String, Group> {
    let mut groups: HashMap<String, Group> = HashMap::new();

    for collection in collections {
        for (key, files) in collection.files {
            let group_key = key.as_string();

            let group = groups.entry(group_key)
                .or_insert_with(|| Group {
                    files: HashMap::new(),
                });

            group.files
                .entry(collection.tag.clone())
                .or_insert_with(Vec::new)
                .extend(files);
        }
    }

    groups
}
