//! Module graph formation [MOD-1] and module source discovery [MOD-2].
//!
//! A module program is selected by one `modules.wfg` file. Its rows register
//! the program's modules in written order, each with the exact list of earlier
//! modules it may name, and its entries name the functions an invocation may
//! run with their environment requirements. The file's directory is the
//! package root: every registered module is one directory below it holding
//! one interface record, `module.wfm`, and the direct `.wf` implementation
//! records beside it.

use std::path::{Path, PathBuf};

use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::syntax::{FinalizedExtent, NodeId};
use crate::{
    CanonicalSyntaxUnit, ModuleId, ModuleRecord, Production, SourceRole, SyntaxCoordinate,
};

/// The logical path of a module program's graph record.
pub const GRAPH_FILE_NAME: &str = "modules.wfg";

/// The file name of every module's interface record [MOD-2].
pub const INTERFACE_FILE_NAME: &str = "module.wfm";

/// One named entry of a module graph [MOD-9].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEntry {
    name: String,
    module: ModuleId,
    function: String,
    no_heap: bool,
    coordinate: SyntaxCoordinate,
    /// The entry's location in the graph record and that line, for a
    /// rejection that cites the entry [MOD-9].
    written: Option<(crate::SourceLocation, String)>,
}

impl GraphEntry {
    /// Returns the entry's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the module that declares the entry function.
    #[must_use]
    pub const fn module(&self) -> ModuleId {
        self.module
    }

    /// Returns the entry function's name within its module.
    #[must_use]
    pub fn function(&self) -> &str {
        &self.function
    }

    /// Reports whether the entry states the no-heap requirement [STOR-8].
    #[must_use]
    pub const fn no_heap(&self) -> bool {
        self.no_heap
    }

    /// Returns the entry's coordinate in the graph record.
    #[must_use]
    pub const fn coordinate(&self) -> SyntaxCoordinate {
        self.coordinate
    }

    /// Returns the entry's location in the graph record and the line
    /// holding it.
    #[must_use]
    pub fn written(&self) -> Option<(&crate::SourceLocation, &str)> {
        self.written.as_ref().map(|(at, line)| (at, line.as_str()))
    }
}

/// A formed module graph: its registered modules in row order and its named
/// entries in written order [MOD-1].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleGraph {
    modules: Vec<ModuleRecord>,
    entries: Vec<GraphEntry>,
}

impl ModuleGraph {
    /// Returns every registered module in row order.
    #[must_use]
    pub fn modules(&self) -> &[ModuleRecord] {
        &self.modules
    }

    /// Returns every named entry in written order.
    #[must_use]
    pub fn entries(&self) -> &[GraphEntry] {
        &self.entries
    }

    /// Returns the named entry with this name.
    #[must_use]
    pub fn entry(&self, name: &str) -> Option<&GraphEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// Returns every module a module may name through its dependencies'
    /// interfaces: its direct dependencies and, because an interface closes
    /// over the interfaces it names, theirs in turn [MOD-8].
    #[must_use]
    pub fn dependency_closure(&self, module: ModuleId) -> Vec<ModuleId> {
        let mut closure = Vec::new();
        let mut pending = self
            .modules
            .get(module.index())
            .map_or_else(Vec::new, |record| record.dependencies().to_vec());
        while let Some(next) = pending.pop() {
            if closure.contains(&next) {
                continue;
            }
            closure.push(next);
            if let Some(record) = self.modules.get(next.index()) {
                pending.extend(record.dependencies().iter().copied());
            }
        }
        closure.sort();
        closure
    }

    /// Records where each entry is written, rendered by `locate` from its
    /// coordinate in the graph record.
    pub(crate) fn locate_entries(
        &mut self,
        locate: impl Fn(SyntaxCoordinate) -> Option<(crate::SourceLocation, String)>,
    ) {
        for entry in &mut self.entries {
            entry.written = locate(entry.coordinate);
        }
    }

    /// Returns the module registered at this qualified name, `pkg` or
    /// `pkg::a::b`.
    #[must_use]
    pub fn module_named(&self, qualified: &str) -> Option<ModuleId> {
        let path = module_components(qualified)?;
        self.modules
            .iter()
            .position(|module| module.path() == path.as_slice())
            .and_then(ModuleId::from_index)
    }
}

/// The path components of a written qualified module name, `pkg::a::b`.
fn module_components(qualified: &str) -> Option<Vec<String>> {
    let mut parts = qualified.split("::");
    if parts.next()? != "pkg" {
        return None;
    }
    Some(parts.map(str::to_owned).collect())
}

/// Why a graph row or entry is refused [MOD-1].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphIssueKind {
    /// A second row registers an already registered module.
    DuplicateModule {
        /// The module's qualified name.
        path: String,
    },
    /// A row lists its own module as a dependency.
    SelfDependency {
        /// The module's qualified name.
        path: String,
    },
    /// A row lists one dependency twice.
    DuplicateDependency {
        /// The dependency's qualified name.
        path: String,
    },
    /// A row lists a module that no row registers.
    UnregisteredDependency {
        /// The dependency's qualified name.
        path: String,
    },
    /// A row lists a module registered only by a later row; dependencies
    /// point to earlier rows, which is what keeps the graph acyclic.
    LaterDependency {
        /// The dependency's qualified name.
        path: String,
    },
    /// A second entry takes an already used entry name.
    DuplicateEntry {
        /// The entry name.
        name: String,
    },
    /// An entry's target names no function of a registered module.
    UnregisteredEntryModule {
        /// The written target.
        target: String,
    },
}

/// One refused graph row or entry, at its written path [MOD-1].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphIssue {
    coordinate: SyntaxCoordinate,
    kind: GraphIssueKind,
}

impl GraphIssue {
    /// Returns the numbered rule owning the rejection.
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        "MOD-1"
    }

    /// Returns the coordinate of the offending written path.
    #[must_use]
    pub const fn coordinate(&self) -> SyntaxCoordinate {
        self.coordinate
    }

    /// Returns the structured payload.
    #[must_use]
    pub const fn kind(&self) -> &GraphIssueKind {
        &self.kind
    }
}

/// Trusted graph-formation invariant failure, never a source rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphCompilerFailure {
    /// The canonical graph tree did not have the `graph_file` shape.
    InvalidGraphTree,
}

impl core::fmt::Display for GraphCompilerFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("the canonical graph tree does not have the graph_file shape")
    }
}

/// One written module path and where it is written.
struct WrittenPath {
    components: Vec<String>,
    coordinate: SyntaxCoordinate,
}

impl WrittenPath {
    fn qualified(&self) -> String {
        let mut text = String::from("pkg");
        for component in &self.components {
            text.push_str("::");
            text.push_str(component);
        }
        text
    }
}

/// Forms the graph a canonical `graph_file` unit writes [MOD-1].
///
/// Rows are read in written order. Each row registers one module; each of
/// its dependencies must be an earlier row, other than itself, listed once.
/// Each entry takes a fresh name and targets a function of a registered
/// module: the last component of its written path is the function's name and
/// the components before it name the module. The first refusal in written
/// order is returned.
pub(crate) fn form_graph(
    unit: &CanonicalSyntaxUnit<'_, '_, '_>,
) -> Result<Result<ModuleGraph, GraphIssue>, GraphCompilerFailure> {
    let topology = &unit.finalized.topology;
    let classified = unit.classified_bundle();
    let mut direct = vec![Vec::new(); topology.nodes.len()];
    for (index, terminal) in topology.terminals.iter().enumerate() {
        let owner = terminal
            .owner
            .ok_or(GraphCompilerFailure::InvalidGraphTree)?;
        direct
            .get_mut(owner.index())
            .ok_or(GraphCompilerFailure::InvalidGraphTree)?
            .push(index);
    }
    let spelling = |terminal: usize| -> Result<String, GraphCompilerFailure> {
        let token = classified
            .tokens()
            .get(terminal)
            .ok_or(GraphCompilerFailure::InvalidGraphTree)?
            .token();
        std::str::from_utf8(token.span().bytes())
            .map(str::to_owned)
            .map_err(|_| GraphCompilerFailure::InvalidGraphTree)
    };
    let has = |terminal: usize, predicate: TerminalPredicate| {
        classified
            .tokens()
            .get(terminal)
            .is_some_and(|token| token.terminals().contains(predicate))
    };
    let coordinate = |node: NodeId| -> Result<SyntaxCoordinate, GraphCompilerFailure> {
        let record = topology
            .node(node)
            .ok_or(GraphCompilerFailure::InvalidGraphTree)?;
        let FinalizedExtent::Source { source, start, end } = record.extent else {
            return Err(GraphCompilerFailure::InvalidGraphTree);
        };
        Ok(SyntaxCoordinate::new(source, start, end))
    };
    let children_with = |node: NodeId, production: Production| -> Vec<NodeId> {
        topology
            .node_children(node)
            .unwrap_or(&[])
            .iter()
            .copied()
            .filter(|child| {
                topology
                    .node(*child)
                    .is_some_and(|record| record.production == production)
            })
            .collect()
    };
    let written_path = |node: NodeId| -> Result<WrittenPath, GraphCompilerFailure> {
        let mut components = Vec::new();
        for terminal in direct.get(node.index()).map_or(&[][..], Vec::as_slice) {
            if has(*terminal, TerminalPredicate::Identifier) {
                components.push(spelling(*terminal)?);
            }
        }
        Ok(WrittenPath {
            components,
            coordinate: coordinate(node)?,
        })
    };

    let root = topology.root;
    if topology.node(root).map(|record| record.production) != Some(Production::GraphFile) {
        return Err(GraphCompilerFailure::InvalidGraphTree);
    }
    let mut modules: Vec<ModuleRecord> = Vec::new();
    for row in children_with(root, Production::ModuleRow) {
        let paths = children_with(row, Production::ModulePath);
        let Some((module, dependencies)) = paths.split_first() else {
            return Err(GraphCompilerFailure::InvalidGraphTree);
        };
        let module = written_path(*module)?;
        if modules
            .iter()
            .any(|registered| registered.path() == module.components.as_slice())
        {
            return Ok(Err(GraphIssue {
                coordinate: module.coordinate,
                kind: GraphIssueKind::DuplicateModule {
                    path: module.qualified(),
                },
            }));
        }
        let mut edges = Vec::new();
        for dependency in dependencies {
            let dependency = written_path(*dependency)?;
            let issue = |kind| {
                Ok(Err(GraphIssue {
                    coordinate: dependency.coordinate,
                    kind,
                }))
            };
            if dependency.components == module.components {
                return issue(GraphIssueKind::SelfDependency {
                    path: dependency.qualified(),
                });
            }
            let Some(target) = modules
                .iter()
                .position(|registered| registered.path() == dependency.components.as_slice())
                .and_then(ModuleId::from_index)
            else {
                let later = children_with(root, Production::ModuleRow)
                    .into_iter()
                    .filter_map(|later| {
                        children_with(later, Production::ModulePath)
                            .first()
                            .copied()
                    })
                    .map(written_path)
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .any(|registered| registered.components == dependency.components);
                return issue(if later {
                    GraphIssueKind::LaterDependency {
                        path: dependency.qualified(),
                    }
                } else {
                    GraphIssueKind::UnregisteredDependency {
                        path: dependency.qualified(),
                    }
                });
            };
            if edges.contains(&target) {
                return issue(GraphIssueKind::DuplicateDependency {
                    path: dependency.qualified(),
                });
            }
            edges.push(target);
        }
        modules.push(ModuleRecord::new(module.components, edges));
    }

    let mut entries: Vec<GraphEntry> = Vec::new();
    for entry in children_with(root, Production::EntryDecl) {
        let terminals = direct.get(entry.index()).map_or(&[][..], Vec::as_slice);
        let name_terminal = terminals
            .iter()
            .copied()
            .find(|terminal| has(*terminal, TerminalPredicate::Identifier))
            .ok_or(GraphCompilerFailure::InvalidGraphTree)?;
        let name = spelling(name_terminal)?;
        let no_heap = terminals
            .iter()
            .any(|terminal| has(*terminal, TerminalPredicate::Fixed(FixedTerminal::NoHeap)));
        let [target] = children_with(entry, Production::ModulePath)[..] else {
            return Err(GraphCompilerFailure::InvalidGraphTree);
        };
        let target = written_path(target)?;
        if entries.iter().any(|earlier| earlier.name == name) {
            return Ok(Err(GraphIssue {
                coordinate: coordinate(entry)?,
                kind: GraphIssueKind::DuplicateEntry { name },
            }));
        }
        let Some((function, module_path)) = target.components.split_last() else {
            return Ok(Err(GraphIssue {
                coordinate: target.coordinate,
                kind: GraphIssueKind::UnregisteredEntryModule {
                    target: target.qualified(),
                },
            }));
        };
        let Some(module) = modules
            .iter()
            .position(|registered| registered.path() == module_path)
            .and_then(ModuleId::from_index)
        else {
            return Ok(Err(GraphIssue {
                coordinate: target.coordinate,
                kind: GraphIssueKind::UnregisteredEntryModule {
                    target: target.qualified(),
                },
            }));
        };
        entries.push(GraphEntry {
            name,
            module,
            function: function.clone(),
            no_heap,
            coordinate: coordinate(entry)?,
            written: None,
        });
    }
    Ok(Ok(ModuleGraph { modules, entries }))
}

/// One module source read from the package directory [MOD-2].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleSourceFile {
    /// The module the record belongs to.
    pub module: ModuleId,
    /// Whether the record is the module's interface or an implementation.
    pub role: SourceRole,
    /// The record's portable path below the package root.
    pub logical_path: String,
    /// The host path it was read from, for diagnostics.
    pub display_path: String,
    /// The record's exact bytes.
    pub bytes: Vec<u8>,
}

/// Why a package directory cannot supply a registered module's records.
///
/// These are input-envelope failures of the invocation, not source-language
/// rejections: the package directory is not in the shape [MOD-2] requires.
#[derive(Debug)]
pub enum DiscoveryFailure {
    /// A registered module's directory is absent or is not a directory.
    MissingModuleDirectory {
        /// The directory.
        path: PathBuf,
    },
    /// A registered module has no `module.wfm`.
    MissingInterface {
        /// The expected interface path.
        path: PathBuf,
    },
    /// A root, module directory or source record is a symbolic link, whose
    /// ownership would depend on host path resolution.
    SymbolicLink {
        /// The link.
        path: PathBuf,
    },
    /// Two entries of one module directory differ only in letter case.
    CaseCollision {
        /// The first entry.
        first: PathBuf,
        /// The second entry.
        second: PathBuf,
    },
    /// A source record's file name is not a portable logical path component.
    InvalidFileName {
        /// The record.
        path: PathBuf,
    },
    /// The selected graph record is not a file named `modules.wfg`.
    GraphRecordName {
        /// The selected path.
        path: PathBuf,
    },
    /// A directory or record cannot be read.
    Unreadable {
        /// The path.
        path: PathBuf,
        /// The host error.
        error: std::io::Error,
    },
}

impl core::fmt::Display for DiscoveryFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingModuleDirectory { path } => write!(
                formatter,
                "[MOD-2] the registered module's directory {} does not exist",
                path.display()
            ),
            Self::MissingInterface { path } => write!(
                formatter,
                "[MOD-2] the registered module has no interface record {}",
                path.display()
            ),
            Self::SymbolicLink { path } => write!(
                formatter,
                "[MOD-2] {} is a symbolic link; module sources are read only from real directories and files",
                path.display()
            ),
            Self::CaseCollision { first, second } => write!(
                formatter,
                "[MOD-2] {} and {} differ only in letter case",
                first.display(),
                second.display()
            ),
            Self::InvalidFileName { path } => write!(
                formatter,
                "[MOD-2] {} is not a portable source record name",
                path.display()
            ),
            Self::GraphRecordName { path } => write!(
                formatter,
                "[MOD-1] {} is not a graph record; a module program is selected by the file {GRAPH_FILE_NAME} in its package root",
                path.display()
            ),
            Self::Unreadable { path, error } => {
                write!(formatter, "cannot read {}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for DiscoveryFailure {}

/// Reads the graph record a build selects [MOD-1]: a regular file named
/// `modules.wfg`, not a symbolic link, whose directory is the package root.
pub fn read_graph_record(path: &Path) -> Result<Vec<u8>, DiscoveryFailure> {
    if path.file_name() != Some(std::ffi::OsStr::new(GRAPH_FILE_NAME)) {
        return Err(DiscoveryFailure::GraphRecordName {
            path: path.to_path_buf(),
        });
    }
    let unreadable = |error| DiscoveryFailure::Unreadable {
        path: path.to_path_buf(),
        error,
    };
    let metadata = std::fs::symlink_metadata(path).map_err(unreadable)?;
    if metadata.file_type().is_symlink() {
        return Err(DiscoveryFailure::SymbolicLink {
            path: path.to_path_buf(),
        });
    }
    if !metadata.is_file() {
        return Err(DiscoveryFailure::GraphRecordName {
            path: path.to_path_buf(),
        });
    }
    std::fs::read(path).map_err(unreadable)
}

/// Reads every registered module's records from the package directory
/// [MOD-2]: each module's `module.wfm` and then its direct `.wf` files in byte
/// order of their names, modules in row order. Child directories are not
/// read into their parent, and no symbolic link or case-folding collision is
/// followed or tolerated.
pub fn discover_module_sources(
    root: &Path,
    graph: &ModuleGraph,
) -> Result<Vec<ModuleSourceFile>, DiscoveryFailure> {
    let unreadable = |path: &Path, error| DiscoveryFailure::Unreadable {
        path: path.to_path_buf(),
        error,
    };
    let reject_link = |path: &Path| -> Result<std::fs::Metadata, DiscoveryFailure> {
        let metadata = std::fs::symlink_metadata(path).map_err(|error| unreadable(path, error))?;
        if metadata.file_type().is_symlink() {
            return Err(DiscoveryFailure::SymbolicLink {
                path: path.to_path_buf(),
            });
        }
        Ok(metadata)
    };
    reject_link(root)?;
    let mut sources = Vec::new();
    for (index, module) in graph.modules().iter().enumerate() {
        let module_id =
            ModuleId::from_index(index).ok_or_else(|| DiscoveryFailure::Unreadable {
                path: root.to_path_buf(),
                error: std::io::Error::other("too many modules"),
            })?;
        let mut directory = root.to_path_buf();
        for component in module.path() {
            directory.push(component);
            match std::fs::symlink_metadata(&directory) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(DiscoveryFailure::SymbolicLink { path: directory });
                }
                Ok(metadata) if metadata.is_dir() => {}
                _ => return Err(DiscoveryFailure::MissingModuleDirectory { path: directory }),
            }
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&directory).map_err(|error| unreadable(&directory, error))? {
            let entry = entry.map_err(|error| unreadable(&directory, error))?;
            names.push(entry.file_name());
        }
        let mut lowered: Vec<(String, PathBuf)> = Vec::new();
        for name in &names {
            let path = directory.join(name);
            let lower = name.to_string_lossy().to_lowercase();
            if let Some((_, first)) = lowered.iter().find(|(seen, _)| *seen == lower) {
                return Err(DiscoveryFailure::CaseCollision {
                    first: first.clone(),
                    second: path,
                });
            }
            lowered.push((lower, path));
        }
        let prefix = module.path().join("/");
        let logical = |file: &str| {
            if prefix.is_empty() {
                file.to_owned()
            } else {
                format!("{prefix}/{file}")
            }
        };
        let interface = directory.join(INTERFACE_FILE_NAME);
        match std::fs::symlink_metadata(&interface) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(DiscoveryFailure::SymbolicLink { path: interface });
            }
            Ok(metadata) if metadata.is_file() => {}
            _ => return Err(DiscoveryFailure::MissingInterface { path: interface }),
        }
        sources.push(ModuleSourceFile {
            module: module_id,
            role: SourceRole::Interface,
            logical_path: logical(INTERFACE_FILE_NAME),
            display_path: interface.display().to_string(),
            bytes: std::fs::read(&interface).map_err(|error| unreadable(&interface, error))?,
        });
        let mut records = Vec::new();
        for name in names {
            // A record is named by its bytes: a name ending in `.wf` that is
            // not a portable component is refused, never skipped.
            if !name.as_encoded_bytes().ends_with(b".wf") {
                continue;
            }
            let path = directory.join(&name);
            let metadata = reject_link(&path)?;
            if !metadata.is_file() {
                continue;
            }
            let Some(text) = name.to_str().filter(|text| {
                text.bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            }) else {
                return Err(DiscoveryFailure::InvalidFileName { path });
            };
            records.push((text.to_owned(), path));
        }
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        for (name, path) in records {
            sources.push(ModuleSourceFile {
                module: module_id,
                role: SourceRole::Implementation,
                logical_path: logical(&name),
                display_path: path.display().to_string(),
                bytes: std::fs::read(&path).map_err(|error| unreadable(&path, error))?,
            });
        }
    }
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryFailure, read_graph_record};

    /// A fresh directory for one test, removed by the returned guard.
    struct Directory(std::path::PathBuf);

    impl Directory {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("whitefoot-graph-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create the test directory");
            Self(path)
        }
    }

    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// [MOD-1] a module program is selected by its `modules.wfg` alone: a
    /// graph under another name, and one reached through a symbolic link,
    /// are refused before any byte of them is read.
    #[test]
    fn only_a_regular_modules_wfg_is_a_graph_record() {
        let directory = Directory::new("record");
        let graph = directory.0.join("modules.wfg");
        std::fs::write(&graph, b"pkg: [];\n").expect("write the graph");
        assert_eq!(
            read_graph_record(&graph).expect("the graph record"),
            b"pkg: [];\n"
        );
        let other = directory.0.join("other.wfg");
        std::fs::write(&other, b"pkg: [];\n").expect("write another graph");
        assert!(matches!(
            read_graph_record(&other),
            Err(DiscoveryFailure::GraphRecordName { .. })
        ));
        #[cfg(unix)]
        {
            let linked = directory.0.join("linked");
            std::fs::create_dir_all(&linked).expect("create a directory");
            std::os::unix::fs::symlink(&graph, linked.join("modules.wfg")).expect("link the graph");
            assert!(matches!(
                read_graph_record(&linked.join("modules.wfg")),
                Err(DiscoveryFailure::SymbolicLink { .. })
            ));
        }
    }

    /// [MOD-2] a directory entry named by bytes ending in `.wf` is a record
    /// of its module, so one whose name is not a portable component is
    /// refused rather than skipped.
    #[cfg(unix)]
    #[test]
    fn a_record_name_that_is_not_utf8_is_refused() {
        use std::os::unix::ffi::OsStrExt;

        let directory = Directory::new("names");
        std::fs::write(directory.0.join("modules.wfg"), b"pkg: [];\n").expect("write the graph");
        std::fs::write(directory.0.join("module.wfm"), b"\n").expect("write the interface");
        let name = std::ffi::OsStr::from_bytes(b"bad\xff.wf");
        std::fs::write(directory.0.join(name), b"\n").expect("write the record");
        let graph = crate::form_module_graph(
            crate::SourceInput::new("modules.wfg", b"pkg: [];\n"),
            crate::CompilerLimits::default(),
        )
        .expect("the graph forms");
        assert!(matches!(
            super::discover_module_sources(&directory.0, &graph),
            Err(DiscoveryFailure::InvalidFileName { .. })
        ));
    }
}
