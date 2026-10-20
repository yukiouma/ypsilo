use crate::{
    Result,
    config::{Migration, Node},
    error::ScaffoldError,
};
use std::{fs, io, path::Path};

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

    if let Some(migration) = &node.migration {
        return copy_migration(migration, &target);
    }

    if let Some(reference) = &node.reference {
        return copy_reference(reference, &target);
    }

    if node.directory {
        create_dir(&target)?;
        return build_tree(&node.children, &target);
    }

    create_file(&target)
}

/// Copy the immediate children of `migration.directory` whose extension is in
/// `migration.filter` into `target`, which is created if absent.
///
/// The source directory is not searched recursively.
fn copy_migration(migration: &Migration, target: &Path) -> Result<()> {
    let source: &Path = &migration.directory;
    if !source.exists() {
        return Err(ScaffoldError::MigrationDirNotFound(source.to_path_buf()));
    }

    create_dir(target)?;

    let entries = fs::read_dir(source).map_err(|e| ScaffoldError::io(source, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| ScaffoldError::io(source, e))?;
        let path = entry.path();

        if !path.is_file() || !extension_is_allowed(&path, &migration.filter) {
            continue;
        }

        let destination = target.join(entry.file_name());
        if destination.exists() {
            continue;
        }
        copy_file(&path, &destination)?;
    }

    Ok(())
}

/// Whether `path`'s extension appears in `filter`. Filter entries carry a leading dot.
fn extension_is_allowed(path: &Path, filter: &[String]) -> bool {
    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let extension = format!(".{extension}");
    filter.iter().any(|allowed| allowed == &extension)
}

/// Copy `reference` to `target`, leaving an existing target alone.
fn copy_reference(reference: &Path, target: &Path) -> Result<()> {
    if !reference.exists() {
        return Err(ScaffoldError::ReferenceNotFound(reference.to_path_buf()));
    }
    if target.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        create_dir(parent)?;
    }
    copy_file(reference, target)
}

/// Copy `source` to `destination`, keeping the source's timestamps.
///
/// [`fs::copy`] alone leaves the destination stamped with the moment of the
/// copy, which hides how old a template really is. The access and modification
/// times are restored; the creation time stays that of the copy, because no
/// platform-independent way exists to set it.
fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::metadata(source).map_err(|e| ScaffoldError::io(source, e))?;

    fs::copy(source, destination)
        .map(|_| ())
        .map_err(|e| ScaffoldError::io(destination, e))?;

    let mut times = fs::FileTimes::new().set_modified(
        metadata
            .modified()
            .map_err(|e| ScaffoldError::io(source, e))?,
    );
    if let Ok(accessed) = metadata.accessed() {
        times = times.set_accessed(accessed);
    }

    let file = open_for_timestamps(destination).map_err(|e| ScaffoldError::io(destination, e))?;
    file.set_times(times)
        .map_err(|e| ScaffoldError::io(destination, e))
}

/// Open `path` so that its timestamps can be rewritten.
///
/// Write access is what [`fs::File::set_times`] needs on Windows. On Unix the
/// timestamps can be set through a read-only handle, which is the only way in
/// when the copy inherited a read-only permission from its source.
fn open_for_timestamps(path: &Path) -> io::Result<fs::File> {
    match fs::File::options().write(true).open(path) {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            fs::File::options().read(true).open(path)
        }
        result => result,
    }
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
    use crate::{ScaffoldError, config::Migration};
    use std::{
        fs,
        path::Path,
        time::{Duration, SystemTime},
    };
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

    /// A fixed instant well before any test run, so a copied file's timestamp
    /// cannot be confused with the moment it was copied.
    fn long_ago() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000)
    }

    /// Stamp `path` with [`long_ago`], so its timestamps stand out from the copy.
    fn age_file(path: &Path) {
        let file = fs::File::options().write(true).open(path).unwrap();
        file.set_times(
            fs::FileTimes::new()
                .set_accessed(long_ago())
                .set_modified(long_ago()),
        )
        .unwrap();
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

    fn reference_node(name: &str, reference: &Path) -> Node {
        Node {
            name: name.to_string(),
            directory: false,
            reference: Some(reference.to_path_buf()),
            migration: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn copies_a_reference_file_to_its_configured_name() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template v1.0.xlsx"), b"workbook").unwrap();

        build_tree(
            &[reference_node(
                "AK-XX SDTM Specification v1.0.xlsx",
                &src.path().join("template v1.0.xlsx"),
            )],
            dst.path(),
        )
        .unwrap();

        let copied = dst.path().join("AK-XX SDTM Specification v1.0.xlsx");
        assert_eq!(fs::read(&copied).unwrap(), b"workbook");
    }

    #[test]
    fn creates_the_parent_directory_for_a_reference_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template.xlsx"), b"x").unwrap();

        build_tree(
            &[reference_node(
                "template.xlsx",
                &src.path().join("template.xlsx"),
            )],
            &dst.path().join("specs"),
        )
        .unwrap();

        assert!(dst.path().join("specs").join("template.xlsx").is_file());
    }

    #[test]
    fn errors_when_the_reference_source_is_missing() {
        let dst = TempDir::new().unwrap();
        let err = build_tree(
            &[reference_node(
                "out.xlsx",
                Path::new("/nonexistent/template.xlsx"),
            )],
            dst.path(),
        )
        .unwrap_err();
        assert!(matches!(err, ScaffoldError::ReferenceNotFound(_)));
    }

    #[test]
    fn does_not_overwrite_an_existing_copied_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template.xlsx"), b"fresh").unwrap();
        fs::write(dst.path().join("template.xlsx"), b"edited").unwrap();

        build_tree(
            &[reference_node(
                "template.xlsx",
                &src.path().join("template.xlsx"),
            )],
            dst.path(),
        )
        .unwrap();

        assert_eq!(
            fs::read(dst.path().join("template.xlsx")).unwrap(),
            b"edited"
        );
    }
    fn migration_node(name: &str, directory: &Path, filter: &[&str]) -> Node {
        Node {
            name: name.to_string(),
            directory: true,
            reference: None,
            migration: Some(Migration {
                directory: directory.to_path_buf(),
                filter: filter.iter().map(|f| f.to_string()).collect(),
            }),
            children: Vec::new(),
        }
    }

    /// A source directory holding two `.sas` files, one `.xlsx`, and a subdirectory.
    fn general_marcos_fixtures() -> TempDir {
        let src = TempDir::new().unwrap();
        fs::write(src.path().join("m-forest-v1-0.sas"), b"macro").unwrap();
        fs::write(src.path().join("m-seriesplot-v1-0 .sas"), b"macro").unwrap();
        fs::write(src.path().join("Macro category information.xlsx"), b"book").unwrap();
        fs::create_dir(src.path().join("nested")).unwrap();
        fs::write(src.path().join("nested").join("deep-v1.sas"), b"deep").unwrap();
        src
    }

    #[test]
    fn copies_only_files_matching_the_extension_filter() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(
            &[migration_node("macros", src.path(), &[".sas"])],
            dst.path(),
        )
        .unwrap();

        let macros = dst.path().join("macros");
        assert!(macros.is_dir());
        assert!(macros.join("m-forest-v1-0.sas").is_file());
        assert!(macros.join("m-seriesplot-v1-0 .sas").is_file());
        assert!(!macros.join("Macro category information.xlsx").exists());
    }

    #[test]
    fn does_not_recurse_into_subdirectories_of_the_source() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(
            &[migration_node("macros", src.path(), &[".sas"])],
            dst.path(),
        )
        .unwrap();

        assert!(!dst.path().join("macros").join("nested").exists());
    }

    #[test]
    fn creates_an_empty_directory_when_nothing_matches_the_filter() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(
            &[migration_node("macros", src.path(), &[".txt"])],
            dst.path(),
        )
        .unwrap();

        let macros = dst.path().join("macros");
        assert!(macros.is_dir());
        assert_eq!(fs::read_dir(&macros).unwrap().count(), 0);
    }

    #[test]
    fn leaves_already_migrated_files_alone() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();
        fs::create_dir(dst.path().join("macros")).unwrap();
        fs::write(
            dst.path().join("macros").join("m-forest-v1-0.sas"),
            b"local edit",
        )
        .unwrap();

        build_tree(
            &[migration_node("macros", src.path(), &[".sas"])],
            dst.path(),
        )
        .unwrap();

        assert_eq!(
            fs::read(dst.path().join("macros").join("m-forest-v1-0.sas")).unwrap(),
            b"local edit"
        );
    }

    #[test]
    fn errors_when_the_migration_source_is_missing() {
        let dst = TempDir::new().unwrap();
        let err = build_tree(
            &[migration_node(
                "macros",
                Path::new("/nonexistent/macros"),
                &[".sas"],
            )],
            dst.path(),
        )
        .unwrap_err();
        assert!(matches!(err, ScaffoldError::MigrationDirNotFound(_)));
    }

    #[test]
    fn keeps_the_modification_time_of_a_copied_reference_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let template = src.path().join("template.xlsx");
        fs::write(&template, b"workbook").unwrap();
        age_file(&template);

        build_tree(&[reference_node("template.xlsx", &template)], dst.path()).unwrap();

        let copied = dst.path().join("template.xlsx");
        assert_eq!(
            fs::metadata(&copied).unwrap().modified().unwrap(),
            long_ago()
        );
    }

    #[test]
    fn keeps_the_modification_time_of_a_migrated_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("m-forest-v1-0.sas"), b"macro").unwrap();
        age_file(&src.path().join("m-forest-v1-0.sas"));

        build_tree(
            &[migration_node("macros", src.path(), &[".sas"])],
            dst.path(),
        )
        .unwrap();

        let copied = dst.path().join("macros").join("m-forest-v1-0.sas");
        assert_eq!(
            fs::metadata(&copied).unwrap().modified().unwrap(),
            long_ago()
        );
    }

    /// A read-only source makes a read-only copy, whose timestamps can only be
    /// rewritten through a handle that was not opened for writing.
    #[cfg(unix)]
    #[test]
    fn keeps_the_modification_time_of_a_copy_of_a_read_only_file() {
        use std::os::unix::fs::PermissionsExt;

        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let template = src.path().join("template.xlsx");
        fs::write(&template, b"workbook").unwrap();
        age_file(&template);
        fs::set_permissions(&template, fs::Permissions::from_mode(0o444)).unwrap();

        build_tree(&[reference_node("template.xlsx", &template)], dst.path()).unwrap();

        let copied = dst.path().join("template.xlsx");
        assert_eq!(
            fs::metadata(&copied).unwrap().modified().unwrap(),
            long_ago()
        );
    }
}
