# 0001 — Case Configuration Format: TOML via serde

**Status:** Accepted

## Context

FlowVysAI simulations are parameterised by many values: domain dimensions,
fluid properties, time-step count, input geometry path, and (in future sprints)
thermal and environmental parameters.  A structured, human-readable file format
is needed so that:

- simulations are reproducible without re-compiling,
- users can share and version-control case files,
- the CLI can validate inputs before starting a long run.

The format must be:

- easy to read and write by engineers who are not software developers,
- natively supported by the Rust ecosystem,
- cross-platform (Windows & Linux),
- extensible as new physics modules are added.

## Decision

Case configuration files use **TOML** (`*.toml`) parsed by the
[`toml`](https://crates.io/crates/toml) crate via
[`serde`](https://crates.io/crates/serde) derive macros.

The workspace `Cargo.toml` already lists `serde` as a workspace dependency.
The `toml` crate will be added when the case-file feature is implemented
(Sprint S3).

A minimal case file looks like:

```toml
[domain]
nx = 100
ny = 50
nz = 1        # 1 = 2-D simulation

[fluid]
viscosity  = 0.01
density    = 1.0

[simulation]
steps      = 5000
output_dir = "results/"

[geometry]
stl_path   = "cases/geometry/naca0012.stl"
voxel_res  = 0.01
```

## Alternatives Considered

| Alternative | Reason rejected |
|-------------|-----------------|
| JSON        | No comments; verbose for nested numeric data |
| YAML        | Indentation-sensitive; historically quirky parsers; no first-class Rust derive story |
| RON (Rusty Object Notation) | Non-standard; unfamiliar to engineers without Rust background |
| Custom INI  | Not expressive enough for nested sections (thermal, environment) |

## Consequences

- **Positive:** Engineers already familiar with `Cargo.toml` find the syntax
  intuitive.  Tables map cleanly to Rust structs via `serde`.  Comments are
  supported.
- **Positive:** `toml` crate is `no_std`-compatible and has zero unsafe code.
- **Neutral:** Adding `toml` increases compile time by a few seconds.
- **Watch-out:** TOML arrays of tables (`[[env.conditions]]`) can feel verbose
  for large parameter sweeps; a CSV/script layer may be added later for that
  use-case.

## References

- [`toml` crate](https://crates.io/crates/toml)
- [`serde` crate](https://crates.io/crates/serde)
- [TOML spec v1.0](https://toml.io/en/v1.0.0)
