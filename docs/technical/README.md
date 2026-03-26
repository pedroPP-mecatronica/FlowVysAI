# Technical Documentation

> **English** | [Português abaixo ↓](#documentação-técnica)

---

This section targets **developers and researchers** working on or studying the AeroCore engine internals.

## Contents

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System architecture, crate layout, module responsibilities, design decisions (SoA, zero-alloc, etc.) |
| [Mesh I/O](mesh-io.md) | STL parser (ASCII + binary), `SoaMesh` data structure, voxelization pipeline, file formats |
| [Solver Roadmap](solver-roadmap.md) | Sprint-based implementation plan: D2Q9 → D3Q19 → thermal coupling |

## Key Concepts

- **Data-Oriented Design (DOD)**: all hot-path data is stored in Structure-of-Arrays (SoA) layout for SIMD and cache efficiency.
- **Zero-cost abstractions**: Rust generics and traits compile down to zero-overhead machine code.
- **Cross-platform**: `cargo build` works identically on Windows and Linux.
- **Memory strategy**: arena allocators (`bumpalo`) for per-timestep temporaries; pool allocators (`slab`) for stable entity handles; `memmap2` for large file I/O.

## How to Navigate

Start with [architecture.md](architecture.md) for the big picture, then dive into [mesh-io.md](mesh-io.md) for the I/O layer, and [solver-roadmap.md](solver-roadmap.md) for the numerical core.

---

---

# Documentação Técnica

> [English above ↑](#technical-documentation)

---

Esta seção é destinada a **desenvolvedores e pesquisadores** que trabalham ou estudam os internos do motor AeroCore.

## Conteúdo

| Documento | Descrição |
|-----------|-----------|
| [Arquitetura](architecture.md) | Arquitetura do sistema, layout dos crates, responsabilidades dos módulos, decisões de design (SoA, zero-alloc, etc.) |
| [I/O de Malha](mesh-io.md) | Parser STL (ASCII + binário), estrutura de dados `SoaMesh`, pipeline de voxelização, formatos de arquivo |
| [Roadmap do Solver](solver-roadmap.md) | Plano de implementação por sprints: D2Q9 → D3Q19 → acoplamento térmico |

## Conceitos Chave

- **Design Orientado a Dados (DOD)**: todos os dados em caminhos quentes são armazenados em layout Structure-of-Arrays (SoA) para eficiência SIMD e de cache.
- **Zero-cost abstractions**: generics e traits do Rust compilam para código de máquina sem overhead.
- **Multiplataforma**: `cargo build` funciona identicamente no Windows e no Linux.
- **Estratégia de memória**: alocadores arena (`bumpalo`) para temporários por passo de tempo; alocadores pool (`slab`) para handles de entidades estáveis; `memmap2` para I/O de arquivos grandes.

## Como Navegar

Comece com [architecture.md](architecture.md) para uma visão geral, depois mergulhe em [mesh-io.md](mesh-io.md) para a camada de I/O e [solver-roadmap.md](solver-roadmap.md) para o núcleo numérico.
