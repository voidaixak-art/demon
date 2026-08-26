use crate::engine::AxiomEngine;
use crate::relations::Relation;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub source_id: u64,
    pub source_concept: String,
    pub target_id: u64,
    pub target_concept: String,
    pub similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReport {
    pub total_trials: usize,
    pub successful_synapses: Vec<MutationResult>,
    pub consolidated_orphans: usize,
}

pub struct VectorEngine;

impl VectorEngine {
    pub const VECTOR_DIM: usize = 128;

    pub fn compute_concept_vector(concept: &str, dimension: &str) -> Vec<f64> {
        let mut vector = vec![0.0; Self::VECTOR_DIM];
        let full_text = format!("{} {}", concept.to_lowercase(), dimension.to_lowercase());
        let words: Vec<&str> = full_text.split_whitespace().collect();

        for (w_idx, word) in words.iter().enumerate() {
            if word.len() <= 2 {
                continue;
            }
            // Word hash
            let mut h = 5381u64;
            for b in word.bytes() {
                h = ((h << 5).wrapping_add(h)).wrapping_add(b as u64);
            }
            let idx = (h as usize) % Self::VECTOR_DIM;
            let weight = 1.0 + (1.0 / (w_idx as f64 + 1.0));
            vector[idx] += weight;

            // Character n-grams (3-grams) for morphological similarity (100% Unicode-safe)
            let chars: Vec<char> = word.chars().collect();
            if chars.len() >= 3 {
                for i in 0..=(chars.len() - 3) {
                    let mut gh = 0u64;
                    for c in &chars[i..i + 3] {
                        let val = *c as u32 as u64;
                        gh = ((gh << 3).wrapping_add(gh)).wrapping_add(val);
                    }
                    let g_idx = (gh as usize) % Self::VECTOR_DIM;
                    vector[g_idx] += 0.5;
                }
            }
        }

        // L2 Normalization
        let norm: f64 = vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-6 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }

        vector
    }

    pub fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
        if v1.is_empty() || v2.is_empty() || v1.len() != v2.len() {
            return 0.0;
        }

        let mut dot = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for (a, b) in v1.iter().zip(v2.iter()) {
            dot += a * b;
            norm1 += a * a;
            norm2 += b * b;
        }

        let denom = norm1.sqrt() * norm2.sqrt();
        if denom < 1e-6 {
            0.0
        } else {
            dot / denom
        }
    }

    /// Phase 2.2: Compute a DISTRIBUTIONAL vector from co-occurrence data.
    /// Uses PPMI (Positive Pointwise Mutual Information) weighting.
    /// Falls back to hash-based vector if insufficient co-occurrence data.
    pub fn compute_distributional_vector(
        engine: &crate::engine::AxiomEngine,
        concept: &str,
        dimension: &str,
    ) -> Vec<f64> {
        let clean = concept.to_lowercase();
        let cooc = engine.cooccurrence.read().unwrap();

        // Check if we have enough co-occurrence data for this word
        let word_contexts = cooc.get(&clean);
        let has_data = word_contexts.map_or(false, |ctx| ctx.len() >= 3);

        if !has_data {
            // Fallback to hash-based vector (not enough distributional data yet)
            return Self::compute_concept_vector(concept, dimension);
        }

        let contexts = word_contexts.unwrap();

        // Calculate total co-occurrence counts for PPMI
        let total_cooc: f64 = cooc.values()
            .flat_map(|ctx| ctx.values())
            .sum();

        let word_total: f64 = contexts.values().sum();

        // Build PPMI-weighted vector using random projection
        // (deterministic hash to project context words into 128-D space)
        let mut vector = vec![0.0; Self::VECTOR_DIM];

        for (context_word, &count) in contexts.iter() {
            // Calculate PPMI
            let context_total: f64 = cooc.values()
                .filter_map(|ctx| ctx.get(context_word))
                .sum();

            if context_total < 1.0 || total_cooc < 1.0 { continue; }

            let p_xy = count / total_cooc;
            let p_x = word_total / total_cooc;
            let p_y = context_total / total_cooc;

            let pmi = (p_xy / (p_x * p_y)).ln();
            let ppmi = pmi.max(0.0); // Positive PMI only

            if ppmi > 0.0 {
                // Project this context word into vector space using hash
                let mut h = 5381u64;
                for b in context_word.bytes() {
                    h = ((h << 5).wrapping_add(h)).wrapping_add(b as u64);
                }
                // Spread the PPMI weight across multiple dimensions for robustness
                let idx1 = (h as usize) % Self::VECTOR_DIM;
                let idx2 = (h.wrapping_mul(31) as usize) % Self::VECTOR_DIM;
                let idx3 = (h.wrapping_mul(97) as usize) % Self::VECTOR_DIM;
                vector[idx1] += ppmi;
                vector[idx2] += ppmi * 0.5;
                vector[idx3] += ppmi * 0.25;
            }
        }

        // L2 Normalize
        let norm: f64 = vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-6 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        } else {
            // If PPMI produced a zero vector, fall back to hash
            return Self::compute_concept_vector(concept, dimension);
        }

        vector
    }

    pub fn run_dream_mutation(
        engine: &AxiomEngine,
        trials: usize,
        threshold: f64,
    ) -> io::Result<DreamReport> {
        let mut rng = rand::thread_rng();
        let (nodes_snapshot, ids): (Vec<_>, Vec<u64>) = {
            let graph = engine.graph.read().unwrap();
            let n: Vec<_> = graph.values().cloned().collect();
            let ids: Vec<u64> = graph.keys().cloned().collect();
            (n, ids)
        };

        if ids.len() < 2 {
            return Ok(DreamReport {
                total_trials: 0,
                successful_synapses: Vec::new(),
                consolidated_orphans: 0,
            });
        }

        // Ensure all nodes have vectors
        for node in &nodes_snapshot {
            if node.vector.is_none() {
                let vec = Self::compute_concept_vector(&node.concept, &node.dimension);
                let _ = engine.add_or_get_node(&node.concept, &node.dimension, Some(vec));
            }
        }

        let updated_nodes = {
            let graph = engine.graph.read().unwrap();
            graph.values().cloned().collect::<Vec<_>>()
        };

        let mut successful = Vec::new();

        for _ in 0..trials {
            let n1 = updated_nodes.choose(&mut rng).unwrap();
            let n2 = updated_nodes.choose(&mut rng).unwrap();

            if n1.id == n2.id {
                continue;
            }

            let vec1 = n1.vector.clone().unwrap_or_else(|| Self::compute_concept_vector(&n1.concept, &n1.dimension));
            let vec2 = n2.vector.clone().unwrap_or_else(|| Self::compute_concept_vector(&n2.concept, &n2.dimension));

            let score = Self::cosine_similarity(&vec1, &vec2);

            if score >= threshold {
                // Check if link already exists
                let already_linked = n1.links.iter().any(|l| l.target_id == n2.id);
                if !already_linked {
                    let added = engine.add_link(
                        n1.id,
                        n2.id,
                        Relation::Evolved_Vector_Link,
                        (score * 100.0).round() / 100.0,
                    )?;

                    if added {
                        successful.push(MutationResult {
                            source_id: n1.id,
                            source_concept: n1.concept.clone(),
                            target_id: n2.id,
                            target_concept: n2.concept.clone(),
                            similarity: (score * 1000.0).round() / 1000.0,
                        });
                    }
                }
            }
        }

        let orphans = engine.find_orphan_nodes();
        let orphan_count = orphans.len();

        let storage = engine.storage.read().unwrap();
        if !successful.is_empty() {
            storage.log_stream(
                "DREAM",
                &format!(
                    "🌙 Dream cycle complete: {} trials, formed {} new semantic synapses.",
                    trials,
                    successful.len()
                ),
            )?;
        }

        Ok(DreamReport {
            total_trials: trials,
            successful_synapses: successful,
            consolidated_orphans: orphan_count,
        })
    }

    /// Resonates a raw string (sentence) against the entire vector space
    /// to find the most conceptually similar nodes in the graph.
    /// This acts as the "Native Ear" of the engine, bypassing NLP parsing.
    pub fn resonate_sentence(
        engine: &AxiomEngine,
        sentence: &str,
        top_k: usize,
    ) -> Vec<(u64, String, f64)> {
        let sentence_vec = Self::compute_concept_vector(sentence, "");
        let graph = engine.graph.read().unwrap();
        let mut scores = Vec::new();

        for (&id, node) in graph.iter() {
            let node_vec = node.vector.clone().unwrap_or_else(|| Self::compute_concept_vector(&node.concept, &node.dimension));
            let score = Self::cosine_similarity(&sentence_vec, &node_vec);
            
            // Lower threshold because a long sentence will dilute the vector compared to a single word
            if score > 0.01 { 
                scores.push((id, node.concept.clone(), score));
            }
        }

        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scores.into_iter().take(top_k).collect()
    }

    // === DisCoCat Tensor Mathematics ===

    /// Adjectives act as 2D Matrices.
    /// In a 128-dim space, full matrix is expensive, so we approximate with a diagonal matrix operation.
    pub fn apply_adjective_matrix(noun_vec: &[f64], adj: &str) -> Vec<f64> {
        let adj_vec = Self::compute_concept_vector(adj, "adjective");
        let mut result = vec![0.0; Self::VECTOR_DIM];
        for i in 0..Self::VECTOR_DIM {
            result[i] = noun_vec[i] * (1.0 + adj_vec[i]); 
        }
        
        let norm: f64 = result.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-6 {
            for v in result.iter_mut() {
                *v /= norm;
            }
        }
        result
    }

    /// Verbs act as 3D Tensors (Noun x Noun -> Meaning Space).
    /// Performs ASYMMETRIC tensor contraction: Subject and Object are projected
    /// into different dimensional subspaces to break symmetry.
    /// "Dog bites Man" ≠ "Man bites Dog"
    pub fn tensor_contraction(subj: &[f64], verb: &str, obj: &[f64]) -> Vec<f64> {
        let verb_vec = Self::compute_concept_vector(verb, "verb");
        let mut meaning_vector = vec![0.0; Self::VECTOR_DIM];
        
        for i in 0..Self::VECTOR_DIM {
            let s = subj[i];
            let o = obj[i];
            let v = verb_vec[i];
            
            // ASYMMETRIC contraction: Subject dominates even dimensions,
            // Object dominates odd dimensions. This ensures S↔O swap
            // produces a DIFFERENT meaning vector.
            if i % 2 == 0 {
                // Even dims: Subject-heavy
                meaning_vector[i] = (s * v * 1.5) + (o * v * 0.5) + (s * o);
            } else {
                // Odd dims: Object-heavy  
                meaning_vector[i] = (s * v * 0.5) + (o * v * 1.5) + (s * o);
            }
        }
        
        let norm: f64 = meaning_vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-6 {
            for v in meaning_vector.iter_mut() {
                *v /= norm;
            }
        }
        meaning_vector
    }

    /// De-contracts a meaning vector back into the closest word. (The Native Mouth)
    pub fn tensor_decontraction(engine: &AxiomEngine, meaning_vec: &[f64]) -> String {
        let graph = engine.graph.read().unwrap();
        let mut best_word = String::from("unknown");
        let mut best_score = -1.0;

        for node in graph.values() {
            let node_vec = node.vector.clone().unwrap_or_else(|| Self::compute_concept_vector(&node.concept, &node.dimension));
            let score = Self::cosine_similarity(meaning_vec, &node_vec);
            if score > best_score {
                best_score = score;
                best_word = node.concept.clone();
            }
        }
        
        best_word
    }
}
