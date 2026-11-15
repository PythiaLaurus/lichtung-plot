"""Maturin hooks with a lock-consistent source-distribution boundary."""

from __future__ import annotations

import argparse
from collections.abc import Mapping
import copy
from dataclasses import dataclass
from io import BytesIO
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import shutil
import subprocess
import tarfile
import tempfile

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

from maturin import (
    build_editable as build_editable,
    build_wheel as build_wheel,
    get_requires_for_build_editable as get_requires_for_build_editable,
    get_requires_for_build_sdist as get_requires_for_build_sdist,
    get_requires_for_build_wheel as get_requires_for_build_wheel,
    prepare_metadata_for_build_editable as prepare_metadata_for_build_editable,
    prepare_metadata_for_build_wheel as prepare_metadata_for_build_wheel,
)


PackageId = tuple[str, str, str | None]


def _toml(data: bytes) -> dict:
    return tomllib.loads(data.decode("utf-8"))


def _same_value(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict) and isinstance(right, dict):
        return left.keys() == right.keys() and all(
            _same_value(value, right[key]) for key, value in left.items()
        )
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(
            _same_value(a, b) for a, b in zip(left, right)
        )
    return left == right


def _packages(lock: dict) -> dict[PackageId, dict]:
    result = {}
    packages = lock.get("package")
    if not isinstance(packages, list):
        raise ValueError("Cargo.lock must contain a package array")
    for package in packages:
        if not isinstance(package, dict):
            raise ValueError("invalid Cargo.lock package")
        name, version, source = (
            package.get("name"),
            package.get("version"),
            package.get("source"),
        )
        if (
            not isinstance(name, str)
            or not name
            or not isinstance(version, str)
            or not version
            or (source is not None and (not isinstance(source, str) or not source))
        ):
            raise ValueError("invalid Cargo.lock package identity")
        identity = (name, version, source)
        if identity in result:
            raise ValueError(f"duplicate Cargo.lock package: {identity!r}")
        result[identity] = package
    return result


def _resolve_dependency(value: str, inventory: dict[PackageId, dict]) -> PackageId:
    head, separator, qualifier = value.rpartition(" (")
    source: str | None = None
    if separator:
        if not qualifier.endswith(")") or not qualifier[:-1]:
            raise ValueError(f"invalid Cargo.lock dependency: {value!r}")
        source = qualifier[:-1]
    else:
        head = value
    parts = head.split()
    if len(parts) not in (1, 2):
        raise ValueError(f"invalid Cargo.lock dependency: {value!r}")
    name = parts[0]
    version = parts[1] if len(parts) == 2 else None
    matches = [
        identity
        for identity in inventory
        if identity[0] == name
        and (version is None or identity[1] == version)
        and (source is None or identity[2] == source)
    ]
    if len(matches) != 1:
        raise ValueError(f"unknown or ambiguous Cargo.lock dependency: {value!r}")
    return matches[0]


def _edges(package: dict, inventory: dict[PackageId, dict]) -> set[PackageId]:
    dependencies = package.get("dependencies", [])
    if not isinstance(dependencies, list) or not all(
        isinstance(value, str) for value in dependencies
    ):
        raise ValueError("invalid Cargo.lock dependency array")
    return {_resolve_dependency(value, inventory) for value in dependencies}


def _validate_lock(before_bytes: bytes, after_bytes: bytes) -> None:
    """Allow pruning, not pin changes or retargeting an existing dependency."""
    before, after = _toml(before_bytes), _toml(after_bytes)
    if not _same_value(
        {key: value for key, value in before.items() if key != "package"},
        {key: value for key, value in after.items() if key != "package"},
    ):
        raise ValueError("Cargo.lock top-level fields changed")
    original, retained = _packages(before), _packages(after)
    for identity, package in retained.items():
        if identity not in original:
            raise ValueError(f"new Cargo.lock package: {identity!r}")
        previous = original[identity]
        if not _same_value(
            {key: value for key, value in previous.items() if key != "dependencies"},
            {key: value for key, value in package.items() if key != "dependencies"},
        ):
            raise ValueError(f"Cargo.lock package fields changed: {identity!r}")
        if not _edges(package, retained) <= _edges(previous, original):
            raise ValueError(f"Cargo.lock dependency retargeted or added: {identity!r}")


def _relative_parts(value: str) -> tuple[str, ...]:
    path = PurePosixPath(value)
    if (
        not value
        or "\0" in value
        or "\\" in value
        or path.is_absolute()
        or any(PureWindowsPath(part).drive for part in path.parts)
        or any(
            part in ("", ".", "..") for part in value.removesuffix("/").split("/")
        )
    ):
        raise ValueError(f"unsafe source-distribution path: {value!r}")
    return path.parts


@dataclass(frozen=True)
class _Member:
    info: tarfile.TarInfo
    parts: tuple[str, ...]
    data: bytes | None


def _read_sdist(path: Path) -> tuple[str, list[_Member]]:
    members = []
    seen = set()
    roots = set()
    files = set()
    with tarfile.open(path, "r:gz") as archive:
        for info in archive:
            parts = _relative_parts(info.name)
            if parts in seen:
                raise ValueError(f"duplicate source-distribution member: {info.name!r}")
            seen.add(parts)
            roots.add(parts[0])
            if info.type == tarfile.DIRTYPE:
                if info.size != 0:
                    raise ValueError(
                        "source-distribution directories cannot have payloads"
                    )
                data = None
            elif (
                info.type in (tarfile.REGTYPE, tarfile.AREGTYPE)
                and info.sparse is None
            ):
                if len(parts) < 2 or info.name.endswith("/") or info.size < 0:
                    raise ValueError("source-distribution root must be a directory")
                stream = archive.extractfile(info)
                if stream is None:
                    raise ValueError(f"unreadable source member: {info.name!r}")
                with stream:
                    data = stream.read()
                if len(data) != info.size:
                    raise ValueError(f"incomplete source member: {info.name!r}")
                files.add(parts)
            else:
                raise ValueError(f"link or special source member: {info.name!r}")
            members.append(_Member(info, parts, data))
    if len(roots) != 1:
        raise ValueError("source distribution must have exactly one root")
    for member in members:
        if any(member.parts[:index] in files for index in range(1, len(member.parts))):
            raise ValueError("a source-distribution file cannot own child paths")
    return roots.pop(), members


def _extract(members: list[_Member], directory: Path) -> None:
    # No tarfile extraction API, links, ownership changes, or executable hooks.
    for member in members:
        target = directory.joinpath(*member.parts)
        if member.data is None:
            target.mkdir(parents=True, exist_ok=True)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as output:
                output.write(member.data)


def _layout(project: Path) -> tuple[Path, Path]:
    document = _toml((project / "pyproject.toml").read_bytes())
    system = document.get("build-system", {})
    if (
        not isinstance(system, dict)
        or system.get("build-backend") != "_build_backend"
        or system.get("backend-path") != ["."]
        or not (project / "_build_backend.py").is_file()
    ):
        raise ValueError("pyproject.toml requires its root-local _build_backend.py")
    tool = document.get("tool", {})
    if not isinstance(tool, dict) or not isinstance(tool.get("maturin", {}), dict):
        raise ValueError("invalid tool.maturin table")
    manifest_name = tool.get("maturin", {}).get("manifest-path", "Cargo.toml")
    if not isinstance(manifest_name, str):
        raise ValueError("invalid maturin manifest-path")
    manifest = project.joinpath(*_relative_parts(manifest_name))
    if not manifest.is_file():
        raise ValueError("binding Cargo.toml is missing")
    for directory in (manifest.parent, *manifest.parent.parents):
        candidate = directory / "Cargo.toml"
        if candidate.is_file() and isinstance(
            _toml(candidate.read_bytes()).get("workspace"), dict
        ):
            lock = directory / "Cargo.lock"
            if not lock.is_file():
                raise ValueError("workspace Cargo.lock is missing")
            return manifest, lock
    raise ValueError("binding workspace root was not found")


def _write_raw(project: Path, directory: Path, env: dict[str, str]) -> Path:
    selector = env.get("LICHTUNG_MATURIN", "maturin")
    if os.path.dirname(selector):
        selector = os.path.abspath(selector)
    result = subprocess.run(
        [
            selector,
            "pep517",
            "write-sdist",
            "--sdist-directory",
            str(directory),
        ],
        cwd=project,
        env=env,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise ValueError("maturin did not return a source-distribution filename")
    name = Path(lines[-1])
    path = name if name.is_absolute() else directory / name
    if (
        path.parent.resolve() != directory.resolve()
        or not path.name.endswith(".tar.gz")
        or path.is_symlink()
        or not path.is_file()
    ):
        raise ValueError("maturin returned an unexpected source-distribution path")
    return path


def _metadata(manifest: Path, workspace: Path, env: dict[str, str]) -> None:
    subprocess.run(
        [
            "cargo",
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest),
        ],
        cwd=workspace,
        env=env,
        check=True,
        stdout=subprocess.DEVNULL,
    )


def _verify_stage(
    members: list[_Member], directory: Path, lock_parts: tuple[str, ...]
) -> None:
    for member in members:
        target = directory.joinpath(*member.parts)
        if target.is_symlink() or not target.resolve().is_relative_to(
            directory.resolve()
        ):
            raise ValueError("staged source member escaped its private directory")
        if member.data is None:
            if not target.is_dir():
                raise ValueError(f"staged directory changed: {member.info.name!r}")
        elif not target.is_file() or (
            member.parts != lock_parts and target.read_bytes() != member.data
        ):
            raise ValueError(f"staged source member changed: {member.info.name!r}")


def _repack(
    members: list[_Member], path: Path, lock_parts: tuple[str, ...], lock_bytes: bytes
) -> None:
    with tarfile.open(path, "w:gz") as archive:
        for member in members:
            info = copy.copy(member.info)
            data = lock_bytes if member.parts == lock_parts else member.data
            if member.parts == lock_parts:
                info.size = len(lock_bytes)
                info.pax_headers = dict(info.pax_headers)
                info.pax_headers.pop("size", None)
            archive.addfile(info, None if data is None else BytesIO(data))
    _, published_members = _read_sdist(path)
    if len(published_members) != len(members):
        raise ValueError("repacked source-distribution membership changed")
    for before, after in zip(members, published_members):
        expected = lock_bytes if before.parts == lock_parts else before.data
        if (
            before.info.name != after.info.name
            or before.info.type != after.info.type
            or expected != after.data
        ):
            raise ValueError("repacking changed a source member other than Cargo.lock")


def _unchanged(path: Path, original: bytes) -> None:
    if not path.is_file() or path.read_bytes() != original:
        raise RuntimeError("source workspace Cargo.lock changed; it was not restored")


def _publish(archive: Path, output_directory: Path) -> None:
    output_directory.mkdir(parents=True, exist_ok=True)
    # A same-filesystem private copy permits atomic, no-replacement publication.
    with tempfile.TemporaryDirectory(
        prefix=".lichtung-sdist-", dir=output_directory
    ) as name:
        temporary = Path(name) / archive.name
        shutil.copyfile(archive, temporary)
        os.link(temporary, output_directory / archive.name)


def build_sdist(
    sdist_directory: str, config_settings: Mapping[str, object] | None = None
) -> str:
    """Build a fresh archive whose projected Cargo graph has a matching lock."""
    if config_settings:
        raise ValueError("maturin 1.16's sdist writer does not accept config_settings")
    project = Path(__file__).resolve().parent
    source_manifest, source_lock = _layout(project)
    original_lock = source_lock.read_bytes()
    try:
        helper = (project / "_build_backend.py").read_bytes()
        output_directory = Path(sdist_directory).absolute()
        env = os.environ.copy()
        env["CARGO_NET_OFFLINE"] = "true"
        env["MATURIN_NO_INSTALL_RUST"] = "1"
        with tempfile.TemporaryDirectory(prefix="lichtung-sdist-") as name:
            temporary = Path(name)
            raw = temporary / "raw"
            raw.mkdir()
            original = _write_raw(project, raw, env)
            _unchanged(source_lock, original_lock)
            root_name, members = _read_sdist(original)
            stage = temporary / "stage"
            stage.mkdir()
            _extract(members, stage)
            root = stage / root_name
            manifest, lock = _layout(root)
            if (
                lock != root / "Cargo.lock"
                or manifest.relative_to(root)
                != source_manifest.relative_to(source_lock.parent)
                or (root / "_build_backend.py").read_bytes() != helper
            ):
                raise ValueError("source-distribution root/backend layout changed")
            lock_parts = (root_name, "Cargo.lock")
            staged_lock = lock.read_bytes()
            _validate_lock(original_lock, staged_lock)
            _metadata(manifest, root, env)
            _verify_stage(members, stage, lock_parts)
            normalized_lock = lock.read_bytes()
            _validate_lock(staged_lock, normalized_lock)
            _validate_lock(original_lock, normalized_lock)
            result = temporary / original.name
            _repack(members, result, lock_parts, normalized_lock)
            _unchanged(source_lock, original_lock)
            _publish(result, output_directory)
            return result.name
    finally:
        # Also check failure paths. Never conceal a write by restoring the lock.
        _unchanged(source_lock, original_lock)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True)
    arguments = parser.parse_args()
    print(build_sdist(str(Path(arguments.out).absolute())))
