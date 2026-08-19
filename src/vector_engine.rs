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
        storage.log_stream(
            "DREAM",
            &format!(
                "🌙 Dream cycle complete: {} trials, formed {} new semantic synapses.",
                trials,
                successful.len()
            ),
        )?;

        Ok(DreamReport {
            total_trials: trials,
            successful_synapses: successful,
            consolidated_orphans: orphan_count,
        })
    }
}
