# 0002 — Binary STL Parser: memmap2 + nom

**Status:** Proposed — implementation planned for Sprint S2

## Context

STL is the de-facto interchange format for 3-D geometry in engineering tools.
FlowVysAI must ingest STL files (both ASCII and binary) to feed the voxeliser
pipeline introduced in Sprint S1 (PR #3).

Requirements:

- Handle files from tens of KB to several hundred MB without reading the whole
  file into a `Vec<u8>`.
- Work correctly on both Linux and Windows (file-backed memory maps behave
  differently across OS).
- Produce zero-copy slices over the raw bytes where possible to avoid
  allocation in the hot path.
- Remain free of `unsafe` in parser logic (use well-audited crates).

## Decision

The binary STL parser will be implemented in Sprint S2 using:

- **[`memmap2`](https://crates.io/crates/memmap2)** for memory-mapped file
  I/O.  `memmap2` is the maintained fork of `memmap` and is cross-platform.
  It is `unsafe` internally (mapping memory is inherently unsafe), but the API
  is well-audited and widely used.
- **[`nom`](https://crates.io/crates/nom)** for zero-copy byte-slice parsing.
  `nom` combinators (`le_u32`, `le_f32`, `count`) will parse the 84-byte header
  and triangle records directly from the mapped slice without any intermediate
  allocation.

ASCII STL will continue to be handled by a separate code path using standard
`str` scanning, detected by the `solid` prefix.

Both crates will be added to `aerocore_core` as regular dependencies in Sprint S2.

## Alternatives Considered

| Alternative | Reason rejected |
|-------------|-----------------|
| `std::fs::read` (full load) | Reads entire file into heap; impractical for large STLs |
| `std::io::BufReader` + manual parsing | More boilerplate than `nom`; no zero-copy |
| `winnow` (nom successor) | API is still stabilising; team familiarity with `nom` is higher |
| `stl_io` crate | Does not use memory mapping; allocates a `Vec<Triangle>` |
| Custom unsafe pointer arithmetic | Unnecessary given `nom`'s coverage; harder to audit |

## Consequences

- **Positive:** Large STL files are parsed in O(1) memory overhead relative to
  file size (the OS pages in only the accessed regions).
- **Positive:** `nom` combinators are unit-testable with synthetic byte
  buffers without touching the file system.
- **Neutral:** When `memmap2`-based I/O is introduced, any required `unsafe`
  will be confined to a small, well-documented I/O adapter and reviewed on every
  PR that modifies it.
- **Watch-out:** On Windows, a memory-mapped file cannot be deleted or
  truncated while the map is open.  Tests must drop the `Mmap` before cleanup
  (use `drop(mmap)` explicitly or rely on scope).

## References

- [`memmap2` crate](https://crates.io/crates/memmap2)
- [`nom` crate](https://crates.io/crates/nom)
- [Binary STL format spec](https://en.wikipedia.org/wiki/STL_(file_format)#Binary)
- PR #3 — LBM boundary mask + STL voxeliser pipeline
