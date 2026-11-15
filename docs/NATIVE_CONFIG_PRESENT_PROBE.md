# Offline observation with exact owner configuration present

Task `a3d58c579542`, 2026-11-15 UTC.

Root authorized one new immutable snapshot after the exact configuration path
returned `ENOENT` and the original file's schema was inspected without printing
values. The [configuration receipt](NATIVE_CLIENT_CONFIG.md) records that
source and the absence of explicit account/credential fields.

**VERIFIED:** The new snapshot contains 454 regular files, totaling
5,115,044,929 bytes. All 453 previous hashes are preserved. Only the exact
256-byte `Data/Configs/ClientConfig.xml` was added, with SHA-256
`a319ef50b21d3d7e84b5b3c0894b41805c809e7d0eae728b7b4be7f1d3717c5b`.
All destination inodes are independent, files are `0400` and directories `0500`.
There are no symlinks or hard links in the game snapshot. Old snapshots and the
owner install remain untouched.

Private run: `evidence/native-retail-reference/run-20261115T103701Z/`.
Snapshot manifest SHA-256:
`e7889c5f0e3676e7792974d270f207923b6d1f4dd5e6619fd511d1655dbd1fb6`.
Assembly receipt SHA-256:
`a6162914804fb278f35cb267c95c6c9157ddf4faf52ab4c4f021fb03fa30e53a`.

A new 1,755-file clean prefix clone matched the baseline hashes. The original
normal standalone arguments and verified namespace/Xvfb/cleanup controls remain
unchanged. The snapshot is still partial; adding this one file does not imply
all retail distribution dependencies are available.

Because an existing file changes strace path canonicalization, a harmless
present-file metadata preflight tested the same twelve target/parent path forms.
All 36 intended metadata calls were recorded and all six unrelated negative
probes were excluded. The 42-operation fixture used only stat/access and
metadata-only `O_PATH` handles; no configuration contents were read. Preflight
receipt SHA-256:
`9cd5e47cf8c96b22dc1d90de8531f28befa4de6a7fdd159882df4c5b7a91a0a5`.
The known syscall/path-form and unannotated-relative-access limitations from
`NATIVE_TARGETED_METADATA_TRACE.md` still apply.

**OBSERVED:** The single native attempt used the same 45-second inner deadline,
55-second outer bound, 20,000-call limit and 8-MiB collector cap. At
10:40:50.084–10:40:50.086 UTC, retail PID 12 made nine successful `newfstatat`
checks of the exact configuration, reporting a regular 256-byte file. At
10:40:50.088409, owned `wineserver` PID 15 issued `openat` with
`O_RDONLY|O_NONBLOCK`, receiving fd 433 for that exact path.

| Frozen input set | Actual configuration-path observation |
| --- | --- |
| Previous 453 files | Retail status check and parent check returned `ENOENT` |
| New 454 files | Nine successful retail status checks, then a successful read-only Wine-server open |

This resolves the observed missing-path condition. It proves metadata success
and handle acquisition, not XML content consumption, deserialization, a direct
native callsite or successful frontend initialization.

The tracer started at 10:40:31.589 UTC. The supervisor reached its deadline at
10:41:16.405 and completed owned cleanup at 10:41:17.028. All ten selected calls
and 2,178 emitted bytes were retained without cap, discard or collector error.
The tracer was stopped with SIGTERM by owned cleanup; Xvfb and the REA wrapper
exited zero. No owned live process remained. Additional inaccessible host PIDs
were identified by metadata as unrelated kernel workers or container/SSH-service
ancestry; no unrelated process was signaled or attached.

Eighteen periodic frames include eleven observations of the same permitted
primary game window, which remains essentially black. No new prompt/error
window or UI input was observed. `Launch.log` is empty, and no observed fatal
marker triggered the stop. The final exceptional-window list being empty does
not erase the preceding primary-window observations. No visual, login, wire or
runtime parity is claimed.

**VERIFIED:** After cleanup, all 454 files, including the original configuration,
retain their pre-run sizes and hashes. Private hosts and supervisor hashes also
match. Post-run receipt SHA-256:
`1fa97fad39c1f3557260ca19d02f939d94232e65d3cc6f4b7e148e333a4ed143`.
Raw syscall trace SHA-256:
`3c0da72b08d10349dc9b5bffadc0a7301f33d30a349d04ce60250ee335ea029c`.

REA Evidence:
`ev_6d61fab2857fbacb249ec86643afbc26d383c7a3e6e1e776b4e702845320bb8f`.
Exported bundle SHA-256:
`5efcd6b26b77a851d03b259ad83b28ff80a11b52aac59632f5438dab49f6c0c6`.
REA marks this capture **truncated** because its 1-MiB aggregate filesystem
hash budget omitted the 483,621-byte `prefix-clone.json` digest in both before
and after snapshots. File enumeration, terminal capture and sampled process
records were not marked truncated. This omission is preserved; a later digest
cannot retroactively supply a before-capture digest. The original prefix files
were checked during preparation, while the separate post-run verifier fully
checks the actual 454 game inputs. REA was exported and reset. Ordinary process
sampling and executable image-binding limitations remain.

Aggregate private finding receipt SHA-256:
`1212e0a487d37e847a0483ee138fa8783460711bd7a02340798870478cbedf97`.

Sources: exact owner-install configuration and immutable game inputs,
proprietary/private compatibility study only; original copy/hash, supervisor,
collector and fixture scripts; installed strace/runtime sources already listed
in the preceding observation documents. Configuration values and game bytes
remain private and outside source control/builds. No new external implementation
was copied or dependency installed.
