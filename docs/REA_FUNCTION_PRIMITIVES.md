# Function prototype string and value callbacks

**VERIFIED static native contract:** the original Function prototype installs
distinct `toString` and `valueOf` functions with descriptor flags **1**.
`toString` returns the active context's builtin string `[type Function]` and
does not inspect the receiver or arguments. `valueOf` converts the supplied
receiver interface back into a native Value; it does not copy the original
ActionScript receiver wrapper.

This is a bounded compatibility result for the owner's build `1.52.0.1700`.
No native callback, original script or Rust VM was executed in this study.
Unknown receiver and reference states remain explicit implementation gates.

## Original registration and lookup

[FRESH_GLOBAL_BOOTSTRAP.md](FRESH_GLOBAL_BOOTSTRAP.md) establishes the actual
Function prototype producer and its four-method inventory. Its table binds
`toString` to **`0x14018c7c0`** and `valueOf` to **`0x140196710`**. Both
entries have flags 1 and distinct function identities. The Object prototype's
similarly named callbacks are different; this proof does not implement them.

These functions are among the thirteen early native methods created before
Function prototype cache publication. They have null internal parents and an
own `__proto__` marker, flags 3. Installing their callbacks does not change that
history or make their own missing `call`/`apply` members appear later.

The private `avm1-bootstrap-audit/verification.json` binds the final Function
prototype interface to normal getter `0x1401e8670` and raw getter
`0x1401e7c40`. The normal thunk passes the Environment's StringContext to the
raw getter. These two ordinary method names reach the actual own function
Values; the getter does not invoke their callbacks. No dispatch by method text
alone is justified when a property has been replaced or borrowed.

## Exact toString callback

The complete **74-byte** function is
`0x14018c7c0..0x14018c80a`, matching its PE function extent. It:

1. Reads the Environment pointer from `FnCall+0x30` and result pointer from
   `FnCall+0x08`.
2. Follows `Environment+0xe8`, then the selected context's `+0x28`, then that
   object's `+0x18`, to the root holding the builtin-string array.
3. Cleans the old result when its native tag is at least 5.
4. Writes native **tag 5**, copies the builtin node at root `+0x3d0` into the
   result's `+0x08`, and increments the node's reference count at `+0x18`.

There is no read of `FnCall.this`, argument count, argument array, receiver
category, function name, source code, or source-version byte. There is no
receiver conversion, property lookup or string formatter in this callback.
It requires a valid active Environment and its original builtin context;
the receiver cannot supply that context.

The literal binding is independently joined to retained initialization.
The selected root constructor call at `0x1401d45db..0x1401d45eb` supplies
root `+0x138` to complete helper `0x140181020`. That helper passes the original
pointer table and count `0x9c` to `0x140097f70`. The complete binder preserves
table order while interning each supplied NUL-terminated string and storing its
node in the corresponding destination slot. Thus root `+0x3d0` is index **83**.
The original index-83 pointer and its literal bytes were checked against the
owned PE: **15 ASCII characters, `[type Function]`, followed by NUL**.
The return string excludes the terminator.

The ordinary initialized, unchanged builtin array is a condition of this join.
The callback returns that context's existing node; it does not allocate a new
native string on every call. A Rust implementation may charge its existing
string budget for its own representation, without claiming native allocation
equivalence. Native old-result cleanup and reference accounting remain real
effects. A fresh Undefined result takes the no-cleanup branch; arbitrary old
managed results require the established bounded cleanup/collector contract.

**Implementation boundary:** admit this callback only with an explicitly bound
active builtin context. Once the callback is reached, otherwise-valid receivers
and arguments are ignored; a function-only receiver or zero-argument rule would
be a conservative extra restriction, not a native test. A direct API call with
no active context must not infer one from the function or receiver. No host call
or user coercion is required by the supported branch.

## Exact valueOf callback and conversion

The complete **33-byte** leaf is
`0x140196710..0x140196731`. It has no PE unwind entry; both exits are tail jumps
to `0x1401936f0`, followed by padding outside the selected body. It reads only
`FnCall.this` at `+0x10` and the result pointer at `+0x08`:

- A nonnull interface pointer is adjusted by `-0x20` to its object base and
  supplied to `0x1401936f0`.
- A null interface supplies a null object pointer to the same converter.

Neither this leaf nor the complete converter reads argc, arguments or the
Environment. The leaf does not possess the original receiver Value's auxiliary
frame or reference flags. Generic primitive-to-interface mapping at the caller
is therefore not established by this callback.

Complete **`0x1401936f0..0x1401937e6`** has two paths. For a nonnull object it
queries the actual interface category through slot `+0x10`. Only category
**`0x17`** takes the function-reference path; every other category and a null
object take the tag-6 path. There are no additional category-2/5 tests here.

### Proved ordinary native and ScriptFunction receivers

The ordinary native interface **`0x142514430`** and actual ScriptFunction
interface **`0x142513090`** both bind category getter `0x1401e1420`, returning
`0x17`, and FunctionRef getter **`0x14018c580`** at slot `+0x60`.
[REA_SCRIPT_FUNCTION_CONVERSION.md](REA_SCRIPT_FUNCTION_CONVERSION.md) binds
the latter interface to both actual DefineFunction producers.

The complete FunctionRef getter returns the same object and explicitly writes
**null auxiliary frame and zero reference flags**. It retains the object but
does not change its code, captured context or scope storage. The converter
passes this new reference to `0x14018de50` and cleans the temporary reference
afterward, including its documented reference/collector bookkeeping.

For a **fresh Undefined result**, the setter produces native **tag 8**, the
same function object, null auxiliary frame and zero flags. This is not a copy
of a receiver wrapper with a captured activation. The established ordinary
CallFunction path initializes its result to tag 0 at `0x14020a78e`; a Rust
return-value API likewise needs an explicit fresh-result model for this rule.

The named **CallMethod** producer is also joined, using its retained complete
case `0x14020b6ef..0x14020bfbb`. At **`0x14020b757`** it writes tag 0 to
the separate result at `[rbp+0xd8]`. Ordinary receiver branches place that
address in `FnCall+0x08` at `0x14020b999/0x14020b9a7` before invocation at
`0x14020b9d5`; another inline branch does the same at
`0x14020bb19/0x14020bb27` before `0x14020bb61`. The named function-receiver
branch passes the identical result address in RDX at `0x14020bc50` to
`0x14019aba0` at `0x14020bc6a`. That complete constructor stores RDX into
`FnCall+0x08` at `0x14019abb3`, preserves the slot, and supplies the resulting
FnCall to invocation at `0x14020bc8e`.

The concrete ordinary native, ScriptFunction, OP and FP interfaces all bind
their `+0x78` predicate to the complete false leaf `0x1405b93b0`. Therefore
the native invocation wrapper `0x14018c5b0` selects its ordinary path at
`0x14018c6c0/0x14018c6c3`, forwarding the same FnCall without changing the
result. This closes the fresh destination for the supported original named
method route, including a function receiver, without relying on synthetic-test
setup. It retains ordinary direct method lookup and the existing no-reentry
conditions; it is not a proof of arbitrary getter, resolver or caller coercion.

There is a material exception: `0x14018de50` first checks whether the existing
destination is already tag 8 with the same function pointer. If so, it returns
without replacing that destination's auxiliary frame or flags. Do not describe
the callback as unconditionally clearing every destination reference.

**Implementation boundary:** the known ordinary native callbacks, Object and
Function callables, and the early-method functions may preserve object identity
under this fresh-result profile. An ordinary ScriptFunction additionally needs
the existing conservative target/global-only scope guard until the VM can
represent a new wrapper with a dropped auxiliary frame independently of the
original wrapper. That guard is stricter than native behavior. Category `0x17`
alone does not prove that another specialized function class uses the same
virtual FunctionRef getter. Such constructor/method kinds need their concrete
slot witness before being added to this profile.

### Proved Object and Function prototype receivers

The fresh producer and exact category slots establish category **6** for the
cached Object prototype and Function prototype. They take the converter's
fallback at `0x1401937aa`: after any necessary old-result cleanup, it writes
**tag 6 with the identical object base** and retains that object. No member
lookup or virtual FunctionRef getter occurs on this path.

This supports returning the exact cached OP/FP identity when those prototypes
are explicitly supplied as receivers. It does not make Function.prototype
callable. A bounded implementation can admit those two identities without
generalizing to every ordinary or display object.

### Null and unsupported receiver representations

A null `FnCall.this` takes the same fallback and writes **tag 6 with a null
pointer**. It writes neither native tag-1 Null nor tag-0 Undefined. The current
Rust Value representation cannot express that native state as an ordinary
ObjectId, so mapping it to `Value::Null` or `Value::Undefined` would be invented
behavior. Keep the null-interface case gated until it is represented and its
caller mapping is established.

The generic fallback also must not be mistaken for preservation of public
display tag 7. Primitive, deferred, display, super and other specialized
receiver paths remain outside the initial profile. Input object memory is not
rewritten to remove a captured frame; result construction and temporary
ownership changes are separate from the original receiver wrapper.

## Effects and validation limits

The selected callbacks do not execute function bodies, read receiver properties,
dispatch a host operation, resolve a deferred builtin or invoke a user coercion.
They do update result and reference state. Managed-result cleanup and the
function conversion's temporary cleanup can reach the previously studied
collector helpers; positive counts alone are not a blanket no-work guarantee.
The supported profile retains valid nonoverflowing ownership, ordinary
initialized string/collector state and absence of destructive collection or
reentry. Arbitrary cleanup callbacks and allocation failure are not proved
harmless here.

The private verifier independently matches **955 instruction spans / 3,762
bytes**, **392 retained REA data bytes**, **16 previously recorded category
pointer bytes**, and **32 concrete predicate-slot bytes** against the pinned
executable. The predicate slots are explicit independent PE reads joined to
retained producer identities and the retained false leaf. The selected root callsite is
not presented as a new complete root-initialization proof. A binder extent
check corrected its two-byte return boundary before the successful final check.
Padding and the neighboring partial body in the initial valueOf capture are
excluded from semantic coverage.

Two new target-free REA calls completed with exit zero, untruncated output and
unchanged target hash. They supplied the two callback bodies and the selected
literal/pointer read. The complete bundle was exported, then saved identically
while resetting the investigation; no target remains open. There were no Rust
tests, native invocation, original replay or product changes in this increment.

## Evidence and provenance

Private evidence root: `evidence/rea-function-primitives/` under the project
data directory. `verification.json`, `finding-summary.json`, `complete.bundle.json`
and `reset.bundle.json` bind source hashes, selected ranges and limits.

- New callback record:
  `ev_50ca51101f9509c4aba02f2513c2c6157b1c8b61a9f505c04f0736a2ea531874`.
- New selected builtin-pointer/literal record:
  `ev_4901f073442fa067ed54afa44a03873fb906d42eb13ec1d8635aa8fe03c23044`.
- `fresh-global-bootstrap/prototype-producers/` supplies actual registration,
  flags, producer identities and OP/FP category slots.
- `avm1-bootstrap-audit/verification.json` supplies concrete prototype getter
  dispatch and the early-method parent distinction.
- `rea-function-calls/function-binding.bundle.json`, `function-context.bundle.json`,
  `rea-script-function-conversion/getter.bundle.json`, and the latter study's
  producer/validation receipts supply the complete converter, FunctionRef getter,
  result setter and actual ScriptFunction binding.
- `rea-movieclip-binding/final.bundle.json`, records `ev_c30fe...` and
  `ev_4d8736...`, supplies complete CallMethod and FnCall construction;
  `rea-external-interface/final.bundle.json`, `ev_0c83...`, supplies the complete
  native invocation wrapper. Exact record IDs and hashes are in the receipt.
- `rea-native-alternative/extends-name-binding-final.bundle.json` supplies the
  selected root-array setup and complete builtin binder. The private receipt
  identifies every reused record and bundle digest.

| Source | Licence / status | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-held build; private interoperability study | Exact callback bodies, literal, tables and independent PE checks |
| Cited project documents and private REA records | Original project research over the owned artifact | Reuse registration, conversion, initialization and cleanup evidence |
| Installed Reverse Engineer Anything skill and native/evidence guidance | REA MIT; Rizin LGPL-3.0-only tooling | Bounded static reads, complete export and reset |
| Python standard library | PSF tooling | Original selected-span and hash verification |

No external VM implementation was used. Native bytes, full tables and extracted
artifacts remain private; this commit contains only the compatibility findings.
