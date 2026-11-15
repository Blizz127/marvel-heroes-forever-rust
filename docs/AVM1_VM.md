# Bounded AVM1 execution increment

`avm1::vm` executes an explicitly limited subset of the original decoded actions.
It is an execution core for compatibility work, not a complete AVM1 runtime or a
claim of working frontend startup. No third-party VM implementation was used.

## Public API and ownership

`Vm::new(movie_version, Limits)` creates a private heap and global/root objects.
The current coercion profile accepts versions 7–19. `register(Arc<Program>)`
returns a stable `CodeId`. Functions retain that ID and their defining source
offset, so additional movies can register after execution starts. Code is shared
immutably; function definitions do not deep-copy code or borrow a caller's stack.

`bind_loaded_movie_target(target, header_version)` records an integration-validated
successfully loaded movie target. It rejects invalid IDs and a version differing
from the VM's profile. The integration must also validate every registered movie's
header; registering raw Program bytes cannot establish that relationship. Failed
loads, empty definitions and arbitrary objects must not be bound. Function bodies
retain their captured movie target separately from `this`, and nested invocation
restores the caller target. This context permits the verified loaded-header
property-flag default below; unbound or native/direct-call contexts retain the
outcome-equality gate. An arbitrary receiver does not confer loaded-target status.

`execute(code, target, host)` runs an action stream. `call(callable, this, args,
host)` invokes an already created function. `Execution` reports the return value,
remaining root stack, instruction count and host-call count. `get`, `set`,
`object`, `native`, `global` and `root` support explicit integration/setup.

Errors include a VM-local CodeId/source-offset origin, opcode and a typed reason. An error
stops execution immediately; earlier writes remain visible. Execution is not
transactional, and retrying a stream is not an automatic recovery strategy.
`instruction_count()` remains available after failure.

`Value` distinguishes undefined, null, Boolean, double, UTF-16 string and object
identity. Objects have data/accessor properties and an explicit prototype. Script functions
and native bindings are object kinds. Arrays expose indexed properties and length.
Function bodies capture lexical object scopes and the current constant pool.
Function2 parameter registers, supported preloads, local variables, arguments and
return values are represented directly. With blocks add a temporary scope.

## Host boundary

A native function must first be created with `native(binding_id)` and installed
on an object. `Host::call` receives that binding, receiver, arguments and source
offset. It can return a value or an error; it receives no implicit operating
system, networking or game-client access. Returned object IDs are checked.

Timeline controls, trace and URL instructions become typed `HostCommand` intents.
`Host::command` must acknowledge or reject each intent. `RejectHost` rejects all
native calls and commands. No default success stubs exist, and the executor
contains no filesystem, network, browser, process or native-client execution.
An integrating host is responsible for the behavior of bindings it explicitly
installs. A recording host alone is not a display-list implementation.

## Execution limits and current gaps

Defaults cap each invocation at 250,000 instructions, 64 nested function/With
executions, 16,384 stack values, 10,000 host calls and 4,096 arguments. Persistent
caps are 100,000 heap objects, one million cumulative properties, eight million
allocated string units, 20,000 registered programs and 64 MiB of source spans.
Arrays are capped at one million elements, prototype traversal at 256 links, and
property visitation and native hash-reconstruction work at one million units per invocation. Flag updates reserve
work for both preflight and mutation before touching any member.
Configured function depth greater than 128 is rejected. Pool references,
registers, branches, object IDs and stack accesses are checked. Loops consume the
same instruction budget as straight-line code.

This increment has no garbage collector. Activation/argument objects remain in
the bounded arena, so sustained gameplay will eventually require reclamation.
Host-created Rust values are supplied by trusted integration code; the VM's
allocation counters cover its own allocations and installed property names.

Unsupported operations return errors, including Try/Throw, unrestricted
enumeration/property flags (the bounded supported profile is below), movieclip
paths, general super-member dispatch and parent preloads, automatic
primitive boxing, and uninstalled standard-library/native functions. Missing
ordinary properties correctly yield undefined; attempting to call that value
stops with `UnboundCall`. The optional `install_object_constructor()` installs the documented Object
constructor, prototype, documented `Object.prototype.addProperty`, the
`Object.registerClass` host seam, and a shared Function-prototype intermediate
with the supported call/apply methods. Primitive boxing remains unsupported.
The separate `install_function_conversion()` enables the native-backed limited
Function conversion below. `install_movie_clip_constructor()` enables the separate
unbound MovieClip script-object profile below. Scene attachment, Stage, Mouse and
other Scaleform interfaces still require explicit integration.

Primitive conversion is deliberately narrower than the eventual compatibility
profile. It supports modern Boolean behavior, ordinary decimal numeric strings,
checked primitive equality, numeric operations, UTF-16 string concatenation and
comparison, and modulo-based Int32/Uint32 operations. Legacy arithmetic currently
requires numeric operands. Unknown empty/whitespace/non-ASCII/hexadecimal numeric
string cases, null-to-number, object primitive conversion, NaN relational edges,
and general non-integral/large number formatting stop explicitly. They must be
measured against the owner’s runtime before broadening this profile. Strings and
floating-point values are not silently flattened into Rust debug formatting.

## Owned-script execution evidence

**OBSERVED:** The private harness executes original decoded instructions from
the owner's movies with explicitly supplied object state and `RejectHost`:

| Probe | Source span | Controlled result | Executed instructions |
| --- | --- | --- | ---: |
| Manager `UpdateStageScale` | Definition 6197+106; body 6249+54 | Original cursor 128×64, scales 0.5×2 → width 64, height 128 | 18 |
| Login width getter, missing child | Definition 64967+56 | Returns 0 | 6 |
| Login width getter, present child | Same | Child width 320 → returns 320 | 8 |

Both functions execute without host calls. The manager setup uses its contiguous
original constant-pool/function prefix at 5967+336. The login probe selects its
original enclosing constant-pool record and the original anonymous definition;
it does not claim that the containing class initialization ran. Instructions and
function bodies retain their source coordinates. The expected results follow
from the supplied data and decoded operations; no native-retail comparison has
been performed.

**OBSERVED:** Running the full manager root action with that minimal environment
stops at offset 6314, GetMember, after six instructions because a required namespace
object is absent. At core commit `c645c4e`, a separate fresh-empty-VM run of each of its 468 action tags
completes none: 251 stop at Extends, 181 at missing/primitive object receivers,
19 at unbound calls, 15 at rejected Stop commands, and two at qualified paths.
That inventory is explicitly not normal startup ordering or wire/behavior parity.

The manager's `setDocumentClass` is script-defined; treating it as a successful
native stub would hide required behavior. Its observed call chain reaches
prototype manipulation, Function/apply, and subsequent Mouse handling. A full
initialization still requires class bootstrap, imported movies, standard objects,
actual host bindings, event dispatch and a display-list runtime.

Private evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/avm1-runtime/`:

- `startup-probes.json`: input hashes and isolated probe selection.
- `run-probes.rs`, `probe-results.tsv`, `execution-evidence.json`: reproducible
  original Rust execution harness, actual results, hashes and limitations.
- `vm-tests.txt`: focused synthetic execution checks.
- `*-private.txt`: private source-coordinate disassembly and dependency evidence.

**VERIFIED:** Twenty-five focused VM tests pass for operand ordering, pool capture,
function calls, later code registration, property/prototype behavior, explicit
native bindings, unsupported paths, shadowing, and instruction/call/stack/heap
budgets. These tests supplement the owned probes; they do not prove retail parity.

## Ordered initialization and inheritance increment

`Extends` now creates a subclass prototype linked to the superclass prototype,
sets its `__constructor__` metadata, and installs it without invoking the
superclass. This has a focused test using a rejecting native superclass binding;
zero native calls occur. Adobe v19 text extraction loses underscores in this
section, so the authored v9 PDF was also checked. Setting plain `constructor`
instead would be an incorrect interpretation of that extraction.

**OBSERVED:** The manager contains 452 unique DoInitAction sprite IDs, all in root
frame zero. Adobe specifies an initialization phase in serialized order before
normal frame actions and implicit sprite creation. The ordinary root action at
5967 appears earlier in the file than the first initializer at 7239; executing
all tags as encountered would therefore use the wrong phase order.

`run-init-order.rs` registers the 452 programs and preserves one VM. It installs
the standard Object constructor/accessor method and an explicit MovieClip native
constructor binding with a linked prototype. RejectHost rejects any attempt to
invoke that native constructor; no unrelated namespace is precreated. The
original initializer itself constructs `_global.gfx`, `gfx.core.UIComponent` and
its prototype. The earlier Object-only intrinsic profile executes 218 instructions before
stopping at the unbound `ASSetPropFlags` call at 12374. Zero initializers complete,
and subsequent initializers are not executed after that failure. That baseline is
in `init-order-after-pop.tsv`; `init-order-results.tsv` records the earlier strict
stack-underflow investigation.
The first initializer reaches UIComponent extending MovieClip at 8705 and six
prototype `addProperty` calls starting at 12178 before requiring `ASSetPropFlags`
at 12374.
The second initializer creates FocusHandler and invokes its singleton constructor,
which calls `Selection.addListener` at 13552. These are specific dependencies,
not authorization to manufacture all missing namespaces or stub their effects.
`Object.registerClass` must maintain a name-to-constructor registry for later
instances; returning true without registering a class would be incorrect.

`init-order-results.tsv`, `run-init-order.rs`, `init-order-private.json` and
`inheritance-evidence.json` retain the private ordering, dependency and execution
measurements. The earlier isolated-inventory counts describe the core baseline;
they do not claim that adding Extends completes those initializers.

## Executable accessor increment

`Object.prototype.addProperty` installs a getter/setter pair lazily and replaces
an existing own property. Invalid names or callable arguments return false
without creating a property. A read invokes the getter with zero arguments; a
write invokes the setter with the assigned value and ignores its return. An
inherited accessor receives the original instance as `this`. A null setter makes
writes harmless, as Adobe explicitly specifies. No getter runs during registration.

GetMember/SetMember and variable reads/assignments use executable access when a
descriptor is present. Getter/setter functions consume the existing instruction,
call and host budgets. `get`/`set` remain data-only integration helpers and return
an explicit error on accessors. Integrators use `read_property`/`write_property`
with a Host to execute an accessor and receive an Execution report.

**OBSERVED:** `run-owned-accessor.rs` installs the original isolated login getter
at 64967 as an accessor on a controlled prototype. Registration executes zero
script instructions. Reads from a derived instance execute the original six/eight
instructions and return 0/320 for absent/present child state. Writing the null-setter
property preserves the child's width at 320. All operations use RejectHost and
produce zero host calls. This tests original getter execution with controlled
registration, not successful retail class initialization.

`owned-accessor-results.tsv`, `accessor-evidence.json` and `vm-tests-accessor.txt`
retain that evidence and eleven passing focused tests. The synthetic accessor test
also verifies a real script setter mutates the instance, not the prototype, and
invalid registration leaves no property behind. That increment still stopped at
the unresolved property-flags binding; the later flag implementation and its exact
remaining default-clear stop are described below.

## Explicit function method values

`function_method(FunctionMethod::Call/Apply)` creates standard callable method
values for an explicitly installed function prototype. It does not invent a
global Function constructor or assume a Function-cast contract. Both methods
support an explicit object receiver; Call forwards trailing arguments, and Apply
reads a supplied array in index order or accepts omitted/null/undefined argument
lists as zero arguments. The latter cases are grounded by the later native study
below. Null/primitive receivers and other primitive argument lists remain explicit
unsupported profiles; timeline fallback and boxing are not guessed. Full document
class setup still depends on typed allocation and native behavior beyond this
method adapter.
All invocations, including recursive builtin Call/Apply chains, consume nesting
budget. A cyclic Apply argument array reaches a bounded error rather than Rust
stack exhaustion. Focused tests and the owned adapter probes below validate this implementation;
no native runtime comparison has been performed.

## REA-backed zero-depth ActionPop

**VERIFIED by static owned-binary REA evidence:** The native raw-opcode dispatcher
maps 0x17 to handler 0x140209749. Its boundary helper 0x140185840 reaches
0x1401858e3 for the last stack page. That branch restores the decremented top
pointer, resets the sentinel tag to zero and returns normally. The same
dispatcher's depth calculation establishes that one page with top at page base
has logical depth zero. The original manager's empty Pop at 8511 is therefore
accepted by the VM, preserving an empty stack. This change applies only to
ActionPop; other underflows remain explicit errors.

Private evidence:
`evidence/rea-native-alternative/avm1-pop-boundary.bundle.json`, REA evidence
`ev_22b3e9316258dea0ee949de2291bbf916ba35f5605696c3d60877a53959d8dcd`, plus dispatch
mapping evidence `ev_9ad09fdbeaf5ea2d7e1863c3061aebe83f0432cd76e1d6119d12299f6dbb4853`.
`native-pop-execution-evidence.json` records the resulting 218-instruction ordered
probe and thirteen passing focused tests. Static REA evidence and original-code
execution are distinct from running the native client; no native comparison has
been claimed.

The subsequent native-backed property-flag increment below replaces the missing
intrinsic with a bounded implementation. Its remaining default-clear ambiguity
is reported explicitly instead of being hidden behind a successful callback.

## Native-backed property flags

`install_property_flags()` explicitly installs the bounded `ASSetPropFlags`
intrinsic. It accepts a generic object with a null or supported exact own-name
selector and integer numeric masks; three or four arguments are supported.
The narrow unbound MovieClip prototype selected-name path is described below.
Other selectors and native object classes,
and unverified numeric coercions stop explicitly. Null selects stored own members,
including already hidden members. It never visits a prototype, synthesizes absent
virtual builtins, resolves a getter or invokes a native host. Successful calls
return Undefined.

**VERIFIED, static REA:** Both masks are limited to their low three bits. The new
flag byte is `(old & ~clear) | set`; setting wins overlap and upper existing bits
are preserved. Bit 1 prevents enumeration, bit 2 prevents own-member deletion,
and bit 4 prevents ordinary own-data assignment. Existing flags survive ordinary
value replacement, including prior strings, functions and objects. Extends creates
stored `__proto__` and `__constructor__` entries with flags 3. These entries are
therefore included by the null selector even though they are hidden by default.
No symbolic behavior is claimed for upper flag bits.

**VERIFIED, static REA:** For the traced successfully loaded target class, the
cached environment byte is the parsed movie-header version. The parser reads the
fourth validated FWS/CWS/GFX/CFX header byte; the loader copies it into the
DataDef state; concrete definition/target virtual getters supply the cache. The
native three-argument call uses clear 7 for version 5 and clear 0 otherwise. The
VM's accepted profile is 7–19, so an explicitly bound loaded activation target
uses clear 0. No version-5 execution support is claimed.

This rule follows the captured script target, independently of its receiver.
Nested accessor/function calls restore the caller context. Raw Program registration
alone supplies no target provenance. Empty definitions can still contain a -1
version, and failed/import/native contexts outside the traced loaded path are not
promoted to loaded status. For an unbound target or a direct intrinsic call with no
script activation, the VM compares both possible defaults for every selected
member before any mutation. Outcome-independent calls run; otherwise
`AmbiguousPropertyFlags { old, set }` stops at the original callsite. Explicit
four-argument calls bypass this ambiguity. The failing flag operation is atomic;
earlier actions in the stream still remain visible.

`own_property_flags()` and `own_enumerable_names()` expose stored state for focused
integration checks without executing getters. The latter uses lexical inspection
order, not native runtime hash order. ActionEnumerate/Enumerate2 implement verified
hidden-member filtering and prototype traversal only where zero or one visible
name remains. Multiple names and visible duplicate/shadow cases stop until native
ordering is recovered. Own generic-member Delete returns false for absent or
protected members; deleting unprotected prototype metadata remains unsupported.
ReadOnly ordinary own-data writes leave the value unchanged, as the native
SetMember path discards its false result. Inherited ReadOnly and flagged-accessor
write ordering, prototype writes after its metadata flags change, and addProperty
replacement of flagged members remain explicit gaps.

**OBSERVED baseline before loaded-target binding:** With the intrinsic installed,
the unmodified first manager initializer reaches source 12374 after 218
instructions, stopping at
`AmbiguousPropertyFlags { old: 3, set: 1 }`. Both metadata entries remain flags 3;
ordinary entries remain untouched. This is more precise execution evidence, not a
completed initializer or a claim of native parity. No later initializer runs.
`run-init-order-property-flags.rs`, `init-order-property-flags.tsv`,
`vm-tests-property-flags.txt` and `property-flags-evidence.json` preserve the focused
checks and controlled owned-script result privately.

An additional owned-bytecode probe selects the original manager constant pool at
55506+118 and unmodified property-flag suffix at 55787+37, including the ASSet call
at 55821. The 11-instruction probe uses the actual owned GFX version 15. Its explicitly
supplied generic target has one data property initially
at flags 0. The call changes that property to flags 1, preserves its value and
finishes with an empty action stack and zero host calls. Both native defaults give
the same result for that controlled state. The constructor and containing class
initializer are not run, and this does not bypass the first initializer's real
metadata ambiguity. `run-owned-property-flags.rs` and
`owned-property-flags-results.tsv` retain that distinct probe.

Native evidence is retained under the private evidence root:

- `rea-native-alternative/asflags-scope-proof-receipt.json` and
  `asflags-scope-final.bundle.json`: generic-object binding, own-member visitor and
  Enum/Delete/SetMember flag consumers.
- `rea-native-alternative/extends-name-binding.bundle.json`: exact metadata names,
  flags 3 baselines, and ordinary assignment/cleanup preserving flag bytes.
- `rea-asflags-native/finding-summary.json`: masks, transformation and native
  default-clear branch.
- `rea-asflags-followup-return/finding-summary.json` and `final.bundle.json`:
  native registration, wrapper, caller-initialized Undefined and untouched return
  slot, bridged to raw Push type 3.
- `rea-asflags-followup-version/finding-summary.json`: original partial cache
  investigation, superseded for the concrete loaded target by the following chain.
- `rea-asflags-followup-version-proof/finding-summary.json`: concrete target
  factory, constructors, definition getter and cache association.
- `rea-movie-header/finding-summary.json`: validated header parsing and loader
  descriptor forwarding.
- `rea-native-alternative/movie-version-proof-receipt.json` and
  `movie-version-final.bundle.json`: definition/state getter
  chain and final descriptor version copy, including evidence
  `ev_d37bdccbdab5682db5e6270ea46dc77ec4ccc20dda3863619e3b3b6a1940fbf8`.

## Loaded startup and Function inheritance

**OBSERVED:** Binding the actual loaded manager root (GFX version 15) makes the
first initializer complete all 220 instructions. With the real bounded Selection
listener host, the next initializer registers one listener and initially stops at
16355: the constructor function cannot find inherited `addProperty`. This is a
separate prototype-chain gap, not a missing native success callback.

The [Adobe Function reference](https://open-flash.github.io/mirrors/as2-language-reference/Function.html)
identifies Function's Object inheritance and inherited `addProperty`. The VM now
creates one shared Function-prototype intermediate linked to Object.prototype;
script and builtin function values link through it. Existing call/apply method
values are installed there. The function object's separate `prototype` member
continues to describe constructed instances. This foundation alone does not
install global Function; its limited native conversion is installed separately.
String-based function construction is unsupported. Invocation, typeof and the
constructor descriptor of the intermediate prototype itself remain explicit gaps
pending their complete native contracts.
Object construction must be installed before script function definitions; late
installation is rejected rather than silently retrofitting script-modified state.

**OBSERVED:** The same unmodified ordered manager probe now completes seven
initializers, retaining one coherent global/root/heap state. FocusHandler's
instance accessor returns the exact listener object retained by SelectionHost;
reading it does not register another listener. This read also exposed object/null
Equals2 at13730: ECMA abstract equality returns false without object coercion, so
null/undefined checks now precede the unsupported object-to-primitive path. Its
four Selection extension flags, `gfxExtensions`, and `_focusrect` are stored, but
this probe does not implement or claim their eventual input/focus effects. The
next stop at that increment is DataProvider's Extends at30980 (CodeId7), before
Array is installed. Normal root actions and later initializers do not run after
that failure; the following Array increment advances this baseline.

`run-init-order-selection.rs`, `init-order-selection.tsv` (the earlier function
inheritance stop), `init-order-function-prototype.tsv`,
`vm-tests-loaded-default.txt`, `vm-tests-function-prototype.txt`, and
`loaded-target-startup-evidence.json` retain this progression privately. Focused
checks also cover loaded target versus receiver identity, restoration after a
getter from an unbound target, version mismatch, direct intrinsic ambiguity and
the shared intermediate prototype identity. These are original-Rust observations,
not execution of the retail client.

## Bounded Array foundation

`install_array_constructor()` explicitly installs Array after Object and before
any script definitions or array creation. The documented creation forms are
implemented: zero arguments, one bounded nonnegative integral Number as length,
or an element list otherwise. Passing an existing array produces a new one-element
array containing that reference; it is not a cast or flatten operation. Fractional,
negative and nonfinite numeric lengths stop explicitly, and exceeding the configured
length limit produces a resource-budget error. Adobe documents integer conversion
for fractional input, so no ECMAScript/AS3 RangeError behavior is substituted.

Constructed arrays and InitArray results share an Array-prototype identity linked
to Object.prototype. **VERIFIED, static REA:** The fresh native prototype is an
actual Array-category object with initial length zero, an Object-prototype parent,
and an exact constructor.prototype link. The VM therefore represents it as an empty
Array, allowing its length and missing initial index reads. Prototype-specific
indexed/length mutation and changes to the native constructor's prototype binding
remain explicit unsupported cases. Real array instances expose their own length
and stored elements. Array methods are not fabricated. Native evidence is in
`rea-array/finding-receipt.json` and `final.bundle.json`; the earlier opaque
prototype was an evidence gate, not a claimed native category.

**OBSERVED:** The original manager now completes 52 serialized root-frame
initializers in the same VM, with the original FocusHandler accessor returning the
one retained Selection listener. The next stop is NewMethod at323687 (CodeId52,
initializer321958), entering a constructor that requests unsupported super/parent
register preloading. No initializer after that stop or ordinary root action runs.
No Array-prototype mutation is attempted in this probe.

`run-init-order-array.rs`, `init-order-array.tsv`, `vm-tests-array.txt` and
`array-startup-evidence.json` retain the owned result and 23 passing focused VM
checks. The Array check exercises creation forms, nested reference identity,
constructor opcodes, bounds, version-independent unsupported coercions, and the
prototype-specific guards. This is an execution increment, not completed Array
or frontend parity.

## Constructor-only super dispatch

Function2 super preloading now works for a constructor entered by NewObject or
NewMethod through an unchanged recorded ActionExtends relationship. A bounded
internal reference retains the receiver and immediate superclass constructor.
Nested super calls advance relative to the constructor currently executing,
without allocating a replacement receiver or rediscovering the most-derived
class. Both register and nonsuppressed named-super bindings use that reference.
The prototype, parent link and owned `__constructor__` metadata are validated;
modified chains, ordinary super.method lookup, parent preloading and ungrounded
constructor call/apply contexts remain explicit unsupported cases. Object/Array
superclass initialization also stays gated separately from their ordinary native
factory behavior.

Adobe SWF19, printed pages98–99, specifies that NewObject/NewMethod retain the
constructed instance and discard script-constructor return values. The VM now
follows that rule, while the explicitly supported Object/Array factories select
the actual native-constructor result separately. A controlled three-level fixture
verifies same-receiver nested super calls, preserved super-call results, rejection
of modified metadata and discarded script-constructor object returns.

**OBSERVED:** Original Tween initialization still completes 52 preceding
initializers, then executes90 instructions before reaching MovieClip native binding1
at322430 (CodeId52). The request has zero arguments and the exact same instance
whose prototype is Tween.prototype; it is not the MovieClip function object. The
host rejects this unresolved native boundary. Selection still retains one listener,
and the original instance accessor returns that listener without adding another.

That earlier controlled probe used generic Rust allocation and did not establish
native receiver-category parity. Subsequent REA found that native construction
consults the public `prototype.constructor` and produces a category-11 MovieClip
object for the unchanged Tween chain. The typed profile below supersedes this
placeholder; its allocation-side-effect gate is explicit.

`run-init-order-super.rs`, `init-order-super.tsv`, `vm-tests-super.txt` and
`constructor-super-evidence.json` preserve the controlled host request and24 passing
focused VM checks. `tween-super-dispatch-private.json` records the unchanged source
instructions and the following comma-list flag dependency. Primary source sections
and licences are retained in `avm1-reference/super-sources.json`.

## Typed unbound MovieClip profile

`install_movie_clip_constructor()` installs a separate MovieClip constructor and
unbound prototype after Object initialization. `movie_clip_role(ObjectId)` reports
`UnboundPrototype` or `UnboundInstance` only for genuinely typed VM objects;
inheriting a MovieClip prototype does not change a plain object's category.
`movie_clip_has_mouse_event_names(ObjectId)` exposes the allocation-time marker
for typed instances, separately from listener registration.
These script objects have no Scene node, timeline, renderer, event dispatcher or
native resource attached. This is an explicit standard intrinsic with no host
callback or display allocation.

**VERIFIED static native behavior:** The constructor callback returns the same
receiver when its native category is 11; a different category or null receiver
returns undefined. The callback does not allocate, read arguments, change receiver
properties or create a display object. NewObject/NewMethod allocation is a
separate operation. Native allocation consults the public `prototype.constructor`,
selects its allocator, invokes the original script constructor and discards that
constructor's return. It installs the original constructor as the receiver's own
hidden `__constructor__` with flags 3, separately from the superclass reference
stored on an Extends-created prototype.

The supported native allocator creates a typed unbound instance for zero arguments
with unchanged constructor/prototype links. Public-constructor mutation cannot
fall through to a generic receiver: it stops explicitly. Typed instance mutation,
native/display members, accessor behavior, typeof, event-name writes, prototype
metadata replacement and display attachment remain gated. Existing stored ordinary
method reads and non-event ordinary writes on the unbound prototype use the proved
raw lookup/assignment paths. Missing native members stop instead of returning
invented values. Generic subclass-prototype writes remain generic; assigning its
`onLoad` does not invoke an inherited MovieClip assignment hook.

**VERIFIED static allocation hook:** A fresh instance's marker starts false.
Assigning a category-11 prototype skips visitation. Assigning a generic prototype
visits every own and inherited stored key, including hidden members, without
reading values or evaluating getters; any of the twelve native mouse-event names
sets the marker. The complete native MovieClip/Object prototype inventory contains
53 stored entries and none of those names. The VM can therefore determine the
marker from all retained script-added keys without inventing native method behavior.
A native baseline method that has not been implemented remains unreadable; both
member reads and existence lookup stop explicitly instead of yielding undefined
or false. Instance mutation and future event dispatch remain separate contracts.

**OBSERVED:** The original Tween initializer at 321958 now completes all 153
instructions with a typed unbound MovieClip receiver. Its `super()` returns the
same instance; its own hidden constructor remains the original Tween function with
flags 3. The allocation marker is false. All six original function objects are
copied onto MovieClip.prototype, and the original comma-selector call sets flags 1
on `tween__start`, `tween__to`, and `tween__run`; the other three retain flags 0.

At the typed-allocation checkpoint, the ordered original manager probe completes
**118 initializers**, retaining
one real Selection listener. It next stops at Enumerate2 750939 in CodeId 113,
called by initializer 118 at 757488, because native enumeration order with multiple
visible names remains unsupported. The new state is original Rust script execution,
not an original-client runtime or display-instance parity claim.

Evidence is joined in `rea-movieclip-constructor/combined-proof-receipt.json`,
`rea-movieclip-binding/finding-summary.json`, `rea-script-allocation/finding-summary.json`
and the research lane's `docs/REA_MOVIECLIP_CONSTRUCTOR.md`. The read/visitor
algorithm is retained in `rea-script-allocation/instance-read-receipt.json`.
Closed inventories are
`rea-movieclip-prototype-inventory/movieclip-prototype-inventory.json` and
`rea-object-prototype-inventory/object-prototype-inventory.json`; they include
later native bootstrap preservation and distinguish subsequent script writes.
`run-init-order-movieclip-events.rs`, `init-order-movieclip-events.tsv`,
`vm-tests-movieclip-events.txt` and `movieclip-event-startup-evidence.json` retain
the private original-script probe and 28 passing focused implementation checks.
Earlier `movieclip-runtime-evidence.json` preserves the superseded 52-initializer
gated foundation measurement.

## Native Object prototype inventory

The native inventory proves exactly nine initial own stored Object.prototype keys,
a null parent and no stored `__proto__` entry. Its special `constructor` entry has
flags 3; all eight native method entries have flags 1. The VM now preserves that
inventory: implemented `addProperty` and the seven remaining method identities
(`hasOwnProperty`, `watch`, `unwatch`, `isPropertyEnumerable`, `isPrototypeOf`,
`toString`, `valueOf`) are present and callable. Invoking an unimplemented method
fails with `UnsupportedNativeFunction` at the original call origin. It does not
return success or invent native behavior. Both the Object constructor's `prototype`
link and the prototype's `constructor` link retain their proved flags 3.

This corrects the previous ordinary-set flags on `constructor` and `addProperty`;
those inherited members must not appear in enumeration of a plain object. Native
method presence must not silently become undefined in original existence checks.
The closed receipt is
`rea-object-prototype-inventory/object-prototype-inventory.json`. Focused tests
verify all nine entries, inherited presence, hidden flags, no stored parent key,
and explicit invocation failures. The ordered original startup still completes
118 initializers and stops at the same multi-name Enumerate2 gate.

`vm-tests-object-inventory.txt`, `run-init-order-object-inventory.rs`,
`init-order-object-inventory.tsv` and `object-inventory-runtime-evidence.json`
retain 29 passing focused checks and the private startup observation. Generic
literal construction metadata and native hash layout remain separate evidence
requirements; this inventory does not establish every ordinary-object factory.

## InitObject metadata and native ASCII enumeration order

**VERIFIED static construction:** InitObject invokes the current native Object
through its zero-argument New path before consuming literal pairs. For the supported
unchanged binding, the allocator first stores `__proto__`, shared construction
stores `__constructor__`, and the Object callback stores public `constructor`.
All three own entries have flags 3; both constructor values refer to global Object.
Literal value/name pairs are then popped in reverse push-pair order and passed
through the ordinary member setter. New ordinary fields have flags 0; replacing
an existing data value retains its flags. Inherited supported accessors receive
the newly allocated literal as `this`; their return values do not replace the
literal or determine success. Flagged accessor ordering remains unsupported,
and fresh native watch state is zero; native watch installation is not implemented.

**OBSERVED:** The focused inherited-setter test records a write on the fresh
literal, confirms no own data field shadows the accessor, and ignores the setter
returning false. Its flagged-accessor variant still stops explicitly. The private
`avm1-runtime/vm-tests-initobject-setter.txt` retains the passing check.

`object_literal()` implements that path separately. It checks the installed global
Object, original prototype/constructor links and null parent before allocation.
Numeric counts and ASCII String names are the supported input profile; unsupported
name coercion stops explicitly. Automatic ScriptFunction prototypes and direct
Object calls retain their separate existing profiles. They do not inherit this
literal-specific metadata contract.

**VERIFIED static hash algorithm:** Native names hash from the last byte to the
first, fold ASCII uppercase, start at 5381, and repeatedly compute `(h * 33) XOR b`
with wrapping arithmetic, retaining the low 24 bits. A fresh table has eight slots.
Before insertion it grows when `5 * existing_count > 4 * capacity`. Collision
placement relocates the occupied home entry to the first wrapped linear free slot,
repairs a displaced chain when necessary, and puts the new entry at its home.
Growth doubles capacity and reinserts old occupied slots in ascending index order.
Updates of existing keys leave their slots unchanged.

The VM records complete insertion history for proved InitObject and native Object
prototype profiles. Fresh ASCII hash layout is reconstructed under a work budget;
slots are visited in ascending order and hidden entries are filtered afterward.
The operand stack receives names above a null sentinel, so the original loop pops
them in reverse visitation order. Deletion histories, unproved initial layouts,
non-ASCII hashes, and the still-gated duplicate/shadow profiles stop explicitly.
The bounded table model does not use Rust's lexical BTreeMap order for native
multi-name enumeration.

**DERIVED static reference, OBSERVED Rust agreement:** The original seven fields
occupy a 16-slot table together with three hidden metadata keys. Their loop-pop
order is `preferredHeight`, `maxHeight`, `visible`, `minWidth`, `maxWidth`,
`preferredWidth`, `minHeight`. The focused test checks the complete ten-entry
slot order, including the hidden collision chain, against the independent native
receipt. Updates preserve that order; deletion invalidates the supported history.

**OBSERVED:** The ordered original manager now completes **447 of 452 root-frame
initializers**. UIObject.mergeClipParameters at 750939 executes through the native
order profile. Startup next stops at GetVariable 2055355 in initializer 447
(CodeId 447), because the existing `gfx.controls.ListItemRenderer` qualified lookup
still requires its own native resolution proof. The other dotted final lookup is
`gfx.controls.ScrollingList`; these are class namespace references, with no
`__Packages`, slash, colon, root or level path. No namespace is fabricated.

The construction receipt is
`rea-enumeration-callback/initobject-finding-summary.json`. Native hashing,
collision placement, resizing, visitation, callback and the final derived slot
layout are joined in `rea-enumeration-order/combined-proof-receipt.json`; the
research lane documents them in `docs/REA_ENUMERATION.md`. The original source
join is `avm1-runtime/enumerate2-mergeclip-private.json` and final registration
inventory is `avm1-runtime/final-registrations-private.json`. No retail executable
was run; the reference order is a static derivation, not a native runtime capture.

`vm-tests-native-hash.txt`, `run-init-order-native-hash.rs`,
`init-order-native-hash.tsv` and `native-hash-startup-evidence.json` retain
31 passing focused checks and the private original-script observation. The earlier
instrumented receiver trace had only one hidden metadata key and incorrect inherited
method flags; this new native construction/inventory evidence supersedes that
historical Rust shape. Its original pair-pop insertion sequence remains valid.

## Exact own-name property flags

ASSetPropFlags now accepts existing own ASCII identifiers, including nonempty
comma-separated identifier lists. Every selected key is checked before mutation.
There is no whitespace trimming, case folding, primitive conversion, inherited
lookup or getter invocation. Missing/inherited keys, empty components, non-ASCII
names and unsupported native descriptors stop explicitly. Selection parsing and
both visitation passes consume allocation/member budgets.

The same selected-name path is supported for stored ordinary members of the actual
unbound MovieClip prototype. Public constructor metadata, native virtual members,
null-selector visitation of that prototype, instance flags and display objects
remain outside this profile. Flags preserve stored values and apply the existing
low-three-bit mask rule and bound-target version default. Null selection of generic
objects is unchanged. Read-only behavior and unsupported accessor ordering remain
explicitly checked by the existing consumers.

Native receipts are `rea-asflags-selected-name/finding-summary.json` and
`rea-asflags-comma-selector/finding-summary.json`. The latter follows the original
`tween__start,tween__to,tween__run` string through exact ASCII splitting in source
order. MovieClip.prototype's raw lookup/flag-setter binding is established by the
joined constructor evidence. Focused checks verify those three own values and
flags, a generic `dispatchQueue` field, and rejection without partial flag writes
for missing or unsupported selectors. This bounded rejection policy is not a claim
about native failure handling outside the supported profile.

## Qualified class namespace reads in the modeled root

**VERIFIED static, bounded profile:** GetVariable splits the observed three-part
ASCII dot path, resolves its first name using the existing target-before-global
scope, and resolves later names on each returned namespace object without a scope
restart. Existing ordinary own data members use exact names for loaded versions
above 6. The final FunctionRef copy preserves the function object and auxiliary
frame reference; it does not perform `Function(value)` conversion. Receipts are
`rea-qualified-variable/combined-proof-receipt.json` and
`rea-qualified-variable-entry/finding-summary.json`; the research lane records
them in `docs/REA_QUALIFIED_VARIABLES.md`.

The VM accepts only a three-segment ASCII identifier path with scopes exactly
`[target, global]`, no local activation, and plain parentless target/global objects.
Intermediate namespaces must be Plain objects with existing own data properties;
the final value must be a ScriptFunction. Missing or primitive intermediates,
accessors, inherited namespace fields, native/display categories, reserved/root
paths, With scopes, other grammars and qualified writes remain explicit errors.
Segment allocation and member work are bounded. Only opcode GetVariable uses this
new route; CallFunction and constructor name resolution retain their prior gates.

**OBSERVED controlled Rust execution:** All **452 original manager root-frame
initializers** complete in serialized order with the explicit minimal globals,
SelectionHost and the actual bounded ClassRegistry. The final five registrations
retain the original constructor identities for ListItemRenderer, Cursor,
NotificationManager, Manager and ScrollingList. The Selection singleton remains
one listener, and the earlier typed unbound Tween checks still pass. Two focused
checks cover scope shadowing without global restart, exact function identity,
unsupported accessors without invocation, and the retained grammar/scope gates.

This count is diagnostic progress, not complete native startup parity. The VM's
parentless Plain root does not establish absence of `gfx` from a retail attached
root's script storage, named children, prototype or resolver. The global baseline
is also explicitly incomplete; omitted real native globals can change existence
branches. Attached-root resolution and complete native global registration require
separate evidence and integration. No retail process, display attachment or server
was used by this probe.

Private `avm1-qualified-variable/namespace-profile-receipt.json` captures the
actual modeled namespace owners and class identities before the change.
`avm1-runtime/run-init-order-qualified-variable.rs`,
`init-order-qualified-variable.tsv`, `vm-tests-qualified-variable.txt` and
`qualified-variable-runtime-evidence.json` retain the new controlled execution.

## Original frame action after the controlled initializers

**OBSERVED:** With the same modeled Plain root and minimal globals, all 452 original
manager initializers still complete after the host-result change. The original
root-frame DoAction at 5967 then reaches 31 instructions, counting the failed
instruction. It stops at the original MarvelUtils adapter's Function call at
855912, CodeId 139, opcode `0x3d`, with `unimplemented MovieClip native member`.
No unknown native host callback was invoked.

Private inspection establishes the immediate cause. Global Function already exists.
The adapter has assigned a script prototype to the modeled root, so ordinary variable
lookup searches its activation, then that root's chain, then global. The root chain
passes through UIComponent.prototype and the unbound native MovieClip prototype.
There is no modeled own `Function` along that chain; the explicit native-member
boundary rejects before global fallback. This is neither a failure of the restored
Function conversion nor proof that a real attached root lacks that member. No missing
native member lookup was broadened to advance the diagnostic.

`avm1-runtime/frame-after-inits-evidence.json`,
`run-manager-frame-after-inits.rs` and `manager-frame-after-inits.tsv` retain the
result; `avm1-frame-after-inits/lookup-context.txt` records the immutable inspection.
The complete native global inventory and attached owner/storage/child semantics
remain separate requirements. Completing the controlled initializers does not
establish a completed retail frontend startup.

## Native Function conversion and empty apply lists

**Corrected native binding, VERIFIED static:** The earlier research incorrectly
classified interface `0x142513260` as the ordinary script-function table. Both
DefineFunction (`0x9b`, type 1) and DefineFunction2 (`0x8e`, type 2) are now traced
to constructor `0x1401e14c0`, actual interface `0x142513090`, and its `+0x60`
getter `0x14018c580`. That getter preserves the function object while clearing
auxiliary frame and reference flags. The creators capture an active frame when
present, and invocation consumes that auxiliary pointer as a frame parent.
Therefore general closure conversion is not object-identity conversion.

The temporary all-Script gate is replaced by the proved target/global-only
profile. `rea-script-function-conversion/finding-summary.json`,
`definefunction-type1.bundle.json` and the research lane's
`docs/REA_SCRIPT_FUNCTION_CONVERSION.md` retain the corrected producer, getter,
capture and invocation evidence. Both old and newer original script producers
have been checked; no table category is inferred from a nearby interface.

`install_function_conversion()` installs the verified one-argument global Function
path after Object initialization. An existing supported callable returns the same
underlying VM function object. Noncallable values return null. Other arities,
`new Function`, superclass construction and internal super references stop
explicitly; no source-string function construction is inferred.

The native function-reference getter clears auxiliary reference/frame fields while
retaining the underlying function object. The VM currently models ordinary
function-object references, not every native wrapper form. Supported script
conversion requires scopes exactly `[captured target, global]`; nested activation
and With scopes remain explicit errors. This is not an assertion that a native
function-value wrapper is byte-for-byte unchanged.

Native apply forwards zero arguments for omitted, null and undefined lists and
preserves the supplied object receiver. This does not replace lexical scope,
`_root`, `_global` or the captured movie target. Primitive/null receiver fallback,
other primitive list types and constructor-super semantics through call/apply
remain outside the supported profile. Evidence is in
`rea-function-calls/finding-receipt.json` and `final.bundle.json`; the detailed
native trace is documented in `docs/REA_FUNCTION_CALLS.md` by the research lane.

**OBSERVED:** The original manager adapter at 855879+41 executes its eight original
instructions, assigning the supplied receiver's prototype and invoking the original
empty MarvelUtils constructor at 855764 (body 855772+0) through Function/apply. It
uses a controlled plain receiver and zero host calls. A second controlled case
supplies the original login getter at 64967 as the callable, producing 16 total
instructions; the supplied child width remains 320, and a direct getter call
confirms its 320 return. These are selected original constant pools/functions;
full containing initializers and a native MovieClip receiver are not substituted
or claimed to have run.

`run-owned-function-conversion.rs`, `owned-function-conversion-results.tsv`,
`vm-tests-function-conversion.txt` and `function-conversion-evidence.json` retain
these probes and 25 passing focused checks. They cover callable identity, explicit
receivers, zero-argument list forms, native/noncallable values, arity, captured
activation rejection and unsupported construction. These historical probes are superseded for the corrected table binding by
`run-owned-function-conversion-verified.rs`,
`owned-function-conversion-verified.tsv`,
`vm-tests-function-conversion-verified.txt` and
`function-conversion-verified-evidence.json`: the original type1 empty constructor
adapter still runs 8 instructions; the original type2 getter variant runs 16 and
returns the controlled width 320. One focused test verifies identity and retained
captured-frame rejection. These are selected-script observations, with plain
controlled receivers; no full containing initializer or native wrapper execution
is claimed.

## Active host-call context

`HostCall.context: Option<ObjectId>` exposes the active interpreter target,
separately from the supplied `this` value and call-site `CodeOrigin`. It is copied
only from the current execution frame. A direct native `Vm::call` has no script
Environment and supplies None. A script call runs with its captured target; nested
calls and Object-result setter reentry restore the caller's target on success or
failure. An attached target is identified by its owner ObjectId, which the frontend
must explicitly associate with the correct live movie/root/handler.

**VERIFIED static:** The original named CallMethod builds its native FnCall from
the current interpreter Environment. ExternalInterface.call selects its handler
through that Environment's target, AS2 root and MovieImpl. The function's creation
origin and its `this` receiver do not select that handler. This specific native
bridge is documented in `docs/EXTERNAL_INTERFACE_BINDING.md`, including REA record
`ev_13a35134d29edfd62e7aecc19583c957c3182e870d48cd99b5cf85505ec9f4ba`.
That evidence does not independently prove all preceding script target-selection
semantics; the seam preserves the VM's existing bounded frame model.

**OBSERVED:** Two focused tests cover distinct receiver/context/origin identities,
cross-target calls, Object-result script-setter reentry, error restoration and a
subsequent direct native call with no context. The original selected localization
caller still executes 40 instructions and its two direct wrapper calls execute 25
each. All three host requests carry the original functions' captured root target,
distinct from their controlled ExternalInterface receiver. Private
`avm1-runtime/host-context-evidence.json`, `run-owned-host-context.rs`,
`owned-host-context.tsv` and `vm-tests-host-context.txt` retain these checks.
The controlled namespace/host fixture does not install a native ExternalInterface
factory, select a retail handler or resolve actual locale resources.

## Fresh ExternalInterface graph

`build_external_interface(call_binding)` constructs the verified fresh callable,
separate prototype and two native method identities. It returns their ObjectIds
without installing a namespace entry or factory cache. It uses the cached Function
and Object prototypes directly, preserving them even when public bindings change.
The callable owns `available` (flags 0), `addCallback` and `call` (flags 7), and
`prototype` (flags 3); linked-parent and prototype-constructor metadata retain flags 3.
The prototype adds no methods. The complete setup and rights/evidence ledger are
in `docs/EXTERNAL_INTERFACE_BINDING.md`.

The native callable callback returns undefined without invoking a host.
`addCallback` remains an identified unsupported native function. `available` is an
explicit native special-property marker whose active-handler query rejects until
implemented; it is never represented by a guessed Boolean or missing property.
The supplied `call_binding` forwards through the existing HostCall/HostResult
boundary. No callback registration or handler is created by the builder.
Construction with `new`, native-object mutation, accessor installation and native
metadata rewriting remain gated. A failed unpublished factory allocation restores
its newly allocated graph and counters; this is an engineering rollback policy,
not a native out-of-memory behavior claim.

**OBSERVED:** Five focused checks cover the exact fresh ledger, cached-parent
selection, distinct identities, allocation-budget rollback, callback boundaries,
unsupported available/construction/mutation paths and rejecting default host.
The original selected localization caller still runs 40 instructions; two wrapper
calls run 25 each, return distinct four-field objects and carry the captured root
context at original CallMethod 110311. This time the receiver is the grounded
ExternalInterface callable graph. The surrounding namespace and returned locale
data remain controlled fixtures, and no lazy-token materialization or actual engine
handler is claimed. Private `avm1-runtime/external-interface-evidence.json`,
`run-owned-external-interface.rs`, `owned-external-interface.tsv` and
`vm-tests-external-interface.txt` retain these results.

## Retained deferred builtin values

`Value::DeferredBuiltin(DeferredBuiltinName)` represents native tag 11 separately
from Undefined and an already materialized function. The opaque wrapper retains
only a validated name; it carries neither a factory nor a creation Environment.
One VM admits one AS2 context. Its name-keyed registry/cache is separate from
script properties, and `bind_builtin_context_target(ObjectId)` requires an explicit
host association before any resolution, including cached resolution. A direct
native `Vm::call` supplies no Environment and cannot infer one from `this`.

`register_deferred_builtin(namespace, name, factory, flags)` installs an actual
token without invoking its factory. The first profile accepts bounded ASCII
identifiers, flags 0/1, and an ordinary namespace with original Object-parent,
descriptor and resolver-history guards. An existing registry name leaves its entry
and namespace unchanged. `BuiltinFactory::Unsupported` preserves real existence
and scalar behavior but stops at materialization; it never reports a missing/null
factory. The supported ExternalInterface registration requires its exact name and
flags 0. It builds the fresh graph, performs guarded ordinary namespace publication,
then caches the callable. Existing retained aliases stay tokens; later namespace
reads see the published callable. Failed unpublished allocation is not cached.

**VERIFIED static:** A retained token is Boolean true, TypeOf string `undefined`,
ToString `[type Function]`, and numeric positive zero without factory lookup. String
conversion requires an active Environment for its builtin string. Two tokens compare
equal under both equality opcodes even when their names differ; Rust name equality
remains a separate registry/inspection operation. Token-left Equals2 against a
non-token is false without resolution. Boolean/Number/String-left against a token
on the right attempts materialization and then returns false on normal completion.
The opcode preserves native lower-stack-left/top-stack-right order. StrictEquals
never materializes a token. Details and exact evidence are in
`docs/DEFERRED_BUILTIN_VALUES.md` and
`rea-deferred-builtin-values/combined-proof-receipt.json`.

Ordinary reads and copies retain tokens. Explicit member receivers, named-call
receivers and selected callees resolve with the active context. NewObject/NewMethod
resolve selected callable/prototype metadata before applying their existing
construction gates. Qualified lookup retains a terminal token; its existing
namespace-kind/activation guards still limit intermediate use. Unnamed CallMethod
token receiver binding, Function conversion of a token, deferred accessor arguments,
Add2/relational primitive-conversion order, reentrant factories and multiple AS2
contexts remain explicit errors. No eager universal global bootstrap is installed.

**OBSERVED:** Ten focused registry/executor checks pass. They cover scalar and
equality non-materialization, observable reversed-operand factory effects, real
named-call resolution, retained aliases and stable cached identity, current-target
admission, namespace mutations, unsupported factories, allocation rollback/retry
and fresh registration limits. Private `avm1-runtime/vm-tests-deferred.txt` retains
the report. These tests alone do not establish native startup behavior.

**OBSERVED original selected code:** A separate frozen seven-source-file probe
registers ExternalInterface lazily and binds the controlled root context before
executing the unchanged localization functions. Registration leaves object count
19 unchanged, and the namespace retains its token through definition setup.
The first 40-instruction caller publishes constructor 28, methods 29/30 and
prototype 31 at its original named CallMethod path (110311). Both retained aliases
(a Rust value and a script property) stay tokens. Repeated 25-instruction wrappers
reuse those graph identities and return distinct result objects 35/38, allocating
only their three ordinary invocation/result objects each. All three callbacks
validate original origin 110311, root context 1 and distinct constructor receiver 28.
Intermediate operand stacks were not instrumented. Private
`avm1-deferred-localization-probe/verification.json`, `sources.json` and `run.stdout`
retain the source hashes, native-free compile/run and exact observations. The
surrounding namespace and locale payload remain controlled; no actual engine
handler, complete global baseline, full initializer or native process is claimed.

## Fresh Object results from native hosts

`Host::call_result` is backward compatible: its default calls the existing `call`
and wraps the result in `HostResult::Value`. A host can instead return
`HostResult::ObjectFields(Vec<(Text, Value)>)`. After the callback returns, the VM
constructs one fresh Object through the proved unchanged native Object profile
and performs ordinary `write_member` operations in the supplied order. The host
does not hold a mutable VM reference, so inherited setters can call the host again
without nested mutable ownership. No result-object identity is cached.

**VERIFIED static native boundary:** Public GFxValue CreateObject with no class
name and zero arguments joins current global Object construction. Its result has
own `__proto__`, `__constructor__` and `constructor` metadata with flags 3. The public
SetMember conversion reaches the ordinary internal setter with flags 0; the outer
struct converter ignores its Boolean result. The saved unpatched LocalizedObjectData
fields are, in order, `localizedPrototypeIdStr`, `localizedText`,
`localizedTooltipText` (strings), and `hasTooltip` (Boolean). The native receipt is
`localized-result-object/finding-summary.json`, with byte validation in
`verification.json`; the research lane records the bridge and exact limits in
`docs/LOCALIZED_RESULT_OBJECT.md`.

The current materializer accepts unique ASCII identifier keys and well-formed,
NUL-free UTF-16 String or Boolean values. It checks and charges conservative input,
stored-name, field, member-work, object and host-call budgets. The original global
Object and prototype/constructor links must still be own data bindings; inherited
fallback or accessor replacement cannot substitute for them. Fresh native watch
state is zero, and native watch installation remains unsupported. Existing ordinary
accessor writes use the VM's established setter path; flagged accessor and inherited
readonly ordering still reject explicitly. Failures after construction or a setter
can retain prior writes, as with other VM execution; there is no rollback claim.

**OBSERVED focused checks:** Two calls produce distinct objects with ordered own
fields and exact metadata flags. Mutating one leaves the other unchanged. Native
setters receive that fresh object as `this`, execute in field order, reenter the
host, and can return false without replacing the result object. Invalid payloads,
modified constructor binding, exhausted string/property budgets and exhausted
nested host-call budgets reject. The legacy direct-value Host implementation still
passes its existing check. Four checks are retained in
`avm1-runtime/vm-tests-host-object-fields.txt` and
`host-object-fields-evidence.json`.

**OBSERVED original selected scripts with controlled data:** The actual
MarvelGlobals.GetLocalizedStringByLabel definition at 110321 runs with its original
GetLocalizedObjectDataByLabel and IsEmptyString definitions and constant pool. It
executes 40 instructions, makes one strictly checked host request at 110311, reads
`localizedText` at 110384 and returns the supplied text. Two separate calls to the
original object-data wrapper each execute 25 instructions and retain distinct fresh
objects with four flags-0 fields and three flags-3 metadata entries. Private
`avm1-localized-result-probe/run-localized-selected.rs`, `probe.tsv`,
`execution.json` and `source-snapshot.json` retain the frozen original code and
source coordinates. The provided MarvelGlobals/ExternalInterface receivers and
result data are explicitly controlled fixtures; no native global baseline, whole
class initializer or locale resolver is substituted or claimed to run.

This seam does not install a localization binding, select reflected fields, parse
locale data or claim that a live native struct has no patched fields. The integrating
host owns those data and patch-profile decisions. Neither the focused checks nor the selected-script probe executes the retail
native callback or establishes runtime parity.

## Explicit attached source MovieClip bridge

`Value::DisplayClip(AttachedClipId)` is distinct from an ordinary object value.
`create_attached_movie_clip(AttachedClipSpec)` allocates a new owner identity and
retains the host's DisplayTargetId, loaded movie identity, sprite definition,
version, complete case-3 parent chain and actual raw event state. It never retags a
Plain or unbound MovieClip object. Native MovieClip/prototype selection is admitted
only through the unchanged own-data bindings. Each live target identity is unique.

`attached_owner` supplies the separate ObjectId used as an execution context or
explicit host mapping key. `attached_value` supplies the display reference.
Bound script storage is lazy and has its own opaque diagnostic identity; it cannot
be passed back as an ordinary Value. `set_attached_prototype` creates storage first,
updates its linked parent and mouse-event marker, then updates the owner's prototype.
The public `attached_info` record exposes these distinctions. `adopt_attached_root`
requires a pristine old root and no registered programs/script closures; integration
must call it before installing a host whose context depends on that root. Existing
Plain-object diagnostics retain their separate behavior.

**VERIFIED static admission:** The initially selected names `gfx`, `Function`, `disabled`,
`enableInitCallback`, `localTextLabel`, `button` and `localizedText_tf` take the
ordinary route under the original standard-member map and concrete case-3 owner.
This is an exact-name profile backed by `rea-attached-member-names/combined-proof-receipt.json`,
not an underscore heuristic. Native `enabled`, `tabEnabled`, `transform` and
`scale9Grid` demonstrate why arbitrary non-underscore names cannot be accepted.
The private finite inventory remains outside git.

`read_attached_member` returns `Missing` or `Present(Value)`. Own storage is probed
without inheritance, then the complete unique named-child snapshot, then the
prototype. Own undefined/null values win. A supported matching child returns its
public DisplayClip reference, not its storage. Unsupported named-child categories
remain explicit entries and stop lookup rather than disappearing. The host installs
one complete snapshot with `set_attached_children`; direct children must be created
before that installation. Duplicate names/target identities, later addition,
replacement, rename, removal and eligibility-bit changes reject. These restrictions
keep the native child-cache profile unambiguous; they are not retail restrictions.

Ordinary writes target storage and never replace a child. The normal setter skips
inherited data while looking for deeper accessors, shadows inherited data including
readonly data, preserves existing own flags, and leaves own readonly data unchanged.
Accessors, watches, resolvers and native special effects stay explicit boundaries.
Only the admitted names are written; their pre-write hook has no target-state write
under the required homogeneous live case-3 parent chain. Generic/unbound setters
are not reused to infer bound-storage semantics.

The unchanged native MovieClip/Object/null fallback is separately verified to miss
`Function`, `gfx` and `onLoad` without modifying its raw output. The attached path
checks current own members, original builtin links and absence of resolver mutation
history before using that fact. Other missing native prototype members reject.
A found undefined remains found, preserving target-before-global lookup. The existing
Plain-root fallback is not relaxed. The three-segment qualified class lookup may
use this explicitly admitted attached first-segment route; all its other guards remain.

`AttachedEventState` contains caller-owned target/load/source raw fields, without a
universal Default. The concrete owner-offset byte at target +0x6d overlaps the high
byte of target_flags and must be 0x52 in this profile. Updates preserve that byte and
child eligibility bits 7/8; unmodeled cache/lifetime transitions cannot enter through
a raw-state update. The separate attachment helper clears load bit 3 only on its
proved raw-argument branch. Eligibility is captured before construction callbacks;
presence marking never clears an existing bit. Wrapper admission checks target bit 4
before clearing bit 5 for Load, then applies the extra load gate. No helper executes
callbacks or proves delivery. `attached_load_present` consumes actual saved-load-event
presence or raw storage/prototype presence, ignores named children, invokes no getter
and does not require a callable value.

**OBSERVED:** Eleven focused bridge/event checks pass, together with 37 existing
VM checks. The original manager cursor handler at 7096+77 runs on an explicit attached
source identity for character 27 (`cursor_mc`), with a controlled geometry host.
It executes 14 instructions and six indexed-property callbacks, moving supplied
coordinates (10,20) to (100,200). Owner and display reference remain distinct, and
no script storage is created by the geometry host. Private
`avm1-runtime/attached-bridge-evidence.json`, `run-owned-attached-cursor.rs`,
`owned-attached-cursor.tsv` and the focused reports retain the result.

This probe supplies declared raw state and geometry; it does not establish a native
root binding, Scene association, placement/class lifecycle, event queue execution or
retail display parity. Numeric/string/Boolean coercion, script equality/typeof,
Function.apply display-receiver conversion, attached accessor execution, enumeration
and retirement remain gated. Integration owns verified Scene/movie/target mappings.
The static contracts are in `docs/ATTACHED_MOVIECLIP.md`,
`docs/ATTACHED_MEMBER_NAMES.md`, `rea-prototype-function-miss/finding-summary.json`
and the retained event-wrapper receipts cited by `attached_events.rs`.

**OBSERVED controlled attached-root boundary:** The original manager's saved first
root frame has exactly three named local sprite placements: `escapeMenuFadeIn_mc`
(sprite 32/depth 1), `manager_mc` (sprite 29/depth 3), and `cursor_mc`
(sprite 27/depth 5). A separate probe creates these explicit source identities,
adopts a fresh attached root before host setup/compilation, and freezes the complete
unique named-child snapshot. All four targets use declared raw-state fixtures;
placement callbacks, class construction, frame actions and child events are not run.

The first initializer (tag 379, sprite 33, source 7239+5138) stops after 12
instructions at NewObject 8509 because the exact attached-name profile does not
yet admit `Object`. Original pool 7239 index 2 and Push 8493 establish that operand.
Zero initializers complete, no host/class/Selection calls occur, and root storage
remains absent. Private `avm1-attached-manager-init/receipt.json` and
`rejected-operand.json` retain the frozen source hashes, complete child accounting
and parsed operand evidence. This is distinct from the earlier 452-initializer
Plain-root diagnostic; neither fixture establishes native startup parity.

The following bounded increment admits exact `Object`, `MarvelGlobals`, `flash`
and `String` names. Their absence from the complete native standard-member map
and canonical builtin prototype inventory is separately verified in
`rea-prototype-function-miss/object-extension.json`; all storage/child/prototype
precedence and mutation guards remain. This extends an exact-name profile rather
than inferring that arbitrary identifiers are ordinary native properties.

Raw NewObject (`0x40`) with zero arguments resolving the unchanged native Object
now uses the same proved allocation/metadata path as InitObject: own `__proto__`,
`__constructor__`, `constructor` in that order, each flags 3. The direct `Object()`
call and automatic script prototypes are separate paths. Nonzero-argument and
NewMethod Object construction remain gated. The native opcode-to-shared-allocator
join is retained in
`rea-deferred-builtin-materialization/constructor/object-newobject-zero-arg.json`.
One focused test verifies metadata, unchanged-global checks and argument gating;
the five existing attached bridge tests also pass. This supersedes the old
NewObject shortcut that created only the linked prototype metadata.

**OBSERVED:** The next frozen attached-root probe passes Object 8509 and stops
after 58 instructions at GetVariable 8704, whose original Push 8699 refers to
pool index 8, `MovieClip`; Extends follows at 8705. Zero initializers complete.
The same complete saved child fixture is retained, and no host, Selection or
registry callback occurs. Evidence is in
`avm1-attached-manager-init-increment2/receipt.json`. Exact `MovieClip` admission
and unchanged-native-chain raw miss are now separately verified by
`rea-prototype-function-miss/movieclip-extension.json`; the next increment admits
that one name while retaining all preceding lookup and mutation guards.

## Indexed movie-property host seam

GetProperty/SetProperty now use `Host::get_property` / `set_property`. Both have
default rejecting implementations, so existing hosts retain explicit failures.
Requests contain the execution-context ObjectId, original target value, a typed
Adobe movie-property index, and the source instruction offset. The host owns
path resolution, local/global coordinate conversion, frame state and actual
mutation. Returned object IDs are checked; invalid indices and exhausted host
budgets stop before the callback. Nothing accesses a native client automatically.

**OBSERVED:** The original manager MouseMove ClipAction at 7096+77 executes all
14 instructions against a controlled geometry host. Starting at (10,20) with a
supplied pointer at (100,200), it sets the cursor to (100,200). Its six callbacks
are reads of X/MouseX at 7127/7138, write X at 7140, reads Y/MouseY at 7158/7169,
and write Y at 7171. No native function is invoked. This verifies original script
execution and the host seam with supplied coordinate behavior; it does not prove
retail pointer transforms or automatic event dispatch.

`run-owned-cursor.rs`, `owned-cursor-results.tsv`, `cursor-execution-evidence.json`
and `vm-tests-movie-properties.txt` retain the private probe and fourteen passing
focused tests. The integrating frontend must supply actual geometry/event state;
returning invented values from a default host would be incorrect.

## Class-registration host seam

`Object.registerClass` now calls `Host::register_class(ClassRegistration)`, whose
default implementation rejects the request. The host returns an explicit Boolean
only after applying its registry policy; the VM does not report universal success,
look up exports, fabricate classes, instantiate a constructor or mutate old
instances. A supported call has a nonempty String name and a callable function
object or explicit null. Null means unregister; undefined never means unregister.

Adobe documents replacement, registration before a matching symbol exists,
unregistration without changing existing instances, and Boolean success/failure.
It does not document the precise invalid-argument behavior. Empty/nonstring names,
missing/excess arguments and undefined/nonfunction constructors therefore stop
explicitly. The addProperty validation table must not be reused for registerClass.
The host also owns the Boolean outcome of removing an absent entry.

`ClassRegistration` retains the constructor ObjectId, the calling `CodeOrigin`,
and a separate constructor-definition origin for script functions. Native/builtin
constructors have no script definition origin. `CodeOrigin` combines the VM-local
CodeId and movie source offset; offsets alone are not unique across movies.
`HostCall`, `PropertyRequest` and VM errors also expose this origin while retaining
their earlier offset field. Caller origins are restored after accessor/function
calls, so a getter in another program cannot misattribute the eventual native
request. Hosts associate CodeIds with loaded movie identities for provenance.
The native registry belongs to the current AS2 root context; call-site SWF and
constructor-definition origin do not select separate registries. The integrating
host owns that context. Static evidence is retained in
`rea-class-registry/{entry,helpers,owner,lookup}.bundle.json`, including owner
`ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a`
and lookup `ev_6a3ef294e0f93be00362ab76b3f85adad17bf1242bc644ad9fa95b6009d76dc4`.

**OBSERVED:** Two original owned registration snippets each execute eight
instructions against a real controlled in-memory registry:

| Registration stream | Linkage | Constructor's original manager definition |
| --- | --- | --- |
| Manager 2055420+64, call 2055481 | `Cursor` | `MarvelCursor`, definition 55654 |
| Login 167848+76, call 167921 | `TextOnlyButtonWide` | `MarvelButton`, definition 152783 |

The harness selects each original enclosing constant pool and constructor
definition, binds the resulting function to the identified variable explicitly,
and runs the unmodified registration stream. Those constructor bodies and full
class initializers are not executed. The registry retains the exact function
objects; login registration and manager definition have distinct CodeIds. The
same manager snippet with RejectHost stops at call 2055481 with its original code
origin, proving there is no default success path.

`run-owned-register-class.rs`, `owned-register-class-results.tsv`,
`register-class-evidence.json` and `vm-tests-register-class.txt` retain the probe
and seventeen passing focused tests. Registry tests cover actual replacement and
removal, retained constructor objects, explicit host rejection/false responses,
unsupported arguments, overlapping definition offsets in different programs,
and call-origin restoration after cross-program accessor lookup. The integrating
frontend still owns export association and later instance construction.

## Typed native String boxing and bounded ASCII uppercase

`BuiltinFactory::String` registers the exact GLOBAL.String token with flags 1.
It uses the existing explicitly associated active context and deferred cache;
raw reads and retained aliases stay tokens. Materialization publishes the native
callable through the ordinary setter, preserving existing descriptor flags.
A failed unpublished graph/publication restores heap/property/string counters
and its canonical String cache. `build_string_graph()` is separately available
for explicit eager integration; it does not install a global binding or context.

The typed graph has distinct callable, prototype, method and temporary wrapper
roles. String.prototype retains the native `[type Object]` payload and cached
Object parent; its callable/methods retain cached Function parents. The supported
uppercase method is an identity-bearing native function. `fromCharCode` also has
an explicit callable identity and flags 7, with invocation unsupported. Thirteen
other native prototype methods are intentionally omitted from this bounded graph;
unknown reads, presence/descriptor lookup, mutation and reflection therefore
reject rather than report missing. These guards also apply during prototype
traversal through a Plain object. The sparse graph and retained temporary arena
are not native allocation-count or garbage-collection parity.

Only named CallMethod on a primitive NUL-free ASCII String uses the new boxing
route. It checks the active context, current data-only parentless GLOBAL.String
and raw GLOBAL.Object identities, actual public prototype/constructor metadata,
cached parents and resolver/accessor history. A modified or alternative graph
rejects. A fresh category-8 wrapper retains the input payload and the proved own
metadata insertion order: `__proto__` flags 3, native `length` marker flags 0,
`__constructor__` flags 3. The final normal lookup must supply an actual function
value; this primitive branch does not materialize a returned deferred method.

The native uppercase intrinsic accepts the typed wrapper, zero arguments in
this initial profile, and code units 1..127 (or an empty string). It maps only
ASCII a–z to A–Z, preserves all other admitted units, and returns a primitive
String without modifying the input or box. Output allocation is bounded before
a fallible reserve and charged once. Native argc is actually ignored; the zero
argument limit is an explicit implementation slice. No host callback or external
execution is fabricated. General primitive member coercion, explicit construction,
direct String() invocation, borrowed/prototype method receivers, other methods,
NUL and non-ASCII behavior remain unsupported.

Canonical `length` reads compute the admitted ASCII payload's length only under
the proved marker/miss route. The prototype has no own length marker: an inherited
ordinary override or accessor rejects instead of being ignored. Non-ASCII native
length counts encoded scalars, so this is not a general UTF-16 length rule.

**VERIFIED static:** `STRING_NATIVE_METHODS.md` joins the native constructor,
current-global boxing, exact primitive-call method admission, graph/marker state,
callback and all 128 ASCII mapping inputs. Private combined receipt
`rea-string-uppercase/combined-proof-receipt.json`, SHA256
`c0a824584a7fc3ec2c6ec04a601b8739144827eb0b1e02c4a3c80c51f028c6b8`,
records 17 new REA observations with independently checked instruction/data bytes.
Source rights remain the owned proprietary executable/movie and original project
static analysis; no external VM implementation or extracted method inventory is
included here.

**OBSERVED controlled Rust checks:** six module tests cover graph/metadata,
current globals/context, deferred reuse, fresh boxes, ASCII/control/chunk edges,
unknowns, graph edits and bounded allocation. Three bytecode/integration tests
cover repeated calls, publication flags and retained tokens, direct/other-opcode
rejection, ancestor mutation/reflection gates and no alternate factory execution
for an unproved replaced-method token. Seven existing deferred-registry checks
exercise the preserved ExternalInterface/unsupported-token behavior. Reports and
tested module hashes are in `avm1-runtime/string-bridge-evidence.json`. The owned
94251 call-snippet diagnostic is separate; these tests do not claim the full
label update, native loading or retail UI parity.

**OBSERVED owned-call snippet:** frozen commit `7389005` executes the unchanged
manager action span 94235+17, including CallMethod 94251, twice. A synthetic
21-byte prefix sets root register 2; the unchanged original 103-entry constant
pool at source 90399+1577 is relocated in the controlled program, while the call
retains its source coordinate. Each seven-instruction execution returns
`MARVEL HEROES 123 !?` from the supplied `mArVeL hErOeS 123 !?`, with zero VM/host
callbacks. The graph is cached, the global descriptor retains flags 1, and wrapper
IDs 20 and 21 are distinct. Input and owned movie hashes remain unchanged.
`avm1-owned-string-call/execution.json`, SHA256
`bf012a034b15c2a5c9be475d02c6d659757f56a1f1d4393e816bd0349338bca3`,
retains nine frozen source hashes and explicit setup/coordinate mapping. This
executes the original call snippet, not the complete method or native client.

## Optional verified attached-name inventory

`install_attached_name_profile(Arc<NativeNameProfile>)` installs one immutable,
authenticated owner-private profile before any program registration. The VM
re-exports the parser and classification types from its public
`attached_name_profile` module; the crate root should re-export that same module,
not compile a second copy. The integrating loader owns trusted expected hashes,
actual executable verification and the SHA-256 callback. No extracted inventory
is embedded in the VM, source tests or packages. Without this optional profile,
the preceding exact-name slice remains unchanged.

With a profile, the concrete case-3 bridge admits exact nonstandard, nonunderscore
ASCII names through ordinary handling. Standard IDs and unsupported names still
reject; membership is not a native property implementation. General writes also
explicitly reject the `on` prefix and rendererString/rendererFloat/disableBatching
pre-write routes. The former whitelist never admitted these hooks. Homogeneous
case-3 parent/lifetime assumptions, storage/child/prototype ordering, accessors,
resolver insertion history, readonly behavior and the native context gates remain.

The full private builtin inventory is consulted separately from the Rust object
maps. A missing native MovieClip own key stops before walking to Object.prototype,
so a script-added ancestor key cannot accidentally replace an omitted native
function. Matching live storage/child/script own entries still take precedence.
Only verified baseline absence after the existing native identity, constructor,
parent and resolver checks becomes `AttachedMember::Missing`. A known native or
reserved key that is still unmodeled remains unsupported. These facts do not
turn every absent Rust key into native absence, and do not change global lookup
or deferred builtin behavior.

**OBSERVED:** three focused integration checks cover installation phase, broader
ordinary reads/writes, standard/underscore/Unicode boundaries, unsupported child
shadowing, own Undefined, native-member omission before ancestor lookup, accessor
and deleted-resolver guards, and unsupported pre-write hooks. They use generated
synthetic inventory names. Six parser tests cover bounded authentication and
shape checks. The separate private actual-profile/startup diagnostic is recorded
with its own source and executable hashes; synthetic tests are not native parity.
The native theorem and full source/rights ledger are in
`ATTACHED_NAME_CLASSIFICATION.md`, with private receipts under
`rea-attached-name-classification/` and `rea-prototype-general-classification/`.

**OBSERVED actual saved-script boundary with authenticated inventory:** the
committed `0272965` attached-root probe completes the first manager initializer.
The second (CodeId 1, tag 383/sprite 34) stops after 130 instructions in its
FocusHandler constructor at SetMember 13632: original Push 13623 supplies
preloaded register 2 (`_root`), `_focusrect`, and Boolean false. The verified
inventory identifies this as standard native member 17, whose setter is not yet
implemented. The original Selection.addListener executes once, retaining listener
116. Class-registration and unknown-native counts remain zero, and root ordinary
storage remains absent. This does not bypass the standard-member gate.

The exact same controlled three-child/raw-state/minimal-global fixture is used;
no TextField, imported-version or full native-global initialization is claimed.
The full 48,544,080-byte owned executable and exact 4,226-byte private inventory
match their independently expected digests; nine committed VM/decoder modules
and both frozen original host snapshots match after compilation/run. Build and
run exit zero with empty stderr, while the VM reports the explicit unsupported
native member. `avm1-attached-manager-init-increment4/receipt.json`, SHA256
`67926731aa7377f276e51001365dffa6fb6b1691ce098913847cb78c6c57fd61`,
retains original source coordinates, operand proof and partial state. This is
separate from the historical parentless Plain-root initializer count.

## Supplied native focus rectangle state

`bind_attached_focus_rect(AttachedClipId, AttachedFocusRectState)` binds a supplied
raw `target_flags_b8` word once. There is no Default. This is native target DWORD
+0xb8, separate from +0x6a event flags and from ordinary MovieClip storage. An
unbound snapshot remains unsupported; it never becomes an invented Null/false.
`attached_focus_rect` returns the optional supplied/current state and bounded
last-write CodeOrigin metadata. Public host writes record no stale interpreter
origin. Existing ClipSpec literals and other adoption APIs are unchanged.

For exact `_focusrect`, the typed case-3 member route now uses proved standard
member 17 before ordinary storage/children/prototype lookup. If an authenticated
name profile is installed it must agree on that exact ID. Read returns native
Null for mask 0, or Boolean(mask == 0x180) for the other masks. Only exact Boolean
false assignment is admitted: `(word & !0x80) | 0x100`, preserving every other bit.
The result is `AttachedWrite::NativeUpdated`, with no ordinary member insertion,
lazy storage allocation, accessor/watch execution or fabricated host callback.
This does not broaden Boolean true/coercion, other display kinds, alias spellings,
or indexed GetProperty/SetProperty host routing.

**VERIFIED static:** `ATTACHED_FOCUSRECT.md` and private
`rea-attached-focusrect/finding-summary.json` (SHA256
`38e23de370824c68b6ae4abf5d4a82660086bf47c033c1504de6a9ed0dafe49c`)
prove receiver-local setter/getter behavior and the complete return-only helper.
No rendering/focus-navigation consumer or native initial word is inferred.

**OBSERVED controlled Rust checks:** two pure state tests cover all focus masks
and preservation/idempotence; integration tests cover unknown snapshots, duplicate
binding, root/child isolation, rejection of other input types, no lazy storage,
ordinary accessor/readonly shadow bypass, unchanged event flags, original code
provenance and zero host dispatch. Supplied test words are explicit synthetic
state, not observed native defaults. The original manager frontier is exercised
separately; no startup result is inferred from these tests.

**OBSERVED original-script execution on a controlled attached root:** committed
`171a67e` now completes all 452 selected manager root-frame initializers, in
serialized order, with 49,244 instructions. This uses the same authenticated
private name inventory, three saved immediate children, explicit event words and
minimal globals as increment 4. The only added host admission is a supplied root
focus word `0x5a5a0180`; it is synthetic state, not a native default or capture.
After initializer 1, the word is `0x5a5a0100`, with last-write origin CodeId 1,
source 13632. Root ordinary storage remains absent at every initializer checkpoint,
and the separate event word stays `0x52000180`. Selection retains one listener
after one add and no removals; the class registry records five calls/revisions.
No unknown native callback is accepted.

The first harness run completed that scope but exited 101 because its inherited
final assertion still required an unsupported stop. Its source, output and failure
receipt remain unchanged. A single reporting-only correction, plus separate
private output paths, produced build/run exit 0 and identical VM/host observations.
`avm1-attached-manager-init-increment5/attempt2/receipt.json`, SHA256
`1b885efc1a439bee819e7b7c2fdc4c635742a7eff230eb372bf1f3d6d525d934`,
retains both attempts and unchanged source/input hashes. This receipt ends before
the containing-frame DoAction, clip handlers or display activation. The precreated
saved-child snapshot and minimal globals do not establish native lifecycle order,
complete global initialization or retail startup parity.

The separate containing-frame diagnostic replays those same initializers in a
fresh process, then runs the one original root-frame DoAction at 5967+518 on the
same in-process VM and hosts. **OBSERVED:** its 33rd instruction stops at
CallMethod 855918, CodeId 139, with `Function method null/primitive thisObject`.
The entry program is CodeId 452. Original code has already assigned the root's
prototype, leaving storage 15888 and prototype 15254; these effects are retained
after failure. Focus/event words, Selection and class-registry state remain
unchanged, and no unknown host callback runs.

**INFERRED operand interpretation:** the original MarvelUtils adapter calls
`Function(cls).apply(mc, null)`, where the frame passes its typed root as `mc`
and `marvel.managers.MenuManager` as `cls`. The frozen FunctionMethod branch
accepts only `Value::Object` receivers; its error also rejects a `DisplayClip`
and does not prove that the actual source supplied a null or primitive receiver.
The probe does not instrument the intrinsic argument list. Native attached
receiver conversion must be established before broadening that gate. No clip
handlers, native implicit sprite construction or additional global bindings are
included in this containing-frame probe.
`avm1-attached-manager-frame-action-increment1/receipt.json`, SHA256
`f4a9e28ae9249cad6c0a491ffc043cfc8bdd9c8ded37fce3c4bdb41d92ac96bc`,
retains the unchanged frozen sources, full initializer prefix, exact call-chain
coordinates and build/run exit 0 with the explicit VM stop.

## Bounded attached receiver for Function.apply

The explicit Apply path now preserves a canonical live `DisplayClip` receiver
for an ordinary ScriptFunction2 with exactly two arguments and an explicit null
or undefined list. It requires an active attached caller, a supported captured
target/global scope, and live receiver/caller/captured targets under the same
adopted root. The shared-root condition is a conservative implementation limit;
it is not presented as a native root-identity comparison. Existing typed identity
admission supplies the coherent target/public-handle association; retirement and
rebinding remain unsupported. No owner or lazy storage is exposed as an ordinary
Object value, and changing `this` does not replace the captured Environment.

`REA_ATTACHED_FUNCTION_APPLY.md` records the **VERIFIED static** tag7-to-owner and
Function2 owner-to-tag7 bridges, including both preloaded and named `this`.
Legacy DefineFunction, captured activations/With scopes, native/Super callables,
TextField receivers, Function.call, array lists and direct calls without an active
caller remain outside this attached extension. Apply remains ordinary invocation;
it does not invent constructor context to satisfy the separate Super-preload gate.

**OBSERVED focused synthetic checks:** both Function2 this routes preserve the
display value while nested execution uses the captured target and restores the
caller context. Unsupported profiles reject without fabricated callbacks or lazy
receiver storage. A Function2 requiring Super still reports that existing gate.
The two new checks and three existing call/apply, Function-conversion and captured
Environment checks pass. Private `avm1-runtime/attached-apply-final-evidence.json`,
SHA256 `3a05b7c62800f829f124c1c1ee8a4468d7100224b89bf7198a54cd4b797b4955`,
retains source/binary hashes and exact filtered results; these checks do not
establish original frame completion or native runtime parity.

**OBSERVED original-script follow-up:** the frozen `e537cad` replay completes the
same 452 initializers and reaches the same original Apply call at 855918 after
33 frame instructions. The attached-receiver gate is passed; invocation now stops
at `super preload requires recorded constructor context`. No MenuManager body
instruction has run yet. Public root storage/prototype, focus/event words,
Selection and class registry match the previous partial result, with no new
host call. Private `avm1-attached-manager-frame-action-increment2/receipt.json`,
SHA256 `dba8037690bc67af74debbe28cdef158cbc742cc4038e9139e5680a8da3e0f70`,
retains source/input hashes, unchanged setup/action selection and build/run exit 0
with that explicit VM error.

## Ordinary applied superclass invocation

Apply now carries a separate internal attached invocation context. Its initial
prototype cursor comes from the actual receiver owner. When the Function2 flags
require Super, `applied_super.rs` validates that selected cursor against the
recorded Extends prototype and reads its exact own hidden constructor. Each fresh
opaque Super reference retains that callable, the unchanged display receiver and
the next parent-prototype cursor. A nested unnamed zero-argument call advances
only the cursor; it does not alter the receiver's prototype or allocate a new
display instance. The existing NewObject/NewMethod construction context remains
separate.

`REA_APPLIED_SUPER.md` provides the **VERIFIED static** ordinary invocation,
null-method-hint, receiver/prototype separation and nested forwarding contract.
It also proves the terminal original native MovieClip callback returns Undefined
for the attached owner's category 2, without substituting category-11 storage or
writing receiver properties. No host success stub is involved.

Initial admission is deliberately narrower than native lookup: supported
target/global-only Function2 classes, exact original data links and flags-3
hidden/prototype metadata, a nonnull parent, and ordinary Function2 or canonical
MovieClip base. Metadata mutations, missing/marker/deferred/accessor fallback,
ambiguous recorded prototypes, non-Undefined method names, arguments and named
super methods remain unsupported. Unrelated prototype accessors are not evaluated
or rejected by these exact own-data checks. The actual cursor is checked before
using the class relation; a cached class relation never replaces receiver lookup.

**OBSERVED synthetic validation:** three helper checks cover cursor progression,
fresh wrapper identities, metadata rejection before allocation, captured-scope and
native-parent gates, and heap budgets. An opcode-level two-class Apply check
observes the same typed receiver at both script levels, Undefined from the native
terminal, preserved captured context and unchanged receiver prototype/storage.
The two existing attached Apply checks and both existing constructor-super and
typed MovieClip allocation checks also pass: eight focused checks total.
`avm1-runtime/applied-super-checks.json`, SHA256
`f20d1f29d9413e777a32362859daa47ae1c110b036e89ebfb605869c17de739f`,
retains final source/binary hashes and filtered output. Original frame progression
is measured separately; these checks do not establish native lifecycle parity.

## Attached own-property flags after applied startup

**OBSERVED original-script diagnostic:** frozen `e41ceea` again completed the
452 selected manager initializers, then executed 154 instructions of the original
containing-frame action. The first stop moved to CodeId 2, CallMethod at source
17373, inside `gfx.events.EventDispatcher.initialize`. Its retained source builds
`_global.ASSetPropFlags(target, "dispatchQueue", 1)` after assigning target methods
and `dispatchQueue`. The previous primitive-receiver gate rejected the typed
display reference. The argument flow is source-derived; that run did not capture
the property's actual value or previous flags, so neither is assumed to be zero.
Private `avm1-attached-manager-frame-action-increment3/receipt.json`, SHA256
`d0affcefd6b9255b5534603938dae30c061ffcd6e920ba4353fc2966fc1a04e0`,
retains the unchanged fixture/action selection, frozen inputs and exact failure.
The controlled raw target flags, complete saved child snapshot, selected
initializer order and minimal globals still limit this result; it is not retail
startup or event-delivery evidence.

The VM now admits this bounded attached flags route: an active live same-root
Environment with a known loaded version 7..19, an existing separate bound storage
object, exactly one ASCII identifier naming an existing own ordinary data slot,
and the three arguments with set mask 1. `REA_ATTACHED_PROPERTY_FLAGS.md` grounds
the **VERIFIED static** owner-interface-to-storage raw lookup and flag update.
This path does not read named children or inherited members and does not invoke
normal getters, setters or event hooks. The property's value remains unchanged;
the resulting flags are the previous byte OR 1. The loaded active Environment
determines the omitted clear mask. The named CallMethod result remains Undefined.

Missing storage remains unsupported because the native hook can allocate it.
Missing/inherited slots, accessors, special metadata, TextField receivers, broad
selectors, other masks and direct calls without an active target remain explicit
boundaries. Bound storage is never exposed as an ordinary script object. The
same-root admission is an implementation restriction, not a native root-equality
test inferred from the call path.

**OBSERVED focused checks:** two new checks cover existing own Undefined data
with prior ReadOnly flags, inherited accessor isolation, retained value/identity,
no storage creation on rejection, special-metadata/selector/context/type gates
and zero host calls. Two existing flag checks and both attached Apply checks also
pass. Final source/binary hashes and all six results are in
`avm1-runtime/attached-flags-final-checks.json`, SHA256
`80c7d3630f1c88a9424eb087830bba27a86d91238db283936a07cd0dc04f1b5e`.
These synthetic checks do not establish original-frame completion.

**OBSERVED next original-frame diagnostic:** frozen `5734d6b`, with the same
452-initializer selection and fixture, reaches 159 frame instructions. Read-only
public snapshots show root `dispatchQueue` changing from Missing/no storage to
Present(ObjectId 139) in storage 15888, with own flags 1. The first new stop is
CodeId 15, SetMember at 60254 in `_global.MarvelObject`'s constructor: original
`this.tabEnabled = false` after its superclass call. The authenticated native
name profile identifies standard member 29; its setter is still unsupported.
Focus/event state, Selection and class-registry observations are unchanged, and
there are no unknown native calls. Private
`avm1-attached-manager-frame-action-increment4/receipt.json`, SHA256
`edbe93eefca2f22f283102bcc1596c31fc0dc4dc9bd44034d6ef12cd867cd67e`,
retains all frozen inputs and build/run exit 0 with that explicit VM stop.

## Dedicated saved-action execution

The saved-action APIs and current scope are documented in
`SAVED_ACTION_STATE.md`, with native frame/source contracts in
`SAVED_EVENT_EXECUTION.md` and `SAVED_CLIP_ACTION_LOADING.md`. Installed wrappers
retain independent source pools; adopted typed-owner Environments retain their
supplied operand vector and four registers. The first bounded executor runs pure
Construct/Load mode4 with explicit raw state, distinguishes a bit4 skip, and
halts on returned body errors after retaining partial state. It does not replace
the generic frame diagnostic or claim that parsed ClipAction records have been
delivered. Native event selection, eligibility, ordering, same-installation mask
splitting and actual source-to-owner association remain host responsibilities.

## Explicit attached TextField snapshot and native member seam

`adopt_attached_text_field(AttachedTextFieldSpec)` accepts a caller-established
live case-4 snapshot. It creates a distinct `AttachedTextFieldId` and owner
ObjectId; `Value::DisplayTextField` retains display-reference identity. Source
movie/EditText ID/header version, parent clip, supplied opaque native prototype,
raw target flags and watch state remain explicit. This API neither executes the
native constructor nor establishes placement/lifecycle effects. It never creates
a generic Object prototype for the field. `AttachedChild::TextField` preserves
this type in the complete named-child snapshot; own storage, including a present
Undefined, still shadows the child.

`attached_text_field_info` exposes source, parent, environment_owner, target,
target_flags_6a, prototype, watch_state, owner and optional storage directly.
The first parent with target flag 0x400 must match the supplied environment owner.
That ancestry is checked again on each request, since supported clip state updates
can change the marker. The owner, receiver target and active script context are
three separate facts. `TextFieldMemberRequest` additionally carries the inherited
environment owner, original CodeOrigin/source offset and exact member.

Only case-sensitive `text`, `html` and `htmlText` member operations are admitted.
`Host::get_text_field_member` and `Host::set_text_field_member` reject by default.
The integrating host must bind actual Scene/source identities and explicit native
backend state. Html uses exact Bool; text/htmlText use bounded valid NUL-free
UTF-16 Strings. The VM does not normalize text or fabricate native defaults.
Unknown/present watch state rejects writes. Admitted writes allocate separate
category-13 storage before calling the backend; backend rejection preserves this
allocation and spent host-call budget. Engineering admission failures, including
exhausted budgets, occur before allocation. Reads do not allocate storage and
charge returned String units. All other field members, ordinary-object conversion,
raw property inspection and use of the field as an execution Environment remain
explicitly unsupported.

**VERIFIED static:** the case-4 owner, lazy storage, actual parent Environment and
member-slot contracts are grounded in `TEXTFIELD_RUNTIME.md` and its private
`textfield-runtime/rea-attached-identity-member-bridges.json` and
`rea-attached-text-accessors.json` evidence. In particular native receiver identity
selects the field; the active caller does not substitute for it. This remains an
adoption API, since the complete native construction hooks have not been modeled.
The supplied opaque prototype is a host identity, not an invented VM object.

A separate narrow native comparison proof admits live display tag 7 versus
Undefined or Null under Equals2/StrictEquals, in either order: the result is false
without comparison-side display resolution or primitive coercion. It does not
admit display/display equality or general display conversions. The opcode still
performs its ordinary operand cleanup. The privately retained
`avm1-reference/textfield-null-equality.json` (SHA256
`e9186d2c73e37d2586bf9a5e36ed688590b60084a64b5c527fc6d85527502801`)
independently checks 117 native instructions/392 bytes and the original manager
method's first field-versus-Undefined comparison at 94126. Its saved source is
`MarvelGame-frontend_manager.gfx`, SHA256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.

**OBSERVED controlled Rust checks:** six module tests cover identity, ancestry,
watch/context guards, budgets, exact Unicode values and backend-error storage
retention. Two bytecode integration tests cover request receiver/environment/
context separation, child shadowing, prohibited owner coercion, and both display
kinds versus both null-like primitives in both operand orders. They use explicit
synthetic same-version snapshots and a controlled routing host; they do not claim
native text parsing or actual login execution. Results and tested source hashes
are retained in `avm1-runtime/textfield-bridge-evidence.json`.

The original adoption entry points retain their same-version gates. The additive
`create_attached_movie_clip_with_binding(spec,
AttachedMovieBinding::LocalSourceOrImportedLocalExport)` accepts the separately
proved bounded route for versions 7..19. The host must establish a successful,
eligible local resource or ordinary imported local export whose **final** selected
resource-binding pair is unchanged. A later native resource-list substitution,
custom alias or failed definition is outside the witness. A source filename or
function-definition origin is not proof of that binding.

`AttachedInfo` exposes `environment_movie`, `environment_header_version` and
optional `binding` separately from `source`. These identity roles are equal only
under the admitted binding profile; the native constructor itself stores them
independently. The selected target's loaded-version map retains its actual value,
and `bind_loaded_movie_target` rejects reassignment to a different header version.
No code-origin or placing-parent version is substituted for it. Controlled Plain
targets retain the VM-version admission rule. Field owners and storage cannot be
registered as independently owned execution Environments.

`adopt_attached_text_field_from_parent_movie(spec)` is the additive local EditText
snapshot route. It verifies the source movie/header against the actual immediate
parent's selected MovieDef, while still independently checking the first 0x400
ancestor against the supplied environment owner. The raw source, selected target
MovieDef, inherited field Environment and captured script-call target are separate
facts. It does not admit arbitrary imported field definitions, run constructor
hooks or infer any state from the method's originating movie. The interpreter's
implemented instruction profile is already restricted to versions 7..19; its
existing version-sensitive ASSetPropFlags default reads the actual selected target
version. No new legacy or per-version language semantics are invented.

**OBSERVED controlled Rust checks:** two focused tests retain imported source11,
target11 and parent/root15 identities, reject legacy API mismatches and unsupported
version ranges, prohibit target-version reassignment, reject an incorrect field
source/parent association, and exercise both distinct active/inherited Environment
orders. Changing the first marked ancestor invalidates a previously supplied field
association. These synthetic checks establish routing and admission behavior, not
actual native import execution. The source/Environment evidence is documented in
`TEXTFIELD_SOURCE_ENV_VERSION.md` and privately retained under `source-version-env/`.
A captured attached script target additionally requires its live eligibility bit7
and clear bit4; unsupported caller-Environment substitution or target resolution
stops explicitly. A focused script-call check retains captured Environment15 with
a receiver bound to movie11, then rejects a bit4 change before the callback.

The 18 passing focused Clip, TextField, loaded-target flag and captured-context checks, together
with tested source hashes, are retained in
`avm1-runtime/source-env-bridge-evidence.json`.
The actual original label update still needs its separately unsupported String
uppercase path; this increment does not claim completed login or native lifecycle
parity.

Source licence/provenance for this increment: the two original project runtime
and API studies above, the privately retained owned Gazillion/Marvel executable
and GFX movie (proprietary, compatibility study only), and retained REA/Rizin
static outputs (REA MIT/Rizin LGPL tooling). No extracted bytecode, assets or
native member inventories are included in source or builds.

## Exact initialization timing reference

Adobe SWF19, Chapter 5, SWF 6 action model, DoInitAction, printed page 108
([PDF page 110](https://open-flash.github.io/mirrors/swf-spec-19.pdf#page=110)) ties
initialization to the containing frame's first encounter, not an additional
first-placement gate. Same-frame initializers run in serialized order before
normal frame actions and before implicit ActionScript sprite-object creation.
Later visits to that frame skip them; one DoInitAction is allowed per sprite per
SWF. This should not be expanded into a claim about every display placement phase.

The detailed Adobe v9 section, printed pages 126–127
([PDF reference](https://www.mobilefish.com/download/flash/swf_file_format_spec_v9.pdf#page=126)), agrees.
Its overview at page 28 also describes encounter-time processing, leaving precise
streaming/load-phase reconciliation outside these paragraphs. Neither detailed
section establishes reload/reset lifecycle or a per-loaded-movie ledger key.
Those remain scheduler design questions requiring additional evidence. A flattened
structural tag/action inventory is not runtime order. Private
`doinit-timing-and-registration-private.json` records exact sections, observed
manager/login root-frame distributions and registration dependencies.

## Source and licence ledger

| Source | Licence/rights | Use |
| --- | --- | --- |
| [Adobe SWF19 specification](https://open-flash.github.io/mirrors/swf-spec-19.pdf) | © 2006–2012 Adobe, all rights reserved | Published AVM1 operation, function, register and stack descriptions; study only. |
| [ECMA-262 edition 3](https://ecma-international.org/wp-content/uploads/ECMA-262_3rd_edition_december_1999.pdf) | Published ECMA standard; no implementation licence asserted | Sections 9.4–9.6, 11.6.1, 11.8.5 and 11.9: integer conversions and newer comparison/addition algorithms referenced by Adobe. |
| [Adobe AS2 global functions](https://open-flash.github.io/mirrors/as2-language-reference/global_functions.html) and [constants](https://open-flash.github.io/mirrors/as2-language-reference/constants.html) | © 2004–2008 Adobe, all rights reserved | Version-specific primitive conversion cautions and the global Array creation forms. |
| [Adobe compiler migration warnings, AIR SDK mirror](https://airsdk.dev/reference/actionscript/3.0/compilerWarnings.html) | Adobe API documentation, copyrighted; maintained mirror | Warning 1039 documents AS2/AS3 empty/whitespace numeric conversion differences; these cases remain unsupported here. |
| [Adobe SWF v9 specification, authored PDF mirror](https://www.mobilefish.com/download/flash/swf_file_format_spec_v9.pdf) | © 2006–2008 Adobe, all rights reserved | Cross-check Extends `__proto__` / `__constructor__` fields obscured in extracted v19 text. |
| [Adobe AS2 Object reference](https://open-flash.github.io/mirrors/as2-language-reference/Object.html) and [Function reference](https://open-flash.github.io/mirrors/as2-language-reference/Function.html) | Adobe copyrighted documentation, all rights reserved | Object construction and prototype behavior; registerClass and function call/apply contracts for remaining bootstrap work. |
| [Adobe AS2 super statement](https://open-flash.github.io/mirrors/as2-language-reference/statements.html#super) | © 2004–2010 Adobe, all rights reserved | Constructor-super invocation/receiver semantics, combined with SWF19 preload flags and NewObject/NewMethod behavior; no general method-super implementation inferred. |
| [Adobe AS2 Array reference](https://open-flash.github.io/mirrors/as2-language-reference/Array.html) | © 2004–2010 Adobe, all rights reserved | Array/Object inheritance and bounded documented constructor forms. Prototype-specific ECMAScript behavior is not assumed to be AS2 behavior. |
| [Macromedia Learning AS2](https://open-flash.github.io/mirrors/learning-as2.pdf) | © 2005 Macromedia, all rights reserved | Cast behavior study; Array conversion takes precedence over an identity cast. The native-backed Function conversion is separately constrained by owned REA evidence. |
| [Autodesk VisitMembers](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/01886.html) and [cursor example](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/as2_reference/mouse_extensions/changing_cursor.html) | Autodesk proprietary documentation | ASSetPropFlags usage context; native implementation decisions use the cited owned REA evidence. |
| Owner's pinned retail Win64 executable; exact path/hash in the cited REA bundles | Proprietary owner-held client, private compatibility study | Native opcode dispatch, zero-depth ActionPop, property-flag transformation/visitor/consumers, metadata, Array prototype state, Function conversion/apply, typed MovieClip construction/prototype hooks and return-value behavior; static REA evidence only. |
| Owner's private frontend-manager and login GFX bytes | Proprietary retail material, private compatibility study | Original-script isolated probes and actual unsupported-dependency measurements. |

The integrating lead owns additions to `docs/PROVENANCE.md`. This lane changes
only this document, the VM module and its coordinated `avm1.rs` module export.
