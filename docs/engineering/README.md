# Engineering Documentation

> **English** | [Português abaixo ↓](#documentação-de-engenharia)

---

This section targets **mechanical and aeronautical engineers, product evaluators, and clients** who want to understand what FlowVysAI/AeroCore can do, how to use it, and why its results can be trusted.

You do **not** need to understand Rust or the internal architecture to benefit from this section.

## Contents

| Document | Description |
|----------|-------------|
| [Product Vision](product-vision.md) | What the product is, what it does, intended use cases, current capabilities, and honest limitations |
| [Workflows](workflows.md) | Step-by-step simulation workflow — from CAD geometry to results |
| [Validation & Trust](validation-and-trust.md) | How results are validated against known solutions; what "trust" means in CFD |

## Quick Summary

**What it is:** A high-performance open-source CFD solver based on the Lattice Boltzmann Method (LBM), built in Rust.

**Who it's for:** Engineers and researchers who need aerodynamic or thermodynamic flow simulations, especially in contexts where performance, reproducibility, and cross-platform support matter.

**Current state:**  
- ✅ Geometry pipeline: STL mesh loading, bounding-box analysis, voxelization  
- 🔄 2D flow solver (Lattice Boltzmann D2Q9) — in progress  
- 📋 3D solver, thermal coupling, GUI — planned

**How to evaluate:** See [validation-and-trust.md](validation-and-trust.md) for our approach to quantitative validation.

---

---

# Documentação de Engenharia

> [English above ↑](#engineering-documentation)

---

Esta seção é destinada a **engenheiros mecânicos e aeronáuticos, avaliadores de produto e clientes** que querem entender o que o FlowVysAI/AeroCore pode fazer, como usá-lo e por que seus resultados podem ser confiáveis.

Você **não** precisa entender Rust ou a arquitetura interna para se beneficiar desta seção.

## Conteúdo

| Documento | Descrição |
|-----------|-----------|
| [Visão do Produto](product-vision.md) | O que o produto é, o que faz, casos de uso pretendidos, capacidades atuais e limitações honestas |
| [Fluxos de Trabalho](workflows.md) | Fluxo de trabalho de simulação passo a passo — da geometria CAD aos resultados |
| [Validação e Confiança](validation-and-trust.md) | Como os resultados são validados contra soluções conhecidas; o que "confiança" significa em CFD |

## Resumo Rápido

**O que é:** Um solver CFD de alto desempenho de código aberto baseado no Método de Lattice Boltzmann (LBM), construído em Rust.

**Para quem é:** Engenheiros e pesquisadores que precisam de simulações de escoamento aerodinâmico ou termodinâmico, especialmente em contextos onde performance, reprodutibilidade e suporte multiplataforma importam.

**Estado atual:**  
- ✅ Pipeline de geometria: carregamento de malha STL, análise de bounding box, voxelização  
- 🔄 Solver de fluxo 2D (Lattice Boltzmann D2Q9) — em andamento  
- 📋 Solver 3D, acoplamento térmico, GUI — planejado

**Como avaliar:** Veja [validation-and-trust.md](validation-and-trust.md) para nossa abordagem de validação quantitativa.
