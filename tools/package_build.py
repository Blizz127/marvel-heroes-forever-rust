#!/usr/bin/env python3
"""Stage an already-built original executable in the owner's private build root.

This does not upload, alter launcher channels, bundle game assets or overwrite
an existing build. Run the focused startup/feature checks before packaging.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import tomllib

repo = Path(__file__).resolve().parent.parent
version = tomllib.loads((repo / 'Cargo.toml').read_text())['package']['version']
binary = repo / 'target/release/mhf'
reported = subprocess.check_output([str(binary), '--version'], text=True).strip()
if reported != 'mhf ' + version:
    raise SystemExit('binary does not match Cargo version; rebuild first')
root = Path.home() / '.local/share/marvel-heroes-forever-rust/builds'
root.mkdir(mode=0o700, parents=True, exist_ok=True)
destination = root / version
destination.mkdir(mode=0o700)
shutil.copy2(binary, destination / 'mhf')
# Explicit original-only allowlist. Never walk the private assets/evidence tree.
for name in ['README.md', 'FINDINGS.md']:
    shutil.copy2(repo / name, destination / name)
shutil.copytree(repo / 'docs', destination / 'docs', ignore=shutil.ignore_patterns('*.bin','*.upk','*.sip'))
rows=[]
for path in sorted(destination.rglob('*')):
    if path.is_file():
        with path.open('rb') as stream:
            digest = hashlib.file_digest(stream,'sha256').hexdigest()
        rows.append({'path':str(path.relative_to(destination)), 'bytes':path.stat().st_size, 'sha256':digest})
commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
manifest={'version':version,'created_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'source_commit':commit,'source_dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True)),
          'scope':'original asset tooling; not playable; no game assets or retail binaries','files':rows}
(destination/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'version':version,'path':str(destination),'files':len(rows),'binary_sha256':next(f['sha256'] for f in rows if f['path']=='mhf')},indent=2))
