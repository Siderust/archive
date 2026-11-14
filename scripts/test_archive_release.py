"""Release tests using disposable Git repositories and mocked remote services."""

import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import Mock, patch
from urllib.error import HTTPError

spec = importlib.util.spec_from_file_location(
    "archive_release", Path(__file__).with_name("archive-release.py")
)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)

TAG = "v0.1.6"
VERSION = TAG[1:]
NOTES = "### Fixed\n- Release test."
CHANGELOG = f"# Changelog\n\n## [Unreleased]\n\n## [{VERSION}]\n\n{NOTES}\n"
ENTRY = {"vers": VERSION}
GITHUB_RELEASE = {"draft": False, "prerelease": False, "body": NOTES}


class MetadataTest(unittest.TestCase):
    def test_invalid_tags(self):
        for tag in ("0.1.6", "v0.01.6", "v0.1", "v0.1.6-01", "v0.1.6+", "v0.1.6/a"):
            with self.subTest(tag=tag), self.assertRaises(release.Blocked):
                release.tag_version(tag)

    def test_semver_and_patch_selection(self):
        self.assertEqual(release.tag_version("v2.3.4-alpha.2+build.06"), "2.3.4-alpha.2+build.06")
        self.assertEqual(release.next_patch("2.3.99"), "2.3.100")
        with self.assertRaisesRegex(release.Blocked, "stable release baseline"):
            release.next_patch("2.3.4-alpha")

    def test_changelog_insertion_and_idempotency(self):
        old = f"# Changelog\n\n## [{VERSION}]\n\nPrevious release.\n"
        updated, notes = release.change_entry(old, "0.1.7", "2026-11-12", NOTES)
        self.assertIn("## [Unreleased]\n\n## [0.1.7]", updated)
        self.assertIn("Previous release.", updated)
        self.assertEqual(notes, NOTES)
        self.assertEqual(release.change_entry(updated, "0.1.7", "2026-11-13", "Different"),
                         (updated, NOTES))

    def test_missing_empty_and_duplicate_changelog(self):
        for text in ("# Changelog\n", f"## [{VERSION}]\n", CHANGELOG + f"\n## [{VERSION}]\n"):
            with self.subTest(text=text), self.assertRaises(release.Blocked):
                release.change_entry(text, VERSION)

    def test_registry_archive_identity(self):
        archive = io.BytesIO()
        package = f"{release.CRATE}-{VERSION}"
        with tarfile.open(fileobj=archive, mode="w:gz") as output:
            files = {
                ".cargo_vcs_info.json": json.dumps({"git": {"sha1": "reviewed"}, "path_in_vcs": ""}),
                "Cargo.toml": f'[package]\nname = "{release.CRATE}"\nversion = "{VERSION}"\n',
            }
            for path, body in files.items():
                data = body.encode()
                member = tarfile.TarInfo(f"{package}/{path}")
                member.size = len(data)
                output.addfile(member, io.BytesIO(data))
        data = archive.getvalue()
        entry = {"cksum": hashlib.sha256(data).hexdigest()}
        with patch.object(release, "request", return_value=data):
            release.verify_published(VERSION, "reviewed", entry)
            with self.assertRaisesRegex(release.Blocked, "does not match release commit"):
                release.verify_published(VERSION, "different", entry)
            with self.assertRaisesRegex(release.Blocked, "checksum mismatch"):
                release.verify_published(VERSION, "reviewed", {"cksum": "incorrect"})

    def test_transient_registry_error_retries(self):
        error = HTTPError("https://index.crates.io", 503, "temporary", {}, None)
        with (patch.object(release, "urlopen", side_effect=[error, io.BytesIO(b"ok")]) as http,
              patch.object(release.time, "sleep") as sleep):
            self.assertEqual(release.request(error.url), b"ok")
        self.assertEqual(http.call_count, 2)
        sleep.assert_called_once_with(1)


class GitReleaseTest(unittest.TestCase):
    def setUp(self):
        scratch = Path(self.enterContext(tempfile.TemporaryDirectory()))
        old = Path.cwd()
        checkout = scratch / "checkout"
        checkout.mkdir()
        os.chdir(checkout)
        self.addCleanup(os.chdir, old)
        self.output = scratch / "outputs"
        self.enterContext(patch.dict(os.environ, {
            "GITHUB_REPOSITORY": "example/archive",
            "GITHUB_OUTPUT": str(self.output),
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_AUTHOR_NAME": "Fixture",
            "GIT_AUTHOR_EMAIL": "fixture@example.org",
            "GIT_COMMITTER_NAME": "Fixture",
            "GIT_COMMITTER_EMAIL": "fixture@example.org",
            "CARGO_REGISTRY_TOKEN": "",
        }))
        os.environ.pop("GITHUB_STEP_SUMMARY", None)
        self.remote = str(scratch / "remote.git")
        release.git("init", "--initial-branch=main")
        release.git("init", "--bare", "--initial-branch=main", self.remote)
        release.git("remote", "add", "origin", self.remote)
        Path("Cargo.toml").write_text(f'[package]\nname = "siderust-archive"\nversion = "{VERSION}"\n')
        Path("Cargo.lock").write_text(f'[[package]]\nname = "siderust-archive"\nversion = "{VERSION}"\n')
        Path("CHANGELOG.md").write_text(CHANGELOG)
        self.data = Path(release.DATASETS[0])
        for name in release.DATA_PATHS:
            path = Path(name)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("scientific data")
        self.base = self.commit()
        release.git("push", "origin", "HEAD:refs/heads/main")
        self.registry = self.enterContext(patch.object(release, "registry_entry", return_value=None))
        self.github = self.enterContext(patch.object(release, "github_release", return_value=None))
        self.validate = self.enterContext(patch.object(release, "validate_candidate"))
        self.verify = self.enterContext(patch.object(release, "verify_published"))

    def commit(self):
        release.git("add", "-A")
        release.git("commit", "-m", "Fixture")
        return release.git("rev-parse", "HEAD")

    def tag(self, name=TAG, *, push=True):
        release.git("tag", "-am", "Fixture", name)
        if push:
            release.git("push", "origin", f"refs/tags/{name}")

    def stage_refresh(self):
        self.data.write_text("revised scientific data")
        return [self.data.as_posix()]

    def mock_publish_commands(self, cargo=None, github=None):
        original = release.run

        def dispatch(*args, **kwargs):
            if args[:2] == ("cargo", "publish"):
                return cargo(*args, **kwargs)
            if args[:2] == ("gh", "release"):
                return github(*args, **kwargs)
            return original(*args, **kwargs)

        return patch.object(release, "run", side_effect=dispatch)

    def test_valid_manual_release_uses_exact_tag_when_main_advanced(self):
        self.tag()
        self.data.write_text("later main data")
        new_head = self.commit()
        release.git("push", "origin", "HEAD:refs/heads/main")
        release.validate_tagged_candidate(TAG, release.git("rev-parse", f"refs/tags/{TAG}"))
        self.assertEqual(release.git("rev-parse", "HEAD"), self.base)
        self.assertNotEqual(new_head, self.base)
        self.validate.assert_called_once_with()
        self.assertEqual(release.git("tag", "--list"), TAG)

    def test_missing_tag_cannot_create_a_release(self):
        with self.assertRaisesRegex(release.Blocked, "Push the reviewed annotated version tag"):
            release.validate_tagged_candidate(TAG, "")

    def test_tag_moved_since_event(self):
        self.tag()
        with self.assertRaisesRegex(release.Blocked, "expected"):
            release.validate_tagged_candidate(TAG, "1" * 40)

    def test_divergent_tag(self):
        self.stage_refresh()
        self.commit()
        self.tag()
        release.git("reset", "--hard", self.base)
        with self.assertRaisesRegex(release.Blocked, "divergent from main"):
            release.validate_tagged_candidate(TAG, "")
        self.validate.assert_not_called()

    def test_tag_cargo_mismatch(self):
        self.tag("v0.1.7")
        with self.assertRaisesRegex(release.Blocked, "Version mismatch"):
            release.validate_tagged_candidate("v0.1.7", "")

    def test_lock_mismatch(self):
        path = Path("Cargo.lock")
        path.write_text(path.read_text().replace(VERSION, "0.1.5"))
        with self.assertRaisesRegex(release.Blocked, "Version mismatch"):
            release.check_metadata()

    def test_no_change_does_not_require_baseline_or_create_output(self):
        with patch.object(release, "refresh", return_value=[]):
            release.prepare()
        self.assertEqual(release.git("rev-parse", "HEAD"), self.base)
        self.assertFalse(self.output.exists())
        self.registry.assert_not_called()
        self.validate.assert_not_called()

    def test_download_and_dataset_validation_fail_without_reserving_patch(self):
        for label in ("download failed", "snapshot coverage regressed"):
            with (self.subTest(label=label),
                  patch.object(release, "refresh", side_effect=release.Blocked(label)),
                  self.assertRaisesRegex(release.Blocked, label)):
                release.prepare()
            self.assertEqual(release.git("rev-parse", "HEAD"), self.base)
            self.assertFalse(self.output.exists())
        self.registry.assert_not_called()

    def test_unrelated_changes_even_if_reverted(self):
        self.tag()
        other = Path("README.md")
        other.write_text("non-data")
        self.commit()
        other.unlink()
        release.git("push", "origin", self.commit() + ":refs/heads/main")
        with (patch.object(release, "refresh", side_effect=self.stage_refresh),
              self.assertRaisesRegex(release.Blocked, "Non-data changes")):
            release.prepare()
        self.registry.assert_not_called()

    def test_stale_local_main(self):
        release.git("update-ref", release.MAIN, self.base)
        self.stage_refresh()
        self.commit()
        with (patch.object(release, "refresh") as refresh,
              self.assertRaisesRegex(release.Blocked, "Checkout is stale")):
            release.prepare()
        refresh.assert_not_called()

    def prepare_from_published_baseline(self):
        self.tag()
        self.registry.side_effect = [ENTRY, None]
        self.github.side_effect = [GITHUB_RELEASE, None]
        with patch.object(release, "refresh", side_effect=self.stage_refresh):
            release.prepare()

    def test_valid_data_reserves_one_annotated_patch(self):
        self.prepare_from_published_baseline()
        commit = release.tag_commit("0.1.7")
        self.assertEqual(release.check_metadata(), "0.1.7")
        self.assertEqual(release.git("rev-parse", f"{commit}^"), self.base)
        self.assertEqual(release.git("rev-parse", release.MAIN), commit)
        self.assertEqual(release.git("cat-file", "-t", release.tag_ref("0.1.7")), "tag")
        self.assertEqual(self.output.read_text(), f"tag=v0.1.7\ncommit={commit}\n")
        self.validate.assert_called_once_with()

    def test_remote_main_advances_during_validation(self):
        def advance():
            ahead = release.git("commit-tree", f"{self.base}^{{tree}}", "-p", self.base,
                                "-m", "A reviewed change reached main")
            release.git("--git-dir", self.remote, "fetch", str(Path.cwd()), ahead)
            release.git("--git-dir", self.remote, "update-ref", "refs/heads/main", ahead)
        self.validate.side_effect = advance
        with self.assertRaises(subprocess.CalledProcessError):
            self.prepare_from_published_baseline()
        self.assertFalse(self.output.exists())
        self.assertEqual(release.git("ls-remote", "origin", release.tag_ref("0.1.7")), "")

    def test_competing_tag_rejects_atomic_push_without_updating_main(self):
        def reserve():
            tag = "v0.1.7"
            release.git("tag", "-am", "Competing tag", tag, self.base)
            release.git("push", "origin", f"refs/tags/{tag}")
            release.git("tag", "-d", tag)
        self.validate.side_effect = reserve
        with self.assertRaises(subprocess.CalledProcessError):
            self.prepare_from_published_baseline()
        self.assertFalse(self.output.exists())
        self.assertEqual(release.git("ls-remote", "origin", "refs/heads/main").split()[0], self.base)

    def test_existing_crate_and_release_are_verified_without_mutation(self):
        self.tag()
        self.registry.return_value = ENTRY
        self.github.return_value = GITHUB_RELEASE
        with self.mock_publish_commands() as commands:
            release.publish(TAG, self.base)
        self.verify.assert_called_once_with(VERSION, self.base, ENTRY)
        self.assertFalse(any(call.args[0] in ("gh", "cargo") for call in commands.call_args_list))

    def test_existing_release_with_wrong_notes_is_synchronized_after_verification(self):
        self.tag()
        self.registry.return_value = ENTRY
        self.github.return_value = {**GITHUB_RELEASE, "body": "old notes"}
        github = Mock(side_effect=lambda *args, **kwargs: self.assertEqual(self.verify.call_count, 1))
        with self.mock_publish_commands(github=github):
            release.publish(TAG, self.base)
        github.assert_called_once_with("gh", "release", "edit", TAG, "--title",
                                       f"{release.CRATE} {TAG}", "--notes", NOTES, "--prerelease=false")

    def test_interrupted_upload_and_release_retry(self):
        self.tag()
        cargo = Mock(return_value=Mock(returncode=1))
        github = Mock(side_effect=subprocess.CalledProcessError(1, "gh"))
        self.registry.side_effect = [None, None, ENTRY, ENTRY]
        with (patch.dict(os.environ, CARGO_REGISTRY_TOKEN="fixture"),
              patch.object(release.time, "sleep"),
              self.mock_publish_commands(cargo, github)):
            with self.assertRaises(subprocess.CalledProcessError):
                release.publish(TAG, self.base)
            github.side_effect = None
            release.publish(TAG, self.base)
        cargo.assert_called_once()
        self.assertEqual(github.call_count, 2)
        self.assertEqual(self.verify.call_count, 2)

    def test_failed_publication_leaves_github_release_untouched(self):
        self.tag()
        github = Mock()
        with (patch.dict(os.environ, CARGO_REGISTRY_TOKEN="fixture"),
              patch.object(release.time, "sleep"),
              self.mock_publish_commands(Mock(return_value=Mock(returncode=1)), github),
              self.assertRaisesRegex(release.Blocked, "has not indexed")):
            release.publish(TAG, self.base)
        github.assert_not_called()


if __name__ == "__main__":
    unittest.main()
