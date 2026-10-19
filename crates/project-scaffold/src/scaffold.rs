use crate::{Result, config::Node, error::ScaffoldError};
use std::{fs, path::Path};

/// Materialise `nodes` beneath `base`.
///
/// Anything that already exists is left untouched, so this is safe to re-run.
pub fn build_tree(nodes: &[Node], base: &Path) -> Result<()> {
    for node in nodes {
        create_node(node, base)?;
    }
    Ok(())
}

fn create_node(node: &Node, base: &Path) -> Result<()> {
    let target = base.join(&node.name);

    if node.directory {
        create_dir(&target)?;
        return build_tree(&node.children, &target);
    }

    create_file(&target)
}

fn create_dir(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|e| ScaffoldError::io(path, e))
}

fn create_file(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::File::create(path)
        .map(|_| ())
        .map_err(|e| ScaffoldError::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScaffoldError;
    use std::fs;
    use tempfile::TempDir;

    fn dir(name: &str) -> Node {
        Node {
            name: name.to_string(),
            directory: true,
            reference: None,
            migration: None,
            children: Vec::new(),
        }
    }

    fn file(name: &str) -> Node {
        Node {
            directory: false,
            ..dir(name)
        }
    }

    fn with_children(name: &str, children: Vec<Node>) -> Node {
        Node {
            children,
            ..dir(name)
        }
    }

    #[test]
    fn creates_a_single_directory() {
        let tmp = TempDir::new().unwrap();
        build_tree(&[dir("documents")], tmp.path()).unwrap();
        assert!(tmp.path().join("documents").is_dir());
    }

    #[test]
    fn creates_a_nested_tree() {
        let tmp = TempDir::new().unwrap();
        build_tree(
            &[with_children(
                "documents",
                vec![with_children(
                    "specs",
                    vec![file("AK-XX SDTM Specification v1.0.xlsx")],
                )],
            )],
            tmp.path(),
        )
        .unwrap();
        let leaf = tmp
            .path()
            .join("documents")
            .join("specs")
            .join("AK-XX SDTM Specification v1.0.xlsx");
        assert!(leaf.is_file());
    }

    #[test]
    fn creates_the_whole_project_root_when_it_is_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("ak").join("xx");
        build_tree(&[dir("documents")], &root).unwrap();
        assert!(root.join("documents").is_dir());
    }

    #[test]
    fn skips_a_directory_that_already_exists() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("documents").join("keep.txt"), b"keep").unwrap();

        build_tree(&[dir("documents")], tmp.path()).unwrap();

        assert_eq!(
            fs::read(tmp.path().join("documents").join("keep.txt")).unwrap(),
            b"keep"
        );
    }

    #[test]
    fn skips_a_file_that_already_exists_and_leaves_its_contents_alone() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("notes.txt"), b"original").unwrap();

        build_tree(&[file("notes.txt")], tmp.path()).unwrap();

        assert_eq!(fs::read(tmp.path().join("notes.txt")).unwrap(), b"original");
    }

    #[test]
    fn running_twice_is_a_no_op() {
        let tmp = TempDir::new().unwrap();
        let tree = vec![with_children("documents", vec![dir("crf")])];
        build_tree(&tree, tmp.path()).unwrap();
        build_tree(&tree, tmp.path()).unwrap();
        assert!(tmp.path().join("documents").join("crf").is_dir());
    }

    #[test]
    fn reports_the_path_of_a_node_it_could_not_create() {
        // `documents` is a file, so creating `documents/specs` fails with ENOTDIR.
        // A top-level node sitting on an existing path would simply be skipped.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("documents"), b"not a directory").unwrap();

        let err = build_tree(
            &[with_children("documents", vec![dir("specs")])],
            tmp.path(),
        )
        .unwrap_err();

        assert!(matches!(err, ScaffoldError::Io { .. }));
        assert!(err.to_string().contains("specs"));
    }
}
