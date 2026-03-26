# 0003 — CI Matrix: Windows + Ubuntu with fmt / clippy / test

**Status:** Accepted

## Context

FlowVysAI targets academic users running both Linux workstations and Windows
laptops.  Without a CI gate on Windows, platform-specific regressions (path
separators, `memmap2` file-locking semantics, CRLF line endings, Windows
API differences) go undetected until a user reports a bug.

The project also needs a consistent style and lint baseline so that
contributions from students with varying Rust experience can be merged without
manual style reviews.

## Decision

A single GitHub Actions workflow (`.github/workflows/ci.yml`) runs on every
`push` and every pull request, using a **2×1 matrix**:

| Dimension | Values                          |
|-----------|---------------------------------|
| `os`      | `ubuntu-latest`, `windows-latest` |

Each job runs the following steps in order, all with
`working-directory: aerocore` (where the Cargo workspace lives):

1. `cargo fmt --check` — rejects unformatted code.
2. `cargo clippy --all-targets --all-features -- -D warnings` — rejects any
   Clippy warning.
3. `cargo test --all-targets --all-features` — runs all unit and integration
   tests.

The Rust toolchain is installed via
[`dtolnay/rust-toolchain@stable`](https://github.com/dtolnay/rust-toolchain)
with the `rustfmt` and `clippy` components.  Cargo registry and `target/` are
cached with [`actions/cache@v4`](https://github.com/actions/cache) keyed on
`Cargo.lock` to keep CI fast.

## Alternatives Considered

| Alternative | Reason rejected |
|-------------|-----------------|
| Linux-only CI | Does not catch Windows-specific regressions |
| `macOS` added to matrix | Adds CI minutes with little value for target audience; can be added later |
| `cargo deny` / `cargo audit` | Valuable, but out of scope for initial CI setup; add in a follow-up |
| `nightly` toolchain | Nightly can break unexpectedly; `stable` is sufficient |
| `actions-rs/toolchain` | Deprecated; `dtolnay/rust-toolchain` is the community-maintained replacement |

## Consequences

- **Positive:** Pull requests cannot merge with formatting violations or Clippy
  warnings.
- **Positive:** Windows regressions are caught before merge, not after a user
  reports them.
- **Positive:** Cargo cache keeps CI under ~2 minutes per job after the first
  cold run.
- **Neutral:** Contributors must run `cargo fmt` and `cargo clippy` locally
  before pushing.  The PR template checklist reminds them.
- **Watch-out:** `fail-fast: false` is set so that both OS jobs always
  complete, giving full visibility into cross-platform failures in a single CI
  run.

## References

- [`dtolnay/rust-toolchain` action](https://github.com/dtolnay/rust-toolchain)
- [`actions/cache` action](https://github.com/actions/cache)
- [GitHub Actions matrix strategy](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)
- `.github/workflows/ci.yml` (this repository)
