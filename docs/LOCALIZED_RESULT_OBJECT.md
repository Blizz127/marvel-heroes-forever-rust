# Native localization result objects

**VERIFIED static behavior, bounded original-Object profile:** a successful
`LocalizedObjectData` StructProperty conversion creates a new ordinary AS2 object
through the current global `Object` constructor, then assigns the reflected
fields with ordinary AS2 member writes. With the original constructor and
prototype bindings intact, the object has three hidden metadata entries before
the four localization data entries. This is a native allocation/assignment
contract, not a captured client callback or an enabled Rust ExternalInterface.

The original Rust `LocalizedObjectData::into_host_result` prepares exactly these
four fields in saved reflection order. It passes them to the VM's fresh-object
materializer, which retains ordinary setter behavior and its explicit constructor,
value and budget gates. Resource origins remain Rust metadata, not extra AS2
properties. This conversion alone does not bind a native movie handler or execute
the original label-update lifecycle.

All addresses refer to the owner's pinned 1.52.0.1700 Win64 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Recovered instructions, package payloads and detailed receipts remain private.

## Creation and identity

The StructProperty branch of `0x141267200` supplies a null class name, null
argument array and zero argument count to `0x140053890` at `0x1412677ad`. That
helper forwards through `MovieImpl[0x18]`, root virtual slot `+0x148`. The exact
AS2 root table at `0x142511240` binds this slot to `0x1401c6d70`.

Its null-name branch at `0x1401c71b2` obtains the global object and builtin
`Object` key, then calls `0x1401a7c50` at `0x1401c71da` with zero constructor
arguments and first-argument index `-1`. This joins the same named construction
helper already established for InitObject; it does not invoke the InitObject
opcode or use that opcode's reverse literal-pair order.

The existing constructor evidence establishes the following sequence when the
original `Object`, its `prototype`, and the prototype's public `constructor`
binding remain intact as original data properties:

| Initial own entry | Stored native kind | Flags | Meaning |
| --- | --- | --- | --- |
| `__proto__` | Special value, tag `0x0a` | `3` | Created by the parent-link setter; the actual parent is Object.prototype. |
| `__constructor__` | Function reference, tag `8` | `3` | Original constructor retained by the common construction helper. |
| `constructor` | Function reference, tag `8` | `3` | Written by the zero-argument native Object callback. |

Flags `3` hide these entries from ordinary enumeration and prevent deletion.
They still occupy own-property storage. The special `__proto__` entry must retain
the VM's parent-link semantics. The other two entries refer to the same original
Object callable in this profile.

Allocator `0x1401cd330` requests a new generic object and enters constructor
`0x1401ccf40`. The constructor clears its own storage and watch state, links the
Object prototype, and installs the internal object interface at base `+0x20`.
The common construction helper invokes the native constructor on that allocation,
discards the callback's return and retains the allocated receiver. Each successful
conversion therefore has fresh logical object identity; interning string values
does not make result objects shared. Replacement constructors or prototype
bindings require their actual construction semantics and are outside this
bounded result profile.

## The public/native interface joins the AS2 setter

There are two concrete interfaces in this path:

| Interface | Binding and operation |
| --- | --- |
| Public GFxValue interface | Root constructor `0x1401d4490` creates it with table `0x14250f3f0` and stores it at MovieImpl `+0x38`. Slot `+0x28` is `0x1401990d0`. |
| Ordinary AS2 object interface | Generic constructor `0x1401ccf40` installs table `0x1425185c0` at object base `+0x20`. Slot `+0x18` is `0x1401a1020`; raw-write slot `+0x50` is `0x1401e3b20`. |

The public object conversion at `0x1401b4d60` wraps the new ordinary object as a
managed GFxValue of type `0x48`. Its interface pointer is MovieImpl `+0x38` and
its payload is the AS2 object's internal interface pointer. The exact payload
adjustment is established through `0x1401997f0` / `0x140199640`; it is not an
assumed shared vtable layout.

`0x1401990d0` converts the input GFxValue, interns the property name, then calls
that underlying object's ordinary setter at `0x1401991b6`. It supplies a pointer
to a zero flags byte. The missing-entry branch of the ordinary setter forwards
those flags to the raw insertion path. Thus fresh localization data entries have
flags `0`: enumerable, deletable and writable.

This assignment path can invoke a matching inherited accessor setter or an
object watch callback. ReadOnly own entries can reject assignment. The outer
StructProperty conversion does not inspect the setter's boolean result before
advancing to the next reflected property. It cannot be modeled generally as a
callback-free dictionary fill with guaranteed writes.

In the bounded original-Object profile, none of the four localization names is
in the complete original Object.prototype inventory, its parent is null, and
the newly allocated object's watch state is zero. The four writes then create
ordinary data entries without executing a script accessor or watch callback.
This conclusion depends on the actual VM state retaining those conditions.

## Reflected data entries and values

The converter reads the struct's Children pointer at `+0x80`, follows each
field's Next pointer at `+0x60`, and recursively converts each property using
its runtime offset at `+0x8c`. It retrieves each reflected name, then performs
one member assignment before advancing. The source-chain/runtime-loading proof
is grounded by UField serialization at `0x140750bf0` (`Next`), the selected
UStruct serializer prefix at `0x140751330` (`SuperField` and `Children`), and
the ScriptStruct serializer/table binding. The public conversion therefore
follows the reflected linked list; it does not sort by member name or offset.

**OBSERVED saved declaration:** the owned MarvelGame.upk, version 868 / licence
version 3, has LocalizedObjectData at export 5709 (object reference 5710).
Its serialized `Children` chain has the following exact order, `SuperField=0`,
and all four fields have `ArrayDim=1`. Both actual GetLocalizedObjectDataByLabel
return properties reference this scalar struct. This is source metadata, not a
capture of the native heap after linking.

| Order | Field | Native reflected type | GFxValue conversion | AS2 stored value |
| --- | --- | --- | --- | --- |
| 1 | `localizedPrototypeIdStr` | StrProperty | Wide-string tag `7` | String, tag `5` |
| 2 | `localizedText` | StrProperty | Wide-string tag `7` | String, tag `5` |
| 3 | `localizedTooltipText` | StrProperty | Wide-string tag `7` | String, tag `5` |
| 4 | `hasTooltip` | BoolProperty | Boolean tag `2` | Boolean, tag `2` |

**VERIFIED native linking qualification:** the normal Link loop at
`0x14074a9b7` follows `Children` / `Next` in place. The complete function
`0x14074a910..0x14074b923` also contains a conditional merge path. It requires
all three predicates: UObject flags at `+0x10` bit 21, archive `+0x14` nonzero,
and a nonzero third Link argument. That path copies the existing chain in order,
can insert additional candidate fields before their recorded `Next` targets or
append candidates with null `Next`, then rewrites `Next` and `Children`. Existing
entries retain their relative order; it is not an offset sort.

The saved object flags are `0x7000400000000`, without bit 21; saved export flags
are zero. These saved values do not prove that live flags or patch candidates
remain unchanged. The exact four-field contract here therefore applies to the
saved unpatched struct. The archive's concrete object-reference resolver and
global absence of later native field mutations are not established. Property
flags in the UE declaration are distinct from the AS2 member flags `0` above.

The three strings remain strings, including the formatted prototype reference.
The public value converter `0x140196740` sends the nonmanaged wide-string case
through `0x140097450` with length `-1`, then stores an interned AS2 string. The
boolean case copies the boolean byte. No string-to-number or string-to-boolean
coercion is added by this materialization path. The valid, NUL-terminated text
profile from [FRONTEND_LOCALIZATION_RUNTIME.md](FRONTEND_LOCALIZATION_RUNTIME.md)
applies; arbitrary malformed wide strings are not covered here.

Insertion order and hash enumeration order are separate facts. Three hidden
metadata entries precede data writes and affect storage occupancy. Enumeration
must use the separately recovered generic object algorithm rather than assuming
it equals property insertion order.

## Integration boundary

A bounded VM materializer can allocate through the established original Object
construction profile, preserve the three metadata entries and Object.prototype,
then assign the four typed values through ordinary property semantics in the
saved reflected order under that unpatched-struct profile. It should charge the
existing allocation/member budgets and preserve a fresh ObjectId for each
returned object.

Until the VM supports all intervening effects, it must reject changed constructor
or prototype bindings and matching accessors/watchers explicitly. It must not
invent a plain object with no hidden metadata, reuse a cached result object, mark
data fields hidden, or bypass a known script setter. The pure resource resolver's
validation of prototype and locale inputs remains a separate prerequisite.

No native executable, client callback, account, login or server was run for this
investigation. The findings establish selected static paths and their state
requirements; they do not establish retail startup reachability or parity.

## Evidence and sources

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/localized-result-object/`.

The canonical bundle and verification receipt retain commands, Evidence IDs,
input identity, exact candidate spans, prior proof identities and explicit
limitations. Detailed field-order, allocation and value-conversion records remain
private. This documentation contains no recovered implementation code.

| Canonical REA record | Selected proof |
| --- | --- |
| `ev_72c26a1b585275a3b024a88362bb8658c2ed6cffaa9b18db0ae52cc537b9034e` | Actual root creation slot and null-name construction path. |
| `ev_3d6bd4d67cfd5d8cf43066509867e2a19e29a88e958e3df4f34e7af9196bf247` | Public interface, GFxValue conversions, ordinary setter and zero insertion flags. |
| `ev_bd1ecb3816c5f534b130eca67a3c7e888d954af0b2c6d14552b335ef952e92f5` | Payload adjustment and concrete generic AS2 interface table. |
| `ev_cf2350e1a236a88f6656959e8fb7f6ee98d26fb5f02c57db13f9ef361f3be092` | Field/struct serialization, complete Link function and selected table bindings. |
| `ev_a4f369e74c83712f9aee09a8cbf39365a341d39d05a643427399f3fa1a3e4e88` | Field/ScriptStruct constructor identity, names, StrProperty table installation and Link call. |

`native.bundle.json` SHA-256:
`92703ccc07d5975df9cc764d9e7d53399de765d10525e99e9face0e282c74eb1`.
`verify.py` / `verification.json` check all five records as complete, exit 0,
untruncated and input-unchanged, then compare all 11,422 selected instruction/data
bytes directly against the pinned executable (2,738 decoded instructions).
All 53 allocation receipt files and 96 property-order receipt files passed hash
readback. The verification also retains hashes and completion checks for 39
reused canonical constructor/prototype records; it does not claim a new full
byte comparison of those older records.

The pre-existing full StructProperty converter record is
`ev_057b8a514b74f9a7ec59d40ddf66081b01d72d84f701d21c284ab3429a75cb9b`
in `frontend-localization-runtime/aggregate-followup/native/final.bundle.json`.
The constructor/prototype joins are retained in the allocation receipt, including
the existing InitObject, generic Object allocation and prototype inventories.
REA retained no active target; the five new target-free records were exported
and the investigation ledger reset after verification-oriented selection.

| Source read | Rights / licence | Use |
| --- | --- | --- |
| Owner's pinned MarvelHeroesOmega.exe and privately saved MarvelGame.upk | Proprietary owner-held retail data; authorized private interoperability study | Native allocation, reflected field order, interface binding, assignment and value encoding. |
| Existing original project localization, InitObject, Object.prototype and property-flag documentation and private REA receipts | Original project analysis; underlying target material proprietary | Join the new public interface path to already established constructor, metadata and setter behavior. |
| Original private PE mapping / unwind helpers and system Python / objdump tools | Original project helpers; PSF / GNU tooling | Bounded address discovery, chained-unwind ranges, byte and hash verification. No recovered code copied into the client. |
| REA skill and advertised target-free tool schema | Toolkit workflow instructions | Evidence lifecycle and bounded read-only analysis; no shared configuration changes. |
