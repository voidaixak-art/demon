use crate::engine::AxiomEngine;
use std::collections::HashMap;
use rand::seq::SliceRandom;

pub struct Clustering;

impl Clustering {
    /// Runs the Label Propagation Algorithm (LPA) to find communities/domains
    pub fn run_lpa(engine: &AxiomEngine, iterations: usize) -> HashMap<u64, u64> {
        let graph = engine.graph.read().unwrap();
        
        // Initial state: every node is in its own domain (label = its own ID)
        let mut labels: HashMap<u64, u64> = HashMap::new();
        let mut nodes_vec: Vec<u64> = Vec::new();
        
        for (&id, _) in graph.iter() {
            labels.insert(id, id);
            nodes_vec.push(id);
        }

        let mut rng = rand::thread_rng();

        for _ in 0..iterations {
            nodes_vec.shuffle(&mut rng);
            let mut changed = false;

            for &id in &nodes_vec {
                if let Some(node) = graph.get(&id) {
                    if node.links.is_empty() {
                        continue; // Isolated nodes stay in their own domain
                    }

                    // Count the frequency of neighboring labels
                    let mut label_counts: HashMap<u64, usize> = HashMap::new();
                    for link in &node.links {
                        if let Some(&neighbor_label) = labels.get(&link.target_id) {
                            *label_counts.entry(neighbor_label).or_insert(0) += 1;
                        }
                    }

                    // Also consider incoming links (undirected logic for community detection)
                    for (&other_id, other_node) in graph.iter() {
                        if other_node.links.iter().any(|l| l.target_id == id) {
                            if let Some(&neighbor_label) = labels.get(&other_id) {
                                *label_counts.entry(neighbor_label).or_insert(0) += 1;
                            }
                        }
                    }

                    // Find the most frequent label
                    if let Some((&best_label, _)) = label_counts.iter().max_by_key(|&(_, count)| count) {
                        let current_label = labels.get(&id).unwrap();
                        if *current_label != best_label {
                            labels.insert(id, best_label);
                            changed = true;
                        }
                    }
                }
            }
            
            // If equilibrium is reached early
            if !changed {
                break;
            }
        }

        labels
    }
}
