import tempfile
import unittest
from pathlib import Path

from scripts.check_repository import (
    manifest_errors,
    repository_errors,
    test_layout_errors,
)


class ManifestPolicyTests(unittest.TestCase):
    def test_minimal_standalone_manifest_is_accepted(self) -> None:
        manifest = {
            "package": {"name": "lichtung-plot", "publish": False},
            "features": {"default": [], "png": []},
            "dependencies": {"polars": "0.54"},
        }

        self.assertEqual(manifest_errors(manifest), [])

    def test_first_party_and_path_dependencies_are_rejected(self) -> None:
        manifest = {
            "package": {"name": "lichtung-plot", "publish": False},
            "features": {"default": [], "png": []},
            "dependencies": {
                "core": {"package": "lichtung-core", "path": "../core"},
            },
        }

        self.assertEqual(
            manifest_errors(manifest),
            [
                "dependencies.core must not use a path dependency",
                "dependencies.core must not depend on lichtung-core",
            ],
        )


class RepositoryPolicyTests(unittest.TestCase):
    def test_only_the_canonical_integration_root_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            tests = root / "tests"
            tests.mkdir()
            (tests / "integration.rs").write_text("", encoding="utf-8")
            self.assertEqual(test_layout_errors(root), [])

            (tests / "loose_plot_test.rs").write_text("", encoding="utf-8")
            self.assertEqual(
                test_layout_errors(root),
                [
                    "top-level integration test roots must be exactly "
                    "tests/integration.rs; found: tests/integration.rs, "
                    "tests/loose_plot_test.rs"
                ],
            )

    def test_legacy_module_and_boundary_symbol_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative in (
                "README.md",
                "LICENSE-APACHE",
                "LICENSE-MIT",
                "Cargo.lock",
                "specs/01_ARCHITECTURE.md",
                "specs/02_RENDERING_CONTRACT.md",
                "specs/03_DEVELOPMENT.md",
                "docs/getting-started.md",
                "docs/output-contracts.md",
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("fixture\n", encoding="utf-8")
            (root / "Cargo.toml").write_text(
                "[package]\nname = \"lichtung-plot\"\npublish = false\n"
                "[features]\ndefault = []\npng = []\n",
                encoding="utf-8",
            )
            source = root / "src/mod.rs"
            source.parent.mkdir()
            source.write_text("use lichtung_core::Result;\n", encoding="utf-8")

            errors = repository_errors(root)

        self.assertIn("legacy Rust module layout is prohibited: src/mod.rs", errors)
        self.assertIn(
            "forbidden standalone-boundary token 'lichtung_core': src/mod.rs",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
