#!/usr/bin/env python3
"""Verify a private distribution copy against its source inventory.

Nothing is written unless --report selects an output. No account or runtime
files are inspected; only exact regular files in the selected manifest are read.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat


def verify(root, manifest):
    root = root.resolve(strict=True)
    rows = manifest['files']
    seen, verified, problems = set(), [], []
    for entry in rows:
        name = entry['path']
        relative = PurePosixPath(name)
        if relative.is_absolute() or not relative.parts or any(x in ('.', '..') for x in name.split('/')) or '\\' in name or name in seen:
            raise ValueError('unsafe or duplicate manifest path')
        seen.add(name)
        path = root
        try:
            for component in relative.parts:
                path /= component
                if path.is_symlink():
                    raise ValueError('symlink in pin path')
            info = path.stat()
            if not stat.S_ISREG(info.st_mode):
                raise ValueError('not a regular file')
            if info.st_size != entry['bytes']:
                raise ValueError('size mismatch')
            with path.open('rb') as stream:
                digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            if digest != entry['sha256']:
                raise ValueError('sha256 mismatch')
            after = path.stat()
            if (info.st_size, info.st_mtime_ns, info.st_ino) != (after.st_size, after.st_mtime_ns, after.st_ino):
                raise ValueError('file changed while hashing')
            verified.append(name)
        except (OSError, ValueError) as error:
            problems.append({'path':name,'reason':str(error)})
    verified_names = set(verified)
    return {'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'root':str(root),'expected_files':len(rows),'verified_files':len(verified),
            'verified_bytes':sum(f['bytes'] for f in rows if f['path'] in verified_names),
            'complete':not problems,'problems':problems}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root',type=Path)
    parser.add_argument('manifest',type=Path)
    parser.add_argument('--report',type=Path)
    args = parser.parse_args()
    result = verify(args.root, json.loads(args.manifest.read_text()))
    if args.report:
        with args.report.open('x') as stream:
            json.dump(result,stream,indent=2)
            stream.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k != 'problems'},indent=2))
    raise SystemExit(0 if result['complete'] else 1)
