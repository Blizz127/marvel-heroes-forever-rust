# Prototype catalogue

`PrototypeCatalog::open(path)` indexes an explicit private SIP package, loads its
prototype directory and blueprint metadata, and checks declared prototype paths
and IDs. It does not pre-load all prototype bodies. The directory must refer to
members that actually exist in that archive, and IDs/member paths must be unique.

`catalog.reader()` creates one reusable schema index and borrows the archive for
a batch. `read_member`, `read_id` and `read_index` decode one requested prototype.
`verify_all` walks every directory entry and returns structural counts. The
catalogue's contents and source variant are selected by the caller's path; no
server-side or install-side file is silently preferred or modified.

Names are exact archive identifiers with the `Calligraphy/` prefix; they are
never treated as extraction destinations. The byte, nesting and schema bounds
are inherited from the SIP and prototype readers. Error messages identify the
member whose parsing failed.

The catalogue does not merge parent prototype values or perform runtime class,
property, replacement-GUID or game-reference binding. Its count of nested nodes
is a serialized-structure count, not a count of instantiated game entities.

CLI examples:

```sh
mhf prototype verify /private/path/Calligraphy.sip
mhf prototype inspect /private/path/Calligraphy.sip 0xf8ffa9c3a5912250
```

Focused integration receipts are retained privately alongside the build records.
See [prototype format](PROTOTYPE_FORMAT.md) for parser evidence and provenance.
