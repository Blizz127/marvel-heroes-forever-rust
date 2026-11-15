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

`execute(code, target, host)` runs an action stream. `call(callable, this, args,
host)` invokes an already created function. `Execution` reports the return value,
remaining root stack, instruction count and host-call count. `get`, `set`,
`object`, `native`, `global` and `root` support explicit integration/setup.

Errors include the source movie offset, opcode and a typed reason. An error
stops execution immediately; earlier writes remain visible. Execution is not
transactional, and retrying a stream is not an automatic recovery strategy.
`instruction_count()` remains available after failure.

`Value` distinguishes undefined, null, Boolean, double, UTF-16 string and object
identity. Objects have data properties and an explicit prototype. Script functions
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
Arrays are capped at one million elements and prototype traversal at 256 links.
Configured function depth greater than 128 is rejected. Pool references,
registers, branches, object IDs and stack accesses are checked. Loops consume the
same instruction budget as straight-line code.

This increment has no garbage collector. Activation/argument objects remain in
the bounded arena, so sustained gameplay will eventually require reclamation.
Host-created Rust values are supplied by trusted integration code; the VM's
allocation counters cover its own allocations and installed property names.

Unsupported operations return errors, including Extends, Try/Throw, enumeration,
property flags/accessors, movieclip paths, super and parent preloads, automatic
primitive boxing, and uninstalled standard-library/native functions. Missing
ordinary properties correctly yield undefined; attempting to call that value
stops with `UnboundCall`. The supplied heap does not pretend to contain the
Object, Function, MovieClip, Stage, Mouse or Scaleform libraries.

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
object is absent. A separate fresh-empty-VM run of each of its 468 action tags
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

**VERIFIED:** Nine focused VM tests pass for operand ordering, pool capture,
function calls, later code registration, property/prototype behavior, explicit
native bindings, unsupported paths, shadowing, and instruction/call/stack/heap
budgets. These tests supplement the owned probes; they do not prove retail parity.

## Source and licence ledger

| Source | Licence/rights | Use |
| --- | --- | --- |
| [Adobe SWF19 specification](https://open-flash.github.io/mirrors/swf-spec-19.pdf) | © 2006–2012 Adobe, all rights reserved | Published AVM1 operation, function, register and stack descriptions; study only. |
| [ECMA-262 edition 3](https://ecma-international.org/wp-content/uploads/ECMA-262_3rd_edition_december_1999.pdf) | Published ECMA standard; no implementation licence asserted | Sections 9.4–9.6, 11.6.1, 11.8.5 and 11.9: integer conversions and newer comparison/addition algorithms referenced by Adobe. |
| [Adobe AS2 global functions](https://open-flash.github.io/mirrors/as2-language-reference/global_functions.html) and [constants](https://open-flash.github.io/mirrors/as2-language-reference/constants.html) | © 2004–2008 Adobe, all rights reserved | Version-specific primitive conversion cautions. |
| [Adobe compiler migration warnings, AIR SDK mirror](https://airsdk.dev/reference/actionscript/3.0/compilerWarnings.html) | Adobe API documentation, copyrighted; maintained mirror | Warning 1039 documents AS2/AS3 empty/whitespace numeric conversion differences; these cases remain unsupported here. |
| Owner's private frontend-manager and login GFX bytes | Proprietary retail material, private compatibility study | Original-script isolated probes and actual unsupported-dependency measurements. |

The integrating lead owns additions to `docs/PROVENANCE.md`. This lane changes
only this document, the VM module and its coordinated `avm1.rs` module export.
