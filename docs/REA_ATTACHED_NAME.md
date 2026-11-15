# Attached MovieClip `_name` getter and name ownership

**VERIFIED bounded static path:** native `_name`, ID **13**, selector **1**,
reads the attached receiver's actual target. When its public display handle
already exists, the getter returns the StringNode at **handle `+0x10`** as a
primitive String Value. It does not return the handle's separate path field,
look for an ordinary `_name` property, or query a child's/prototype's name.

**The current manager root name remains unknown.** The original script's
receiver and the native field read are established independently of the field's
current text. A supplied known name can support a bounded read; an absent name
witness must not become an empty string, a scene label, the movie filename or
an assumed generated ordinal.

Missing handles take a lazy creation path with native allocation and name
generation. Consequently the unrestricted getter is not globally side-effect
free. The first implementation profile should require a coherent existing
target/handle and a known current name. No setter/rename, parent/path API or
native runtime behavior is implemented by this research document.

## Actual original-script frontier

**OBSERVED controlled Rust run:** frozen `5a16ae4` completes 452 initializers
and reaches GetMember **108644**, CodeId 20, after 167 frame instructions.
It stops at the unsupported native `_name` getter. The earlier original
`tabEnabled=false` assignment has succeeded in this run using supplied status
state; that does not supply a name.

The owned manager movie has SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
The exact method is `_global.MarvelGlobals.GetFullPath`, Function2 definition
108618, body `108637+89`, flags `0x2a`, parameter `object` in register 3.
Its first Push 108637 selects register 3 and `_name` from its retained pool,
then GetMember 108644 performs the read. The enclosing action stream is
`100257+22182`, tag ordinal 459.

The static caller audit identifies original CallMethod 60287 in the
`MarvelObject` constructor. It passes that constructor's preloaded `this`
without a register overwrite, after `tabEnabled=false` at 60254 and
`enableInitCallback=false` at 60264. The pending assignment to `this.m_fullPath`
is at 60288. The root-receiver relationship is **INFERRED from the original
apply/super forwarding chain plus the controlled run's retained identity**,
not from an instrumented runtime register dump. No root-name text was
captured or guessed by that diagnostic.

Private receipts:

- `evidence/avm1-attached-manager-frame-action-increment5/receipt.json`, SHA-256
  `262ae7e06c4ae31635592871f7e4db54a0dbb4121185ca907a586bea5aaa646c`.
- Its `static-context/receipt.json`, SHA-256
  `e00aeedf1f394f143862ebdab205913aaf1027132a0a571500b27cd5bc314b97`.

Subsequent source instructions read `_parent` and ancestor names. Those
operations are not established by this first `_name` read and remain separate
native contracts.

## Exact native dispatch and String result

The complete original standard-member table has row `0x142adcb70`, pointing
to `_name` at `0x14250d3bc`, ID 13 and selector 1. The initialized name flags,
lookup map and concrete class constraints are those in
[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md).

Normal getter `0x1401ddcd0` dispatches through owner primary `+0x118`,
concretely `0x1401dd870`, at `0x1401ddd86`. The derived getter calls base
`0x1401dce90` first. The concrete mask leaf at primary `+0x108` returns
`0xffffffff`, admitting ID 13. Base table word **`0x1401dd814`** selects
**`0x1401dd027`**.

The complete selected case `0x1401dd027..0x1401dd056` obtains target
`[owner+0x20]`, calls **`0x14009f9f0`**, and publishes the result through
**`0x140188fb0`**. That output helper explicitly sets Value tag **5**, copies
the StringNode reference to Value `+8` and retains it. The case releases its
temporary reference and returns native success through the common base,
derived and normal getter success exits.

Thus an empty known name still returns a present String. It is neither a miss
nor Undefined. Ordinary storage, children, prototypes, accessors and resolvers
are not consulted after this native success. Captured neighboring getter
cases beginning at `0x1401dd056` provide no `_name` claims.

## Existing handle: the bounded read

Complete helper `0x14009f9f0..0x14009fa46` first loads target `+0x70`, the
public display/name handle already identified in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md). For a nonnull handle,
`0x14009fa13` reads **handle `+0x10`**. The helper retains that StringNode
and returns it through its supplied output.

This existing-handle branch does not change the target, allocate a handle,
generate another name or read a parent. Its work is the field read and String
reference bookkeeping. The receiving Value helper may release an old output
reference according to the established Value lifetime rules; it does not
convert the name through a script callback.

The name comes from the **receiver target**, independently of the calling
Environment or the source movie's filename. Base case 13 has no Environment
version branch. Existing class, name-map, target-liveness and identity gates
still apply; this does not admit another display class's virtual methods or
all older-version name normalization behavior.

The handle's `+0x18` is a separate field used by the optional path-construction
portion of its constructor. It is not the field selected by `_name`. This
study does not assign a complete public semantic role to every handle field.

## Lazy generation and why it supplies no root default

If target `+0x70` is null, `09f9f0` calls **`0x14009e770`**. If it still
receives null, it takes a manager-derived fallback StringNode path; the exact
fallback text and allocation-failure behavior are not established here.

The complete factory `0x14009e770..0x14009e926` has two distinct branches.
Raw target `+0x6a` mask **`0x10`** selects a manager-default-node branch.
That raw bit is not assigned a guessed root/detached semantic name here.
The ordinary live profile under study keeps that bit clear. In this branch:

1. Target primary virtual **`+0x288`** supplies a temporary name.
2. The factory allocates a `0x28`-byte handle and calls `0x14009a2c0` with
   that name, target `+0x38`, and the target itself.
3. It publishes the resulting handle at target `+0x70` and releases the
   temporary String reference.

For the concrete source target class, constructor `0x1400b6d40` installs
primary table `0x1425069d0`. Its `+0x288` slot, **`0x142506c58`**, points to
**`0x14009a1a0`**. This complete method calls **`0x14005ef90`** using the
target's MovieImpl reached through target `+0x20`, then root-owner `+0x10`.

The complete producer `0x14005ef90..0x14005f093` increments a DWORD counter
at MovieImpl **`+0x3b90`**, passes the counter and literal **`instance{0}`**
to formatting helpers, and interns the produced text. The literal is bound to
`0x1424feae0` by new REA output. This proves a counter-based generated-name
producer exists. Exact formatter behavior, counter initialization/reset,
overflow formatting and prior naming order are not established here. No
particular `instanceN` name follows from this evidence.

Complete handle constructor **`0x14009a2c0..0x14009a405`** stores the supplied
name into handle `+0x10` at `0x14009a2d8`. Its later direct writes leave that
field intact. Optional path work reads that name and can replace **`+0x18`**
at `0x14009a3ba`. That optional work includes a parent virtual call; the
constructor's direct field writes do not prove unrestricted transitive
callback behavior. Its first 101-byte unwind fragment was superseded by the
complete 325-byte body, including chained unwind fragments.

None of these producer paths establishes the actual manager root's handle
state, counter value, explicit placement name, or rename history at the
original GetMember. Reading a serialized placement name can be useful evidence
for a host snapshot only when its exact target association and subsequent
name history are also known. A movie export, asset filename, class name or
scene debugging label is not automatically the native `_name`.

## Bounded implementation consequence

**INFERRED design:** a read-only name bridge can admit a known current name
bound to the same live attached target/handle identity, then return that exact
String without materializing ordinary storage. Preserve case and contents.
A bounded initial text encoding/size profile is valid, but unsupported encoding
or size must not silently change the name.

Distinguish these states explicitly:

| Supplied state | Honest getter behavior |
| --- | --- |
| Known current nonempty name, coherent existing handle | Return that String |
| Known current empty name, coherent existing handle | Return empty String |
| Name unknown, even if a scene/asset label exists | Explicit unsupported boundary |
| Missing handle or untracked name mutation/lifecycle | Require separate producer/lifetime proof |

An explicitly synthetic name can exercise a controlled diagnostic if labeled
as such; it is not a native root-name witness. The snapshot must remain valid
through all supported operations. Rename, replacement, retirement or untracked
external mutation must invalidate it or be modeled before subsequent reads.
This document does not authorize a rename implementation or a fabricated
successful host callback.

## Evidence, checks and provenance

The private pinned Win64 build `1.52.0.1700` executable has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
New canonical records are in `evidence/rea-attached-name/complete.bundle.json`:

| Evidence ID | Coverage |
| --- | --- |
| `ev_acf6ac7d62b53fe487bdd4825e28294c7c6d6fe61bb0196fec088eacf6ab31e7` | Selected native getter case |
| `ev_5b3250e8c1d58ef5d59c902e40d0e2cf290c26955bc0ad09d53b19a848c53315` | Concrete name-producer virtual slot and primitive String output helper |
| `ev_c93d909464106e8f05a3ebe826e5a6a32356a2428ad19c0a5daa5bfe4c699b7e` | Complete name-producer wrapper; initial constructor fragment superseded below |
| `ev_4ef153eb829ac68e77d78d7374fe91f2d2e087eade915043bcaf98bb4345b573` | Complete handle constructor and generated-name producer |
| `ev_d565b9fade0136f1173ae2c32a9d444ea53ae43ae844a273aedb944f8ba55d8e` | Exact generated-name template literal |

Retained attached-movieclip `ev_6f0432...` supplies the complete existing-name
getter and lazy factory. Attached-focusrect `ev_e236...` and `ev_cd942...`,
attached-tab-enabled `ev_0cd9...` and event-dispatch `ev_90b1...` bind the base,
derived and normal getter routes. Attached-name-map `ev_06752...` and
`ev_89cb...` bind `_name` and its finite-table identity. Target-factory
`ev_63afb...` binds the concrete source class. Exact paths, full IDs and ranges
are retained in `verification.json`.

The private original checker matched **682 complete instruction spans, 2,008
table bytes and 18 literal bytes** to the pinned PE, checked original method
header/body and Push/GetMember bytes, and verified both sealed runtime/static
receipts. Its SHA-256 is
`15087ff19bbb118805503cfe4cc4c7f69414000b9ccf4730241aee66abe0a99d`.
The complete new bundle SHA-256 is
`6c86aaa5fc1bcea50d1b22bde6dd997d27815232a01ead5c69628a1a0cb7a4ae`.
REA exported/reset all five records with no retained target. No original
executable or native callback was executed.

Sources read: proprietary owner-installed executable and owned manager movie
for private interoperability; original project attached-object, member-name,
focusrect and saved probe/static-context documents and receipts; local REA
skill/native guidance (MIT toolkit); Rizin (LGPL-3.0-only); Python standard
library (PSF) for original PE/span/receipt validation and unwind-boundary
navigation. No outside VM or SDK implementation was read or copied. Raw
native code, tables, assets and extracted source remain private outside git.
