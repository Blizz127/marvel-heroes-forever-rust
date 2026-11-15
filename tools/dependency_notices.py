#!/usr/bin/env python3
"""Collect selected Cargo dependency notices into a new private directory.

Only installed package notices and explicitly supplied, hash-bound supplements
are copied. This records collection coverage; it is not a legal compliance audit.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys
from urllib.parse import urlparse


MAX_INPUT_BYTES = 64 * 1024 * 1024
MAX_NOTICE_BYTES = 16 * 1024 * 1024
MAX_NOTICE_FILES = 512
NOTICE_PREFIXES = ("license", "copying", "notice")


class CollectionError(Exception):
    pass


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise CollectionError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def no_symlinks(path: Path) -> None:
    if not path.is_absolute() or ".." in path.parts:
        raise CollectionError(f"expected absolute path without '..': {path}")
    current = Path(path.anchor)
    for part in path.parts[1:]:
        current /= part
        if stat.S_ISLNK(current.lstat().st_mode):
            raise CollectionError(f"symlink is not accepted: {current}")


def read_regular(path: Path, limit: int) -> bytes:
    no_symlinks(path)
    observed = path.lstat()
    if not stat.S_ISREG(observed.st_mode) or observed.st_size > limit:
        raise CollectionError(f"not a bounded regular file: {path}")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_size > limit or (
            before.st_dev, before.st_ino
        ) != (observed.st_dev, observed.st_ino):
            raise CollectionError(f"not a bounded regular file: {path}")
        data = stream.read(limit + 1)
        after = os.fstat(stream.fileno())
    if len(data) > limit or (before.st_size, before.st_mtime_ns) != (
        after.st_size, after.st_mtime_ns
    ) or len(data) != before.st_size:
        raise CollectionError(f"file changed or exceeded the read limit: {path}")
    return data


def read_json(path: Path):
    data = read_regular(path, MAX_INPUT_BYTES)
    return json.loads(data, object_pairs_hook=unique_object), digest(data)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_relative(value: str) -> PurePosixPath:
    if not isinstance(value, str) or not value or "\\" in value:
        raise CollectionError(f"invalid relative path: {value!r}")
    parts = value.split("/")
    if any(part in ("", ".", "..") or ":" in part or any(
        ord(char) < 32 or ord(char) == 127 for char in part
    ) for part in parts):
        raise CollectionError(f"unsafe relative path: {value!r}")
    path = PurePosixPath(value)
    if path.is_absolute():
        raise CollectionError(f"absolute notice path is not accepted: {value}")
    return path


def selected_packages(metadata):
    if metadata.get("version") != 1 or not isinstance(metadata.get("resolve"), dict):
        raise CollectionError("expected Cargo format-version 1 with a resolve graph")
    packages = {}
    for package in metadata["packages"]:
        if package["id"] in packages:
            raise CollectionError("duplicate package id")
        packages[package["id"]] = package
    nodes = {}
    for node in metadata["resolve"]["nodes"]:
        if node["id"] in nodes:
            raise CollectionError("duplicate resolve node")
        nodes[node["id"]] = node
    root = metadata["resolve"].get("root")
    roots = [root] if root else metadata.get("workspace_default_members", [])
    if not roots:
        raise CollectionError("metadata has no selected root/default workspace members")
    pending = list(roots)
    visited = set()
    dev_edges = []
    while pending:
        package_id = pending.pop()
        if package_id in visited:
            continue
        if package_id not in packages or package_id not in nodes:
            raise CollectionError(f"unresolved selected package: {package_id}")
        visited.add(package_id)
        for dependency in nodes[package_id]["deps"]:
            kinds = dependency["dep_kinds"]
            if not kinds or any(k["kind"] not in (None, "build", "dev") for k in kinds):
                raise CollectionError("unknown or empty dependency kind")
            if any(k["kind"] != "dev" for k in kinds):
                pending.append(dependency["pkg"])
            else:
                dev_edges.append({"from": package_id, "to": dependency["pkg"]})
    chosen = [packages[key] for key in visited.difference(roots)]
    chosen.sort(key=lambda package: (package["name"], package["version"], package["id"]))
    return chosen, nodes, roots, dev_edges


def crate_directory(package) -> Path:
    manifest = Path(package["manifest_path"])
    no_symlinks(manifest)
    if not manifest.is_file():
        raise CollectionError(f"package manifest is not a regular file: {manifest}")
    return manifest.parent


def installed_notices(package, root: Path):
    candidates = set()

    def visit(path: Path, depth=0):
        if depth > 16:
            raise CollectionError("notice directory nesting exceeds 16 levels")
        no_symlinks(path)
        if path.is_dir():
            for child in sorted(path.iterdir()):
                visit(child, depth + 1)
        elif path.is_file():
            relative = path.relative_to(root).as_posix()
            safe_relative(relative)
            candidates.add(relative)
            if len(candidates) > MAX_NOTICE_FILES:
                raise CollectionError("too many notice files in one package")
        else:
            raise CollectionError(f"special file in notice inventory: {path}")

    for path in sorted(root.iterdir()):
        if path.name.lower().startswith(NOTICE_PREFIXES):
            visit(path)
    declared = package.get("license_file")
    if declared:
        path = Path(declared)
        if path.is_absolute():
            try:
                relative = path.relative_to(root).as_posix()
            except ValueError as error:
                raise CollectionError("declared license_file is outside its crate") from error
        else:
            relative = declared
        safe_relative(relative)
        selected = root / relative
        if not selected.is_file():
            raise CollectionError(f"declared license_file is missing or not regular: {selected}")
        visit(selected)
    return sorted(candidates)


def package_output_name(package) -> str:
    for field in ("name", "version"):
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.+\-]{0,160}", package[field]):
            raise CollectionError(f"unsafe package {field}")
    return f"{package['name']}-{package['version']}-{digest(package['id'].encode())[:12]}"


def matching_supplements(supplement, package):
    matches = [row for row in supplement.get("packages", [])
               if row.get("name") == package["name"] and row.get("version") == package["version"]
               and row.get("package_id", package["id"]) == package["id"]]
    if len(matches) > 1:
        raise CollectionError("duplicate supplement entries for selected package")
    return matches[0] if matches else None


def supplemental_notices(row, package, root, supplement_root):
    if not re.fullmatch(r"[0-9a-fA-F]{40}", row.get("revision", "")):
        raise CollectionError("supplement revision must be an exact 40-digit git commit")
    if row.get("repository") != package.get("repository") or not row.get("repository"):
        raise CollectionError("supplement repository differs from Cargo metadata")
    vcs, vcs_hash = read_json(root / ".cargo_vcs_info.json")
    if vcs.get("git", {}).get("sha1", "").lower() != row["revision"].lower():
        raise CollectionError("supplement revision differs from installed crate VCS metadata")
    if not row.get("files"):
        raise CollectionError("supplement has no files")
    result = []
    for file in row["files"]:
        relative = safe_relative(file["path"])
        kind = file.get("kind", "upstream-notice")
        if kind not in ("upstream-notice", "upstream-declaration", "official-license-text", "upstream-component-notice"):
            raise CollectionError(f"unknown supplemental source kind: {kind}")
        source_revision = file.get("source_revision", None if kind == "official-license-text" else row["revision"])
        component = {}
        if kind == "official-license-text":
            if source_revision is not None:
                raise CollectionError("official licence text must not claim a crate git revision")
        elif kind == "upstream-component-notice":
            if not re.fullmatch(r"[0-9a-fA-F]{40}", source_revision or ""):
                raise CollectionError("component notice needs its exact source revision")
            basis = file["revision_basis"]
            if basis.get("kind") != "gitlink" or basis.get("parent_revision") != row["revision"]:
                raise CollectionError("component notice needs a gitlink at its package revision")
            component_repository = urlparse(file["source_repository"])
            evidence_url = urlparse(basis["evidence_url"])
            for source in (component_repository, evidence_url):
                if source.scheme != "https" or not source.netloc or source.username or source.password:
                    raise CollectionError("component source provenance must use HTTPS without credentials")
            if row["revision"] not in evidence_url.path.split("/"):
                raise CollectionError("gitlink evidence URL must select the exact parent revision")
            gitlink_path = safe_relative(basis["path"]).as_posix()
            tree, tree_hash = read_json(supplement_root / safe_relative(basis["evidence_path"]))
            if tree_hash != basis["evidence_sha256"]:
                raise CollectionError("component gitlink evidence SHA-256 mismatch")
            matches = [entry for entry in tree["tree"] if entry.get("path") == gitlink_path]
            if len(matches) != 1 or matches[0].get("mode") != "160000" or matches[0].get("type") != "commit" or matches[0].get("sha") != source_revision:
                raise CollectionError("component revision does not match the saved gitlink entry")
            component = {"source_repository": file["source_repository"],
                         "revision_basis": {"kind": "gitlink", "parent_revision": row["revision"],
                                            "entry": matches[0], "evidence_url": basis["evidence_url"],
                                            "evidence_sha256": tree_hash}}
        elif source_revision != row["revision"]:
            raise CollectionError("supplement file revision differs from its package revision")
        parsed = urlparse(file["url"])
        if parsed.scheme != "https" or not parsed.netloc or parsed.username or parsed.password:
            raise CollectionError("supplement source must be an HTTPS URL without credentials")
        if not re.fullmatch(r"[0-9a-f]{64}", file.get("sha256", "")):
            raise CollectionError("supplement SHA-256 is missing or malformed")
        data = read_regular(supplement_root / relative, MAX_NOTICE_BYTES)
        if digest(data) != file["sha256"]:
            raise CollectionError("supplement notice SHA-256 mismatch")
        if "bytes" in file and file["bytes"] != len(data):
            raise CollectionError("supplement notice byte length mismatch")
        result.append((relative, data, {
            "origin": "official-license-text" if kind == "official-license-text" else "upstream-supplement",
            "kind": kind, "url": file["url"],
            "repository": row["repository"], "package_revision": row["revision"],
            "source_revision": source_revision,
            "vcs_metadata_sha256": vcs_hash,
            "vcs_metadata_path": str(root / ".cargo_vcs_info.json"),
            "supplement_source_record": file,
            **component,
        }))
    if row.get("upstream_notice_absent"):
        kinds = {entry[2]["kind"] for entry in result}
        if not {"upstream-declaration", "official-license-text"}.issubset(kinds):
            raise CollectionError("absent upstream notice needs its exact declaration and official terms")
    return result


def write_notice(output, relative, data):
    safe_relative(relative.as_posix())
    if not data or b"\x00" in data:
        raise CollectionError("notice is empty or contains NUL bytes")
    target = output / relative
    target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    no_symlinks(target.parent)
    with target.open("xb") as stream:
        os.fchmod(stream.fileno(), 0o600)
        stream.write(data)
    return {"path": relative.as_posix(), "bytes": len(data), "sha256": digest(data)}


def collect(args):
    metadata_path = Path(args.metadata)
    metadata, metadata_hash = read_json(metadata_path)
    packages, nodes, roots, dev_edges = selected_packages(metadata)
    supplement = {}
    supplement_hash = None
    if args.supplements:
        supplement, supplement_hash = read_json(Path(args.supplements))
    output = Path(args.output)
    if not output.is_absolute() or ".." in output.parts:
        raise CollectionError("output must be an absolute path without '..'")
    no_symlinks(output.parent)
    for package in metadata["packages"]:
        source_root = crate_directory(package)
        if output == source_root or source_root in output.parents:
            raise CollectionError("output must not be inside a source crate")
    output.mkdir(mode=0o700)
    result = {
        "schema_version": 1, "complete": False, "issues": [],
        "scope": "selected Cargo normal/build dependency notice collection; not a legal audit",
        "metadata_sha256": metadata_hash, "metadata_path": str(metadata_path),
        "declared_filter_platform": args.target,
        "root_package_ids": roots, "excluded_dev_edges": dev_edges,
        "supplements_sha256": supplement_hash, "packages": [],
        "supplement_exceptions": supplement.get("exceptions", []),
    }
    for package in packages:
        row = {"package_id": package["id"], "name": package["name"],
               "version": package["version"], "source": package.get("source"),
               "license": package.get("license"), "license_file": package.get("license_file"),
               "authors": package.get("authors", []), "repository": package.get("repository"),
               "features": sorted(nodes[package["id"]].get("features", [])),
               "registry_notice_absent": None, "notices": []}
        result["packages"].append(row)
        try:
            root = crate_directory(package)
            row["manifest_path"] = str(root / "Cargo.toml")
            row["cargo_manifest_sha256"] = digest(read_regular(root / "Cargo.toml", MAX_INPUT_BYTES))
            if not row["license"] and not row["license_file"]:
                raise CollectionError("package has no license or license_file declaration")
            paths = installed_notices(package, root)
            row["registry_notice_absent"] = not paths
            destination = PurePosixPath("packages") / package_output_name(package)
            for relative in paths:
                data = read_regular(root / relative, MAX_NOTICE_BYTES)
                copied = write_notice(output, destination / "installed" / relative, data)
                copied.update({"origin": "installed-crate", "source_path": str(root / relative)})
                row["notices"].append(copied)
            extra = matching_supplements(supplement, package)
            if extra:
                row["upstream_notice_absent"] = extra.get("upstream_notice_absent", False)
                row["supplement_record"] = {key: value for key, value in extra.items() if key != "files"}
                for relative, data, provenance in supplemental_notices(
                    extra, package, root, Path(args.supplements).parent
                ):
                    copied = write_notice(output, destination / "supplement" / relative, data)
                    copied.update(provenance)
                    row["notices"].append(copied)
            if not row["notices"]:
                raise CollectionError("no installed or verified supplemental notice texts")
        except (CollectionError, OSError, ValueError, KeyError, TypeError) as error:
            result["issues"].append({"package_id": package["id"], "problem": str(error)})
    result["complete"] = not result["issues"]
    result["package_count"] = len(result["packages"])
    result["notice_count"] = sum(len(row["notices"]) for row in result["packages"])
    data = (json.dumps(result, indent=2, sort_keys=True) + "\n").encode()
    with (output / "manifest.json").open("xb") as stream:
        os.fchmod(stream.fileno(), 0o600)
        stream.write(data)
    print(json.dumps({"complete": result["complete"], "packages": result["package_count"],
                      "notices": result["notice_count"], "issues": result["issues"],
                      "manifest": str(output / "manifest.json"), "manifest_sha256": digest(data)}))
    return 0 if result["complete"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", required=True, help="absolute Cargo metadata JSON path")
    parser.add_argument("--output", required=True, help="new absolute private notices directory")
    parser.add_argument("--supplements", help="absolute manifest of exact-revision upstream notices")
    parser.add_argument("--target", help="record the caller's Cargo --filter-platform value")
    args = parser.parse_args()
    try:
        return collect(args)
    except (CollectionError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"dependency notices: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
