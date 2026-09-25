//! The standard library the toolchain supplies [MOD-10]: its graph record and
//! its modules' records, carried from the compiler's own build so that a
//! program is always checked against the library its compiler ships, and the
//! host modules whose definitions the build supplies [PRE-2].

/// The logical path of the standard library's graph record.
pub(crate) const GRAPH_PATH: &str = "std/modules.wfg";

/// The standard library's graph record, `modules.wfg` below its root.
pub(crate) const GRAPH: &str = include_str!("../../lib/std/modules.wfg");

/// Every record of the standard library's modules, each with its logical
/// path, its path below the library's root after the component `std`
/// [MOD-10], in the order the library's graph registers their modules and
/// each module's interface record first [MOD-2].
pub(crate) const RECORDS: &[(&str, &str)] = &[
    ("std/io/module.wfm", include_str!("../../lib/std/io/module.wfm")),
    ("std/text/module.wfm", include_str!("../../lib/std/text/module.wfm")),
    ("std/fs/module.wfm", include_str!("../../lib/std/fs/module.wfm")),
    ("std/net/module.wfm", include_str!("../../lib/std/net/module.wfm")),
    ("std/process/module.wfm", include_str!("../../lib/std/process/module.wfm")),
];

/// The host modules [PRE-2], by their paths below the standard library: the
/// modules whose functions have no Whitefoot definition, the build supplying
/// each from the runtime units.
pub(crate) const HOST_MODULES: &[&str] = &["io", "text", "fs", "net", "process"];

/// Reports whether a standard library module at `path` is a host module.
pub(crate) fn is_host_module(path: &[String]) -> bool {
    matches!(path, [single] if HOST_MODULES.contains(&single.as_str()))
}

/// The standard library module a record belongs to and its role: the
/// module's path below the library's root and whether the record is its
/// interface [MOD-2].
pub(crate) fn record_module(logical: &str) -> Option<(Vec<String>, crate::SourceRole)> {
    let (directory, file) = logical.strip_prefix("std/")?.rsplit_once('/')?;
    let role = if file == crate::INTERFACE_FILE_NAME {
        crate::SourceRole::Interface
    } else {
        crate::SourceRole::Implementation
    };
    Some((directory.split('/').map(str::to_owned).collect(), role))
}

#[cfg(test)]
mod tests {
    use super::{GRAPH, HOST_MODULES, RECORDS};

    fn library_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lib/std")
    }

    /// Every file of the library's directory is carried, and nothing else,
    /// so adding a module without listing it here cannot go unnoticed.
    #[test]
    fn the_carried_records_are_exactly_the_library_directory() {
        fn walk(root: &std::path::Path, directory: &std::path::Path, found: &mut Vec<String>) {
            for entry in std::fs::read_dir(directory).expect("the library directory reads") {
                let path = entry.expect("a directory entry reads").path();
                if path.is_dir() {
                    walk(root, &path, found);
                } else {
                    found.push(
                        path.strip_prefix(root)
                            .expect("below the root")
                            .to_string_lossy()
                            .replace('\\', "/"),
                    );
                }
            }
        }
        let root = library_root();
        let mut found = Vec::new();
        walk(&root, &root, &mut found);
        found.sort();
        let mut carried: Vec<String> = RECORDS
            .iter()
            .map(|(path, _)| path.strip_prefix("std/").expect("a library path").to_owned())
            .chain(std::iter::once("modules.wfg".to_owned()))
            .collect();
        carried.sort();
        assert_eq!(found, carried);
        for (path, text) in RECORDS {
            let relative = path.strip_prefix("std/").expect("a library path");
            let on_disk = std::fs::read_to_string(root.join(relative)).expect("the record reads");
            assert_eq!(&on_disk, text, "{path} is carried byte for byte");
        }
        assert_eq!(
            std::fs::read_to_string(root.join("modules.wfg")).expect("the graph reads"),
            GRAPH
        );
    }

    /// [PRE-2] the host modules' graph rows and interface records are the
    /// specification's text byte for byte.
    #[test]
    fn the_host_modules_are_the_specification_text() {
        let spec = crate::spec::ACTIVE_KERNEL_SPEC_TEXT;
        let section = spec
            .split("[PRE-2]")
            .nth(1)
            .expect("the specification states PRE-2");
        let fences: Vec<&str> = section
            .split("```\n")
            .skip(1)
            .step_by(2)
            .collect();
        let rows = fences.first().expect("PRE-2 states the graph rows");
        for row in GRAPH.lines() {
            assert!(rows.lines().any(|line| line == row), "{row} is a PRE-2 row");
        }
        for module in HOST_MODULES {
            let path = format!("{module}/module.wfm");
            let (_, text) = RECORDS
                .iter()
                .find(|(record, _)| record.strip_prefix("std/") == Some(path.as_str()))
                .expect("every host module is carried");
            let marker = format!("`std::{module}`, the record `{path}`:\n\n```\n");
            let stated = spec
                .split(&marker)
                .nth(1)
                .and_then(|rest| rest.split("```\n").next())
                .expect("PRE-2 states every host module's record");
            assert_eq!(stated, *text, "{path} is the PRE-2 text");
        }
    }
}
