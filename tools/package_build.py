#!/usr/bin/env python3
"""Stage an already-built original executable in the owner's private build root.

This does not upload, alter launcher channels, bundle game assets or overwrite
an existing build. Run the focused startup/feature checks before packaging.
"""
from pathlib import Path
import argparse
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import tomllib

repo = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=repo / 'target/release/mhf')
parser.add_argument('--profile', choices=['release', 'dev'], default='release')
parser.add_argument('--dependency-notices', type=Path)
args = parser.parse_args()
version = tomllib.loads((repo / 'Cargo.toml').read_text())['package']['version']
binary = args.binary.resolve(strict=True)
reported = subprocess.check_output([str(binary), '--version'], text=True).strip()
if reported != 'mhf ' + version:
    raise SystemExit('binary does not match Cargo version; rebuild first')
status = subprocess.check_output([str(binary), 'status'], text=True)
renderer_compiled = 'Offscreen image renderer compiled: true' in status
notices = None
notice_files = {}
if args.dependency_notices:
    notices = args.dependency_notices.resolve(strict=True)
    manifest_path = notices / 'manifest.json'
    manifest = json.loads(manifest_path.read_text())
    if manifest.get('complete') is not True or manifest.get('issues') != []:
        raise SystemExit('dependency notice collection is incomplete')
    for package in manifest['packages']:
        for notice in package['notices']:
            relative = Path(notice['path'])
            if relative.is_absolute() or '..' in relative.parts:
                raise SystemExit('unsafe notice path')
            source = notices / relative
            if source.is_symlink() or not source.resolve(strict=True).is_relative_to(notices):
                raise SystemExit('notice path leaves the collection')
            data = source.read_bytes()
            if len(data) != notice['bytes'] or hashlib.sha256(data).hexdigest() != notice['sha256']:
                raise SystemExit('dependency notice hash or size changed')
            notice_files[relative] = source
elif renderer_compiled:
    raise SystemExit('renderer builds require --dependency-notices with a complete collection')
root = Path.home() / '.local/share/marvel-heroes-forever-rust/builds'
root.mkdir(mode=0o700, parents=True, exist_ok=True)
destination = root / version
destination.mkdir(mode=0o700)
shutil.copy2(binary, destination / 'mhf')
# Explicit original-only allowlist. Never walk the private assets/evidence tree.
for name in ['README.md', 'FINDINGS.md', 'Cargo.toml', 'Cargo.lock']:
    shutil.copy2(repo / name, destination / name)
shutil.copytree(repo / 'docs', destination / 'docs', ignore=shutil.ignore_patterns('*.bin','*.upk','*.sip'))
if notices:
    notice_destination = destination / 'dependency-notices'
    notice_destination.mkdir(mode=0o700)
    shutil.copy2(notices / 'manifest.json', notice_destination / 'manifest.json')
    for relative, source in notice_files.items():
        target = notice_destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
rows=[]
for path in sorted(destination.rglob('*')):
    if path.is_file():
        with path.open('rb') as stream:
            digest = hashlib.file_digest(stream,'sha256').hexdigest()
        rows.append({'path':str(path.relative_to(destination)), 'bytes':path.stat().st_size, 'sha256':digest})
commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
manifest={'version':version,'created_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'source_commit':commit,'source_dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True)),
          'profile':args.profile, 'renderer_compiled':renderer_compiled,
          'scope':'original asset tooling with registered optional libraries; not playable; no game assets or retail binaries','files':rows}
(destination/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'version':version,'path':str(destination),'files':len(rows),'binary_sha256':next(f['sha256'] for f in rows if f['path']=='mhf')},indent=2))
