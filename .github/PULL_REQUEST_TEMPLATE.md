## Summary

<!--
Briefly describe what this PR does and why.
Link the related Epic issue (e.g. "Closes #<issue>").
-->

## Definition of Done

Please confirm every item is checked before requesting review:

- [ ] `cargo fmt --check` passes (run `cargo fmt` if needed)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all-targets --all-features` passes on **Linux**
- [ ] `cargo test --all-targets --all-features` passes on **Windows** (CI matrix)
- [ ] New public functions/types have doc comments (`///`)
- [ ] Hot-path solver code does **not** introduce heap allocations (verify with tests or comments when applicable)
- [ ] Related ADR added or updated under `docs/adr/` (if this PR makes a significant technology/architecture decision)
- [ ] `docs/ROADMAP.md` sprint checklist updated if a deliverable is complete
- [ ] No unrelated refactors included

## Changes

<!--
List the main files changed and what changed in each.
-->

## Test Plan

<!--
Describe how the new behaviour was tested:
- unit tests added/modified
- integration test scenarios
- manual steps to reproduce on Windows
-->

## Windows Considerations

<!--
Describe any Windows-specific paths, line endings, file permissions, or API
differences that were taken into account.  Write "N/A" if none.
-->

## Screenshots / Benchmark Results

<!--
Add screenshots, MLUPS numbers, or CSV/VTK output samples if applicable.
-->
