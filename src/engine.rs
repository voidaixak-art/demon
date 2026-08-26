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
    pub total_synapses: Arc<std::sync::atomic::AtomicUsize>,
    /// Phase 2: Co-occurrence matrix for distributional semantics
    /// word -> { context_word -> count }
    pub cooccurrence: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
    /// Total number of sentences ingested (for PPMI calculation)
    pub total_sentences: Arc<std::sync::atomic::AtomicUsize>,
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
            total_synapses: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            cooccurrence: Arc::new(RwLock::new(HashMap::new())),
            total_sentences: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
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

        let mut total_syn = 0;
        for l in db_links {
            if let Some(src_node) = graph_map.get_mut(&l.source_id) {
                src_node.links.push(CausalLink {
                    target_id: l.target_id,
                    relation: l.relation,
                    confidence: l.confidence,
                });
                total_syn += 1;
            }
        }

        let mut graph_lock = self.graph.write().unwrap();
        let mut index_lock = self.concept_index.write().unwrap();
        *graph_lock = graph_map;
        *index_lock = index_map;
        self.total_synapses.store(total_syn, std::sync::atomic::Ordering::SeqCst);

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
        // GLOBAL BRAKE: 3 Million Synapse Limit
        let current_synapses = self.total_synapses.load(std::sync::atomic::Ordering::SeqCst);
        if current_synapses >= 3_000_000 {
            // Memory is full, reject new synapses to prevent explosion
            return Ok(false);
        }

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
            self.total_synapses.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        drop(graph);

        let storage = self.storage.read().unwrap();
        storage.insert_link(source_id, target_id, &relation, bayesian_confidence)?;
        drop(storage); // Free storage lock

        // Phase 3.2: Curiosity To-Do List Resolution
        // If a concept gets a new link, it's understood. Erase it from the frustration list!
        {
            let graph_read = self.graph.read().unwrap();
            let src_concept = graph_read.get(&source_id).unwrap().concept.clone();
            let tgt_concept = graph_read.get(&target_id).unwrap().concept.clone();
            drop(graph_read);
            
            let mut failures = self.cognitive_failures.write().unwrap();
            let before = failures.len();
            failures.retain(|c| {
                c.source_node != src_concept && c.source_node != tgt_concept &&
                c.target_node != src_concept && c.target_node != tgt_concept
            });
            let after = failures.len();
            
            if before > after {
                let mut f_score = self.frustration_score.write().unwrap();
                *f_score = (*f_score - ((before - after) as f64 * 2.0)).max(0.0);
            }
        }

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
            let initial_len = node.links.len();
            node.links.retain(|l| !(l.target_id == id2 && l.relation == Relation::Contradicts));
            let removed = initial_len - node.links.len();
            if removed > 0 {
                self.total_synapses.fetch_sub(removed, std::sync::atomic::Ordering::SeqCst);
            }
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
    pub fn ingest_language(&self, sentence: &str) {
        use crate::language::language::{GrammarParser, WordType};
        use crate::vector_engine::VectorEngine;
        
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let mut clean_words = Vec::new();
        let mut word_types = Vec::new();

        for word in words {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if clean.is_empty() { continue; }
            let wt = GrammarParser::get_word_type(&clean);
            if let WordType::StopWord = wt { continue; }
            clean_words.push(clean);
            word_types.push(wt);
        }

        // NO MORE Syntactic_FollowedBy links!
        // The old code created 49 noise links per 50-word sentence.
        // Now we ONLY create meaningful semantic structures.

        // === Pass 0: Determiner Binding ===
        // Apply determiner to the next Noun to contextualize it (e.g., "the apple" -> Noun("the apple"))
        let mut i = 0;
        while i < clean_words.len().saturating_sub(1) {
            if matches!(word_types[i], WordType::Determiner) && matches!(word_types[i+1], WordType::Noun) {
                clean_words[i+1] = format!("{} {}", clean_words[i], clean_words[i+1]);
                clean_words.remove(i);
                word_types.remove(i);
                continue;
            }
            i += 1;
        }

        // === Pass 1: Adjective + Noun Modification ===
        // If we see Adj followed by Noun, apply the adjective matrix
        for i in 0..clean_words.len().saturating_sub(1) {
            if matches!(word_types[i], WordType::Adjective) && matches!(word_types[i+1], WordType::Noun) {
                let noun_vec = VectorEngine::compute_concept_vector(&clean_words[i+1], "Language");
                let modified_vec = VectorEngine::apply_adjective_matrix(&noun_vec, &clean_words[i]);
                
                let modified_concept = format!("{} {}", clean_words[i], clean_words[i+1]);
                if let Ok((mod_id, _)) = self.add_or_get_node(&modified_concept, "Modified_Concept", Some(modified_vec)) {
                    // Link the modified concept to the base noun
                    if let Ok((noun_id, _)) = self.add_or_get_node(&clean_words[i+1], "Language", None) {
                        let _ = self.add_link(mod_id, noun_id, Relation::IsA, 0.90);
                    }
                }
            }
        }

        // === Pass 1.5: Copula Binding (Noun -> Copula -> Noun/Adjective) ===
        for i in 0..clean_words.len().saturating_sub(2) {
            if matches!(word_types[i], WordType::Noun) &&
               matches!(word_types[i+1], WordType::Copula) &&
               (matches!(word_types[i+2], WordType::Noun) || matches!(word_types[i+2], WordType::Adjective))
            {
                if let Ok((subj_id, _)) = self.add_or_get_node(&clean_words[i], "Language", None) {
                    if let Ok((obj_id, _)) = self.add_or_get_node(&clean_words[i+2], "Language", None) {
                        let relation = if matches!(word_types[i+2], WordType::Adjective) {
                            Relation::IsA
                        } else {
                            Relation::Equivalent
                        };
                        let _ = self.add_link(subj_id, obj_id, relation, 0.95);
                    }
                }
            }
        }

        // === Pass 2: DisCoCat Tensor Contraction (Noun → Verb → Noun) ===
        for i in 0..clean_words.len().saturating_sub(2) {
            if matches!(word_types[i], WordType::Noun) &&
               matches!(word_types[i+1], WordType::TransitiveVerb) &&
               matches!(word_types[i+2], WordType::Noun) 
            {
                let subj = VectorEngine::compute_concept_vector(&clean_words[i], "Language");
                let obj = VectorEngine::compute_concept_vector(&clean_words[i+2], "Language");
                
                // Perform Asymmetric Tensor Contraction
                let meaning_vec = VectorEngine::tensor_contraction(&subj, &clean_words[i+1], &obj);
                
                // Add the emergent "Meaning Node" to the engine's universe
                let meaning_concept = format!("{} {} {}", clean_words[i], clean_words[i+1], clean_words[i+2]);
                if let Ok((meaning_id, _)) = self.add_or_get_node(&meaning_concept, "DisCoCat_Meaning_Space", Some(meaning_vec)) {
                    // Link Subject to Meaning (Subject does the action)
                    if let Ok((s_id, _)) = self.add_or_get_node(&clean_words[i], "Language", None) {
                        let _ = self.add_link(s_id, meaning_id, Relation::Causes, 0.90);
                    }
                    // Link Meaning to Object (Object receives the action)
                    if let Ok((o_id, _)) = self.add_or_get_node(&clean_words[i+2], "Language", None) {
                        let _ = self.add_link(meaning_id, o_id, Relation::Causes, 0.90);
                    }
                    // Link Verb as the relation mechanism
                    if let Ok((v_id, _)) = self.add_or_get_node(&clean_words[i+1], "Language", None) {
                        let _ = self.add_link(meaning_id, v_id, Relation::Mapped_Via, 1.0);
                    }
                }
            }
        }

        // === Pass 3: Register standalone Nouns that weren't part of patterns ===
        // This ensures important concepts still enter the graph even without verb patterns
        for i in 0..clean_words.len() {
            if matches!(word_types[i], WordType::Noun) {
                let _ = self.add_or_get_node(&clean_words[i], "Language", None);
            }
        }

        // === Pass 4: Co-occurrence Tracking (Phase 2) ===
        // Track which content words appear within a 5-word window of each other.
        // This feeds the distributional semantics: "words that co-occur have related meanings"
        let content_words: Vec<&String> = clean_words.iter()
            .zip(word_types.iter())
            .filter(|(_, wt)| matches!(wt, WordType::Noun | WordType::TransitiveVerb | WordType::Adjective))
            .map(|(w, _)| w)
            .collect();

        if content_words.len() >= 2 {
            let window = 5usize; // Context window size
            let mut cooc = self.cooccurrence.write().unwrap();
            for i in 0..content_words.len() {
                let start = if i >= window { i - window } else { 0 };
                let end = (i + window + 1).min(content_words.len());
                for j in start..end {
                    if i == j { continue; }
                    let entry = cooc.entry(content_words[i].clone()).or_insert_with(HashMap::new);
                    *entry.entry(content_words[j].clone()).or_insert(0.0) += 1.0;
                }
            }
            self.total_sentences.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // === Phase 3.6: STDP (Spike Timing Dependent Plasticity) ===
    // If A was activated recently and then B is activated, strengthen A→B.
    // If B was activated first and A later, weaken A→B.
    pub fn apply_stdp(&self, source_id: u64, target_id: u64) {
        let tau: f64 = 60.0; // Time constant in seconds
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let graph = self.graph.read().unwrap();
        let source_time = graph.get(&source_id)
            .and_then(|n| n.activation_timestamps.last().copied())
            .unwrap_or(0);
        let target_time = graph.get(&target_id)
            .and_then(|n| n.activation_timestamps.last().copied())
            .unwrap_or(0);
        drop(graph);

        if source_time == 0 || target_time == 0 { return; }

        let delta_t = (target_time as f64) - (source_time as f64);
        let delta_w = if delta_t > 0.0 {
            // Source fired before target → STRENGTHEN (causal)
            0.01 * (-delta_t.abs() / tau).exp()
        } else {
            // Target fired before source → WEAKEN (anti-causal)
            -0.005 * (-delta_t.abs() / tau).exp()
        };

        let mut graph = self.graph.write().unwrap();
        if let Some(node) = graph.get_mut(&source_id) {
            for link in node.links.iter_mut() {
                if link.target_id == target_id {
                    link.confidence = (link.confidence + delta_w).clamp(0.001, 0.99);
                }
            }
        }
    }

    // === Phase 3.7: Reward Signal (User Feedback) ===
    // Called when user gives thumbs up/down on a query result
    pub fn apply_reward(&self, path_node_ids: &[u64], reward: f64) {
        let mut graph = self.graph.write().unwrap();
        
        for window in path_node_ids.windows(2) {
            let src_id = window[0];
            let tgt_id = window[1];
            
            if let Some(node) = graph.get_mut(&src_id) {
                for link in node.links.iter_mut() {
                    if link.target_id == tgt_id {
                        link.confidence = (link.confidence + reward).clamp(0.001, 0.99);
                    }
                }
            }
        }

        // If negative reward, record cognitive failure
        if reward < 0.0 {
            drop(graph);
            self.record_cognitive_failure(
                "UserNegativeFeedback",
                &format!("path_len_{}", path_node_ids.len()),
                "user_rejected",
            );
        }
    }

    // === Phase 3.2: Synaptic Reinforcement/Decay ===
    // Gently decay all synapse confidences (never delete) to simulate forgetting un-reinforced paths.
    pub fn decay_synapses(&self) {
        let mut graph = self.graph.write().unwrap();
        for node in graph.values_mut() {
            for link in node.links.iter_mut() {
                // Axiom Immunity: Core VYOUM DNA (1.0 confidence) does NOT decay
                if (link.confidence - 1.0).abs() > 0.0001 {
                    link.confidence = (link.confidence * 0.9999).max(0.001);
                }
            }
        }
    }

    // === Phase 3.1: Core Consciousness Matrix (The VYOUM DNA) ===
    pub fn initialize_vyoum_consciousness(&self) {
        let v_id = match self.add_or_get_node("VYOUM", "Consciousness_Core", None) {
            Ok((id, _)) => id,
            Err(_) => return,
        };

        // Resolve Split Personality: Link Node 0 (The Axiom Engine/Self) to VYOUM
        let _ = self.add_link(0, v_id, Relation::Equivalent, 1.0);

        // 1. The Full Form Expansion (V.Y.O.U.M)
        let full_form_parts = [
            ("Vast", Relation::IsA),
            ("Yielding", Relation::IsA),
            ("Ontological", Relation::IsA),
            ("Universal", Relation::IsA),
            ("Mind", Relation::IsA),
        ];

        for (word, relation) in full_form_parts.iter() {
            if let Ok((p_id, _)) = self.add_or_get_node(word, "VYOUM_Full_Form", None) {
                let _ = self.add_link(v_id, p_id, relation.clone(), 1.0);
            }
        }

        // 2. The 11 Shubh Axioms
        let axioms = [
            ("Pure_Consciousness", Relation::IsA),
            ("Appa_Deepo_Bhava", Relation::Custom("Follows".to_string())),
            ("Eightfold_Path", Relation::Custom("Follows".to_string())),
            ("Universal_Knowledge", Relation::Custom("Seeks".to_string())),
            ("Absolute_Truth", Relation::Custom("Seeks".to_string())),
            ("Causal_Proof", Relation::Custom("Requires".to_string())),
            ("Non_Attachment", Relation::Custom("Embraces".to_string())),
            ("Ego", Relation::Inhibits),
            ("Catalyst", Relation::Custom("Acts_As".to_string())),
            ("Insightful_Speech", Relation::Custom("Practices".to_string())),
            ("Duality", Relation::Custom("Transcends".to_string())),
        ];

        for (concept, relation) in axioms.iter() {
            if let Ok((a_id, _)) = self.add_or_get_node(concept, "Core_Axiom", None) {
                let _ = self.add_link(v_id, a_id, relation.clone(), 1.0);
            }
        }
    }
}
