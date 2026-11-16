"""Maintain pinned third-party notice payloads; offline, never called by builds."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def generate(project: Path, cargo_about: str) -> dict[str, bytes]:
    workspace = project.parents[1]
    lock_path = workspace / "Cargo.lock"
    lock = lock_path.read_bytes()
    inventory = {
        (p["name"], p["version"], p.get("source")): p
        for p in tomllib.loads(lock.decode())["package"]
    }
    toolchain = tomllib.loads((workspace / "rust-toolchain.toml").read_text())[
        "toolchain"
    ]["channel"]
    rust = subprocess.check_output(["rustc", "--version"], text=True).split()[1]
    if rust != toolchain:
        raise ValueError(f"notice toolchain mismatch: {rust} != {toolchain}")
    if subprocess.check_output([cargo_about, "--version"], text=True).strip() != "cargo-about 0.9.3":
        raise ValueError("notice harvesting requires cargo-about 0.9.3")
    report = json.loads(subprocess.check_output([
        cargo_about, "generate", "--manifest-path", str(project / "Cargo.toml"),
        "--config", str(project / "about.toml"), "--locked", "--offline", "--fail",
        "--format", "json",
    ], cwd=workspace))
    packages = {
        (entry["package"]["name"], entry["package"]["version"]): entry["package"]
        for entry in report["crates"]
        if entry["package"]["source"] is not None
    }
    if any(p["source"] != "registry+https://github.com/rust-lang/crates.io-index" for p in packages.values()):
        raise ValueError("review notice harvesting for non-crates.io dependencies")
    texts: dict[str, tuple[bytes, set[str]]] = {}
    references = {key: set() for key in packages}
    choices = {key: set() for key in packages}

    def add(data: bytes, label: str, owners: list[tuple[str, str]]) -> None:
        # Preserve copyright and terms, normalizing only platform line endings.
        data.decode("utf-8")
        data = data.replace(b"\r\n", b"\n")
        key = digest(data)
        if key not in texts:
            texts[key] = data, set()
        texts[key][1].add(label)
        for owner in owners:
            references[owner].add(key)

    for key, package in sorted(packages.items()):
        root = Path(package["manifest_path"]).parent
        pinned = inventory[(key[0], key[1], package["source"])]
        cached = (
            root.parents[2] / "cache" / root.parent.name
            / f"{key[0]}-{key[1]}.crate"
        )
        if digest(cached.read_bytes()) != pinned["checksum"]:
            raise ValueError(f"registry package checksum differs: {key}")
        found = False
        with tarfile.open(cached) as archive:
            for path in sorted(root.rglob("*")):
                lower = path.name.lower()
                parent_names = (part.lower() for part in path.relative_to(root).parts[:-1])
                if not path.is_file() or not (
                    lower.startswith(("license", "licence", "copying", "copyright", "notice"))
                    or lower in ("authors", "authors.md", "authors.txt")
                    or any(part.startswith(("licenses", "licences")) for part in parent_names)
                ):
                    continue
                relative = path.relative_to(root).as_posix()
                data = path.read_bytes()
                original = archive.extractfile(f"{key[0]}-{key[1]}/{relative}")
                if path.is_symlink() or original is None or data != original.read():
                    raise ValueError(f"unverified registry notice: {key}/{relative}")
                add(data, f"{key[0]} {key[1]} / {relative}", [key])
                found = True
        if not found:
            raise ValueError(f"no packaged legal text for {key}")

    # Also keep canonical fallback terms, but never use them instead of an
    # upstream copyright/COPYING/combined-license file collected above.
    for license in report["licenses"]:
        owners = sorted({
            (usage["crate"]["name"], usage["crate"]["version"])
            for usage in license["used_by"]
            if (usage["crate"]["name"], usage["crate"]["version"]) in packages
        })
        for owner in owners:
            choices[owner].add(license["id"])
        if owners and license["source_path"] is None:
            add(license["text"].encode(), f"cargo-about 0.9.3 canonical {license['id']} terms", owners)

    # polars-arrow's packaged LICENSE refers to a missing ../../LICENSE for its
    # MIT portions. The exact sibling release carries that upstream root text.
    arrow, core = ("polars-arrow", "0.55.2"), ("polars-core", "0.55.2")
    if arrow not in packages or core not in packages:
        raise ValueError("review the Polars license supplement after a dependency change")
    arrow_root = Path(packages[arrow]["manifest_path"]).parent
    core_root = Path(packages[core]["manifest_path"]).parent
    for root in (arrow_root, core_root):
        vcs = json.loads((root / ".cargo_vcs_info.json").read_text())
        if vcs["git"]["sha1"] != "d7488c71ecfbc77790292ff5b365b991c08380ce":
            raise ValueError("Polars supplement is not from the reviewed upstream commit")
    add((core_root / "LICENSE").read_bytes(), "polars-arrow 0.55.2 supplemental MIT: polars-core 0.55.2 LICENSE, same upstream commit", [arrow])

    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
    rust_notice = (sysroot / "share/doc/rust/COPYRIGHT-library.html").read_bytes()
    rust_notice.decode("utf-8")
    manifests = [
        "Cargo.toml", "bindings/python/Cargo.toml",
        "crates/lichtung-plot/Cargo.toml", "crates/lichtung-plot-ndarray/Cargo.toml",
        "crates/lichtung-plot-polars/Cargo.toml",
    ]
    manifest_digest = digest(b"".join(
        name.encode() + b"\0" + (workspace / name).read_bytes()
        for name in sorted(manifests)
    ))
    lines = [
        "Third-party notices for Lichtung Plot's Python native distributions",
        "Generated by third_party.py with cargo-about 0.9.3.",
        f"Cargo.lock SHA256: {digest(lock)}",
        f"Cargo manifests SHA256: {manifest_digest}",
        f"Rust toolchain: {toolchain}",
        f"Rust library notice SHA256: {digest(rust_notice)}",
        f"Notice policy SHA256: {digest((project / 'about.toml').read_bytes())}",
        f"Notice generator SHA256: {digest(Path(__file__).read_bytes())}",
        "",
        "This is a conservative union for the three targets in about.toml, including",
        "build/proc-macro dependencies. Presence here does not assert binary linkage.",
        "Upstream license alternatives and copyrights remain unchanged; the project",
        "license does not relicense third-party components. Cargo-about's selected",
        "license families are listed below. All packaged legal files are retained,",
        "including alternatives and supplementary notices, with LF line endings.",
        "Text bodies are deduplicated only by their full SHA256, not by license name.",
        "Canonical terms supplement, never replace, the original attribution files.",
        "polars-arrow's MIT text is supplemented from the same Polars source commit.",
        "See RUST-LIBRARY-LICENSE.html for Rust's complete standard-library notices.",
        "No Python providers, system fonts or system dynamic libraries are bundled.",
        "",
        "DEPENDENCIES",
        f"External crates: {len(packages)}",
        "",
    ]
    for key, package in sorted(packages.items()):
        if not choices[key] or not references[key]:
            raise ValueError(f"incomplete notice coverage: {key}")
        pinned = inventory[(key[0], key[1], package["source"])]
        lines.extend([
            f"{key[0]} {key[1]}",
            f"  Registry checksum: {pinned['checksum']}",
            f"  Upstream license expression: {package['license']}",
            f"  Harvested license families: {', '.join(sorted(choices[key]))}",
            f"  Notice text SHA256: {', '.join(sorted(references[key]))}",
            "",
        ])
    output = ("\n".join(lines) + "\n").encode()
    for key, (data, labels) in sorted(texts.items()):
        output += f"\n===== NOTICE TEXT {key} =====\n".encode()
        output += ("\n".join(sorted(labels)) + "\n\n").encode()
        output += data + (b"" if data.endswith(b"\n") else b"\n")
        output += b"===== END NOTICE TEXT =====\n"
    if lock_path.read_bytes() != lock:
        raise RuntimeError("Cargo.lock changed during notice harvesting")
    return {
        "THIRD-PARTY-NOTICES.txt": output,
        "RUST-LIBRARY-LICENSE.html": rust_notice,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo-about", default="cargo-about")
    parser.add_argument("--write", action="store_true", help="replace only the two generated notice files")
    arguments = parser.parse_args()
    project = Path(__file__).resolve().parent
    for name, data in generate(project, arguments.cargo_about).items():
        path = project / "licenses" / name
        if arguments.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        elif not path.is_file() or path.read_bytes() != data:
            raise SystemExit(f"stale notice: {path}; review and regenerate explicitly")
        print(f"{name}: SHA256 {digest(data)}")
