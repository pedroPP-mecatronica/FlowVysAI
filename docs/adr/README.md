# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for FlowVysAI.

## What is an ADR?

An ADR is a short document that captures a significant architectural or
technology decision, the context in which it was made, the alternatives that
were considered, and the consequences of the choice.

## Process

1. When a significant decision must be made, open a **Research** issue using
   the template in `.github/ISSUE_TEMPLATE/research.yml`.
2. Pick the next available number and create `docs/adr/XXXX-<slug>.md`.
3. Fill in all sections (see template below).
4. Include the ADR file in the same PR that implements the decision.

## Template

```markdown
# XXXX — Title

**Status:** Proposed | Accepted | Deprecated | Superseded by XXXX

## Context

Why does this decision need to be made?

## Decision

What was decided?

## Alternatives Considered

| Alternative | Reason rejected |
|-------------|-----------------|
| …           | …               |

## Consequences

What becomes easier or harder as a result of this decision?

## References

- Links to crates, papers, benchmarks, RFCs, etc.
```

## Index

| ID   | Title                                          | Status   |
|------|------------------------------------------------|----------|
| 0001 | Case configuration format — TOML + serde       | Accepted |
| 0002 | Binary STL parser — memmap2 + nom              | Accepted |
| 0003 | CI matrix — Windows + Ubuntu, fmt/clippy/test  | Accepted |
