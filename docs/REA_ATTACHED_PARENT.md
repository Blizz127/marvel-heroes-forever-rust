# Attached MovieClip `_parent` getter

**VERIFIED static native contract:** case-3 `_parent`, standard ID **22**,
selector **0**, reads the actual target's parent pointer at **`+0x38`**.
A null pointer produces a present **Undefined** Value (native tag 0), with
native getter success. It does not produce Null. A nonnull pointer produces
display Value tag 7 for that parent's public handle.

The actual manager root's native parent pointer has not been captured. A
controlled scene fixture's `parent=None` supplies that test state; it does not
prove a retail default. Likewise, the explicitly synthetic name used to reach
this instruction is not evidence of the retail root name.

## Original caller and controlled frontier

**OBSERVED:** frozen `72cd244` completes 452 initializers and reaches original
GetMember **108657**, CodeId 20, after 171 frame instructions. Its sealed
private receipt is
`evidence/avm1-attached-manager-frame-action-increment6/receipt.json`, SHA-256
`bddc8d1f1a2593478db2f89e9008e61f21c67ac3403d92c5ca27bb32904e8d2f`.
The only newly supplied name was the explicitly synthetic current-handle
root name `diagnostic_manager_root` for target 1000.

The owned manager movie SHA-256 is
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
In `MarvelGlobals.GetFullPath`, definition 108618, body `108637+89`, flags
`0x2a`, register 3 holds parameter `object`. Push **108650** selects register
3 and `_parent` (pool index 7); GetMember 108657 performs the read. No prior
register-3 write changes that parameter. The source caller remains the
`MarvelObject` constructor's `this`, as documented in
[REA_ATTACHED_NAME.md](REA_ATTACHED_NAME.md). The root relationship is derived
from that call chain and retained fixture identity, not a native register dump.

## Exact native dispatch

The authenticated name-map row **`0x142adcc00`** points to `_parent` at
`0x14250d350`, ID 22, selector 0. Normal getter `0x1401ddcd0` resolves the
standard name, explicitly supplies raw argument 4 as zero at `0x1401ddd79`,
and calls owner primary `+0x118` at `0x1401ddd86`. The concrete case-3 target
is `0x1401dd870`, which first calls base `0x1401dce90`.

Base admission accepts the concrete `+0x108` mask leaf `0x140067900`, returning
`0xffffffff`. Its full 33-entry switch table at `0x1401dd7e0` contains ID 22's
word at **`0x1401dd838`**, selecting **`0x1401dd1b5`**. The earlier 22-entry
table capture was insufficient for this property; the new complete table
capture closes that exact gap.

The complete selected case `0x1401dd1b5..0x1401dd1d3` loads target
`[owner+0x20]`, then parent `[target+0x38]`. There is no Environment-version,
root-name, `_lockroot`, owner-category or ancestor-walking branch inside this
case. Existing class, live-target, name-map and ownership admission remains
required; this does not extend another display subtype's virtual contract.

## Null parent and nonnull parent

For a null parent, the branch enters **`0x1401dd591`**, calls Value cleanup
`0x1401887a0`, writes **tag 0** at **`0x1401dd596`**, then jumps to native
success `0x1401dcf49`. Derived success `0x1401dd8dd` and normal success
`0x1401dde97` retain that success. Therefore even an Undefined result is
present: ordinary storage, children, prototypes, getters and resolvers do not
replace it after native success.

For a nonnull parent, **`0x1401dd1c9`** calls complete helper
**`0x14018ddf0..0x14018de46`**. It reads the parent's public handle at `+0x70`.
If already present, it publishes native display tag **7**, stores that exact
handle and retains it. It first releases an old output Value when necessary;
an already identical tag-7 output avoids redundant replacement. It does not
construct an ordinary script object, consult a bound-storage owner, or return
the original child receiver.

A missing parent handle invokes **`0x14009e770`**, whose conditional allocation
and generated-name behavior is described in REA_ATTACHED_NAME. Consequently
the general nonnull-parent path is not universally allocation-free. If that
factory supplies no handle, the helper can still publish tag 7 with a null
payload; this is not the null-parent branch and must not be conflated with
Undefined or Null by a bounded model.

## Bounded integration consequence

**INFERRED design:** admit either an explicitly known current null parent, or
a known current parent with a coherent, already-existing live public handle.
Return Undefined for the former and the same typed attached-parent identity
for the latter. Keep unknown parent state distinct from known absence.
No lazy handle synthesis, guessed names, fabricated host callbacks or ordinary
Object substitution is justified by this profile.

Parent replacement, retirement, reparenting and untracked host mutation must
invalidate the witness or update it faithfully. This read-only study does not
implement those operations or establish an initial manager-root parent value.
It proves no native startup, renderer or full `GetFullPath` parity.

## Evidence and provenance

The owned Win64 build `1.52.0.1700` executable SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
New canonical records in `evidence/rea-attached-parent/complete.bundle.json`:

- `ev_92b151bf355a724c28452f3595dc107a118307f0ba7ea4011b9f5a2dc4f25b17`:
  complete base standard-property switch table.
- `ev_07ab263b41cea0282c5b43866789a8552dce985a0fb45b9bcf87e75b87dbb805`:
  complete selected parent getter case.

Retained `rea-attached-tab-enabled` record `ev_d549...` supplies the null
result branch; `rea-attached-movieclip` record `ev_6f043...` supplies complete
display conversion and lazy-handle helpers. The private checker also joins
the normal/base/derived return chain, concrete table slots, name map and Value
cleanup. Exact paths, complete IDs and instruction ranges are in
`evidence/rea-attached-parent/verification.json`, SHA-256
`aaefc8ac566124908b65c97b6aadee31d422e1ba9a19aba9f39d90e509816af5`.

It matched **473 instruction spans, 2,044 table bytes and 8 literal bytes**
to the pinned PE and checked the exact original Push/GetMember and sealed
controlled receipt. Complete bundle SHA-256:
`8bf66e9ab00d485d9c469b8a664054c3ef263069f62ae1ae6abcc11306bdef20`.
Both new REA records were exported and reset; no target remained open and no
native client or callback was executed.

Sources: proprietary owner-installed executable and owned manager movie for
private interoperability; original local attached-object, name-map, name,
focusrect, tab-state and controlled-probe evidence; REA skill/toolkit (MIT),
Rizin (LGPL-3.0-only), and Python standard library (PSF) for original byte and
receipt validation. No external VM implementation was read or copied. Native
bytes, tables, assets and extracted instructions remain private outside git.
