"""Check installed declarations and the bounded public type-boundary fixtures.

No package installation, source-package fallback or recursive runtime reflection.
Run with an isolated interpreter that already has the wheel, providers and mypy.
"""

import os
from pathlib import Path
import subprocess
import sys

import lichtung_plot


def main() -> int:
    if not sys.flags.isolated:
        raise SystemExit("run with python -I -B")
    package = Path(lichtung_plot.__file__).resolve().parent
    if not package.is_relative_to(Path(sys.prefix).resolve()):
        raise SystemExit("lichtung_plot must be installed in the selected prefix")
    tests = Path(__file__).resolve().parent
    modules = [
        "lichtung_plot",
        "lichtung_plot._owners",
        "lichtung_plot._native",
        "typing_inputs",
        "typing_lifecycle",
    ]
    environment = os.environ.copy()
    environment["MYPYPATH"] = str(tests)
    return subprocess.call([
        sys.executable,
        "-I",
        "-B",
        "-m",
        "mypy",
        "--no-incremental",
        "--cache-dir",
        os.devnull,
        "--config-file",
        str(tests / "typing_inputs.ini"),
        "--python-executable",
        sys.executable,
        *(argument for module in modules for argument in ("--module", module)),
    ], cwd=tests.parent, env=environment)


if __name__ == "__main__":
    raise SystemExit(main())
