"""Bounded POSIX child execution and ordinary pinned Rust build."""

from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import pwd
import re
import resource
import shutil
import signal
import stat
import subprocess
import tempfile
import time
from typing import Sequence

from runner_inputs import fail, read_regular


BUILD_TIMEOUT_SECONDS = 300
CLEANUP_TIMEOUT_SECONDS = 2.0
TOOLCHAIN = re.compile(
    rb'\[toolchain\]\nchannel = "([0-9]+\.[0-9]+\.[0-9]+)"\n'
    rb'profile = "minimal"\ncomponents = \["clippy", "rustfmt"\]\n\Z'
)


@dataclass(frozen=True)
class ProcessLimits:
    output_bytes: int
    wall_seconds: float
    cpu_seconds: int
    cleanup_seconds: float = CLEANUP_TIMEOUT_SECONDS


def _child_setup(output_bytes: int, cpu_seconds: int) -> None:
    file_hard = resource.getrlimit(resource.RLIMIT_FSIZE)[1]
    sentinel_cap = output_bytes + 1
    file_cap = sentinel_cap if file_hard == resource.RLIM_INFINITY else min(sentinel_cap, file_hard)
    cpu_hard = resource.getrlimit(resource.RLIMIT_CPU)[1]
    cpu_cap = cpu_seconds if cpu_hard == resource.RLIM_INFINITY else min(cpu_seconds, cpu_hard)
    resource.setrlimit(resource.RLIMIT_FSIZE, (file_cap, file_cap))
    resource.setrlimit(resource.RLIMIT_CPU, (cpu_cap, cpu_cap))


def _group_alive(group: int) -> bool:
    try:
        os.killpg(group, 0)
        return True
    except ProcessLookupError:
        return False
    except (PermissionError, OSError):
        return True


def _cleanup_group(process: subprocess.Popen[bytes], timeout: float) -> None:
    """Kill the entire session even after a successful group leader exit."""

    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    except OSError:
        if process.poll() is None:
            process.kill()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=timeout)
    deadline = time.monotonic() + timeout
    while _group_alive(process.pid):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            break
        except (PermissionError, OSError):
            pass
        if time.monotonic() >= deadline:
            fail("child_cleanup", "an engine descendant survived group cleanup")
        time.sleep(0.01)


def run_child(
    name: str,
    command: Sequence[str],
    cwd: Path,
    frame: bytes,
    limits: ProcessLimits,
) -> bytes:
    environment = {
        "LANG": "C",
        "LC_ALL": "C",
        "PYTHONHASHSEED": "0",
        "TZ": "UTC",
    }
    with (
        tempfile.TemporaryFile() as input_file,
        tempfile.TemporaryFile() as output_file,
        tempfile.TemporaryFile() as error_file,
    ):
        input_file.write(frame)
        input_file.seek(0)
        try:
            process = subprocess.Popen(
                tuple(command),
                cwd=cwd,
                env=environment,
                stdin=input_file,
                stdout=output_file,
                stderr=error_file,
                close_fds=True,
                start_new_session=True,
                preexec_fn=lambda: _child_setup(limits.output_bytes, limits.cpu_seconds),
            )
        except (OSError, ValueError) as error:
            fail("child_spawn", f"{name} could not start: {type(error).__name__}")
        timed_out = False
        try:
            process.wait(timeout=limits.wall_seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
        finally:
            _cleanup_group(process, limits.cleanup_seconds)
        output_size = os.fstat(output_file.fileno()).st_size
        error_size = os.fstat(error_file.fileno()).st_size
        if timed_out:
            fail("child_timeout", f"{name} exceeded its wall-clock limit")
        if process.returncode != 0:
            fail("child_exit", f"{name} exited abnormally with status {process.returncode}")
        if output_size > limits.output_bytes or error_size > limits.output_bytes:
            fail("child_output", f"{name} exceeded its output bound")
        if error_size:
            fail("child_stderr", f"{name} wrote nonempty stderr")
        output_file.seek(0)
        raw = output_file.read(limits.output_bytes + 1)
        if len(raw) != output_size:
            fail("child_output", f"{name} output changed or exceeded its bound")
        return raw


def _toolchain_channel(root: Path) -> str:
    raw = read_regular(root / "rust-toolchain.toml", 1_024, "static Rust toolchain lock")
    match = TOOLCHAIN.fullmatch(raw)
    if match is None:
        fail("static_toolchain", "the static Rust toolchain lock has an unknown format")
    return match.group(1).decode("ascii")


def _default_rustup_home() -> Path:
    configured = os.environ.get("RUSTUP_HOME")
    if configured:
        candidate = Path(configured)
        if not candidate.is_absolute():
            fail("static_build", "RUSTUP_HOME must be an absolute path")
        return candidate
    return Path(pwd.getpwuid(os.getuid()).pw_dir) / ".rustup"


def build_static(
    root: Path,
    target: Path,
    cargo_path: Path | None = None,
    rustup_home: Path | None = None,
) -> Path:
    selected_cargo = str(cargo_path) if cargo_path is not None else shutil.which("cargo")
    if selected_cargo is None:
        fail("cargo_missing", "the pinned Cargo launcher is unavailable")
    cargo = Path(selected_cargo).absolute()
    home = rustup_home if rustup_home is not None else _default_rustup_home()
    if not home.is_absolute() or not home.is_dir():
        fail("static_build", "the required Rustup home is unavailable")
    target.mkdir(parents=True, exist_ok=True)
    target_metadata = target.lstat()
    if stat.S_ISLNK(target_metadata.st_mode) or not stat.S_ISDIR(target_metadata.st_mode):
        fail("static_build", "the fresh Cargo target is not a real directory")
    channel = _toolchain_channel(root)
    with tempfile.TemporaryDirectory(prefix="whitefoot-static-build-") as build_directory:
        build_root = Path(build_directory)
        cargo_home = build_root / "cargo-home"
        isolated_home = build_root / "home"
        temporary = build_root / "tmp"
        working = build_root / "cwd"
        for directory in (cargo_home, isolated_home, temporary, working):
            directory.mkdir()
        environment = {
            "CARGO_HOME": str(cargo_home),
            "CARGO_INCREMENTAL": "0",
            "CARGO_NET_OFFLINE": "true",
            "CARGO_TARGET_DIR": str(target.absolute()),
            "CARGO_TERM_COLOR": "never",
            "HOME": str(isolated_home),
            "LANG": "C",
            "LC_ALL": "C",
            "PATH": os.pathsep.join((str(cargo.parent), "/usr/bin", "/bin")),
            "RUSTFLAGS": "-Cdebuginfo=0 -Ccodegen-units=1",
            "RUSTUP_HOME": str(home),
            "RUSTUP_TOOLCHAIN": channel,
            "SOURCE_DATE_EPOCH": "1",
            "TMPDIR": str(temporary),
            "TZ": "UTC",
        }
        command = (
            str(cargo),
            "build",
            "--locked",
            "--offline",
            "--release",
            "--manifest-path",
            str((root / "Cargo.toml").absolute()),
        )
        try:
            process = subprocess.Popen(
                command,
                cwd=working,
                env=environment,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                close_fds=True,
                start_new_session=True,
            )
        except (OSError, ValueError) as error:
            fail("static_build", f"the static engine build failed: {type(error).__name__}")
        timed_out = False
        try:
            process.wait(timeout=BUILD_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            timed_out = True
        finally:
            _cleanup_group(process, CLEANUP_TIMEOUT_SECONDS)
        if timed_out:
            fail("static_build", "the static engine build exceeded its wall-clock limit")
        if process.returncode != 0:
            fail("static_build", "the static engine build returned failure")
    artifact = target / "release" / "whitefoot-static-grammar-auditor"
    try:
        metadata = artifact.lstat()
    except OSError:
        fail("static_artifact", "the declared static executable is absent")
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        fail("static_artifact", "the declared static executable is not a real regular file")
    return artifact
