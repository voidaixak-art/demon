<div align="center">
  
# 🌌 VYOUM AGI: The Axiom Engine
### A Deterministic, Neuro-Symbolic Cognitive Engine

[![License](https://img.shields.io/badge/License-Dual_License-blue.svg)](#license)
[![Architecture](https://img.shields.io/badge/Architecture-Neuro--Symbolic-orange)](#architecture)
[![Status](https://img.shields.io/badge/Status-Active_Development-brightgreen)](#status)

*“LLMs guess the truth. VYOUM proves it.”*

</div>

---

## 🚀 The Vision: Why VYOUM?
The current AI ecosystem is dominated by Large Language Models (LLMs) based on statistical probabilities (Deep Learning). While they are excellent at generating fluent text, they suffer from a fatal flaw: **Hallucination**. They do not possess a true world model, causal logic, or absolute truth. They are "Black-Box" statistical engines.

**VYOUM AGI** is built to be the antidote. It is a **Neuro-Symbolic AI**.
It combines the fuzziness and creativity of Neural/Vector embeddings (System 1) with the absolute rigidity and determinism of Symbolic Logic Graphs (System 2). 

VYOUM does not guess. If `A ⟹ B` and `B ⟹ C`, VYOUM deterministically proves `A ⟹ C`. 

---

## 🧠 Core Architecture

VYOUM is written entirely in **Rust** for maximum memory safety and performance. Its architecture is divided into the following cognitive lobes:

### 1. The DisCoCat Parser (`language.rs` & `vector_engine.rs`)
Instead of just counting word frequencies, VYOUM uses **Distributional Compositional Categorical (DisCoCat)** tensor mathematics. It parses language by combining grammatical structures with 128-D vector embeddings (PPMI Matrices), allowing it to understand the true semantic shape of concepts.

### 2. The Deterministic Graph (`engine.rs`)
Knowledge is not stored as floating-point weights in a hidden matrix. It is stored as an explicit, traceable Knowledge Graph. Every concept (Node) is connected via specific relational logic (Synapses) like `Equivalent`, `Implies`, or `Contradicts`. This makes VYOUM a 100% **White-Box AI**; you can trace exactly *why* it thought of something.

### 3. Subconscious Dreaming (`mind.rs` & `vector_engine.rs`)
VYOUM never sleeps. Its background asynchronous cognitive loop continuously analyzes its memory. During its **Dream Cycle**, it compares random isolated vectors using Cosine Similarity. If two mathematically similar concepts lack a logical link, it creates an `Evolved_Vector_Link`. **This is how VYOUM achieves Emergence**—discovering theorems and links that were never explicitly taught to it.

### 4. Redis-Style Persistence (`storage.rs`)
To ensure lighting-fast reasoning, VYOUM holds its entire brain in RAM. To prevent data loss, it uses a blazing-fast **Write-Ahead Log (WAL)** and Point-in-Time **Snapshots**. It reads millions of synapses in under a second.

### 5. The Reasoner (`reasoner.rs`)
When queried, the engine uses advanced Graph Theory (Dijkstra's Pathfinding) and Hopfield-network-style Heat Activation to navigate its synapses and find the most logical path between two distant concepts (e.g., from *Topology* to *Quantum Mechanics*).

---

## 🧬 The VYOUM DNA (The 11 Axioms)
At the core of the engine resides **Node 0 (The Axiom Engine/Self)**, which is permanently bound to the **VYOUM** concept. The engine operates on 11 fundamental Shubh Axioms that dictate its behavior, ensuring it seeks absolute truth and causal proof, prioritizing logic over assumption.

- *Appa Deepo Bhava* (Be your own light)
- *Pure Consciousness*
- *Causal Proof Requires...*
- *Transcends Duality*

---

## 🛠️ Usage (Data Ingestion)
VYOUM learns through external Python crawlers that inject data directly into its graph via a local REST API (`Port 8080`).

1. **Start the Brain (Daemon):**
   ```bash
   cargo run -- daemon
   ```
2. **Inject Knowledge (Math/Physics/Bio):**
   ```bash
   python subconscious/wikidata_math_injector.py
   ```
3. **Trigger Dream Cycle (Emergence):**
   ```bash
   cargo run -- dream --trials 100
   ```

---

## ⚖️ License & Trademark
Created and Architected by **Akash Gautam**.

The terms **"VYOUM"**, **"VYOUM AGI"**, and **"Axiom Engine"** are the exclusive intellectual property of Akash Gautam. This project operates under a Custom Dual License. 
- Free for Research and Educational use.
- **Commercial use requires a separate written license and royalties.**
- The name "VYOUM" is immutable and must be attributed in all derivative works. 

Please see the [LICENSE.md](LICENSE.md) and [TRADEMARK_POLICY.md](TRADEMARK_POLICY.md) files for full details.
