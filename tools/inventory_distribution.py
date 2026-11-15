#!/usr/bin/env python3
"""Read-only inventory of explicitly selected shipped Marvel Heroes files.

Run on the source machine with an install directory argument. Emits only paths,
sizes and optional SHA-256, never file contents. Does not follow symlinks. Runtime
user data, downloads, patch backups and unclassified files are excluded.
"""
import argparse
from collections import Counter
import datetime
import hashlib
import json
import os
from pathlib import Path
import stat


def is_distribution(relative):
    p = Path(relative)
    parts = p.parts
    suffix = p.suffix.lower()
    if any(x.lower() in {'userdata', 'download', 'logs', 'cache', 'saved', '_mhclientpatch-backup'} for x in parts):
        return False
    if relative.startswith('Data/Game/'):
        return suffix in {'.sip', '.locale', '.string'}
    if relative.startswith('UnrealEngine3/Binaries/'):
        return suffix in {'.exe', '.dll', '.pak'}
    for prefix in ['UnrealEngine3/MarvelGame/CookedPCConsole/', 'UnrealEngine3/Engine/Shaders/Binaries/']:
        if relative.startswith(prefix):
            return suffix in {'.upk', '.tfc', '.pck', '.bin'}
    if relative.startswith('UnrealEngine3/MarvelGame/Movies/'):
        return suffix in {'.bik', '.bk2'}
    if relative.startswith('UnrealEngine3/MarvelGame/Splash/'):
        return suffix == '.bmp'
    if relative.startswith(('UnrealEngine3/Engine/Config/', 'UnrealEngine3/MarvelGame/Config/')):
        return suffix == '.ini' and p.name.startswith(('Base', 'Default'))
    if relative.startswith(('UnrealEngine3/Engine/Localization/', 'UnrealEngine3/MarvelGame/Localization/')):
        return suffix in {'.int', '.cze', '.esn', '.chn', '.hun', '.fra', '.deu', '.pol', '.kor', '.rus', '.ptb', '.slo', '.esm', '.ita', '.jpn', '.por', '.chi', '.spa'}
    return False


def inventory(root, hash_files):
    root = root.resolve(strict=True)
    files, excluded = [], []
    for subtree in ['Data', 'UnrealEngine3']:
        base = root / subtree
        if base.is_symlink():
            raise ValueError('distribution root must not be a symlink: ' + subtree)
        for parent, dirs, names in os.walk(base, followlinks=False):
            for name in list(dirs):
                child = Path(parent) / name
                if child.is_symlink():
                    dirs.remove(name)
                    excluded.append({'path':str(child.relative_to(root)), 'reason':'symlink-directory'})
            for name in sorted(names):
                path = Path(parent) / name
                relative = path.relative_to(root).as_posix()
                info = path.lstat()
                if not stat.S_ISREG(info.st_mode) or not is_distribution(relative):
                    excluded.append({'path':relative, 'reason':'not-selected-distribution-file'})
                    continue
                row = {'path':relative, 'bytes':info.st_size}
                if hash_files:
                    with path.open('rb') as stream:
                        row['sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
                    after = path.lstat()
                    if (info.st_size,info.st_mtime_ns,info.st_ino) != (after.st_size,after.st_mtime_ns,after.st_ino):
                        raise ValueError('file changed while hashing: ' + relative)
                files.append(row)
    files.sort(key=lambda row:row['path'])
    return {'schema':1,'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'source_root':str(root),'hash_algorithm':'sha256' if hash_files else None,
            'build_verified':False,'scope':'selected distribution files; personal and unclassified files excluded',
            'files':files,'file_count':len(files),'bytes':sum(row['bytes'] for row in files),
            'extensions':dict(Counter(Path(row['path']).suffix.lower() for row in files)),
            'excluded':excluded}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root',type=Path)
    parser.add_argument('--hash',action='store_true')
    args = parser.parse_args()
    print(json.dumps(inventory(args.root, args.hash), indent=2))
