#!/usr/bin/env python3
"""Bounded process runner for the non-authorizing v0.9 lexical observer."""

from __future__ import annotations

import os
import subprocess
import threading
from collections.abc import Mapping
from pathlib import Path
from typing import BinaryIO

import v09_lexical_observer as protocol


MAX_RESPONSE_BYTES = 33_554_432
MAX_STDERR_BYTES = 4_096
READ_CHUNK_BYTES = 65_536


class ObserverToolError(RuntimeError):
    """The explicitly selected observer failed instead of publishing a response."""

    def __init__(self, returncode: int, stderr: bytes) -> None:
        super().__init__(f"lexical observer exited with status {returncode}")
        self.returncode = returncode
        self.stderr = stderr


def invoke_observer(
    executable: os.PathLike[str] | str,
    request: protocol.ObserverRequest,
    *,
    timeout_seconds: float = 10.0,
    cwd: os.PathLike[str] | str | None = None,
    environment: Mapping[str, str] | None = None,
) -> protocol.DecodedResponse:
    """Run only an explicitly supplied observer executable and decode its reply."""

    response = invoke_observer_bytes(
        executable,
        request,
        timeout_seconds=timeout_seconds,
        cwd=cwd,
        environment=environment,
    )
    return protocol.decode_response(response, request)


def invoke_observer_bytes(
    executable: os.PathLike[str] | str,
    request: protocol.ObserverRequest,
    *,
    timeout_seconds: float = 10.0,
    cwd: os.PathLike[str] | str | None = None,
    environment: Mapping[str, str] | None = None,
) -> bytes:
    """Run an explicit observer while enforcing live stdout and stderr caps."""

    if type(request) is not protocol.ObserverRequest:
        raise TypeError("observer input must be ObserverRequest")
    if isinstance(timeout_seconds, bool) or timeout_seconds <= 0:
        raise ValueError("observer timeout must be positive")
    executable_path = Path(executable).resolve(strict=True)
    if not executable_path.is_file():
        raise ValueError("observer executable is not a file")

    command = [os.fspath(executable_path)]
    process = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=cwd,
        env={} if environment is None else dict(environment),
        bufsize=0,
    )
    if process.stdin is None or process.stdout is None or process.stderr is None:
        process.kill()
        process.wait()
        raise ObserverToolError(process.returncode, b"observer pipes were unavailable")

    stdout = bytearray()
    stderr = bytearray()
    exceeded: list[str] = []
    pipe_errors: list[str] = []

    def read_bounded(
        pipe: BinaryIO,
        destination: bytearray,
        maximum: int,
        label: str,
    ) -> None:
        try:
            while True:
                chunk = pipe.read(READ_CHUNK_BYTES)
                if not chunk:
                    break
                remaining = maximum - len(destination)
                if len(chunk) > remaining:
                    destination.extend(chunk[: max(remaining, 0)])
                    exceeded.append(label)
                    process.kill()
                    continue
                destination.extend(chunk)
        except OSError:
            pipe_errors.append(label)
            process.kill()
        finally:
            pipe.close()

    def write_request(pipe: BinaryIO) -> None:
        try:
            remaining = memoryview(request.wire_bytes)
            while remaining:
                written = pipe.write(remaining)
                if written is None or written == 0:
                    raise OSError("observer request pipe made no progress")
                remaining = remaining[written:]
            pipe.flush()
        except BrokenPipeError:
            pass
        except OSError:
            pipe_errors.append("stdin")
            process.kill()
        finally:
            pipe.close()

    threads = (
        threading.Thread(
            target=write_request,
            args=(process.stdin,),
            name="whitefoot-observer-stdin",
        ),
        threading.Thread(
            target=read_bounded,
            args=(process.stdout, stdout, MAX_RESPONSE_BYTES, "stdout"),
            name="whitefoot-observer-stdout",
        ),
        threading.Thread(
            target=read_bounded,
            args=(process.stderr, stderr, MAX_STDERR_BYTES, "stderr"),
            name="whitefoot-observer-stderr",
        ),
    )
    for thread in threads:
        thread.start()

    timed_out = False
    try:
        returncode = process.wait(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        timed_out = True
        process.kill()
        returncode = process.wait()
    for thread in threads:
        thread.join()

    if timed_out:
        raise subprocess.TimeoutExpired(
            command,
            timeout_seconds,
            output=bytes(stdout),
            stderr=bytes(stderr),
        )
    if exceeded:
        raise ObserverToolError(
            returncode,
            f"observer {'/'.join(exceeded)} exceeded the harness bound".encode("ascii"),
        )
    if pipe_errors:
        raise ObserverToolError(
            returncode,
            f"observer {'/'.join(pipe_errors)} pipe failed".encode("ascii"),
        )
    diagnostic = bytes(stderr)
    if returncode != 0:
        raise ObserverToolError(returncode, diagnostic)
    if diagnostic:
        raise ObserverToolError(0, b"successful observer wrote to standard error")
    return bytes(stdout)
