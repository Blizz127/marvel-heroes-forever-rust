# Attached MovieClip identity and ordinary member bridge

Status: **VERIFIED static instruction/data paths** in the owner's pinned
1.52.0.1700 x64 executable; **OBSERVED** field layouts and branch conditions.
The proposed Rust seam below is **INFERRED implementation design**, not native
execution evidence. No executable was launched and no runtime parity was tested.
This study covers the source MovieClip path selected by factory case 3. It does
not establish text-field/button behavior, arbitrary native properties, or a
complete display runtime.

The attached MovieClip has three distinct identities: the display target, its
AVM owner/interface, and a lazy script-storage object. Reusing an unbound
MovieClip object as the attached receiver loses native member precedence and
event state. The storage happens to use the same native class family as an
unbound MovieClip object; that does not make the owner interface interchangeable
with it.

## Evidence and provenance

Primary source: the private owner install, proprietary, compatibility study
only. No SDK implementation or third-party client code was used. Source file:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`.
SHA256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses are executable virtual addresses, not relocated runtime addresses.

New canonical REA Evidence is private under
`~/.local/share/marvel-heroes-forever-rust/evidence/rea-attached-movieclip/`:

| Record in `complete.bundle.json` | Coverage |
| --- | --- |
| `ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973` | Owner interface/category, normal/raw write routing, lazy storage construction, dynamic initial prototype lookup, builtin strings |
| `ev_f9c7c7b69028559f5f1e9be45c969fe83eb73034694c2636a0c8d749539ab15e` | Storage interfaces/category/initial state, pre-write hook, base target flags, constructor-tail body, event-presence predicate |
| `ev_6f0432d9252c732a9cfd89a9725e32b9a8916b004fa3d09baf96bd9580fdc8ba` | Complete raw storage setter, child lookup and public display value, attachment flag transition, target/name handles, special-key literals |

`verification.json` records complete/exit-zero/untruncated receipts, unchanged
artifact hashes, instruction-byte checks, and the reused records. Ordinary
selected-range discovery files and helper material under
`evidence/rea-attached-target-state/` are navigation aids; the claims above use
the canonical REA records. Exact instruction output and target bytes stay private.

Reused canonical evidence, all under the same private evidence root:

| Bundle / record | Reused fact |
| --- | --- |
| `rea-asflags-followup-version-proof/environment-owner.bundle.json`, `ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d` | Complete attached owner constructor `0x140203560` |
| `rea-asflags-followup-version-proof/final.bundle.json`, `ev_63afb28a4e5a8587d3d55625096a552ed22a947f9c384cac9c1ace7f5dfa051c` and `ev_6c4d08c2c661c182c698f0e1a41179cf6ef863808887629e100ede2e0dd81409` | Concrete source target factory/constructor, AVM offset binding, base constructor chain |
| `rea-event-dispatch/complete.bundle.json`, `ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54` | Complete attached getter `0x1401ddcd0`, including separate switch data |
| `rea-display-lifecycle/complete.bundle.json`, `ev_f135766caf5aa8db946bf5589308cd0908b6b0e8b5a868c1331ee0bd95c22a30` | Attached prototype setter `0x1401898d0` |
| `rea-display-lifecycle-followup/complete.bundle.json`, `ev_0f5f5a57b16a8e9c9af612ed4f9b30e22c53830e17937046b9f3cf5cca948eac` | Complete source attachment caller `0x14020d8d0` |
| `rea-native-alternative/asflags-scope-final.bundle.json`, `ev_820980a2b7e2c4340592ed0dcd933652c281056d7b90c1a3849e64a5d1a0564a` | Complete generic normal setter `0x1401a1020` |
| `rea-script-allocation/instance-read-final-3.bundle.json`, `ev_fe4c8e82d4bc39d82832325fec5e181f513b775fe9fb9748350682ca3fd34384` | Storage prototype hook `0x140190a50`; raw mapping recorded by its producer |

Prior [event dispatch](FRONTEND_EVENT_DISPATCH.md),
[display lifecycle](REA_DISPLAY_LIFECYCLE.md), and
[MovieClip constructor](REA_MOVIECLIP_CONSTRUCTOR.md) studies supply the queue,
event-name, generic-object, and unbound-object contracts. Their scope does not
automatically extend to attachment.

## Concrete identities and initialization

The case-3 factory constructs the source target with primary table
`0x1425069d0`. Its attached owner begins at target `+0x148`. The binding helper
`0x140099640` records the adjusted owner offset in target byte `+0x6d`; this
layout yields `0x52`, and native callers recover the owner using
`target + 4 * target[0x6d]`.

| Object / field | Observed role |
| --- | --- |
| Target `+0x20` | AS2 root owner, separate from MovieImpl |
| Target `+0x70` | Public display/name handle used by script Value tag 7 |
| Target `+0xd8` | Child-list structure used by named-child reads |
| AVM `+0` | Primary table `0x142517670` |
| AVM `+8` | Object interface `0x1425175c0`; category virtual `+0x10` returns **2** |
| AVM `+0x18` | Current display prototype object |
| AVM `+0x20` | Actual target pointer |
| AVM `+0x28` | Saved clip-event map, initially null |
| AVM `+0x38` | Environment; its target field `+0xe0` refers to the same target |
| AVM `+0x120` | Environment string/global context (`Env +0xe8`) |
| AVM `+0x128` | Cached target SWF version (`Env +0xf0`) |
| AVM `+0x1d8` | Lazy script-storage object, initially null |
| Storage `+0x20` | Interface `0x142511d60`; category virtual `+0x10` returns **11** |
| Storage `+0x30` | Linked prototype parent |
| Storage `+0x38` | Own-member table, initially null |
| Storage `+0x58` | Generic watch-related state, initially null |
| Storage `+0x68` | Target-reference token supplied by `0x140031d70` |
| Storage `+0x70`, `+0x72` | Event-related marker fields, initially zero before prototype visitation |

The storage reference at `+0x68` and the public display handle at target `+0x70`
are different fields with different producer paths. Do not collapse them merely
because both eventually refer to a display target. This study does not establish
a complete native handle retirement/rebinding algorithm.

`0x140203560` initializes the Environment, cached version, zeroed per-frame
initializer ledger, and attached interfaces. It clears target `+0x140` bit 5 and
sets target `+0x6a` bit 11. Neither operation creates an ordinary script member.
Category 2 is an internal virtual result here; it is not a proof of a script
`typeof` result.

Initial display prototype selection is dynamic. The constructor calls
`0x14019b570(context, Env, 6)`, and builtin 6 is **MovieClip**. That helper:

1. Obtains the cached/default object for builtin 6 through `0x140194830`.
2. Raw-reads global `MovieClip` from context `+0x60`.
3. If that value converts to an object, raw-reads its `prototype` member.
4. If that read succeeds, replaces the fallback with the converted member,
   including null when conversion yields null. Otherwise it retains the fallback.

The returned object is retained into AVM `+0x18`. Hard-wiring the original
MovieClip prototype is valid only in a bounded profile that verifies the relevant
global binding and prototype have not changed.

The lazy factory `0x1401de450`, also reached through primary virtual `+0xc8`
via thunk `0x1401de510`, returns existing storage or allocates `0x78` bytes and
calls `0x1401da850(context, target)`. That constructor initializes the generic
base through `0x1401ccd80`, installs the category-11 interfaces, obtains the
target-reference token, and assigns the attached owner's current prototype using
`0x140190a50`. This is a bound storage construction path, distinct from the
unbound allocator `0x1401de5e0 -> 0x1401da7d0`.

Prototype assignment through attached interface virtual `+0x68`
(`0x1401898d0`) has an important order: **obtain/create storage first, forward
the new prototype to storage, then replace AVM `+0x18`**. It does not merely
change the owner and wait for the next write to create storage. Storage's
`0x140190a50` sets the linked parent through `0x14018edf0`, then visits a
non-null prototype whose category is not 11. The existing native proof ties that
visitor to mouse-event markers; it is not an onLoad eligibility flag.

## Ordinary reads

Attached normal getter virtual `+0x20` is `0x140203820`; raw getter `+0x58` is
`0x140203840`. Both adjust the receiver from interface to full AVM owner and
call `0x1401ddcd0`. The normal route supplies Env and no separate string context;
the raw route supplies the string context and null Env. They are not arbitrary
generic-object getters.

The complete getter first admits native/special members through a name flag or
underscore gate, a standard-member ID lookup, primary virtual `+0x118`, and a
builtin switch. Those branches include native state and prototype behavior.
This study deliberately does not implement coordinate, root/parent/path, or
other native properties. A bridge must classify such requests as unsupported
until their actual route is implemented; returning an ordinary missing value
would conceal an unimplemented native branch.

After those special branches, the ordinary route at `0x1401de14b` is:

1. If storage exists, retain its parent, temporarily set storage `+0x30` to null,
   and probe storage using the applicable normal/raw getter. Restore the saved
   parent before returning or continuing. This prevents an inherited member
   from winning before a named child. A present own value wins even when its
   value is null/undefined or otherwise false-like.
2. Probe the target's named children through `0x14012f950`. The case-sensitive
   flag is derived from **the attached owner's cached version greater than 6**.
   A returned child must also have target `+0x6a` bit 7. It is converted to a
   display Value by `0x14018ddf0`.
3. Probe AVM `+0x18`, including that object's normal prototype lookup behavior.
4. If still missing, only underscore names enter `0x140194ba0`. A non-underscore
   ordinary name returns false.

For a plain ordinary key such as `gfx`, false is therefore established only
after the own-storage probe, child probe, and display-prototype probe all miss.
Absence from a standalone VM object's own map is insufficient. Generic resolver
or accessor behavior must also be accounted for or rejected explicitly; the
native probes may return property values or resolver behavior rather than a
simple data-map result.

`0x14012f950` rejects an empty key. It first checks the child-list cache at
list `+0x20`; a matching cached child's name returns that entry. Otherwise it
walks the list's 16-byte entries in increasing list-index order, considering
non-null entries with child `+0x6a` bit 8, and caches the first name match.
The older-version branch compares normalized string nodes; the greater-than-6
branch compares exact interned name nodes. The getter subsequently applies
bit 7 to the selected result; it does not continue searching after that failure.
This proves list-index selection on a cache miss, not a global highest-depth or
last-created rule. Duplicate names, cache invalidation, rename/removal, and list
ordering producers need additional evidence before replacing the host's exact
lookup state with a simplified name map.

`0x14018ddf0` uses the child's existing target `+0x70` handle or creates it via
`0x14009e770`, and publishes **Value tag 7** with that handle at Value `+8`.
It does not publish storage as an ordinary Value tag 6. Name retrieval
`0x14009f9f0` uses the same public handle's name field. Full name creation,
path rebinding, removed-target fallback, and allocation failure are outside the
admitted ordinary live-instance profile.

## Ordinary writes and explicit callback boundaries

Attached normal setter virtual `+0x18` is `0x1401aa130`; raw setter `+0x50` is
`0x1401eb4d0`. Both first have a native/special-member route. The normal setter
also handles the builtin `__proto__` before forwarding. For an ordinary key:

- Normal set obtains/creates storage via primary `+0xc8`, then invokes storage
  interface `+0x18` (`0x1401ad600`) with the original Env, key, value, and flags.
- Raw set obtains/creates storage through `0x1401de450`, then invokes storage
  interface `+0x50` (`0x1401eb5e0`) with the original string context and flags.
- Neither route searches children as write destinations. An ordinary data write
  can shadow a same-named child on the following read.

Both concrete storage setters run `0x1401ad440` **before** the generic setter.
The normal route then calls `0x1401a1020`; the raw route calls `0x1401e3b20`.
The pre-write hook uses the bound token at storage `+0x68`; the unbound proof
that this field is null must not be reused for an attached object.

The hook resolves its target token, checks target virtual `+0x218`, and may
classify a newly visible `on...` name through `0x14017f2b0`. The known classifier
covers twelve mouse events; onLoad/onConstruct are not among those entries.
Separately, name flag bit 31 enables three exact extension comparisons:
`rendererString`, `rendererFloat`, and `disableBatching`, with target-side
conversion/setter calls. All three remain unsupported in the ordinary profile.
The hook also performs reference cleanup; it is not universally side-effect
free merely because the final property write is rejected.

The complete generic normal setter establishes the following bounded rules:

- Probe own members using the calling Env's version (>6 selects exact matching).
- If own lookup misses, walk the linked prototype chain for an accessor
  (Value tag 9), skipping inherited non-accessor data values. Consequently an
  inherited data member is shadowed by a new own member; an inherited accessor
  takes a different callback route.
- The selected own member or inherited accessor whose flags contain `0x04`
  returns false before the generic value update/callback.
- For an own or inherited accessor, query the storage primary virtual `+0x38`
  for its target. When present, the setter callback receiver is the recovered
  attached owner's **interface +8**, not storage's interface +0x20. This fixes
  receiver identity but does not by itself implement accessor execution.
- Watch-related state at storage `+0x58` can invoke `0x140199330` and replace
  the assigned value. A no-watch data-write profile must verify this state is
  absent, rather than silently ignore it.
- Existing ordinary data is updated with its existing flags. A new own entry
  reaches the raw virtual setter with the supplied flags. The full raw setter
  creates/replaces the own value and returns true on its normal path.

The raw setter is not a substitute for normal language assignment: it has no
normal-setter read-only/accessor/watch decision sequence. It also has distinct
special handling for `__proto__`, `__resolve`, `_listeners`, and `onEnterFrame`.
Those keys, accessor creation/dispatch, watches, special native keys, and
allocation errors must remain explicit boundaries unless their corresponding
implementation is already supported. The ordinary storage setter may run the
pre-write hook twice on a new entry (normal override, then virtual raw override);
do not invent a single universal hook position after mutation.

## Construct/onLoad admission on this concrete route

The event executor/wrapper details remain in
[FRONTEND_EVENT_DISPATCH.md](FRONTEND_EVENT_DISPATCH.md). This increment binds
the previously raw initial-state fields and their attachment transition.

The base chain `0x14009c270 -> 0x14009e6d0 -> 0x1400a1130 -> 0x1400b4dd0`
initializes target DWORD `+0x6a` to `0x4000` (also zeroing byte `+0x6d`), then
adds masks `0x0100`, `0x0080`, and `0x0200`. Base `0x14009e6d0` writes WORD
`+0x88 = 0x000a`, so bit 3 starts set. Source constructor `0x1400b6d40` adds
`0x0400` to `+0x6a` and clears/initializes WORD `+0x140`, with bit 7 clear.
Its conditional tail `0x1400b3390` has no direct write to these four fields;
its resource helper callees were not expanded as a universal side-effect proof.

Those fresh defaults are **not final attachment state**. In concrete caller
`0x14020d8d0`, the zero branch of raw argument 7 bit 2 calls target virtual
`+0x198` with EDX=0. The source table resolves this to `0x14009ea20`, which
clears target `+0x88` bit 3 at `0x14009eac9`. A nonzero EDX would set it.
The alternate argument-bit branch instead ORs `+0x88` with 1. These are raw
argument/field observations, not an invented public loader flag name.

Later in that same caller:

1. The new target is selected for the load check only when `+0x6a` bit 10 is set.
   This eligibility is captured before the filtered drain and retained across
   its callbacks; the later presence test uses that selected target.
2. With raw argument 7 bit 2 clear, drain the returned construction token first.
   Then query `0x140099890(target, native-event-mask-1)`; set `+0x140` bit 7
   only if that predicate returns true.
3. With that argument bit set, bypass this filtered drain/predicate and set
   `+0x140` bit 7 for the selected non-null target.

`0x140099890` requires a nonzero owner offset and calls recovered owner primary
virtual `+0x38`, concretely `0x140190490`. The predicate first checks the static
clip-event map through `0x140186fe0`. On miss, it maps the event to its builtin
name, then raw-reads **storage if present, otherwise the display prototype**.
Storage retains its parent in this predicate, so inherited properties count.
It does not probe named children, and it tests lookup success without requiring
a callable value. Therefore a stored non-callable onLoad property can set this
presence bit. This predicate differs from the subsequent event's ordinary
attached member lookup.

The set operation does not clear bit 7 when the predicate returns false.
Furthermore the filtered drain can execute callbacks that mutate target state.
Fresh bit 7 being zero and bit 3 having been cleared at an earlier instruction
do not prove that either remains zero after arbitrary callbacks. A runtime must
observe its actual state at dispatch, not recompute a fresh-state shortcut.

The wrapper `0x1401aa850` rejects target `+0x6a` bit 4 for both events. For
onLoad (native mask 1) it first clears `+0x6a` bit 5, then requires either
`+0x88` bit 3 or `+0x140` bit 7. Construct (native mask `0x40000`) does not use
that extra onLoad condition. Admission and event presence do not certify that
bytecode executed or succeeded. Retirement after arbitrary unload/removal,
later bit writers, and exception unwinding remain outside this bounded profile.

## Proposed typed host seam and supported boundary

**INFERRED design:** give each attached source MovieClip an explicit display
instance identity, owner identity, optional storage identity, current prototype,
root context, cached source version, and live raw event-state fields. Keep
public display references distinct from ordinary object references. A VM object
must acquire an attached role only from this host construction path; it must not
be upgraded automatically because its prototype resembles MovieClip.

Useful operations are:

| Operation | Required result / invariant |
| --- | --- |
| Construct attached source instance | Explicit target-owner association, source definition, context/version, initial prototype selection and state; no automatic event execution |
| Resolve ordinary member | Own-storage result, actual child display reference, prototype result, missing, or an explicit unsupported/callback requirement; preserve that order |
| Write ordinary data member | Lazy bound storage, native normal/raw distinction, exact flags and case policy, shadow children instead of replacing them |
| Assign display prototype | Create storage, assign storage parent/hook first, then owner prototype; retain the same attached receiver identity |
| Evaluate load presence | Saved event or raw storage/prototype presence; no child lookup and no callable requirement |
| Admit Construct/onLoad | Evaluate current target fields at dispatch after the established queue transitions |

A useful first bounded implementation can admit live case-3 instances with
version >6, exact ordinary names, supported generic data members, no watches or
resolvers/accessors, supported unchanged builtin prototype selection, and
unambiguous host-owned child lookup. These restrictions are engineering limits,
not claims that retail rejects other cases. A missing unsupported case must
produce an explicit diagnostic before guessed behavior; in particular, duplicate
child-name/cache behavior, special names, accessor receiver execution, target
retirement/rebinding, and native property effects cannot be success stubs.

Hand-derived integration examples, not runtime tests:

- Own `button = undefined` beats a same-named child and prototype method.
- With no own `button`, a supported named child beats a prototype method.
- Assigning ordinary data to `button` creates/updates storage; it does not replace
  the display child. The next ordinary read returns the own value.
- A prototype accessor named `button` is a setter callback boundary, even if a
  child has that name; the write route does not choose a child destination.
- An absent ordinary `gfx` returns missing only after all three read sources
  miss, allowing the caller's separately proven global fallback.
- A non-callable onLoad property counts for load presence, while later event
  delivery still follows the callable/value rules and actual admission flags.
- An attached prototype update creates storage even before a normal data write.

Validation was focused static evidence review and byte agreement only. No Cargo
tests, native execution, renderer smoke, or live startup claim applies to this
documentation-only increment.
