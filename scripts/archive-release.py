#!/usr/bin/env python3
"""Validate Archive metadata, prepare data-only patches, and publish existing tags."""

import argparse
from datetime import datetime, timezone
import hashlib
from http.client import IncompleteRead
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
NUMBER = r"(?:0|[1-9][0-9]*)"
CORE = rf"{NUMBER}\.{NUMBER}\.{NUMBER}"
PRERELEASE_ID = rf"(?:{NUMBER}|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
BUILD_ID = r"[0-9A-Za-z-]+"
VERSION = rf"{CORE}(?:-{PRERELEASE_ID}(?:\.{PRERELEASE_ID})*)?(?:\+{BUILD_ID}(?:\.{BUILD_ID})*)?"
PUBLISH_ARGS = ("--locked", "--all-features", "-p", CRATE)
RAW = "src/time/eop/raw/"
DATASETS = tuple(RAW + name for name in (
    "UTC-TAI.history", "deltat.data", "deltat.preds", "finals2000A.all",
))
DATA_PATHS = (*DATASETS, RAW + "time_data.provenance.toml",
              "src/time/eop/manifest.toml", "src/time/bundled/snapshot.rs")
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


def check_version(version):
    if not isinstance(version, str) or not re.fullmatch(VERSION, version):
        raise Blocked(f"Invalid SemVer release version {version!r}; expected MAJOR.MINOR.PATCH"
                      " with optional prerelease or build metadata")


def next_patch(version):
    if not re.fullmatch(CORE, version):
        raise Blocked(f"Automatic patches require a stable release baseline; found {version!r}")
    major, minor, patch = map(int, version.split("."))
    return f"{major}.{minor}.{patch + 1}"


def tag_version(tag):
    if not tag.startswith("v"):
        raise Blocked(f"Expected a v-prefixed version tag; got {tag!r}")
    version = tag[1:]
    check_version(version)
    return version


def current_version(ref=None):
    def read(path):
        return tomllib.loads(git("show", f"{ref}:{path}") if ref else Path(path).read_text())

    package = read("Cargo.toml")["package"]
    version = package["version"]
    locked = [item["version"] for item in read("Cargo.lock")["package"]
              if item["name"] == CRATE and "source" not in item]
    if package["name"] != CRATE or locked != [version]:
        raise Blocked(f"Version mismatch at {ref or 'HEAD'}: Cargo.toml={version}, Cargo.lock={locked}")
    check_version(version)
    return version


def change_entry(text, version, date=None, body=None):
    """Read notes or insert a section once, even when Unreleased is absent."""
    headings = list(re.finditer(r"^## (?!#)(.*)$", text, re.MULTILINE))
    entries = {}
    for i, heading in enumerate(headings):
        if match := re.fullmatch(r"\[([^\]]+)\](?: - [0-9]{4}-[0-9]{2}-[0-9]{2})?", heading[1]):
            key = match[1]
            if key != "Unreleased":
                check_version(key)
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
                      "prepare a new version on main and push its tag. Preserve existing tags.")
    result.check_returncode()


def validate_tag(version, commit, head):
    tag = f"v{version}"
    require_ancestor(commit, head, tag)
    if git("cat-file", "-t", tag_ref(version)) != "tag":
        raise Blocked(f"{tag} is not an annotated release tag")
    if current_version(commit) != version:
        raise Blocked(f"Version mismatch between {tag} and its Cargo metadata")


def data_only(path):
    return path in DATA_PATHS


def changes(*args):
    return [path for path in git(*args).split("\0") if path]


def require_data_only_history(base, head):
    # Inspect individual commits, including both sides and resolutions of merges.
    # A later revert of non-data work does not authorize automatic publication.
    history = changes("log", "--format=", "--name-only", "-z", "--diff-merges=separate",
                      "--no-renames", f"{base}..{head}")
    outside = sorted({path for path in history if not data_only(path)})
    if outside:
        raise Blocked("Non-data changes since the release tag: " + ", ".join(outside)
                      + ". Prepare a reviewed release through Publish.")


def request(url, headers=None, allow_missing=False):
    query = Request(url)
    for key, value in (headers or {}).items():
        query.add_unredirected_header(key, value)
    for delay in (1, 2, 4, None):
        try:
            with urlopen(query, timeout=15) as response:
                return response.read()
        except HTTPError as error:
            if error.code == 404 and allow_missing:
                return None
            if delay is None or error.code not in (408, 429, 500, 502, 503, 504):
                raise Blocked(f"HTTP {error.code} checking {url}") from error
        except (URLError, TimeoutError, IncompleteRead) as error:
            if delay is None:
                raise Blocked(f"Could not check {url}: {error}") from error
        time.sleep(delay)


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
        manifest = tomllib.loads(package.extractfile(f"{filename}/Cargo.toml").read().decode())
    if (manifest["package"]["name"], manifest["package"]["version"]) != (CRATE, version):
        raise Blocked(f"Registry archive {filename} has mismatched Cargo metadata")
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


def check_metadata():
    version = current_version()
    change_entry(Path("CHANGELOG.md").read_text(), version)
    return version


def validate_candidate():
    run("bash", "scripts/check-release.sh")
    run("bash", "scripts/check-packaged-docs.sh")
    require_clean()
    summary("Release candidate passed Rust, manifest, packaging, and docs.rs checks.")


def require_clean():
    if git("status", "--porcelain"):
        raise Blocked("Release requires a clean checkout")


def resolve_tag(tag, expected):
    version = tag_version(tag)
    commit = tag_commit(version)
    if not commit:
        raise Blocked(f"Missing {tag}. Push the reviewed annotated version tag before publishing.")
    # A tag-push event can refer to an annotated tag object or its peeled commit.
    if expected and expected not in (commit, git("rev-parse", tag_ref(version))):
        raise Blocked(f"{tag} points at {commit}, expected {expected}")
    validate_tag(version, commit, git("rev-parse", MAIN))
    return version, commit


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
    return datasets


def push_patch(base, version):
    # The candidate is a child of base. The lease also rejects a remote rewind
    # or a concurrent change to main. The tag ref is never forced.
    run("git", "push", "--atomic", f"--force-with-lease=refs/heads/main:{base}",
        "origin", "HEAD:refs/heads/main", tag_ref(version))


def prepare():
    require_clean()
    base = git("rev-parse", "HEAD")
    if base != git("rev-parse", MAIN):
        raise Blocked("Checkout is stale; rerun maintenance from current main")
    if not refresh():
        summary("Upstream scientific payloads are unchanged. No patch required.")
        return
    current = check_metadata()
    commit = tag_commit(current)
    if not commit:
        raise Blocked(f"Missing baseline v{current}. Publish the reviewed release before maintenance.")
    validate_tag(current, commit, base)
    require_data_only_history(commit, base)
    datasets = changes("diff", "--no-renames", "--name-only", "-z", commit, "--", *DATASETS)
    if not datasets:
        summary("The refreshed snapshot matches the published payloads. No patch required.")
        return

    entry = registry_entry(current)
    existing = github_release(current)
    if not entry or not existing or existing["draft"]:
        raise Blocked(f"Release v{current} is incomplete. Resume that version through Publish.")
    verify_published(current, commit, entry)

    version = next_patch(current)
    if tag_commit(version) or registry_entry(version) or github_release(version):
        raise Blocked(f"Automatic patch v{version} already exists; inspect its release state")
    prepare_patch(current, version, datasets)
    check_metadata()
    git("add", "-A", "--", *DATA_PATHS, *RELEASE_FILES)
    git("commit", "-m", f"chore(release): {CRATE} v{version} with refreshed time data")
    validate_candidate()
    commit = git("rev-parse", "HEAD")
    tag = f"v{version}"
    git("tag", "-a", tag, "-m", f"Release {CRATE} {tag}")
    push_patch(base, version)
    repo = os.environ["GITHUB_REPOSITORY"]
    summary(f"Reserved [tag {tag}](https://github.com/{repo}/tree/{tag}) "
            f"at [`{commit}`](https://github.com/{repo}/commit/{commit}). "
            "If publication fails, resume this tag through Publish.")
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        print(f"tag={tag}\ncommit={commit}", file=output)


def await_registry(version):
    for delay in (0, 2, 4, 8, 16, 30, 30):
        if delay:
            time.sleep(delay)
        if entry := registry_entry(version):
            return entry
    raise Blocked(f"crates.io has not indexed {version}; inspect the Cargo output "
                  "and resume Publish for the same tag")


def publish(tag, expected):
    require_clean()
    version, commit = resolve_tag(tag, expected)
    if git("rev-parse", "HEAD") != commit:
        raise Blocked(f"Checkout must be at the validated commit for {tag}")
    _, notes = change_entry(Path("CHANGELOG.md").read_text(), version)
    # Compare with the live tag before either external mutation. Tag rules
    # should prevent moving or deleting version tags.
    live = git("ls-remote", "--exit-code", "origin", tag_ref(version)).split()[0]
    if live != git("rev-parse", tag_ref(version)):
        raise Blocked(f"The remote {tag} changed after checkout")
    entry = registry_entry(version)
    existing = github_release(version)
    if existing and existing["draft"]:
        raise Blocked(f"GitHub Release {tag} is a draft; review it before resuming")
    if not entry:
        if existing:
            raise Blocked(f"GitHub Release {tag} exists without a matching crates.io version")
        if not os.getenv("CARGO_REGISTRY_TOKEN"):
            raise Blocked("Configure CARGO_REGISTRY_TOKEN for the crates.io publication job")
        # A timeout or duplicate response can mean the upload already happened.
        # Reconcile via the index and source identity; never blindly upload again.
        result = run("cargo", "publish", *PUBLISH_ARGS, check=False)
        if result.returncode:
            summary("Cargo publish reported an error. Checking whether crates.io accepted the upload.")
        entry = await_registry(version)
    verify_published(version, commit, entry)
    require_clean()

    prerelease = "-" in version.split("+", 1)[0]
    options = ("--title", f"{CRATE} {tag}", "--notes", notes)
    if not existing:
        flags = ("--prerelease",) if prerelease else ()
        run("gh", "release", "create", tag, "--verify-tag", *options, *flags)
    elif (existing.get("body") or "").strip() != notes or existing["prerelease"] != prerelease:
        run("gh", "release", "edit", tag, *options, f"--prerelease={str(prerelease).lower()}")
    repo = os.environ["GITHUB_REPOSITORY"]
    summary(f"Release complete for [{CRATE} {version}](https://crates.io/crates/{CRATE}/{version}). "
            f"[Commit](https://github.com/{repo}/commit/{commit}) · "
            f"[Tag](https://github.com/{repo}/tree/{tag}) · "
            f"[GitHub Release](https://github.com/{repo}/releases/tag/{tag}).")


def validate_tagged_candidate(tag, expected):
    require_clean()
    _, commit = resolve_tag(tag, expected)
    git("switch", "--detach", commit)
    check_metadata()
    summary(f"Validating release tag `{tag}` at `{commit}`.")
    validate_candidate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check-metadata")
    commands.add_parser("prepare")
    for name in ("validate-tag", "publish"):
        command = commands.add_parser(name)
        command.add_argument("--tag", required=True)
        command.add_argument("--commit", default="")
    args = parser.parse_args()
    try:
        if args.command == "check-metadata":
            summary(f"Cargo metadata and changelog agree on {check_metadata()}.")
        elif args.command == "prepare":
            prepare()
        elif args.command == "validate-tag":
            validate_tagged_candidate(args.tag, args.commit)
        else:
            publish(args.tag, args.commit)
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
