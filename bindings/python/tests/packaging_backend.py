"""Explicit source-only tests; every subprocess is intercepted, never executed.

This filename deliberately does not match pytest's installed-wheel collection.
"""

from __future__ import annotations

import importlib.util
from io import BytesIO
import os
from pathlib import Path
import subprocess
import sys
import tarfile

import pytest


BACKEND = Path(__file__).resolve().parents[1] / "_build_backend.py"
# The same fixtures can be selected explicitly from checkout or unpacked sources.
PROJECT = BACKEND.with_name("pyproject.toml").read_bytes().replace(
    b'manifest-path = "bindings/python/Cargo.toml"\n', b""
)
SDIST_PROJECT = PROJECT + b'\nmanifest-path = "bindings/python/Cargo.toml"\n'
FILENAME = "lichtung_plot-0.1.0.tar.gz"
ROOT = "lichtung_plot-0.1.0"
WORKSPACE = b'[workspace]\nmembers = ["bindings/python"]\nresolver = "2"\n'
BINDING = b'[package]\nname = "binding"\nversion = "0.1.0"\n'
SHARED = b'''[[package]]
name = "shared"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "original"
detail = { enabled = true, revision = 1 }
'''
OTHER_VERSION = SHARED.replace(b'"1.0.0"', b'"2.0.0"').replace(
    b'"original"', b'"other-version"'
)
OTHER_SOURCE = SHARED.replace(
    b"https://github.com/rust-lang/crates.io-index",
    b"https://example.invalid/index",
).replace(b'"original"', b'"other-source"')
BEFORE = (
    b'''version = 4
[[package]]
name = "binding"
version = "0.1.0"
dependencies = [
 "shared 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
 "dev-only",
]
'''
    + SHARED
    + b'dependencies = ["dev-only"]\n'
    + OTHER_VERSION
    + OTHER_SOURCE
    + b'[[package]]\nname = "dev-only"\nversion = "0.1.0"\n'
)
AFTER = (
    b'''version = 4
[[package]]
name = "binding"
version = "0.1.0"
dependencies = ["shared"]
'''
    + SHARED
)


@pytest.fixture
def backend(monkeypatch: pytest.MonkeyPatch):
    spec = importlib.util.spec_from_file_location("_packaging_backend_test", BACKEND)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    monkeypatch.setitem(sys.modules, spec.name, module)
    spec.loader.exec_module(module)

    def unexpected(*args, **kwargs):
        raise AssertionError("source-only tests must not execute subprocesses")

    monkeypatch.setattr(module.subprocess, "run", unexpected)
    return module


def _project(tmp_path, backend, monkeypatch, *, unpacked=False):
    workspace = tmp_path / "source"
    binding = workspace / "bindings/python"
    binding.mkdir(parents=True)
    (workspace / "Cargo.toml").write_bytes(WORKSPACE)
    lock = workspace / "Cargo.lock"
    lock.write_bytes(BEFORE)
    (binding / "Cargo.toml").write_bytes(BINDING)
    project = workspace if unpacked else binding
    (project / "pyproject.toml").write_bytes(SDIST_PROJECT if unpacked else PROJECT)
    helper = project / "_build_backend.py"
    helper.write_bytes(BACKEND.read_bytes())
    monkeypatch.setattr(backend, "__file__", str(helper))
    return project, lock


def _members():
    return {
        ROOT: None,
        f"{ROOT}/Cargo.toml": WORKSPACE,
        f"{ROOT}/Cargo.lock": BEFORE,
        f"{ROOT}/pyproject.toml": SDIST_PROJECT,
        f"{ROOT}/_build_backend.py": BACKEND.read_bytes(),
        f"{ROOT}/bindings/python/Cargo.toml": BINDING,
        f"{ROOT}/bindings/python/_build_backend.py": BACKEND.read_bytes(),
        f"{ROOT}/python/lichtung_plot/payload.bin": b"\0source bytes\xff\n",
    }


def _archive(path, entries):
    with tarfile.open(path, "w:gz") as archive:
        for name, data in entries:
            info = tarfile.TarInfo(name) if isinstance(name, str) else name
            if isinstance(name, str) and data is None:
                info.type = tarfile.DIRTYPE
            info.size = 0 if data is None else len(data)
            archive.addfile(info, None if data is None else BytesIO(data))


@pytest.fixture
def staged(tmp_path, backend, monkeypatch):
    project, lock = _project(tmp_path, backend, monkeypatch)
    members = _members()
    owned = []

    def raw(actual_project, directory, env):
        assert actual_project == project
        owned.append(directory.parent)
        archive = directory / FILENAME
        _archive(archive, members.items())
        return archive

    def metadata(manifest, workspace, env):
        assert manifest == workspace / "bindings/python/Cargo.toml"
        (workspace / "Cargo.lock").write_bytes(AFTER)

    monkeypatch.setattr(backend, "_write_raw", raw)
    monkeypatch.setattr(backend, "_metadata", metadata)
    return backend, lock, members, owned


def test_pruning_resolves_qualifiers_in_each_inventory(backend):
    # Removing another version/source permits both source/version qualifiers to
    # disappear; pruning a shared package's optional edge is also legitimate.
    backend._validate_lock(BEFORE, AFTER)


@pytest.mark.parametrize(
    "changed",
    [
        pytest.param(
            AFTER.replace(b"https://github.com/rust-lang/crates.io-index", b"unknown"),
            id="new-source",
        ),
        pytest.param(AFTER.replace(b'"original"', b'"changed"'), id="checksum"),
        pytest.param(
            AFTER + b'[[package]]\nname = "new"\nversion = "1.0.0"\n',
            id="new-package",
        ),
        pytest.param(
            AFTER.replace(SHARED, OTHER_VERSION), id="existing-version-retarget"
        ),
        pytest.param(
            AFTER.replace(SHARED, OTHER_SOURCE), id="existing-source-retarget"
        ),
        pytest.param(
            AFTER.replace(b"enabled = true", b"enabled = 1"), id="parsed-field-type"
        ),
        pytest.param(AFTER + OTHER_VERSION, id="ambiguous-unqualified-edge"),
    ],
)
def test_lock_changes_cannot_escape_the_original_graph(backend, changed):
    with pytest.raises(ValueError):
        backend._validate_lock(BEFORE, changed)


@pytest.mark.parametrize(
    "hazard",
    [
        "absolute", "nested-drive", "traversal", "duplicate", "symlink",
        "hardlink", "special", "roots", "file-parent",
    ],
)
def test_unsafe_archives_are_rejected_before_extraction(tmp_path, backend, hazard):
    entries = [(f"{ROOT}/file", b"payload")]
    if hazard == "absolute":
        entries = [("/escape", b"payload")]
    elif hazard == "nested-drive":
        entries = [(f"{ROOT}/C:/escape", b"payload")]
    elif hazard == "traversal":
        entries = [(f"{ROOT}/../escape", b"payload")]
    elif hazard == "duplicate":
        entries.append((f"{ROOT}/file", b"replacement"))
    elif hazard in ("symlink", "hardlink", "special"):
        info = tarfile.TarInfo(f"{ROOT}/link-or-special")
        info.type = {
            "symlink": tarfile.SYMTYPE,
            "hardlink": tarfile.LNKTYPE,
            "special": tarfile.FIFOTYPE,
        }[hazard]
        info.linkname = "../../escape"
        entries.append((info, None))
    elif hazard == "roots":
        entries.append(("another-root/file", b"payload"))
    else:
        entries.append((f"{ROOT}/file/child", b"payload"))
    archive = tmp_path / FILENAME
    _archive(archive, entries)
    with pytest.raises(ValueError):
        backend._read_sdist(archive)
    assert not (tmp_path / "escape").exists()


@pytest.mark.parametrize("layout", ["checkout", "unpacked"])
def test_sdist_preserves_members_and_uses_project_local_child_context(
    tmp_path, backend, monkeypatch, layout
):
    project, lock = _project(
        tmp_path, backend, monkeypatch, unpacked=layout == "unpacked"
    )
    selected = "./maturin" if layout == "checkout" else "maturin"
    if layout == "checkout":
        monkeypatch.setenv("LICHTUNG_MATURIN", selected)
    else:
        monkeypatch.delenv("LICHTUNG_MATURIN", raising=False)
    monkeypatch.setenv("CARGO_NET_OFFLINE", "false")
    monkeypatch.setenv("MATURIN_NO_INSTALL_RUST", "0")
    before_env, before_cwd = dict(os.environ), Path.cwd()
    executable = str(before_cwd / selected) if layout == "checkout" else selected
    members, calls, owned = _members(), [], []

    def run(command, *, cwd, env, check, stdout, **kwargs):
        assert check and env is not os.environ
        assert env["CARGO_NET_OFFLINE"] == "true"
        assert env["MATURIN_NO_INSTALL_RUST"] == "1"
        calls.append(command)
        if len(calls) == 1:
            assert command[:4] == [
                executable, "pep517", "write-sdist", "--sdist-directory"
            ]
            assert cwd == project and kwargs == {"text": True}
            assert stdout == subprocess.PIPE
            directory = Path(command[4])
            assert directory.is_absolute()
            owned.append(directory.parent)
            _archive(directory / FILENAME, members.items())
            return subprocess.CompletedProcess(command, 0, FILENAME + "\n")
        assert len(calls) == 2
        workspace = Path(cwd)
        assert workspace.is_relative_to(owned[0]) and workspace != lock.parent
        assert command == [
            "cargo", "metadata", "--offline", "--format-version", "1",
            "--manifest-path", str(workspace / "bindings/python/Cargo.toml"),
        ]
        assert stdout == subprocess.DEVNULL and not kwargs
        assert (workspace / "Cargo.lock").read_bytes() == BEFORE
        (workspace / "Cargo.lock").write_bytes(AFTER)
        return subprocess.CompletedProcess(command, 0)

    monkeypatch.setattr(backend.subprocess, "run", run)
    out = tmp_path / "out"
    out.mkdir()
    sentinel = out / "keep"
    sentinel.write_bytes(b"unrelated output")
    assert backend.build_sdist(str(out), {}) == FILENAME
    assert len(calls) == 2
    root, repacked = backend._read_sdist(out / FILENAME)
    assert root == ROOT
    expected = dict(members)
    expected[f"{ROOT}/Cargo.lock"] = AFTER
    assert {member.info.name: member.data for member in repacked} == expected
    assert lock.read_bytes() == BEFORE
    assert all(not directory.exists() for directory in owned)
    assert set(out.iterdir()) == {out / FILENAME, sentinel}
    assert sentinel.read_bytes() == b"unrelated output"
    assert dict(os.environ) == before_env and Path.cwd() == before_cwd


def test_nested_helper_cannot_replace_the_root_backend(tmp_path, staged):
    backend, lock, members, owned = staged
    del members[f"{ROOT}/_build_backend.py"]
    out = tmp_path / "out"
    with pytest.raises(ValueError):
        backend.build_sdist(str(out))
    assert lock.read_bytes() == BEFORE and not out.exists()
    assert owned and all(not directory.exists() for directory in owned)


def test_metadata_failure_cleans_only_private_directories(
    tmp_path, staged, monkeypatch
):
    backend, lock, _, owned = staged
    original = backend._metadata

    def fail(*args):
        original(*args)
        raise subprocess.CalledProcessError(1, ["cargo", "metadata"])

    monkeypatch.setattr(backend, "_metadata", fail)
    out = tmp_path / "out"
    out.mkdir()
    sentinel = out / "keep"
    sentinel.write_bytes(b"unrelated output")
    with pytest.raises(subprocess.CalledProcessError):
        backend.build_sdist(str(out))
    assert lock.read_bytes() == BEFORE
    assert set(out.iterdir()) == {sentinel}
    assert sentinel.read_bytes() == b"unrelated output"
    assert owned and all(not directory.exists() for directory in owned)


def test_source_lock_drift_is_reported_and_never_restored(
    tmp_path, staged, monkeypatch
):
    backend, lock, _, owned = staged
    original = backend._metadata
    drift = b"source lock changed outside staging\n"

    def metadata(*args):
        original(*args)
        lock.write_bytes(drift)
        raise subprocess.CalledProcessError(1, ["cargo", "metadata"])

    monkeypatch.setattr(backend, "_metadata", metadata)
    out = tmp_path / "out"
    with pytest.raises(RuntimeError, match="not restored"):
        backend.build_sdist(str(out))
    assert lock.read_bytes() == drift and not out.exists()
    assert owned and all(not directory.exists() for directory in owned)


@pytest.mark.parametrize("violation", ["manifest", "resurrect-pruned"])
def test_metadata_can_only_prune_the_staged_lock(
    tmp_path, staged, monkeypatch, violation
):
    backend, lock, members, owned = staged
    if violation == "resurrect-pruned":
        members[f"{ROOT}/Cargo.lock"] = AFTER
    original = backend._metadata

    def metadata(manifest, workspace, env):
        original(manifest, workspace, env)
        if violation == "manifest":
            manifest.write_bytes(b"unexpected manifest rewrite\n")
        else:
            (workspace / "Cargo.lock").write_bytes(BEFORE)

    monkeypatch.setattr(backend, "_metadata", metadata)
    out = tmp_path / "out"
    with pytest.raises(ValueError):
        backend.build_sdist(str(out))
    assert lock.read_bytes() == BEFORE and not out.exists()
    assert owned and all(not directory.exists() for directory in owned)


def test_existing_archive_is_not_overwritten(tmp_path, staged):
    backend, lock, _, owned = staged
    out = tmp_path / "out"
    out.mkdir()
    existing = out / FILENAME
    existing.write_bytes(b"previous archive")
    with pytest.raises(FileExistsError):
        backend.build_sdist(str(out))
    assert existing.read_bytes() == b"previous archive"
    assert set(out.iterdir()) == {existing}
    assert lock.read_bytes() == BEFORE
    assert owned and all(not directory.exists() for directory in owned)
