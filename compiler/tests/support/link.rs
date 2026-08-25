use std::path::{Path, PathBuf};
use std::process::Command;

use whitefoot::{
    COMPLETION_CONTRACT_HEADER, COMPLETION_PLATFORM_FILE_NAME, COMPLETION_PLATFORM_HEADER,
    COMPLETION_PLATFORM_SOURCE, COMPLETION_RUNTIME_SOURCE, FLOOR_RUNTIME_SOURCE,
    PARALLEL_RUNTIME_SOURCE, module_requires_completion_runtime, module_requires_parallel_runtime,
};

/// Adds the exact embedded runtime units selected by the production driver.
///
/// Integration tests have several executable link paths, but runtime
/// selection is one contract: the floor is unconditional, completion joins a
/// module that names an adapter or the parallel ABI, and the parallel unit
/// joins only a module carrying that ABI's fallback marker.
pub(crate) fn add_embedded_runtimes(
    command: &mut Command,
    module: &str,
    directory: &Path,
) -> Vec<PathBuf> {
    let mut artifacts = Vec::new();
    let floor = directory.join("wf_floor.c");
    std::fs::write(&floor, FLOOR_RUNTIME_SOURCE).expect("write the floor runtime");
    command.arg("-pthread").arg("-x").arg("c").arg(&floor);
    artifacts.push(floor);

    let needs_parallel = module_requires_parallel_runtime(module);
    if needs_parallel || module_requires_completion_runtime(module) {
        let header = directory.join("contract.h");
        let platform_header = directory.join("platform.h");
        let shared = directory.join("completion_runtime.c");
        let platform = directory.join(COMPLETION_PLATFORM_FILE_NAME);
        std::fs::write(&header, COMPLETION_CONTRACT_HEADER).expect("write completion contract");
        std::fs::write(&platform_header, COMPLETION_PLATFORM_HEADER)
            .expect("write completion platform contract");
        std::fs::write(&shared, COMPLETION_RUNTIME_SOURCE).expect("write completion runtime");
        std::fs::write(&platform, COMPLETION_PLATFORM_SOURCE).expect("write completion backend");
        command.arg("-I").arg(directory);
        command.arg("-x").arg("c").arg(&shared);
        command.arg("-x").arg("c").arg(&platform);
        artifacts.extend([header, platform_header, shared, platform]);
    }

    if needs_parallel {
        let parallel = directory.join("par_runtime.c");
        std::fs::write(&parallel, PARALLEL_RUNTIME_SOURCE).expect("write the parallel runtime");
        command.arg("-x").arg("c").arg(&parallel);
        artifacts.push(parallel);
    }
    artifacts
}

pub(crate) fn remove_embedded_runtimes(artifacts: Vec<PathBuf>) {
    for artifact in artifacts {
        std::fs::remove_file(artifact).expect("remove an embedded runtime artifact");
    }
}
