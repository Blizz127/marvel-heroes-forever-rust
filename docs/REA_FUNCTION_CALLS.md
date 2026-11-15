# Native Function conversion and apply argument handling

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED, static native control flow:** The pinned client's AS2 global
`Function` accepts an existing callable as its single argument and returns a
reference to the same underlying function object. Its native `apply` forwards
zero arguments for an explicitly null or undefined argument list. The supplied
object receiver is converted to an object interface and passed into the
forwarded call context. These findings support the actual manager bootstrap;
they are not observations of a running retail client.

## Owned call site

**OBSERVED:** The manager function at movie offset 855859 has body
855879 + 41 bytes and parameters `mc` and `cls`. It first assigns the object's
prototype from `cls.prototype`, then uses **CallFunction** at 855912 to resolve
and call global `Function` with `cls`. It calls the result's `apply` method at
855918 with `mc` and explicit null, then pops the result at 855919.
This is a named global call, not ActionCastOp `0x2b`.

The private source is
`evidence/frontend-startup/MarvelGame-frontend_manager.gfx`, 2,055,748 bytes,
SHA-256 `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
The 41-byte body has SHA-256
`3f206a92aed74019c2c1d37e81e7429d60399c5f8371bd91a85acb794db1b7dd`.
The existing original decode is
`evidence/avm1-runtime/manager-registerclass-private.txt`.

## Identifying the native global and prototype methods

**OBSERVED:** Global setup at `0x14020ea40` constructs a native callable whose
callback field receives `0x1401e17a0` at `0x14020ed65`. That object remains in
`r12`. At `0x14020fae7..0x14020fb4b`, setup installs `r12` as a callable value
on the global object using the interned-name slot at offset `0x170`.

The name table has base offset `0x138`, eight bytes per entry; `0x170` is index
7. Its original-file pointer at `0x142add318` resolves to the literal
`Function` at `0x1423f6670`. REA decodes the table constructor at
`0x140181020`, including its source table address `0x142add2e0`. Name/data
pointer reads are separately retained ordinary PE observations, not mislabeled
as REA's recovered symbols.

The `apply` method-name/function-pointer pair at `0x14250e230` contains the
literal address `0x14250c5c4` and callback `0x14019cf00`. REA reads that literal
and pair directly. Setup passes this table to `0x1401e4110` at `0x14020f126`.
The helper reads each name/callback pair and creates native callable members.
This establishes the AS2 method binding; other similarly named AS3 tables in
the executable were excluded.

## Single-argument Function conversion

**VERIFIED, static:** At `0x1401e17aa`, the callback requires exactly one
argument for this conversion path. It accepts internal value tags `8` and
`11` and resolves them through `0x140199640`. For a direct function value,
that helper returns its stored object pointer. The callback passes the result
to `0x1401936f0`, which recognizes native object category `0x17` and obtains
its function reference through the object's virtual slot `+0x60`.

The regular function-reference getter `0x14018c580` returns the same function
object pointer. `0x14018de50` writes the resulting reference as value tag `8`.
Thus the underlying callable object is preserved; the operation does not
parse the supplied callable's string representation into a new function body.
The other single-argument path writes internal tag `1`, already established
as null by the native Push evidence cited below.

This is deliberately **not a byte-for-byte value-wrapper identity claim**.
The regular getter clears its auxiliary pointer and flags before the result
setter copies the reference. Native and ordinary script-function interface
tables both point to that getter in the inspected pin. The function object's
own stored code/context stays in the same object, but semantics for auxiliary
frame state, specialized reference wrappers and bound native functions were
not fully recovered. A Rust implementation must distinguish object identity
from every native wrapper field.

The zero/multiple-argument branch allocates a different native object and was
not needed by the saved bootstrap. Its complete constructor semantics remain
outside this supported profile. No ECMAScript dynamic-source Function
constructor behavior is inferred for AS2.

## Explicit null/undefined apply lists and receiver

**VERIFIED, static:** The complete `apply` body is
`0x14019cf00..0x14019d379`. It initializes forwarded argument count `r14d` to
zero at `0x14019cf32`. When a second argument is supplied, it resolves that
value through `0x140199640` at `0x14019d045`. That helper returns a null pointer
for internal tags `0` and `1`; `apply` branches directly to its invocation
construction at `0x14019d0db`, leaving the forwarded count zero. Omitted lists
take the same zero-count path. Actual array objects use a separate category
check and element loop; this investigation does not generalize every other
value into a supported argument-list form.

Null/undefined tag identities are grounded independently: native Push type 2
writes internal tag 1 at `0x14020caea`, while Push type 3 writes internal tag 0
on the `0x14020cc82` branch. Adobe's SWF specification names those serialized
Push types null and undefined, respectively. The prior REA evidence is retained
in `evidence/rea-asflags-followup-return/final.bundle.json`, records
`ev_baa1bc0e27483ab8df8eee94de03d3795c651dc840a70de0902a005289939e72`
and `ev_34db8dd8c08af191cc8b0cde05b64c344c1de58af13a877333f738d6c08f0305`.

**OBSERVED:** The first argument is resolved by `0x1401997f0` at
`0x14019cf91`. Ordinary object/function values become their object interface;
the display-object value category has a distinct environment-aware resolution
path. The resulting interface pointer is placed into field `+0x10` of the
forwarded call context at `0x14019d13d` or `0x14019d211`. The target function's
virtual invoke slot is then called with that context and the zero argument
count. A temporary result slot is passed into the invocation and copied to
the original result slot afterward at `0x14019d2ee`.

**INFERRED, supported by Adobe's receiver contract:** That context field is the
callee's `this` receiver. This establishes explicit receiver forwarding for
the manager's `mc`; native script execution, display-object resolution under
every environment, primitive receiver boxing and null receiver fallback were
not tested. Lexical scope, `_root`, and `_global` must not be replaced merely
because `apply` supplies a new receiver.

## Function prototype distinction

**OBSERVED:** Setup constructs a separate prototype object with an Object
prototype parent and installs the apply/call method table there. It inserts
this prototype under built-in index 7 and assigns it to the Function
constructor's prototype slot. Its object-interface table is `0x142515be0`;
the category method `0x1403cecd0` returns `6`. Callable function objects use
the category method `0x1401e1420`, which returns `0x17`.

This supports a distinct shared prototype object between script functions
and Object's methods. It does not establish that Function.prototype itself
is callable, its complete script-visible `typeof` result, or every inherited
descriptor. These questions must not be answered by copying ECMAScript's
Function.prototype construction rules.

## Primary documentation and limits

Adobe's AS2 Function reference documents supplied receiver binding, array
arguments and forwarding of the called function's return value. Its generated
`apply` signature says `Void`, which conflicts with the return-value prose.
It does not explicitly specify null/undefined argument lists. The native
argument-count branch above resolves that particular gap for this pin.

ECMA-262 edition 3, section 15.3.4.3, independently specifies no arguments for
null/undefined lists. That is a comparison, not proof of complete AS2/GFx
conformance. Its Function construction rules are different from the observed
single-callable conversion. Macromedia's AS2 casting chapter describes
identity-preserving successful casts but also warns that named global
conversion functions take precedence. Neither that chapter nor ActionCastOp
alone could establish this saved CallFunction's behavior.

No native client execution, authentication, runtime probe, full VM parity,
exception behavior, primitive receiver behavior, or general constructor
compatibility was validated. This document changes no VM code.

## REA evidence and validation

The inspected owner pin is
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Its version evidence is recorded separately in `evidence/retail-version.json`.

**VERIFIED:** All five target-free REA calls completed with exit zero,
untruncated output and unchanged pre/post digest. Code-only calls used raw
mapping base `0x140000c00`, derived from PE image base plus `.text` RVA minus
file offset, with explicit x86/64 decoding. Every returned instruction's bytes
were compared with the corresponding original PE bytes: 2,112 instruction
records, including overlaps. Bounded trailing partial/invalid decodes were
excluded from claims; one range that began inside an instruction was corrected
by a separate decode from its known boundary. PE unwind fragments were not
treated as complete function boundaries.

Private bundles under `evidence/rea-function-calls/`:

| Bundle | Evidence record |
| --- | --- |
| `apply-entry.bundle.json` | `ev_7316eaaa5f9553e5f89d3ec0dcdeb39bcd416dd6dfef009b4b497f9aec9d1178` |
| `apply-conversion.bundle.json` | `ev_aa26bdd087418e61873edce4f4459c8f5c5037b68e30e7c45fcf7c3ad28faf25` |
| `function-global.bundle.json` | `ev_deae116bb617a3a5f0e66497fe739ffe2defb155ebd25edb0da4a75e6c09f329` |
| `function-binding.bundle.json` | `ev_ceb0b6b4241d0b1b4625bb26bb29866a83de58568e4eac93b099ca54166e5092` |
| `function-context.bundle.json` | `ev_9f5bf00c4c1df56a84c20b3f5d17ee3ba42221f241f64704357a27c9187ec501` |

`finding-receipt.json` records scope, counts, hashes and the saved call span.
Candidate scans and readable instruction annotations remain private. REA's
provider identity/version caveats and external-plugin-input caveats are retained
in each complete bundle. No broad analysis, setup/config changes, or target
launch was performed. Workflow instructions were read from
`~/.agents/skills/reverse-engineer-anything/SKILL.md`.

## Outside-source ledger

All sources were studied on 2026-11-15; no external implementation was copied.
Exact private snapshot lengths/hashes and read scope are recorded in
`evidence/rea-function-calls/docs-research/function-apply-contract.json`.

1. Adobe, [AS2 Function reference](https://open-flash.github.io/mirrors/as2-language-reference/Function.html),
   page dated 2010-04-07; receiver/argument/return contract and Function class
   relationship. © 2004–2010 Adobe, all rights reserved; document study only.
2. Adobe, [AS2 global functions](https://open-flash.github.io/mirrors/as2-language-reference/global_functions.html)
   and [top-level summary](https://open-flash.github.io/mirrors/as2-language-reference/package-detail.html);
   checked class versus global-function documentation coverage. Same copyright
   scope; no implementation licence inferred.
3. Macromedia, [*Learning ActionScript 2.0 in Flash*](https://open-flash.github.io/mirrors/learning-as2.pdf),
   September 2005, printed pages 110–112, casting/global conversion precedence.
   © 2005 Macromedia, all rights reserved; document study only.
4. Adobe, [*SWF File Format Specification*, version 19](https://open-flash.github.io/mirrors/swf-spec-19.pdf),
   Push type mapping, CallFunction/CallMethod, and ActionCastOp distinctions.
   © 2006–2012 Adobe, all rights reserved; format study only.
5. Ecma, [ECMA-262 third edition](https://ecma-international.org/wp-content/uploads/ECMA-262_3rd_edition_december_1999.pdf),
   December 1999, sections 15.3.1–15.3.2 and 15.3.4.3; comparison boundaries.
   Ecma copyrighted standards text under its
   [text copyright policy](https://ecma-international.org/policies/by-ipr/ecma-text-copyright-policy/),
   also read for the older-standard notice. No implementation code reused.
6. Owner's pinned executable and manager movie, hashes above; proprietary
   assets used only for interoperability study. Existing original project REA
   bundles, manager decode, and the class-registry lane's bounded ordinary
   disassembly supplied addresses and independent value-tag anchors.

Adobe/Macromedia references are original-author publications preserved by a
mirror. Game bytes, decoded action bodies and native disassembly remain private
and are excluded from source control and builds.
