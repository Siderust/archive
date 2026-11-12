#!/usr/bin/env python3
"""Refresh and release siderust-archive from a clean, full main checkout."""

import argparse
from datetime import datetime, timezone
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time
import tomllib
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

CRATE = "siderust-archive"
VERSION = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
RAW = "src/time/eop/raw/"
DATASETS = tuple(RAW + name for name in (
    "UTC-TAI.history", "deltat.data", "deltat.preds", "finals2000A.all",
))
DATA_PATHS = (RAW, "src/time/eop/manifest.toml", "src/time/bundled/snapshot.rs")
RELEASE_FILES = ("Cargo.toml", "Cargo.lock", "CHANGELOG.md")
MAIN = "refs/remotes/origin/main"


class Blocked(Exception):
    """A release precondition failed."""


def run(*args, capture=False, check=True):
    env = os.environ.copy()
    if args[:2] != ("cargo", "publish") or "--dry-run" in args:
        env.pop("CARGO_REGISTRY_TOKEN", None)
    if args[0] != "gh":
        env.pop("GH_TOKEN", None)
    return subprocess.run(args, check=check, text=True,
                          stdout=subprocess.PIPE if capture else None, env=env)


def git(*args):
    return run("git", *args, capture=True).stdout.rstrip("\n")


def summary(message):
    print(message, flush=True)
    if path := os.getenv("GITHUB_STEP_SUMMARY"):
        with open(path, "a", encoding="utf-8") as output:
            print(message, end="\n\n", file=output)


def version_tuple(version):
    if not re.fullmatch(VERSION, version):
        raise Blocked(f"Invalid release version {version!r}; expected MAJOR.MINOR.PATCH")
    return tuple(map(int, version.split(".")))


def current_version(ref=None):
    def read(path):
        return tomllib.loads(git("show", f"{ref}:{path}") if ref else Path(path).read_text())

    package = read("Cargo.toml")["package"]
    version = package["version"]
    locked = [item["version"] for item in read("Cargo.lock")["package"]
              if item["name"] == CRATE and "source" not in item]
    if package["name"] != CRATE or locked != [version]:
        raise Blocked(f"Version mismatch at {ref or 'HEAD'}: Cargo.toml={version}, Cargo.lock={locked}")
    version_tuple(version)
    return version


def change_entry(text, version, date=None, body=None):
    """Read notes or insert a section once, even when Unreleased is absent."""
    headings = list(re.finditer(r"^## (?!#)(.*)$", text, re.MULTILINE))
    entries = {}
    for i, heading in enumerate(headings):
        if match := re.match(r"\[([^\]]+)\]", heading[1]):
            key = match[1]
            if key in entries:
                raise Blocked(f"Duplicate CHANGELOG.md section [{key}]")
            end = headings[i + 1].start() if i + 1 < len(headings) else len(text)
            # Keep Keep a Changelog link definitions out of the release notes.
            notes = re.split(r"^\[[^\]]+\]:", text[heading.end():end], maxsplit=1, flags=re.MULTILINE)[0]
            entries[key] = (heading.start(), notes.strip())
    if version in entries:
        notes = entries[version][1]
        if not notes:
            raise Blocked(f"CHANGELOG.md [{version}] has no release notes")
        return text, notes
    if body is None:
        raise Blocked(f"Missing CHANGELOG.md section [{version}]")
    # Insert directly below Unreleased, or restore it above the first release.
    if "Unreleased" in entries:
        start = entries["Unreleased"][0]
        position = next((heading.start() for heading in headings if heading.start() > start), len(text))
        prefix = ""
    else:
        position = headings[0].start() if headings else len(text)
        prefix = "## [Unreleased]\n\n"
    # Preserve link definitions at the bottom of a changelog with no older releases.
    links = re.search(r"^\[[^\]]+\]:", text, re.MULTILINE)
    if links:
        position = min(position, links.start())
    before = text[:position].rstrip()
    entry = f"{prefix}## [{version}] - {date}\n\n{body}"
    return f"{before}\n\n{entry}\n\n{text[position:]}".rstrip() + "\n", body


def tag_ref(version):
    return f"refs/tags/v{version}"


def tag_commit(version):
    ref = tag_ref(version)
    result = run("git", "show-ref", "--verify", "--quiet", ref, check=False)
    if result.returncode == 1:
        return None
    result.check_returncode()
    return git("rev-parse", f"{ref}^{{commit}}")


def require_ancestor(commit, head, label):
    result = run("git", "merge-base", "--is-ancestor", commit, head, check=False)
    if result.returncode == 1:
        raise Blocked(f"{label} is divergent from main ({commit} vs {head}); "
                      "prepare a new version on main and use Publish. Preserve existing tags.")
    result.check_returncode()


def validate_tag(version, commit, head):
    tag = f"v{version}"
    require_ancestor(commit, head, tag)
    if git("cat-file", "-t", tag_ref(version)) != "tag":
        raise Blocked(f"{tag} is not an annotated release tag")
    if current_version(commit) != version:
        raise Blocked(f"Version mismatch between {tag} and its Cargo metadata")


def data_only(path):
    return path.startswith(RAW) or path in DATA_PATHS[1:]


def changes(*args):
    return [path for path in git(*args).split("\0") if path]


def automatic_datasets(base, head):
    # Inspect individual commits, including both sides and resolutions of merges.
    # A later revert of non-data work does not authorize automatic publication.
    history = changes("log", "--format=", "--name-only", "-z", "--diff-merges=separate",
                      "--no-renames", f"{base}..{head}")
    outside = sorted({path for path in history if not data_only(path)})
    if outside:
        raise Blocked("Non-data changes since the release tag: " + ", ".join(outside)
                      + ". Prepare a reviewed release through Publish.")
    return changes("diff", "--name-only", "-z", base, head, "--", *DATASETS)


def request(url, headers=None, allow_missing=False):
    try:
        request = Request(url)
        for key, value in (headers or {}).items():
            request.add_unredirected_header(key, value)
        with urlopen(request, timeout=20) as response:
            return response.read()
    except HTTPError as error:
        if error.code == 404 and allow_missing:
            return None
        raise Blocked(f"HTTP {error.code} checking {url}") from error
    except (URLError, TimeoutError) as error:
        raise Blocked(f"Could not check {url}: {error}") from error


def registry_entry(version):
    # Query the public sparse index directly. Any transport error fails closed.
    data = request(f"https://index.crates.io/si/de/{CRATE}", {"Cache-Control": "no-cache"})
    for line in data.splitlines():
        entry = json.loads(line)
        if entry["vers"] == version:
            if entry["yanked"]:
                raise Blocked(f"{CRATE} {version} already exists and is yanked")
            return entry
    return None


def verify_published(version, commit, entry):
    filename = f"{CRATE}-{version}"
    data = request(f"https://static.crates.io/crates/{CRATE}/{filename}.crate")
    if hashlib.sha256(data).hexdigest() != entry["cksum"]:
        raise Blocked(f"Registry checksum mismatch for {filename}")
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as package:
        info = json.load(package.extractfile(f"{filename}/.cargo_vcs_info.json"))
    if info["git"].get("dirty") or info["git"]["sha1"] != commit or info["path_in_vcs"]:
        raise Blocked(f"Existing crates.io version {version} does not match release commit {commit}")


def github_release(version):
    repo = os.environ["GITHUB_REPOSITORY"]
    result = request(
        f"https://api.github.com/repos/{repo}/releases/tags/v{version}",
        {"Authorization": f"Bearer {os.environ['GH_TOKEN']}", "Accept": "application/vnd.github+json"},
        allow_missing=True,
    )
    return json.loads(result) if result else None


def require_credentials(entry):
    if not entry and not os.getenv("CARGO_REGISTRY_TOKEN"):
        raise Blocked("CARGO_REGISTRY_TOKEN is missing; configure it before creating a release tag")


def validate_candidate(*, dirty=False):
    run("cargo", "run", "--locked", "-p", "archive-validate", "--", "MANIFEST.toml")
    run("cargo", "test", "--all-features", "--locked")
    args = ("--allow-dirty",) if dirty else ()
    run("cargo", "publish", "--dry-run", "--locked", "--all-features", "-p", CRATE, *args)
    run("bash", "scripts/check-packaged-docs.sh", *args)
    summary("Release candidate passed manifest, test, packaging, and docs.rs checks.")


def prepare_patch(version, new_version, datasets):
    notes = "### Changed\n- Refreshed the bundled IERS/USNO snapshot.\n" + "\n".join(
        f"  - `{Path(path).name}`" for path in datasets
    )
    changelog = Path("CHANGELOG.md")
    updated, _ = change_entry(changelog.read_text(), new_version,
                              datetime.now(timezone.utc).date().isoformat(), notes)
    # Update only the root package's version and its lock entry. Locked Cargo
    # commands below verify workspace resolution without upgrading dependencies.
    for file, section in (("Cargo.toml", "[package]"), ("Cargo.lock", "[[package]]")):
        path = Path(file)
        pattern = rf'(?m)^({re.escape(section)}\nname = "{CRATE}"\nversion = "){re.escape(version)}"$'
        content, count = re.subn(pattern, lambda match: f'{match[1]}{new_version}"', path.read_text())
        if count != 1:
            raise Blocked(f"Cannot locate the version field in {file}")
        path.write_text(content)
    changelog.write_text(updated)
    current_version()


def refresh():
    summary("Fetching and validating all four IERS/USNO datasets.")
    run("cargo", "run", "--locked", "--release", "--features", "fetch",
        "--bin", "siderust-archive-update-time-data", "--", "--archive-root", ".")
    paths = changes("diff", "--no-renames", "--name-only", "-z", "HEAD") + changes(
        "ls-files", "--others", "--exclude-standard", "-z"
    )
    if any(not data_only(path) for path in paths):
        raise Blocked("Updater changed files outside the IERS/USNO snapshot")
    datasets = sorted(set(paths).intersection(DATASETS))
    if paths and not datasets:
        raise Blocked("Updater changed metadata without a scientific payload change")
    run("cargo", "run", "--locked", "-p", "archive-validate", "--", "MANIFEST.toml")
    summary("Snapshot integrity passed. Datasets changed: "
            + (", ".join(f"`{Path(path).name}`" for path in datasets) or "none") + ".")
    if datasets:
        run("cargo", "test", "--all-features", "--locked")
        git("add", "-A", "--", *DATA_PATHS)
        git("commit", "-m", "chore: refresh IERS/USNO time-data snapshot")
        summary(f"Prepared validated snapshot commit `{git('rev-parse', 'HEAD')}`.")
    return datasets


def push_snapshot(head):
    run("git", "push", "origin", "HEAD:refs/heads/main")
    repo = os.environ["GITHUB_REPOSITORY"]
    summary(f"Saved the validated [snapshot commit](https://github.com/{repo}/commit/{head}) on main.")


def publish(version, commit, entry, release):
    require_credentials(entry)
    if entry:
        verify_published(version, commit, entry)
        summary(f"crates.io already contains {CRATE} {version} from this commit.")
    else:
        if release:
            raise Blocked(f"GitHub Release v{version} exists without a matching crates.io version")
        run("cargo", "publish", "--locked", "-p", CRATE)
        for delay in (1, 2, 4, 8, 16, 0):
            if entry := registry_entry(version):
                verify_published(version, commit, entry)
                summary(f"Verified {CRATE} {version} in the crates.io index.")
                break
            time.sleep(delay)
        else:
            raise Blocked(f"crates.io has not indexed {version} yet; retry Publish for the same version")
    if not release:
        _, notes = change_entry(Path("CHANGELOG.md").read_text(), version)
        run("gh", "release", "create", f"v{version}", "--verify-tag",
            "--title", f"{CRATE} v{version}", "--notes", notes)
    elif release["draft"]:
        raise Blocked(f"GitHub Release v{version} is a draft; review it before completing publication")
    repo = os.environ["GITHUB_REPOSITORY"]
    summary(f"Release complete for [{CRATE} {version}](https://crates.io/crates/{CRATE}/{version}). "
            f"[Commit](https://github.com/{repo}/commit/{commit}) · "
            f"[Tag](https://github.com/{repo}/tree/v{version}) · "
            f"[GitHub Release](https://github.com/{repo}/releases/tag/v{version}).")


def release(mode, requested):
    if os.getenv("GITHUB_REF") != "refs/heads/main":
        raise Blocked("Dispatch this workflow on main")
    if git("status", "--porcelain"):
        raise Blocked("Release requires a clean checkout")
    main = git("rev-parse", MAIN)
    if git("rev-parse", "HEAD") != main:
        raise Blocked("Checkout is stale; dispatch again from the current main")
    refreshed = refresh() if mode == "auto" else []
    head = git("rev-parse", "HEAD")
    current = current_version()
    version = current if mode == "auto" else requested
    version_tuple(version)
    summary(f"Checking release state for `v{version}` at `{head}`.")
    commit = tag_commit(version)
    try:
        if commit:
            validate_tag(version, commit, head)
        elif mode == "auto":
            raise Blocked(f"Missing tag v{current} for main's Cargo version. Use Publish for the prepared release.")
        elif version != current:
            raise Blocked(f"Requested version {version} does not match main's Cargo version {current}")
        datasets = automatic_datasets(commit, head) if mode == "auto" else []
    except Blocked:
        if refreshed:
            push_snapshot(head)
        raise

    entry = registry_entry(version)
    existing = github_release(version)
    if not commit and (entry or existing):
        raise Blocked(f"v{version} already exists externally without a matching Git tag")
    if existing and (existing["draft"] or not entry):
        raise Blocked(f"GitHub Release v{version} is a draft or has no matching crates.io version")

    auto_patch = False
    if mode == "auto":
        if not (entry and existing):
            raise Blocked(f"Release v{version} is incomplete. Resume that version through Publish.")
        if not refreshed:
            summary("Upstream datasets are unchanged. No new patch required.")
            return
        if not datasets:
            push_snapshot(head)
            summary("The refreshed snapshot matches the released scientific payloads. No patch required.")
            return
        verify_published(current, commit, entry)
        major, minor, patch = version_tuple(current)
        version = f"{major}.{minor}.{patch + 1}"
        summary(f"Selecting automatic patch `v{version}` for changes since `v{current}`.")
        if tag_commit(version) or registry_entry(version) or github_release(version):
            raise Blocked(f"Automatic patch v{version} is already reserved; use Publish to recover")
        require_credentials(None)
        prepare_patch(current, version, datasets)
        auto_patch = True
        commit, entry, existing = None, None, None

    require_credentials(entry)
    if commit:
        git("switch", "--detach", commit)
    change_entry(Path("CHANGELOG.md").read_text(), version)
    if not entry:
        validate_candidate(dirty=auto_patch)
    modified = set(changes("diff", "--no-renames", "--name-only", "-z", "HEAD"))
    untracked = changes("ls-files", "--others", "--exclude-standard", "-z")
    if modified != (set(RELEASE_FILES) if auto_patch else set()) or untracked:
        raise Blocked("Release validation left unexpected changes in the checkout")
    if not commit:
        if auto_patch:
            git("add", "--", *RELEASE_FILES)
            git("commit", "-m", f"chore(release): {CRATE} v{version}")
        commit = git("rev-parse", "HEAD")
        if not auto_patch:
            remote_main = git("ls-remote", "--exit-code", "origin", "refs/heads/main").split()[0]
            if remote_main != commit:
                raise Blocked("Main advanced during validation; dispatch again from the reviewed commit")
        git("tag", "-a", f"v{version}", "-m", f"Release {CRATE} v{version}")
        refs = ("HEAD:refs/heads/main", tag_ref(version)) if auto_patch else (tag_ref(version),)
        run("git", "push", "--atomic", "origin", *refs)
        repo = os.environ["GITHUB_REPOSITORY"]
        if auto_patch:
            summary(f"Saved the validated [snapshot commit](https://github.com/{repo}/commit/{head}) on main.")
        summary(f"Reserved [annotated tag v{version}](https://github.com/{repo}/tree/v{version}) "
                f"at [`{commit}`](https://github.com/{repo}/commit/{commit}). "
                "Retry this version through Publish if any remaining step fails.")
    publish(version, commit, entry, existing)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("auto", "manual"))
    parser.add_argument("--version")
    args = parser.parse_args()
    if args.mode == "manual" and not args.version:
        parser.error("manual requires --version")
    try:
        release(args.mode, args.version)
    except Blocked as error:
        summary(f"Release blocked. {error}")
        print("::error::Release blocked. See the job summary.")
        return 1
    except (subprocess.CalledProcessError, OSError, KeyError, ValueError, tarfile.TarError) as error:
        summary(f"Maintenance or publication failed. {error}")
        print("::error::Maintenance or publication failed. See the preceding logs and job summary.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
