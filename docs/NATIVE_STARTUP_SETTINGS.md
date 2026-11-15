# Native startup hostname fault and post-core settings

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED:** The second isolated retail observation's original minidump records
an unchecked null hostname-lookup result. The client dereferenced address `0x18`
at `0x1405ad9e0`, with `RAX = 0`. The corresponding input buffer contains
`devbox`. This happens inside core initialization, before the inspected caller
constructs its post-core settings object or calls its settings-file reader.
The later `m_clientAppSettingsPostCoreInit` verification is not established as
the original cause. No native process was launched during this postmortem.

**INFERRED:** The isolated environment's lack of a `devbox` entry is a concrete
explanation for this lookup failure. Its private hosts file contained only
localhost entries, its resolver configuration used files, and its private
network had only loopback and no routes. A separately authorized corrected
probe will change only the private hostname mapping. It has not been run as
part of this receipt. No host resolver or networking configuration was changed.

## Three distinct failure observations

The earlier capture report describes the settings verification before a later
Wine error. The recovered minidump adds an earlier exception and corrects that
ordering; these events must not be collapsed into a single settings failure.

| Evidence | Concrete observation | Limit |
| --- | --- | --- |
| Retail minidump, ExceptionStream | Thread 36, exception `0xc0000005`, read address `0x18`, RIP `0x1405ad9e0`, RAX zero | This is the captured exception context, not the different ThreadList context |
| Generated `Launch.log` | Verification of `m_clientAppSettingsPostCoreInit`, `ClientApp.cpp` line 1488 | The log does not prove that missing XML caused the original exception |
| Later Wine stderr | Write address `0x3c`, instruction `0x1405bf949` | No matching later exception context was captured; its exact caller and error-handling role remain unverified |

**VERIFIED, native instructions:** At `0x1405ad9c8`, the client calls WSOCK32
ordinal 57 (`gethostname`). On success it passes the stack buffer at
`RBP + 0xd0` to ordinal 52 (`gethostbyname`) at `0x1405ad9da`. The next
instruction reads `[RAX + 0x18]` with no null-result check. The subsequent
instructions read the first address and call ordinal 11 (`inet_ntoa`).
The imported ordinals were matched to the exact Wine `wsock32.dll` export
forwarders in the isolated prefix, independently of guessed function names.

The dump's RBP is `0x7ffffe5c7990`; the buffer at `0x7ffffe5c7a60` maps to
dump offset 100864 and contains `devbox`. The saved stack descriptor bounds
include it. Microsoft documents a null result on failed `gethostbyname` and
identifies the returned structure's address list. These contracts corroborate
the observed native field access; the native offset itself comes from REA.
[Lookup contract](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-gethostbyname),
[hostent](https://learn.microsoft.com/en-us/windows/win32/api/winsock/ns-winsock-hostent).

The exact Winsock error for this call was not recorded. An earlier
`SE_HOST_NOT_FOUND` log message is relevant context, not a captured
`WSAGetLastError` value at the faulting instruction. The later write at
`0x1405bf949` sets `[RCX + 0x3c]`; its reported fault address implies a null
base, but the minidump does not provide that later instruction's registers.

## Settings lifetime and the line-1488 accessor

**VERIFIED, static ordering:** Client initialization at
`0x141756270..0x1417572d2` calls core initialization `0x1405afe20` at
`0x14175661c`. Its return address `0x141756621` appears in the captured native
stack/log evidence. Farther along the normal path, allocation requests
`0x2a50` bytes and calls constructor `0x14174c970` at `0x1417569a3`.
That constructor passes the literal type name `ClientAppSettingsPostCoreInit`
to its base initialization. The caller stores the resulting pointer at owner
field `+0x48` at `0x141756a91`, calls settings-file loading at
`0x141756ab3 -> 0x1418f0650`, then calls a separate override routine
`0x1418efc60`.

Accessors `0x1417538c0` and `0x141753920` check owner field `+0x48` and invoke
verification with the exact member expression, `ClientApp.cpp`, and numeric
line `0x5d0` (1488) when null. On success the first returns a DWORD from
settings field `+0x3cc`; the second branches using that field. The field's
meaning was not recovered. A matching inline verification exists at
`0x141763432`.

**INFERRED:** The null settings verification is consistent with error handling
consulting settings before interrupted core initialization could finish. The
bounded study does not establish a complete exception-handler call graph or
exclude additional independent startup failures.

## Required file path and format

**VERIFIED, static data flow:** `0x1418ed260` obtains the process executable
module path through `GetModuleHandleW(NULL)` and `GetModuleFileNameW`, extracts
its directory, then appends three parent components, `Data`, and `Configs`.
The caller appends `ClientConfig.xml`. Its literal path expression is:

```text
dirname(process_executable)/../../../Data/Configs/ClientConfig.xml
```

The directory comes from the process module, not its current working directory.
For the supplied `UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` layout,
normal parent-path resolution selects `<install-root>/Data/Configs/ClientConfig.xml`.
The join helper appends separators and components; canonicalization itself
was not observed in that helper.
[Module handle](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew),
[module filename](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulefilenamew).

The loader checks the constructed path at `0x1418f07cf`; its false branch
reports that the file is critical for startup. On the accepted branch it
constructs an XML wrapper, calls file reading at `0x140648610`, then configuration
deserialization at `0x14064d3f0`. Native source literals identify
`Core/Core/Xml/RapidXml.cpp` and `Core/Core/Configuration.cpp`. The file reader
opens a stream, reads its full measured size into a terminated buffer, and
invokes its XML parser. This establishes XML format, not the complete schema.
The root element, required properties, valid values, defaults, and malformed-XML
behavior were not recovered. The caller does not immediately test the XML
reader's Boolean result before calling deserialization.

**OBSERVED constraint:** `ClientConfig.xml` was intentionally excluded from the
frozen capture snapshot. Its contents have never been read, copied, or replaced
by this lane. Reaching this later settings branch may therefore expose a
separate, expected missing-input failure. Producing or retrieving settings is
outside this receipt's scope. No guessed XML, endpoint, credentials, Steam
argument, or generated replacement settings are provided.

## Evidence and validation

The only native target was the owner's pinned `MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Five target-free REA Rizin calls completed with exit zero, untruncated output,
and unchanged source digests. All 3,474 instruction records (3,430 distinct
address/byte pairs, including overlaps) were compared against the original PE.
Code decoding used the separately verified `.text` raw mapping `0x140000c00`;
PE data, string, import and export observations used their proper section maps.
No full automatic analysis, debugger command, target launch, shared setup,
server, account, or configuration mutation occurred. REA was exported and reset.

Private evidence is under `evidence/native-startup-settings/`:

| File | Purpose |
| --- | --- |
| `final.bundle.json` | Five complete REA records; SHA-256 `b7fd7f4a87ef17f0246902a8c2dd12a2b1faf02989d5477fb7322b279abcd97d` |
| `finding-receipt.json` | Target, all byte checks, claim limits, subordinate hashes; SHA-256 `46fd1a758a887f5af62f985e568ab93e1005da825da0d203c45dbe7c0448e7b9` |
| `dump-study/dump-report.json` | Original bounded minidump decode, exception/context/stack bounds |
| `winsock-symbol-and-buffer-evidence.json` | Exact runtime DLL forwarders, dump hostname, private hosts contents |
| `config-path-study/config-path-candidates.json` | Module-relative path, call sites, strings, bounded discovery disassembly |
| `primary-docs/sources.json`, `dump-study/sources.json` | Exact source URLs, snapshots, source commits where published, hashes and licences |

REA IDs: `ev_ad213d693aac3e1a6c8019ab69449546d970f7f29c87166b067e4a93a5f72ef1`,
`ev_935950889ee6257698ccdef8e2b0ec624e3c6fc4acbf31763d4f2cf7e3253f4e`,
`ev_0e6f33a5240057cf706ea545ab35ca34b70ad68eb89e9f324ad7f7f4e7c69c69`,
`ev_2c0de35eebcea646889e45b6a79894f949ffcef35945ea671447d9ceb5b25869`,
`ev_4f857cf12a538b258d1ec3dc7390c88c608d2dce204915370f1c715d1572e2ce`.

The minidump SHA-256 is
`526709a87c314968d01db727fedfb97ae9867a59814deae11d793dfb228dfa32`;
it remains only in the private second observation directory. Its module version
is 1.52.0.1700. The runtime DLL inspected for ordinal names has SHA-256
`f4b794ed984fef7ea61fab375c81a8456e420b878be7fbddbae08a651051b6cc`.
No binary bytes, captures, configuration, or disassembly enter this commit.

## Source ledger

- Owner's pinned retail executable and self-generated isolated runtime dump/logs:
  proprietary game artifact, private interoperability study only; instruction,
  path, type-name and fault evidence. No external game implementation copied.
- Installed Wine 10.0 `wsock32.dll` in the private clone: LGPL-2.1-or-later
  runtime metadata, used only to map the client's imported ordinals to API names.
  Runtime/package notice provenance is retained in `NATIVE_REFERENCE_CAPTURE.md`.
- `~/.agents/skills/reverse-engineer-anything/SKILL.md` and
  `references/runtime-observation.md`: mandatory workflow/evidence guidance,
  installed toolkit 6.9.0; no toolkit implementation copied.
- Microsoft SDK documentation below: CC-BY-4.0, document study only. Exact
  snapshot hashes and published source-commit metadata are retained in the
  private source ledgers. They support the original bounded dump decoder and
  Windows API interpretation, not a copied implementation.

- [MINIDUMP_HEADER (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_header)
- [MINIDUMP_EXCEPTION_STREAM (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_exception_stream)
- [CONTEXT (x86 64-bit) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-context)
- [MINIDUMP_EXCEPTION (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_exception)
- [MINIDUMP_THREAD (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_thread)
- [MINIDUMP_MODULE (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_module)
- [MINIDUMP_MEMORY_DESCRIPTOR (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_memory_descriptor)
- [VS_FIXEDFILEINFO (verrsrc.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo)
- [MINIDUMP_SYSTEM_INFO (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_system_info)
- [MINIDUMP_STREAM_TYPE (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ne-minidumpapiset-minidump_stream_type)
- [MINIDUMP_DIRECTORY (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_directory)
- [MINIDUMP_LOCATION_DESCRIPTOR (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_location_descriptor)
- [MINIDUMP_STRING (minidumpapiset.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_string)
- [MicrosoftDocs/sdk-api LICENSE](https://raw.githubusercontent.com/MicrosoftDocs/sdk-api/docs/LICENSE)
- [Microsoft Learn gethostbyname](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-gethostbyname)
- [Microsoft Learn hostent](https://learn.microsoft.com/en-us/windows/win32/api/winsock/ns-winsock-hostent)
- [Microsoft Learn gethostname](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-gethostname)
- [GetModuleHandleW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
- [GetModuleFileNameW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulefilenamew)
