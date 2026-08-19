use crate::relations::Relation;
use crate::storage::{NodeRecord, StorageManager};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub target_id: u64,
    pub relation: Relation,
    pub confidence: f64,
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct AxiomEngine {
    pub graph: Arc<RwLock<HashMap<u64, Node>>>,
    pub concept_index: Arc<RwLock<HashMap<String, u64>>>,
    pub storage: Arc<RwLock<StorageManager>>,
}

impl AxiomEngine {
    pub fn new(storage: StorageManager) -> io::Result<Self> {
        let storage_arc = Arc::new(RwLock::new(storage));
        let engine = AxiomEngine {
            graph: Arc::new(RwLock::new(HashMap::new())),
            concept_index: Arc::new(RwLock::new(HashMap::new())),
            storage: storage_arc,
        };

        engine.reload_from_storage()?;
        Ok(engine)
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

        // Check if link already exists
        let source_node = graph.get_mut(&source_id).unwrap();
        if let Some(existing) = source_node.links.iter_mut().find(|l| l.target_id == target_id && l.relation == relation) {
            existing.confidence = confidence;
        } else {
            source_node.links.push(CausalLink {
                target_id,
                relation: relation.clone(),
                confidence,
            });
        }
        drop(graph);

        let storage = self.storage.read().unwrap();
        storage.insert_link(source_id, target_id, &relation, confidence)?;

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
}
