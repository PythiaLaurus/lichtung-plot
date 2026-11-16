"""Inspect this project's real sdist/wheel without importing its native extension."""

from __future__ import annotations

import argparse
import base64
import csv
from email.parser import BytesParser
from email.policy import default
import hashlib
from io import StringIO
import json
import os
from pathlib import Path, PurePosixPath
import platform as host
import stat
import subprocess
import sys
import tarfile
import zipfile

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def contents(path: Path) -> dict[str, bytes]:
    files = {}
    with zipfile.ZipFile(path) if path.suffix == ".whl" else tarfile.open(path) as archive:
        members = archive.infolist() if isinstance(archive, zipfile.ZipFile) else archive
        seen = set()
        for member in members:
            zipped = isinstance(member, zipfile.ZipInfo)
            name = member.filename if zipped else member.name
            parts = PurePosixPath(name).parts
            require(
                bool(parts)
                and not name.startswith("/")
                and "\\" not in name
                and all(part not in (".", "..") and ":" not in part for part in parts),
                f"unsafe archive member: {name}",
            )
            require(name not in seen, f"duplicate archive member: {name}")
            seen.add(name)
            require(
                not {".git", "target", ".venv", "__pycache__", ".pytest_cache"} & set(parts),
                f"development payload in archive: {name}",
            )
            if member.is_dir() if zipped else member.isdir():
                continue
            require(
                not stat.S_ISLNK(member.external_attr >> 16) if zipped else member.isfile(),
                f"link or special archive member: {name}",
            )
            if zipped:
                files[name] = archive.read(member)
            else:
                stream = archive.extractfile(member)
                require(stream is not None, f"unreadable archive member: {name}")
                with stream:
                    files[name] = stream.read()
    return files


def inspect(path: Path, project: Path, platform: str) -> dict:
    config = tomllib.loads((project / "pyproject.toml").read_text())["project"]
    workspace = project.parents[1]
    version = tomllib.loads((workspace / "Cargo.toml").read_text())["workspace"][
        "package"
    ]["version"]
    licenses = {}
    for pattern in config["license-files"]:
        matches = sorted(project.glob(pattern))
        require(bool(matches), f"license pattern has no files: {pattern}")
        for match in matches:
            require(match.is_file() and not match.is_symlink(), f"invalid license file: {match}")
            licenses[match.relative_to(project).as_posix()] = match.read_bytes()
    notice_name = "licenses/THIRD-PARTY-NOTICES.txt"
    if notice_name in licenses:
        notice = licenses[notice_name].decode()
        lock_digest = hashlib.sha256((workspace / "Cargo.lock").read_bytes()).hexdigest()
        channel = tomllib.loads((workspace / "rust-toolchain.toml").read_text())[
            "toolchain"
        ]["channel"]
        rust_digest = hashlib.sha256(licenses["licenses/RUST-LIBRARY-LICENSE.html"]).hexdigest()
        manifests = [
            "Cargo.toml", "bindings/python/Cargo.toml",
            "crates/lichtung-plot/Cargo.toml", "crates/lichtung-plot-ndarray/Cargo.toml",
            "crates/lichtung-plot-polars/Cargo.toml",
        ]
        manifest_digest = hashlib.sha256(b"".join(
            name.encode() + b"\0" + (workspace / name).read_bytes()
            for name in sorted(manifests)
        )).hexdigest()
        for line in (
            f"Cargo.lock SHA256: {lock_digest}", f"Rust toolchain: {channel}",
            f"Cargo manifests SHA256: {manifest_digest}",
            f"Rust library notice SHA256: {rust_digest}",
            f"Notice policy SHA256: {hashlib.sha256((project / 'about.toml').read_bytes()).hexdigest()}",
            f"Notice generator SHA256: {hashlib.sha256((project / 'third_party.py').read_bytes()).hexdigest()}",
        ):
            require(line in notice.splitlines()[:10], f"stale third-party notice: {line}")
    files = contents(path)
    wheel = path.suffix == ".whl"
    if wheel:
        require(bool(platform), "wheel inspection requires an expected platform tag")
        info_root = f"lichtung_plot-{version}.dist-info"
        metadata_name = f"{info_root}/METADATA"
        license_prefix = f"{info_root}/licenses/"
        package_prefix = "lichtung_plot/"
        tags = BytesParser(policy=default).parsebytes(files[f"{info_root}/WHEEL"])
        require(tags.get_all("Tag") == [f"cp310-abi3-{platform}"], "unexpected wheel tag")
        require(tags["Root-Is-Purelib"] == "false", "native wheel marked as pure Python")
        records = list(csv.reader(StringIO(files[f"{info_root}/RECORD"].decode())))
        require(len(records) == len(files), "wheel RECORD membership differs")
        require({row[0] for row in records} == files.keys(), "wheel RECORD names differ")
        for name, encoded, size in records:
            if name == f"{info_root}/RECORD":
                require(encoded == size == "", "RECORD must not hash itself")
            else:
                digest = base64.urlsafe_b64encode(
                    hashlib.sha256(files[name]).digest()
                ).decode().rstrip("=")
                require(
                    encoded == f"sha256={digest}" and size == str(len(files[name])),
                    f"bad RECORD: {name}",
                )
        extensions = [
            name
            for name in files
            if name.startswith("lichtung_plot/_native.") and name.endswith((".so", ".pyd"))
        ]
        require(len(extensions) == 1, "wheel must contain exactly one native extension")
    else:
        require(path.name == f"lichtung_plot-{version}.tar.gz", "unexpected sdist filename")
        prefix = f"lichtung_plot-{version}/"
        require(all(name.startswith(prefix) for name in files), "unexpected sdist root")
        files = {name.removeprefix(prefix): data for name, data in files.items()}
        metadata_name, license_prefix, package_prefix = "PKG-INFO", "", "python/lichtung_plot/"
        for name in ("Cargo.toml", "Cargo.lock", "bindings/python/Cargo.toml", "_build_backend.py"):
            require(name in files, f"missing source input: {name}")
    info = BytesParser(policy=default).parsebytes(files[metadata_name])
    for key, expected in (
        ("Name", config["name"]), ("Version", version),
        ("Requires-Python", config["requires-python"]), ("License-Expression", config["license"]),
    ):
        require(info[key] == expected, f"unexpected {key}: {info[key]}")
    require(info.get_all("Requires-Dist") is None, "optional providers became runtime dependencies")
    declared = [str(value).strip() for value in info.get_all("License-File", [])]
    require(
        len(declared) == len(licenses) and set(declared) == licenses.keys(),
        "License-File differs from source",
    )
    for name, data in licenses.items():
        require(files.get(license_prefix + name) == data, f"license payload differs: {name}")
    for name in ("__init__.py", "_owners.py", "_native.pyi", "py.typed"):
        require(
            files.get(package_prefix + name)
            == (project / "python/lichtung_plot" / name).read_bytes(),
            f"Python payload differs: {name}",
        )
    urls = {f"{name}, {url}" for name, url in config["urls"].items()}
    require(
        {str(value).strip() for value in info.get_all("Project-URL", [])} == urls,
        "project URLs differ",
    )
    description = files[metadata_name].replace(b"\r\n", b"\n").split(b"\n\n", 1)[1]
    require(
        description.rstrip(b"\n")
        == (project / "README.md").read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n"),
        "README differs",
    )
    return {
        "filename": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "version": version,
        "platform": platform if wheel else "source",
        "files": len(files),
        "license_files": sorted(licenses),
        "python": sys.version,
        "host": {
            "system": host.system(),
            "machine": host.machine(),
            "release": host.release(),
            "macos": host.mac_ver()[0],
            "image_os": os.environ.get("ImageOS"),
            "image_version": os.environ.get("ImageVersion"),
        },
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=workspace, text=True
        ).strip(),
        "source_dirty": bool(subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=workspace, text=True
        ).strip()),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--platform", default="")
    parser.add_argument("--output", default="-")
    arguments = parser.parse_args()
    result = json.dumps(
        inspect(arguments.archive, Path(__file__).resolve().parent, arguments.platform),
        indent=2,
    ) + "\n"
    if arguments.output == "-":
        print(result, end="")
    else:
        output = Path(arguments.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("x", encoding="utf-8") as stream:
            stream.write(result)
