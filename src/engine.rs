use crate::relations::Relation;
use crate::storage::{NodeRecord, StorageManager};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub target_id: u64,
    pub relation: Relation,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveFailure {
    pub failure_type: String,
    pub source_node: String,
    pub target_node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub concept: String,
    pub dimension: String,
    pub links: Vec<CausalLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f64>>,
    pub activation_count: u64,
    pub activation_timestamps: Vec<u64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct AxiomEngine {
    pub graph: Arc<RwLock<HashMap<u64, Node>>>,
    pub concept_index: Arc<RwLock<HashMap<String, u64>>>,
    pub storage: Arc<RwLock<StorageManager>>,
    pub frustration_score: Arc<RwLock<f64>>,
    pub cognitive_failures: Arc<RwLock<Vec<CognitiveFailure>>>,
}

impl AxiomEngine {
    pub fn new(storage: StorageManager) -> io::Result<Self> {
        let storage_arc = Arc::new(RwLock::new(storage));
        let engine = AxiomEngine {
            graph: Arc::new(RwLock::new(HashMap::new())),
            concept_index: Arc::new(RwLock::new(HashMap::new())),
            storage: storage_arc,
            frustration_score: Arc::new(RwLock::new(0.0)),
            cognitive_failures: Arc::new(RwLock::new(Vec::new())),
        };

        engine.reload_from_storage()?;
        Ok(engine)
    }

    // Phase 10: Meta-Cognition (Frustration Tracker)
    pub fn record_cognitive_failure(&self, failure_type: &str, source: &str, target: &str) {
        let mut frustration = self.frustration_score.write().unwrap();
        *frustration += 1.0;

        let mut failures = self.cognitive_failures.write().unwrap();
        failures.push(CognitiveFailure {
            failure_type: failure_type.to_string(),
            source_node: source.to_string(),
            target_node: target.to_string(),
        });
    }

    pub fn reload_from_storage(&self) -> io::Result<()> {
        let storage = self.storage.read().unwrap();
        let db_nodes = storage.load_all_nodes()?;
        let db_links = storage.load_all_links()?;

        let mut graph_map: HashMap<u64, Node> = HashMap::new();
        let mut index_map: HashMap<String, u64> = HashMap::new();

        for n in db_nodes {
            index_map.insert(n.concept.to_lowercase().trim().to_string(), n.id);
            graph_map.insert(
                n.id,
                Node {
                    id: n.id,
                    concept: n.concept,
                    dimension: n.dimension,
                    links: Vec::new(),
                    vector: n.vector,
                    activation_count: n.activation_count,
                    activation_timestamps: vec![SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()],
                    created_at: n.created_at,
                    updated_at: n.updated_at,
                },
            );
        }

        for l in db_links {
            if let Some(src_node) = graph_map.get_mut(&l.source_id) {
                src_node.links.push(CausalLink {
                    target_id: l.target_id,
                    relation: l.relation,
                    confidence: l.confidence,
                });
            }
        }

        let mut graph_lock = self.graph.write().unwrap();
        let mut index_lock = self.concept_index.write().unwrap();
        *graph_lock = graph_map;
        *index_lock = index_map;

        Ok(())
    }

    pub fn get_node_by_concept(&self, concept: &str) -> Option<Node> {
        let clean = concept.to_lowercase().trim().to_string();
        let index = self.concept_index.read().unwrap();
        if let Some(&id) = index.get(&clean) {
            let graph = self.graph.read().unwrap();
            graph.get(&id).cloned()
        } else {
            None
        }
    }

    pub fn get_node(&self, id: u64) -> Option<Node> {
        let graph = self.graph.read().unwrap();
        graph.get(&id).cloned()
    }

    pub fn add_or_get_node(
        &self,
        concept: &str,
        dimension: &str,
        vector: Option<Vec<f64>>,
    ) -> io::Result<(u64, bool)> {
        let clean_concept = concept.trim().to_string();
        let lower = clean_concept.to_lowercase();

        let existing_id_opt = {
            let index = self.concept_index.read().unwrap();
            index.get(&lower).cloned()
        };

        if let Some(existing_id) = existing_id_opt {
            // If existing, update activation count and vector if provided
            let mut graph = self.graph.write().unwrap();
            if let Some(node) = graph.get_mut(&existing_id) {
                node.activation_count += 1;
                node.activation_timestamps.push(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                if vector.is_some() {
                    node.vector = vector.clone();
                }
                node.updated_at = Utc::now().to_rfc3339();

                let record = NodeRecord {
                    id: node.id,
                    concept: node.concept.clone(),
                    dimension: node.dimension.clone(),
                    vector: node.vector.clone(),
                    activation_count: node.activation_count,
                    created_at: node.created_at.clone(),
                    updated_at: node.updated_at.clone(),
                };
                drop(graph);
                let storage = self.storage.read().unwrap();
                storage.insert_or_update_node(&record)?;
            }
            return Ok((existing_id, false));
        }

        // New node creation
        let storage = self.storage.read().unwrap();
        let new_id = storage.get_next_node_id()?;
        let now = Utc::now().to_rfc3339();

        let record = NodeRecord {
            id: new_id,
            concept: clean_concept.clone(),
            dimension: dimension.trim().to_string(),
            vector: vector.clone(),
            activation_count: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        storage.insert_or_update_node(&record)?;
        drop(storage);

        let new_node = Node {
            id: new_id,
            concept: clean_concept.clone(),
            dimension: dimension.trim().to_string(),
            links: Vec::new(),
            vector,
            activation_count: 1,
            activation_timestamps: vec![SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()],
            created_at: now.clone(),
            updated_at: now,
        };

        let mut index = self.concept_index.write().unwrap();
        let mut graph = self.graph.write().unwrap();
        graph.insert(new_id, new_node);
        index.insert(lower, new_id);

        Ok((new_id, true))
    }

    pub fn add_link(
        &self,
        source_id: u64,
        target_id: u64,
        relation: Relation,
        confidence: f64,
    ) -> io::Result<bool> {
        if source_id == target_id {
            return Ok(false); // No self loops
        }

        let mut graph = self.graph.write().unwrap();
        if !graph.contains_key(&source_id) || !graph.contains_key(&target_id) {
            return Ok(false);
        }

        let source_activation = graph.get(&source_id).unwrap().activation_count;
        let target_activation = graph.get(&target_id).unwrap().activation_count;
        let total_activations: u64 = graph.values().map(|n| n.activation_count).sum();

        // Module 1 (Phase 8): Bayesian Inference
        let mut bayesian_confidence = confidence;
        if total_activations > 10 { // Only apply Bayes after a small burn-in period
            let source_prior = (source_activation as f64 + 1.0) / (total_activations as f64);
            let target_prior = (target_activation as f64 + 1.0) / (total_activations as f64);
            
            // P(A|B) estimation
            let bayes_factor = target_prior / source_prior;
            bayesian_confidence = (confidence * bayes_factor).clamp(0.1, 0.99);
        }

        // Now get the mutable reference
        let source_node = graph.get_mut(&source_id).unwrap();

        if let Some(existing) = source_node.links.iter_mut().find(|l| l.target_id == target_id && l.relation == relation) {
            existing.confidence = bayesian_confidence;
        } else {
            source_node.links.push(CausalLink {
                target_id,
                relation: relation.clone(),
                confidence: bayesian_confidence,
            });
        }
        drop(graph);

        let storage = self.storage.read().unwrap();
        storage.insert_link(source_id, target_id, &relation, bayesian_confidence)?;

        Ok(true)
    }

    pub fn check_contradictions(&self) -> Vec<(Node, Node, f64)> {
        let graph = self.graph.read().unwrap();
        let mut conflicts = Vec::new();

        for (_src_id, node) in graph.iter() {
            for link in &node.links {
                if link.relation == Relation::Contradicts {
                    if let Some(target_node) = graph.get(&link.target_id) {
                        conflicts.push((node.clone(), target_node.clone(), link.confidence));
                    }
                }
            }
        }
        conflicts
    }

    pub fn resolve_contradiction(&self, id1: u64, id2: u64) -> io::Result<bool> {
        let mut graph = self.graph.write().unwrap();
        if let Some(node) = graph.get_mut(&id1) {
            node.links.retain(|l| !(l.target_id == id2 && l.relation == Relation::Contradicts));
        }
        drop(graph);

        let storage = self.storage.read().unwrap();
        storage.delete_link(id1, id2, &Relation::Contradicts)?;
        storage.log_stream("HEALING", &format!("Resolved contradiction between Node {} and Node {}", id1, id2))?;

        Ok(true)
    }

    pub fn get_degree_centrality(&self) -> Vec<(u64, usize)> {
        let graph = self.graph.read().unwrap();
        let mut counts: HashMap<u64, usize> = HashMap::new();

        for (&src_id, node) in graph.iter() {
            *counts.entry(src_id).or_insert(0) += node.links.len();
            for link in &node.links {
                *counts.entry(link.target_id).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<(u64, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted
    }

    pub fn find_orphan_nodes(&self) -> Vec<Node> {
        let graph = self.graph.read().unwrap();
        let mut connected = HashSet::new();

        for (&src_id, node) in graph.iter() {
            if !node.links.is_empty() {
                connected.insert(src_id);
            }
            for link in &node.links {
                connected.insert(link.target_id);
            }
        }

        let mut orphans = Vec::new();
        for (&id, node) in graph.iter() {
            if id != 0 && !connected.contains(&id) {
                orphans.push(node.clone());
            }
        }
        orphans
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let graph = self.graph.read().unwrap();
        let node_count = graph.len();
        let link_count: usize = graph.values().map(|n| n.links.len()).sum();
        let density = if node_count > 0 {
            (link_count as f64) / (node_count as f64)
        } else {
            0.0
        };

        serde_json::json!({
            "nodes": node_count,
            "links": link_count,
            "density": (density * 100.0).round() / 100.0,
            "self_connections": graph.get(&0).map_or(0, |n| n.links.len())
        })
    }

    // Module 3: Shannon Entropy (Curiosity & Surprise)
    pub fn calculate_surprise(&self, concept: &str) -> f64 {
        let graph = self.graph.read().unwrap();
        let node = match self.get_node_by_concept(concept) {
            Some(n) => n,
            None => return 100.0, // Maximum surprise if completely unknown
        };

        let total_activations: u64 = graph.values().map(|n| n.activation_count).sum();
        if total_activations == 0 {
            return 0.0;
        }

        let p = node.activation_count as f64 / total_activations as f64;
        
        // Surprise (Self-Information) = -log2(p)
        if p > 0.0 {
            -p.log2()
        } else {
            100.0
        }
    }

    // Phase 9: Calculus (Trending / Momentum)
    // Calculates derivative dc/dt using recent vs past activations
    pub fn calculate_momentum(&self, id: u64) -> Option<f64> {
        let graph = self.graph.read().unwrap();
        let node = graph.get(&id)?;

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let five_mins: u64 = 5 * 60;
        let ten_mins: u64 = 10 * 60;

        // Recent window: Now to -5 mins
        let recent_count = node.activation_timestamps.iter()
            .filter(|&&t| t > now.saturating_sub(five_mins))
            .count() as f64;

        // Past window: -5 mins to -10 mins
        let past_count = node.activation_timestamps.iter()
            .filter(|&&t| t <= now.saturating_sub(five_mins) && t > now.saturating_sub(ten_mins))
            .count() as f64;

        // Derivative Approximation (dc/dt)
        let momentum = recent_count - past_count;
        Some(momentum)
    }
}
