#!/usr/bin/env python3
"""Verify the standalone repository boundary and structural contract."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path


REQUIRED_PATHS = (
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE-APACHE",
    "LICENSE-MIT",
    "specs/01_ARCHITECTURE.md",
    "specs/02_RENDERING_CONTRACT.md",
    "specs/03_DEVELOPMENT.md",
    "docs/getting-started.md",
    "docs/output-contracts.md",
)

FORBIDDEN_SOURCE_TOKENS = (
    "lichtung_core",
    "lichtung_assets",
    "LichtungError",
    "AnalysisSpec",
    "PdfOptions",
    "PdfCompression",
    "svg2pdf",
)

EXPECTED_INTEGRATION_ROOTS = (Path("tests/integration.rs"),)


def manifest_errors(manifest: dict[object, object]) -> list[str]:
    errors: list[str] = []
    package = manifest.get("package", {})
    if not isinstance(package, dict) or package.get("name") != "lichtung-plot":
        errors.append("Cargo.toml must define the root package as lichtung-plot")
    if not isinstance(package, dict) or package.get("publish") is not False:
        errors.append("Cargo.toml package.publish must remain false")

    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        dependencies = manifest.get(section, {})
        if not isinstance(dependencies, dict):
            errors.append(f"Cargo.toml [{section}] must be a table")
            continue
        for name, specification in dependencies.items():
            package_name = name
            if isinstance(specification, dict):
                package_name = specification.get("package", name)
                if "path" in specification:
                    errors.append(f"{section}.{name} must not use a path dependency")
            if str(package_name).startswith("lichtung-"):
                errors.append(f"{section}.{name} must not depend on {package_name}")

    features = manifest.get("features", {})
    if not isinstance(features, dict) or set(features) != {"default", "png"}:
        errors.append("Cargo.toml features must be exactly default and png")
    return errors


def test_layout_errors(root: Path) -> list[str]:
    test_root = root / "tests"
    actual = tuple(path.relative_to(root) for path in sorted(test_root.glob("*.rs")))
    if actual == EXPECTED_INTEGRATION_ROOTS:
        return []
    rendered = ", ".join(str(path) for path in actual) or "none"
    return [
        "top-level integration test roots must be exactly "
        f"tests/integration.rs; found: {rendered}"
    ]


def repository_errors(root: Path) -> list[str]:
    errors = [
        f"required repository file is missing: {relative}"
        for relative in REQUIRED_PATHS
        if not (root / relative).is_file()
    ]
    manifest_path = root / "Cargo.toml"
    if manifest_path.is_file():
        with manifest_path.open("rb") as handle:
            errors.extend(manifest_errors(tomllib.load(handle)))
    errors.extend(test_layout_errors(root))

    rust_paths = sorted((*root.glob("src/**/*.rs"), *root.glob("tests/**/*.rs")))
    for path in rust_paths:
        relative = path.relative_to(root)
        if path.name == "mod.rs":
            errors.append(f"legacy Rust module layout is prohibited: {relative}")
        text = path.read_text(encoding="utf-8")
        for token in FORBIDDEN_SOURCE_TOKENS:
            if token in text:
                errors.append(f"forbidden standalone-boundary token {token!r}: {relative}")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args(argv)
    errors = repository_errors(args.root.resolve())
    if errors:
        print("repository policy failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("lichtung-plot repository policy OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
