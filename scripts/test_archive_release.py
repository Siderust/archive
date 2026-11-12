"""Release state tests using local Git repositories and mocked publication."""

import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "archive_release", Path(__file__).with_name("archive-release.py")
)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)

NOTES = "### Fixed\n- Release test."
CHANGELOG = f"# Changelog\n\n## [Unreleased]\n\n## [0.1.6]\n\n{NOTES}\n"


class ChangelogTest(unittest.TestCase):
    def test_missing_unreleased_and_repeated_execution_with_new_date(self):
        old = "# Changelog\n\n## [0.1.6]\n\nPrevious release.\n"
        updated, notes = release.change_entry(old, "0.1.7", "2026-11-12", NOTES)
        self.assertIn("## [Unreleased]\n\n## [0.1.7]", updated)
        self.assertIn("Previous release.", updated)
        self.assertEqual(notes, NOTES)
        self.assertEqual(release.change_entry(updated, "0.1.7", "2026-11-13", "Different"),
                         (updated, NOTES))

    def test_duplicate_version_is_rejected(self):
        with self.assertRaisesRegex(release.Blocked, "Duplicate"):
            release.change_entry(CHANGELOG + "\n## [0.1.6] - 2026-11-12\n", "0.1.6")


class GitReleaseTest(unittest.TestCase):
    def setUp(self):
        scratch = self.enterContext(tempfile.TemporaryDirectory())
        old = Path.cwd()
        os.chdir(scratch)
        self.addCleanup(os.chdir, old)
        self.enterContext(patch.dict(os.environ, {
            "GITHUB_REF": "refs/heads/main",
            "GITHUB_REPOSITORY": "example/archive",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_AUTHOR_NAME": "Fixture",
            "GIT_AUTHOR_EMAIL": "fixture@example.org",
            "GIT_COMMITTER_NAME": "Fixture",
            "GIT_COMMITTER_EMAIL": "fixture@example.org",
            "CARGO_REGISTRY_TOKEN": "",
        }))
        os.environ.pop("GITHUB_STEP_SUMMARY", None)
        release.git("init", "--initial-branch=main")
        Path("Cargo.toml").write_text('[package]\nname = "siderust-archive"\nversion = "0.1.6"\n')
        Path("Cargo.lock").write_text('[[package]]\nname = "siderust-archive"\nversion = "0.1.6"\n')
        Path("CHANGELOG.md").write_text(CHANGELOG)
        self.data = Path(release.DATASETS[0])
        self.data.parent.mkdir(parents=True)
        self.data.write_text("scientific data")
        self.base = self.commit()
        release.git("update-ref", release.MAIN, self.base)
        self.refresh = self.enterContext(patch.object(release, "refresh", return_value=[]))
        self.registry = self.enterContext(patch.object(release, "registry_entry", return_value=None))
        self.github = self.enterContext(patch.object(release, "github_release", return_value=None))
        self.validate = self.enterContext(patch.object(release, "validate_candidate"))

    def commit(self):
        release.git("add", "-A")
        release.git("commit", "-m", "Fixture")
        return release.git("rev-parse", "HEAD")

    def tag(self):
        release.git("tag", "-am", "Fixture", "v0.1.6")

    def test_divergent_baseline_fails_before_registry_access(self):
        self.data.write_text("branch-only update")
        self.commit()
        self.tag()
        release.git("reset", "--hard", self.base)
        with self.assertRaisesRegex(release.Blocked, "v0.1.6 is divergent"):
            release.release("auto", "")
        self.registry.assert_not_called()

    def test_lock_mismatch_is_explicit(self):
        Path("Cargo.lock").write_text(Path("Cargo.lock").read_text().replace("0.1.6", "0.1.5"))
        with self.assertRaisesRegex(release.Blocked, "Version mismatch"):
            release.current_version()

    def test_data_only_even_if_non_data_change_was_reverted(self):
        self.data.write_text("revised scientific data")
        data_commit = self.commit()
        self.assertEqual(release.automatic_datasets(self.base, data_commit), [self.data.as_posix()])
        other = Path("README.md")
        other.write_text("non-data")
        self.commit()
        other.unlink()
        with self.assertRaisesRegex(release.Blocked, "Non-data changes"):
            release.automatic_datasets(self.base, self.commit())

    def test_no_change_does_not_create_tag_or_validate_release_candidate(self):
        self.tag()
        self.registry.return_value = {"vers": "0.1.6"}
        self.github.return_value = {"draft": False}
        release.release("auto", "")
        release.release("auto", "")
        self.assertIsNone(release.tag_commit("0.1.7"))
        self.validate.assert_not_called()

    def test_incomplete_release_requires_explicit_retry(self):
        self.tag()
        with self.assertRaisesRegex(release.Blocked, "Release v0.1.6 is incomplete"):
            release.release("auto", "")
        self.assertIsNone(release.tag_commit("0.1.7"))
        self.validate.assert_not_called()

    def test_existing_registry_version_without_tag_is_rejected(self):
        self.registry.return_value = {"vers": "0.1.6"}
        with self.assertRaisesRegex(release.Blocked, "exists externally"):
            release.release("manual", "0.1.6")
        self.assertIsNone(release.tag_commit("0.1.6"))

    def test_missing_credentials_before_tag(self):
        with self.assertRaisesRegex(release.Blocked, "CARGO_REGISTRY_TOKEN is missing"):
            release.release("manual", "0.1.6")
        self.assertIsNone(release.tag_commit("0.1.6"))
        self.validate.assert_not_called()

    def test_manual_release_tags_the_reviewed_main_commit(self):
        with tempfile.TemporaryDirectory() as bare:
            release.git("init", "--bare", "--initial-branch=main", bare)
            release.git("remote", "add", "origin", bare)
            release.git("push", "origin", "HEAD:refs/heads/main")
            with (patch.dict(os.environ, CARGO_REGISTRY_TOKEN="fixture"),
                  patch.object(release, "publish") as publish):
                release.release("manual", "0.1.6")
            self.assertEqual(release.tag_commit("0.1.6"), self.base)
            self.assertEqual(release.git("rev-parse", release.MAIN), self.base)
            self.assertEqual(release.git("cat-file", "-t", release.tag_ref("0.1.6")), "tag")
            self.validate.assert_called_once_with(dirty=False)
            publish.assert_called_once_with("0.1.6", self.base, None, None)

    def test_valid_data_prepares_patch_and_pushes_one_annotated_tag(self):
        with tempfile.TemporaryDirectory() as bare:
            release.git("init", "--bare", "--initial-branch=main", bare)
            release.git("remote", "add", "origin", bare)
            self.tag()
            release.git("push", "origin", "HEAD:refs/heads/main", release.tag_ref("0.1.6"))
            self.data.write_text("corrected scientific data")
            new_data = self.commit()
            release.git("push", "origin", "HEAD:refs/heads/main")
            self.refresh.return_value = [self.data.as_posix()]
            self.registry.side_effect = [{"vers": "0.1.6"}, None]
            self.github.side_effect = [{"draft": False}, None]
            with (patch.dict(os.environ, CARGO_REGISTRY_TOKEN="fixture"),
                  patch.object(release, "verify_published"),
                  patch.object(release, "publish") as publish):
                release.release("auto", "")
            self.assertEqual(release.current_version(), "0.1.7")
            tag = release.tag_commit("0.1.7")
            self.assertEqual(release.git("rev-parse", f"{tag}^"), new_data)
            self.assertEqual(release.git("cat-file", "-t", release.tag_ref("0.1.7")), "tag")
            self.assertEqual(release.git("rev-parse", release.MAIN), tag)
            publish.assert_called_once_with("0.1.7", tag, None, None)
            self.validate.assert_called_once_with(dirty=True)

    def test_retry_after_crate_upload_only_creates_github_release(self):
        self.registry.return_value = {"vers": "0.1.6"}
        with (patch.dict(os.environ, CARGO_REGISTRY_TOKEN="fixture"),
              patch.object(release, "verify_published"),
              patch.object(release, "run") as run):
            run.side_effect = [None, subprocess.CalledProcessError(1, "gh")]
            with self.assertRaises(subprocess.CalledProcessError):
                release.publish("0.1.6", self.base, None, None)
            self.assertEqual(run.call_args_list[0].args[:2], ("cargo", "publish"))

        self.tag()
        original = release.run
        with (patch.object(release, "verify_published") as verify,
              patch.object(release, "run", wraps=original) as run):

            def skip_gh(*args, **kwargs):
                if args[0] != "gh":
                    return original(*args, **kwargs)
                return None

            run.side_effect = skip_gh
            release.release("manual", "0.1.6")
        verify.assert_called_once_with("0.1.6", self.base, self.registry.return_value)
        self.assertFalse(any(call.args[:2] == ("cargo", "publish") for call in run.call_args_list))
        self.assertTrue(any(call.args[:3] == ("gh", "release", "create") for call in run.call_args_list))


if __name__ == "__main__":
    unittest.main()
