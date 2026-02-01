use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::util::manifest::Manifest;

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    pub name: String,
    pub children: HashMap<String, TreeNode>,
    pub uuid: Option<Uuid>,
    pub full_path: Option<PathBuf>,
}

impl TreeNode {
    pub fn insert_path(&mut self, path: &Path, uuid: Uuid) {
        let mut current = self;

        let components: Vec<_> = path.components().collect();
        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy().to_string();
            let is_last = i == components.len() - 1;

            current = current.children.entry(name).or_insert_with(|| TreeNode {
                name: component.as_os_str().to_string_lossy().to_string(),
                children: HashMap::new(),
                uuid: if is_last { Some(uuid) } else { None },
                full_path: if is_last {
                    Some(path.to_path_buf())
                } else {
                    None
                },
            });

            if is_last && current.uuid.is_none() {
                current.uuid = Some(uuid);
                current.full_path = Some(path.to_path_buf());
            }
        }
    }
}

pub fn build_tree_from_manifest(manifest: &Manifest) -> TreeNode {
    let mut tree = TreeNode {
        name: "footnote.wiki".to_string(),
        children: HashMap::new(),
        uuid: None,
        full_path: None,
    };

    for (uuid, entry) in manifest {
        tree.insert_path(&entry.path, *uuid);
    }
    tree
}
