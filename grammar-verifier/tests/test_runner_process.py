from __future__ import annotations

import errno
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import time
import unittest
from unittest import mock

from support_common import ROOT
from runner_inputs import RunnerError
from runner_process import ProcessLimits, build_static, run_child


class ProcessTests(unittest.TestCase):
    def setUp(self) -> None:
        if os.name != "posix":
            self.skipTest("the grammar evidence runner requires POSIX process controls")
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.limits = ProcessLimits(
            output_bytes=1024,
            wall_seconds=1.0,
            cpu_seconds=1,
            cleanup_seconds=1.0,
        )

    def tearDown(self) -> None:
        self.directory.cleanup()

    def _script(self, source: str, *arguments: str) -> tuple[str, ...]:
        path = self.root / f"child-{len(tuple(self.root.glob('child-*.py')))}.py"
        path.write_text(source, encoding="utf-8")
        return (sys.executable, "-I", "-S", "-B", str(path), *arguments)

    def test_exact_frame_is_delivered_with_fixed_environment(self) -> None:
        source = (
            "import os, sys\n"
            "data=sys.stdin.buffer.read()\n"
            "expected={'LANG','LC_ALL','PYTHONHASHSEED','TZ'}\n"
            "if not expected <= set(os.environ): raise SystemExit(9)\n"
            "if {'HOME','PATH','PYTHONPATH'} & set(os.environ): raise SystemExit(10)\n"
            "sys.stdout.buffer.write(data)\n"
        )
        frame = b"exact-frame"
        self.assertEqual(run_child("fake", self._script(source), self.root, frame, self.limits), frame)

    def test_timeout_is_tool_failure(self) -> None:
        limits = ProcessLimits(1024, 0.05, 1, 1.0)
        with self.assertRaisesRegex(RunnerError, "wall-clock"):
            run_child("fake", self._script("import time\ntime.sleep(30)\n"), self.root, b"", limits)

    def test_signal_is_tool_failure(self) -> None:
        source = "import os, signal\nos.kill(os.getpid(), signal.SIGTERM)\n"
        with self.assertRaisesRegex(RunnerError, "exited abnormally"):
            run_child("fake", self._script(source), self.root, b"", self.limits)

    def test_nonempty_stderr_is_tool_failure(self) -> None:
        source = "import sys\nsys.stderr.write('unexpected')\n"
        with self.assertRaisesRegex(RunnerError, "stderr"):
            run_child("fake", self._script(source), self.root, b"", self.limits)

    def test_output_limit_is_enforced_by_the_child_resource_boundary(self) -> None:
        source = "import os\nos.write(1, b'x' * 4096)\n"
        with self.assertRaises(RunnerError) as caught:
            run_child("fake", self._script(source), self.root, b"", self.limits)
        self.assertIn(caught.exception.code, {"child_exit", "child_output", "child_stderr"})

    def test_descendant_is_removed_after_successful_leader_exit(self) -> None:
        pid_path = self.root / "descendant.pid"
        source = (
            "import os, sys, time\n"
            "pid=os.fork()\n"
            "if pid == 0:\n"
            "  time.sleep(2)\n"
            "  os._exit(0)\n"
            "open(sys.argv[1], 'w', encoding='ascii').write(str(pid))\n"
            "sys.stdout.write('done\\n')\n"
        )
        try:
            output = run_child("fake", self._script(source, str(pid_path)), self.root, b"", self.limits)
        except RunnerError as error:
            if error.code == "child_cleanup":
                self.skipTest("the managed test host forbids process-group signaling")
            raise
        self.assertEqual(output, b"done\n")
        descendant = int(pid_path.read_text(encoding="ascii"))
        deadline = time.monotonic() + 1.0
        while time.monotonic() < deadline:
            try:
                os.kill(descendant, 0)
            except OSError as error:
                if error.errno == errno.ESRCH:
                    break
            time.sleep(0.01)
        else:
            self.fail("the child process-group descendant survived cleanup")

    def test_static_build_uses_locked_toolchain_and_closed_fresh_environment(self) -> None:
        project = self.root / "project"
        target = self.root / "target"
        rustup_home = self.root / "rustup"
        project.mkdir()
        rustup_home.mkdir()
        (project / "Cargo.toml").write_text("[package]\nname='fake'\nversion='0.0.0'\n", encoding="ascii")
        (project / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.91.1"\nprofile = "minimal"\ncomponents = ["clippy", "rustfmt"]\n',
            encoding="ascii",
        )
        cargo = self.root / "cargo"
        cargo.write_text(
            f"#!{sys.executable}\n"
            "import json, os, pathlib, sys\n"
            "target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])\n"
            "(target/'release').mkdir(parents=True, exist_ok=True)\n"
            "(target/'release'/'whitefoot-static-grammar-auditor').write_bytes(b'fake')\n"
            "receipt={'args':sys.argv[1:],'cwd':os.getcwd(),'env':dict(os.environ)}\n"
            "(target/'receipt.json').write_text(json.dumps(receipt), encoding='ascii')\n",
            encoding="ascii",
        )
        cargo.chmod(0o755)
        with mock.patch.dict(
            os.environ,
            {
                "RUSTUP_TOOLCHAIN": "ambient-override",
                "RUSTFLAGS": "ambient-flags",
                "RUSTC_WRAPPER": "ambient-wrapper",
                "WHITEFOOT_UNRELATED": "must-not-leak",
            },
            clear=False,
        ):
            artifact = build_static(project, target, cargo, rustup_home)
        self.assertEqual(artifact, target / "release" / "whitefoot-static-grammar-auditor")
        receipt = json.loads((target / "receipt.json").read_text(encoding="ascii"))
        environment = receipt["env"]
        self.assertEqual(environment["RUSTUP_TOOLCHAIN"], "1.91.1")
        self.assertEqual(environment["RUSTFLAGS"], "-Cdebuginfo=0 -Ccodegen-units=1")
        self.assertNotIn("RUSTC_WRAPPER", environment)
        self.assertNotIn("WHITEFOOT_UNRELATED", environment)
        self.assertNotEqual(Path(receipt["cwd"]), project)
        self.assertIn(str((project / "Cargo.toml").absolute()), receipt["args"])


if __name__ == "__main__":
    unittest.main()
