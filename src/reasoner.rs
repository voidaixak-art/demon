use crate::engine::{AxiomEngine, Node};
use crate::relations::Relation;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStep {
    pub step_number: usize,
    pub from_node: String,
    pub to_node: String,
    pub relation: Relation,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    pub proven: bool,
    pub start_concept: String,
    pub goal_concept: String,
    pub steps: Vec<ProofStep>,
    pub explanation: String,
}

pub struct Reasoner;

#[derive(Clone, PartialEq)]
struct State {
    cost: f64,
    node: u64,
    temporal_direction: i32, // 0 = Neutral, 1 = Forwards (Before), -1 = Backwards (After)
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap ordering
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Reasoner {
    pub fn find_causal_path(
        engine: &AxiomEngine,
        start_id: u64,
        goal_id: u64,
    ) -> Option<Vec<(u64, u64, Relation, f64)>> {
        if start_id == goal_id {
            return Some(Vec::new());
        }

        let graph = engine.graph.read().unwrap();
        if !graph.contains_key(&start_id) || !graph.contains_key(&goal_id) {
            return None;
        }

        // Dijkstra's Algorithm using Min-Heap
        let mut heap: BinaryHeap<State> = BinaryHeap::new();
        let mut distances: HashMap<u64, f64> = HashMap::new();
        let mut parent_map: HashMap<u64, (u64, Relation, f64)> = HashMap::new();

        heap.push(State { cost: 0.0, node: start_id, temporal_direction: 0 });
        distances.insert(start_id, 0.0);

        let mut found = false;
        while let Some(State { cost, node: current, temporal_direction }) = heap.pop() {
            if current == goal_id {
                found = true;
                break;
            }

            if cost > *distances.get(&current).unwrap_or(&f64::MAX) {
                continue;
            }

            if let Some(graph_node) = graph.get(&current) {
                for link in &graph_node.links {
                    if link.relation.is_traversable() {
                        // Module 3 (Phase 8): Temporal Consistency
                        let edge_time = match link.relation {
                            Relation::TemporalBefore => 1,
                            Relation::TemporalAfter => -1,
                            _ => 0,
                        };

                        if temporal_direction != 0 && edge_time != 0 && temporal_direction != edge_time {
                            continue; // Time paradox (cannot go forward then backward in time)
                        }

                        let next_temporal = if edge_time != 0 { edge_time } else { temporal_direction };

                        let safe_conf = link.confidence.max(0.001).min(1.0);
                        let edge_cost = -safe_conf.ln();
                        let next_cost = cost + edge_cost;

                        let is_better = next_cost < *distances.get(&link.target_id).unwrap_or(&f64::MAX);
                        if is_better {
                            distances.insert(link.target_id, next_cost);
                            parent_map.insert(link.target_id, (current, link.relation.clone(), link.confidence));
                            heap.push(State { cost: next_cost, node: link.target_id, temporal_direction: next_temporal });
                        }
                    }
                }
            }
        }

        if !found {
            let s_concept = graph.get(&start_id).map(|n| n.concept.clone()).unwrap_or_default();
            let g_concept = graph.get(&goal_id).map(|n| n.concept.clone()).unwrap_or_default();
            engine.record_cognitive_failure("PathfindingDeadEnd", &s_concept, &g_concept);
            return None;
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut curr = goal_id;
        while curr != start_id {
            if let Some(&(p_id, ref rel, conf)) = parent_map.get(&curr) {
                path.push((p_id, curr, rel.clone(), conf));
                curr = p_id;
            } else {
                break;
            }
        }

        path.reverse();
        Some(path)
    }

    pub fn prove_deduction(
        engine: &AxiomEngine,
        start_concept: &str,
        goal_concept: &str,
    ) -> ProofResult {
        let start_node = engine.get_node_by_concept(start_concept);
        let goal_node = engine.get_node_by_concept(goal_concept);

        if start_node.is_none() || goal_node.is_none() {
            let missing = if start_node.is_none() && goal_node.is_none() {
                format!("Neither '{}' nor '{}' exists in memory.", start_concept, goal_concept)
            } else if start_node.is_none() {
                format!("'{}' does not exist in memory.", start_concept)
            } else {
                format!("'{}' does not exist in memory.", goal_concept)
            };

            return ProofResult {
                proven: false,
                start_concept: start_concept.to_string(),
                goal_concept: goal_concept.to_string(),
                steps: Vec::new(),
                explanation: missing,
            };
        }

        let start = start_node.unwrap();
        let goal = goal_node.unwrap();

        if let Some(raw_path) = Self::find_causal_path(engine, start.id, goal.id) {
            let graph = engine.graph.read().unwrap();
            let mut steps = Vec::new();
            let mut explanation = format!(
                "✅ DEDUCTION PROVEN: A rigorous causal chain exists from [{}] to [{}]:\n",
                start.concept, goal.concept
            );

            for (idx, (src_id, tgt_id, rel, conf)) in raw_path.iter().enumerate() {
                let src_name = graph.get(src_id).map(|n| n.concept.as_str()).unwrap_or("Unknown");
                let tgt_name = graph.get(tgt_id).map(|n| n.concept.as_str()).unwrap_or("Unknown");

                steps.push(ProofStep {
                    step_number: idx + 1,
                    from_node: src_name.to_string(),
                    to_node: tgt_name.to_string(),
                    relation: rel.clone(),
                    confidence: *conf,
                });

                explanation.push_str(&format!(
                    "  [Step {}] [{}] {} [{}] (confidence: {:.2})\n",
                    idx + 1,
                    src_name,
                    rel.symbol(),
                    tgt_name,
                    conf
                ));
            }

            explanation.push_str(&format!(
                "➔ CONCLUSION: [{}] logically leads to [{}] through {} verified axiomatic steps.",
                start.concept,
                goal.concept,
                steps.len()
            ));

            ProofResult {
                proven: true,
                start_concept: start.concept,
                goal_concept: goal.concept,
                steps,
                explanation,
            }
        } else {
            ProofResult {
                proven: false,
                start_concept: start.concept.clone(),
                goal_concept: goal.concept.clone(),
                steps: Vec::new(),
                explanation: format!(
                    "❌ UNPROVEN: No unbroken axiomatic pathway currently connects [{}] to [{}] in the neural graph.",
                    start.concept, goal.concept
                ),
            }
        }
    }

    pub fn retrieve_subgraph_context(engine: &AxiomEngine, query: &str, limit: usize) -> String {
        let query_words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|s| s.to_string())
            .collect();

        let graph = engine.graph.read().unwrap();
        let mut scored_nodes: Vec<(&Node, usize)> = graph
            .values()
            .map(|n| {
                let mut score = 0;
                let c_lower = n.concept.to_lowercase();
                let d_lower = n.dimension.to_lowercase();
                for w in &query_words {
                    if c_lower.contains(w) {
                        score += 3;
                    }
                    if d_lower.contains(w) {
                        score += 1;
                    }
                }
                (n, score)
            })
            .collect();

        scored_nodes.sort_by(|a, b| b.1.cmp(&a.1));

        let mut context = String::from("=== AXIOM ENGINE KNOWLEDGE GRAPH CONTEXT ===\n");
        let mut matched = 0;

        for (node, score) in scored_nodes {
            if score > 0 && matched < limit {
                context.push_str(&format!(
                    "• Node [#{}] '{}' (Dimension: {})\n",
                    node.id, node.concept, node.dimension
                ));

                for link in &node.links {
                    let tgt_name = graph.get(&link.target_id).map(|t| t.concept.as_str()).unwrap_or("Unknown");
                    context.push_str(&format!(
                        "    └── {} [{}] (conf: {:.2})\n",
                        link.relation.symbol(),
                        tgt_name,
                        link.confidence
                    ));
                }
                matched += 1;
            }
        }

        if matched == 0 {
            context.push_str("No direct axiomatic matches found in current memory.\n");
        }

        context
    }

    pub fn run_transitive_dream(
        engine: &AxiomEngine,
        trials: usize,
    ) -> std::io::Result<Vec<(String, String, f64)>> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let nodes: Vec<_> = {
            let graph = engine.graph.read().unwrap();
            graph.values().cloned().collect()
        };

        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let mut successful = Vec::new();

        for _ in 0..trials {
            let a = nodes.choose(&mut rng).unwrap();
            
            if a.links.is_empty() { continue; }
            
            let link_ab = a.links.choose(&mut rng).unwrap();
            if !link_ab.relation.is_traversable() { continue; }
            
            let b_node = {
                let graph = engine.graph.read().unwrap();
                graph.get(&link_ab.target_id).cloned()
            };

            if let Some(b) = b_node {
                if b.links.is_empty() { continue; }
                
                let link_bc = b.links.choose(&mut rng).unwrap();
                if !link_bc.relation.is_traversable() { continue; }

                let c_id = link_bc.target_id;
                
                // Don't link to self
                if c_id == a.id { continue; }
                
                // Check if A->C already exists
                let exists = a.links.iter().any(|l| l.target_id == c_id);
                
                if !exists {
                    let mut inferred_conf = (link_ab.confidence * link_bc.confidence * 0.9 * 100.0).round() / 100.0;
                    
                    // === Phase 3.5: FCA Lattice Gate ===
                    // A and C must share some attribute/meaning overlap.
                    // If they are completely orthogonal, do not infer a relation.
                    let c_node = {
                        let graph = engine.graph.read().unwrap();
                        graph.get(&c_id).cloned()
                    };
                    
                    if let Some(c) = c_node {
                        if let (Some(vec_a), Some(vec_c)) = (&a.vector, &c.vector) {
                            let sim = crate::vector_engine::VectorEngine::cosine_similarity(vec_a, vec_c);
                            if sim < 0.1 {
                                continue; // Too dissimilar, reject the dream
                            }
                            // Boost confidence based on similarity
                            inferred_conf = (inferred_conf + sim * 0.2).min(0.99);
                        }
                        
                        if inferred_conf >= 0.5 { // Only solidify somewhat strong thoughts
                            if let Ok(true) = engine.add_link(a.id, c_id, Relation::Implies, inferred_conf) {
                                successful.push((a.concept.clone(), c.concept.clone(), inferred_conf));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(successful)
    }

    // Module 4: Analogical Reasoning (Graph Isomorphism proxy)
    // Matches the "structural shape" of a node (its outgoing link relations)
    // to find another concept that behaves exactly like it.
    pub fn find_analogy(
        engine: &AxiomEngine,
        concept: &str,
    ) -> Option<(String, f64)> {
        let node = engine.get_node_by_concept(concept)?;
        
        let graph = engine.graph.read().unwrap();
        if graph.len() < 2 || node.links.is_empty() {
            return None;
        }

        // Create a structural signature for the target concept
        // E.g., counts of "IsA", "Causes", "Produces"
        let mut target_sig: HashMap<Relation, usize> = HashMap::new();
        for link in &node.links {
            *target_sig.entry(link.relation.clone()).or_insert(0) += 1;
        }

        let mut best_match = String::new();
        let mut best_score = 0.0;

        for (&id, other) in graph.iter() {
            if id == node.id || other.links.is_empty() {
                continue;
            }

            let mut other_sig: HashMap<Relation, usize> = HashMap::new();
            for link in &other.links {
                *other_sig.entry(link.relation.clone()).or_insert(0) += 1;
            }

            // Calculate cosine similarity of structural signatures
            let mut dot = 0.0;
            let mut norm1 = 0.0;
            let mut norm2 = 0.0;

            let all_keys: HashSet<_> = target_sig.keys().chain(other_sig.keys()).collect();
            for k in all_keys {
                let v1 = *target_sig.get(k).unwrap_or(&0) as f64;
                let v2 = *other_sig.get(k).unwrap_or(&0) as f64;
                dot += v1 * v2;
                if target_sig.contains_key(k) { norm1 += v1 * v1; }
                if other_sig.contains_key(k) { norm2 += v2 * v2; }
            }

            let denom = norm1.sqrt() * norm2.sqrt();
            let score = if denom < 1e-6 { 0.0 } else { dot / denom };

            if score > best_score {
                best_score = score;
                best_match = other.concept.clone();
            }
        }

        if best_score > 0.5 {
            Some((best_match, (best_score * 100.0).round() / 100.0))
        } else {
            None
        }
    }

    // Module 2: PageRank (Attention / Working Memory)
    // Simulates attention spreading from an active concept.
    pub fn compute_attention(
        engine: &AxiomEngine,
        active_concept: &str,
    ) -> Vec<(String, f64)> {
        let start_node = match engine.get_node_by_concept(active_concept) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let graph = engine.graph.read().unwrap();
        let mut attention: HashMap<u64, f64> = HashMap::new();
        let damping = 0.85;
        let iterations = 20;

        // Initialize all to 0 except start_node
        for &id in graph.keys() {
            attention.insert(id, if id == start_node.id { 1.0 } else { 0.0 });
        }

        for _ in 0..iterations {
            let mut next_attention: HashMap<u64, f64> = HashMap::new();
            
            // Random walk with restart
            for (&id, _node) in graph.iter() {
                next_attention.insert(id, (1.0 - damping) * if id == start_node.id { 1.0 } else { 0.0 });
            }

            for (&id, node) in graph.iter() {
                let current_score = *attention.get(&id).unwrap_or(&0.0);
                if current_score > 0.0 && !node.links.is_empty() {
                    let share = (current_score * damping) / node.links.len() as f64;
                    for link in &node.links {
                        *next_attention.entry(link.target_id).or_insert(0.0) += share;
                    }
                }
            }

            attention = next_attention;
        }

        let mut results: Vec<(u64, f64)> = attention.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results.into_iter()
            .take(10) // Top 10 things in working memory
            .filter_map(|(id, score)| {
                graph.get(&id).map(|n| (n.concept.clone(), (score * 1000.0).round() / 1000.0))
            })
            .collect()
    }

    // Module 2 (Phase 8): Hopfield Networks (Intuition)
    // Energy Minimization to find an "Attractor State" (Gut Feeling)
    pub fn run_intuition(
        engine: &AxiomEngine,
        active_concepts: &[String],
    ) -> Vec<String> {
        let graph = engine.graph.read().unwrap();
        let mut state: HashMap<u64, f64> = HashMap::new();

        // Initialize state
        for concept in active_concepts {
            if let Some(node) = engine.get_node_by_concept(concept) {
                state.insert(node.id, 1.0);
            }
        }

        let mut energy_changed = true;
        let mut iterations = 0;

        while energy_changed && iterations < 10 {
            energy_changed = false;
            let mut next_state = state.clone();

            for (&id, node) in graph.iter() {
                let mut sum = 0.0;
                // Sum weights * state of neighbors
                for link in &node.links {
                    if let Some(&neighbor_val) = state.get(&link.target_id) {
                        sum += link.confidence * neighbor_val;
                    }
                }
                
                // Activation function (Sigmoid/Threshold)
                let new_val = if sum > 0.5 { 1.0 } else { 0.0 };
                let old_val = *state.get(&id).unwrap_or(&0.0);
                
                if (new_val - old_val).abs() > 0.1 {
                    next_state.insert(id, new_val);
                    energy_changed = true;
                }
            }
            state = next_state;
            iterations += 1;
        }

        state.into_iter()
            .filter(|(_, v)| *v > 0.5)
            .filter_map(|(id, _)| graph.get(&id).map(|n| n.concept.clone()))
            .filter(|c| !active_concepts.contains(c)) // Return only inferred intuition
            .take(5)
            .collect()
    }

    // Module 4 (Phase 8): Game Theory (Internal Debate / Nash Equilibrium)
    // Plays a Minimax game between two contradicting concepts.
    // Returns the ID of the winner (the concept with stronger logical backing).
    pub fn internal_debate(engine: &AxiomEngine, id_a: u64, id_b: u64) -> Option<u64> {
        let graph = engine.graph.read().unwrap();
        
        let node_a = graph.get(&id_a)?;
        let node_b = graph.get(&id_b)?;

        // Calculate Payoff (Evidence strength = Sum of incoming confidence * source activation count)
        let mut payoff_a = 0.0;
        let mut payoff_b = 0.0;

        for (_, n) in graph.iter() {
            for link in &n.links {
                if link.target_id == id_a {
                    payoff_a += link.confidence * (n.activation_count as f64);
                } else if link.target_id == id_b {
                    payoff_b += link.confidence * (n.activation_count as f64);
                }
            }
        }

        if payoff_a > payoff_b {
            Some(id_a)
        } else if payoff_b > payoff_a {
            Some(id_b)
        } else {
            engine.record_cognitive_failure("CognitiveDissonance", &node_a.concept, &node_b.concept);
            None // Tie
        }
    }

    pub fn run_internal_debate_cycle(engine: &AxiomEngine) -> Result<Vec<(String, String, String)>, String> {
        let mut contradictions = Vec::new();
        let graph = engine.graph.read().unwrap();
        
        // Find pairs of concepts that contradict each other
        for (id, node) in graph.iter() {
            for link in &node.links {
                if link.relation == Relation::Contradicts {
                    contradictions.push((*id, link.target_id));
                }
            }
        }
        drop(graph);

        let mut resolutions = Vec::new();

        for (id_a, id_b) in contradictions {
            if let Some(winner_id) = Reasoner::internal_debate(engine, id_a, id_b) {
                let loser_id = if winner_id == id_a { id_b } else { id_a };
                
                // Fetch names
                let graph = engine.graph.read().unwrap();
                let winner_name = graph.get(&winner_id).unwrap().concept.clone();
                let loser_name = graph.get(&loser_id).unwrap().concept.clone();
                drop(graph);

                // Lower the activation count and links of the loser (Penalty)
                let mut graph = engine.graph.write().unwrap();
                if let Some(loser_node) = graph.get_mut(&loser_id) {
                    loser_node.activation_count = loser_node.activation_count.saturating_sub(1);
                    // Remove the contradiction link so it doesn't loop forever
                    loser_node.links.retain(|l| l.target_id != winner_id || l.relation != Relation::Contradicts);
                }
                if let Some(winner_node) = graph.get_mut(&winner_id) {
                    winner_node.activation_count += 2;
                    winner_node.links.retain(|l| l.target_id != loser_id || l.relation != Relation::Contradicts);
                }
                
                resolutions.push((winner_name, loser_name, "Nash Equilibrium Penalty applied to loser".to_string()));
            }
        }

        Ok(resolutions)
    }

    // Module 2 (Phase 9): Group Theory (Domain Fusion / Structural Isomorphism)
    // Searches for structurally identical subgraph patterns (symmetries) between two distinct nodes/domains
    // If it finds a perfect match, it creates an Equivalent link, effectively "Fusing" knowledge.
    pub fn fuse_domains(engine: &AxiomEngine, domain_a_id: u64, domain_b_id: u64) -> Option<(String, String, bool)> {
        let graph = engine.graph.read().unwrap();
        
        let node_a = graph.get(&domain_a_id)?;
        let node_b = graph.get(&domain_b_id)?;

        let mut a_relations: HashMap<Relation, usize> = HashMap::new();
        for link in &node_a.links {
            *a_relations.entry(link.relation.clone()).or_insert(0) += 1;
        }

        let mut b_relations: HashMap<Relation, usize> = HashMap::new();
        for link in &node_b.links {
            *b_relations.entry(link.relation.clone()).or_insert(0) += 1;
        }

        let is_isomorphic = a_relations.len() > 1 && a_relations == b_relations;
        
        let mut shared_keys = 0;
        for key in a_relations.keys() {
            if b_relations.contains_key(key) {
                shared_keys += 1;
            }
        }
        let similarity = if !a_relations.is_empty() && !b_relations.is_empty() {
            (shared_keys as f64) / (a_relations.len().min(b_relations.len()) as f64)
        } else {
            0.0
        };

        let a_concept = node_a.concept.clone();
        let b_concept = node_b.concept.clone();
        drop(graph);

        if is_isomorphic {
            let _ = engine.add_link(domain_a_id, domain_b_id, Relation::Equivalent, 0.95);
            let _ = engine.add_link(domain_b_id, domain_a_id, Relation::Equivalent, 0.95);
            return Some((a_concept, b_concept, true));
        } else if similarity >= 0.5 {
            // Category Theory (Functor) Mapping
            let conf = (similarity * 100.0).round() / 100.0;
            let _ = engine.add_link(domain_a_id, domain_b_id, Relation::Mapped_Via, conf);
            return Some((a_concept, b_concept, false));
        }

        engine.record_cognitive_failure("TopologyMismatch", &a_concept, &b_concept);
        None
    }

    pub fn run_contradiction_dream(engine: &AxiomEngine, trials: usize) -> Result<Vec<(String, String)>, String> {
        use rand::seq::SliceRandom;
        let graph = engine.graph.read().unwrap();
        let mut keys: Vec<u64> = graph.keys().cloned().collect();
        let mut rng = rand::thread_rng();
        keys.shuffle(&mut rng);
        
        let mut to_contradict = Vec::new();

        for &id_a in keys.iter().take(trials) {
            if let Some(node_a) = graph.get(&id_a) {
                // If A is pure math/logic
                let is_a_abstract = node_a.dimension.contains("Math") || node_a.dimension.contains("Logic");
                
                for link in &node_a.links {
                    if let Some(node_b) = graph.get(&link.target_id) {
                        let is_b_applied = node_b.dimension.contains("Physics") || node_b.dimension.contains("Applied");
                        
                        // If they are linked but from wildly different dimensions
                        if (is_a_abstract && is_b_applied) || (is_b_applied && is_a_abstract) {
                            // Check if they share any neighbors (a rigorous bridge)
                            let a_neighbors: std::collections::HashSet<_> = node_a.links.iter().map(|l| l.target_id).collect();
                            let b_neighbors: std::collections::HashSet<_> = node_b.links.iter().map(|l| l.target_id).collect();
                            
                            let intersection: Vec<_> = a_neighbors.intersection(&b_neighbors).collect();
                            if intersection.is_empty() {
                                // NO bridge! This is a hallucination.
                                to_contradict.push((id_a, link.target_id, node_a.concept.clone(), node_b.concept.clone()));
                            }
                        }
                    }
                }
            }
        }
        drop(graph);

        let mut results = Vec::new();
        for (id_a, id_b, concept_a, concept_b) in to_contradict {
            engine.record_cognitive_failure("CognitiveDissonance", &concept_a, &concept_b);
            let _ = engine.add_link(id_a, id_b, Relation::Contradicts, 0.99);
            results.push((concept_a, concept_b));
        }

        Ok(results)
    }

    pub fn run_proof_dream(engine: &AxiomEngine, trials: usize) -> Result<Vec<(String, String)>, String> {
        use rand::seq::SliceRandom;
        let graph = engine.graph.read().unwrap();
        let mut keys: Vec<u64> = graph.keys().cloned().collect();
        let mut rng = rand::thread_rng();
        keys.shuffle(&mut rng);

        let mut to_prove = Vec::new();

        for &id_a in keys.iter().take(trials) {
            if let Some(node_a) = graph.get(&id_a) {
                for link_ab in &node_a.links {
                    if link_ab.relation == Relation::Implies {
                        let id_b = link_ab.target_id;
                        if let Some(node_b) = graph.get(&id_b) {
                            for link_bc in &node_b.links {
                                if link_bc.relation == Relation::Implies {
                                    let id_c = link_bc.target_id;
                                    // If A implies B, and B implies C, and B is Formal Logic / Mathematics
                                    if node_b.dimension.contains("Math") || node_b.dimension.contains("Logic") {
                                        to_prove.push((id_a, id_c));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(graph);

        let mut proven = Vec::new();
        for (id_a, id_c) in to_prove {
            let conf = 0.95; // Rigorous proof
            if let Ok(_) = engine.add_link(id_a, id_c, Relation::Proves, conf) {
                let graph = engine.graph.read().unwrap();
                if let (Some(a), Some(c)) = (graph.get(&id_a), graph.get(&id_c)) {
                    proven.push((a.concept.clone(), c.concept.clone()));
                }
            }
        }
        Ok(proven)
    }

    pub fn run_evolutionary_dream(engine: &AxiomEngine, trials: usize) -> Result<Vec<(String, String, String)>, String> {
        use rand::seq::SliceRandom;
        let graph = engine.graph.read().unwrap();
        let mut keys: Vec<u64> = graph.keys().cloned().collect();
        let mut rng = rand::thread_rng();
        keys.shuffle(&mut rng);

        let mut to_invent = Vec::new();

        // Evolutionary rule: if nodes are from different dimensions but share >= 3 neighbors, invent a bond.
        for &id_a in keys.iter().take(trials) {
            let node_a = match graph.get(&id_a) {
                Some(n) => n,
                None => continue,
            };

            for &id_b in keys.iter().take(trials).rev() {
                if id_a == id_b { continue; }
                let node_b = match graph.get(&id_b) {
                    Some(n) => n,
                    None => continue,
                };

                if node_a.dimension != node_b.dimension && !node_a.dimension.is_empty() && !node_b.dimension.is_empty() {
                    let a_neighbors: std::collections::HashSet<_> = node_a.links.iter().map(|l| l.target_id).collect();
                    let b_neighbors: std::collections::HashSet<_> = node_b.links.iter().map(|l| l.target_id).collect();
                    
                    let shared = a_neighbors.intersection(&b_neighbors).count();
                    if shared >= 3 {
                        // INVENT A LAW
                        let dim_a = node_a.dimension.split_whitespace().next().unwrap_or("Alpha");
                        let dim_b = node_b.dimension.split_whitespace().next().unwrap_or("Omega");
                        let law_name = format!("{}-{}_Emergence", dim_a, dim_b);
                        to_invent.push((id_a, id_b, node_a.concept.clone(), node_b.concept.clone(), law_name));
                        break; // one invention per node_a
                    }
                }
            }
        }
        drop(graph);

        let mut invented_laws = Vec::new();
        for (id_a, id_b, concept_a, concept_b, law_name) in to_invent {
            let conf = 0.88;
            if let Ok(_) = engine.add_link(id_a, id_b, Relation::Custom(law_name.clone()), conf) {
                invented_laws.push((concept_a, concept_b, law_name));
            }
        }
        
        Ok(invented_laws)
    }

    /// Spreading Activation (Heat Diffusion on Graph) — The "Mouth" Mathematics
    /// 
    /// Models how the human brain serializes parallel neural activation into
    /// sequential speech output (Broca's Area linearization).
    ///
    /// Algorithm: ∂u/∂t = -L·u (Heat Equation on Graph)
    /// - Seed nodes are "heated" (activation = 1.0)
    /// - Heat spreads through synapses weighted by confidence
    /// - Decay ensures distant concepts get less activation
    /// - Output is read in activation order (highest first) = natural serial chain
    /// - No artificial limits — energy decay is the natural stop condition
    pub fn trace_thought_path(
        engine: &AxiomEngine,
        seed_concepts: &[String],
    ) -> String {
        let graph = engine.graph.read().unwrap();
        
        if graph.is_empty() {
            return "My neural graph is currently empty. I have no thoughts to express.".to_string();
        }

        // --- Phase 1: Find Seed Nodes ---
        let mut activation: HashMap<u64, f64> = HashMap::new();
        let mut activated_via: HashMap<u64, (u64, Relation, f64)> = HashMap::new(); // child -> (parent, relation, confidence)
        let mut seeds_found = Vec::new();

        for concept in seed_concepts {
            let clean = concept.to_lowercase();
            let clean = clean.trim();
            
            // Try exact match first, then substring match
            let mut found_id = None;
            for (&id, node) in graph.iter() {
                if node.concept.to_lowercase().trim() == clean {
                    found_id = Some(id);
                    break;
                }
            }
            
            // Fuzzy substring fallback
            if found_id.is_none() {
                for (&id, node) in graph.iter() {
                    if node.concept.to_lowercase().contains(clean) || clean.contains(&node.concept.to_lowercase()) {
                        found_id = Some(id);
                        break;
                    }
                }
            }

            if let Some(id) = found_id {
                activation.insert(id, 1.0);
                seeds_found.push(id);
            }
        }

        if seeds_found.is_empty() {
            return format!(
                "I searched my neural graph but could not find any concept matching: [{}]. This is a blind spot in my knowledge.",
                seed_concepts.join(", ")
            );
        }

        // --- Phase 2: Heat Diffusion (Spreading Activation) ---
        // NOW WITH: Sheaf Context, Hebbian Learning, Lateral Inhibition, Winner-Take-All
        let damping: f64 = 0.7;    // How much energy passes through each synapse
        let decay: f64 = 0.5;      // How much energy the sender retains after spreading
        let iterations = 8;         // Diffusion depth (how many hops)
        let min_threshold = 0.01;   // Below this = not activated = natural stop
        let top_k = 30;             // Winner-Take-All: only top-K survive per iteration

        // Determine seed dimensions for Sheaf contextual gating
        let seed_dimensions: HashSet<String> = seeds_found.iter()
            .filter_map(|id| graph.get(id).map(|n| n.dimension.clone()))
            .collect();

        // Track links used for Hebbian reinforcement
        let mut hebbian_links: Vec<(u64, u64)> = Vec::new();

        for iter in 0..iterations {
            let mut new_activation: HashMap<u64, f64> = HashMap::new();

            for (&node_id, &current_energy) in activation.iter() {
                if current_energy < min_threshold {
                    continue;
                }

                if let Some(node) = graph.get(&node_id) {
                    let num_links = node.links.len().max(1) as f64;
                    
                    for link in &node.links {
                        if !link.relation.is_traversable() {
                            // Phase 3: Inhibitory links produce NEGATIVE heat
                            if link.relation == Relation::Custom("Inhibits".to_string()) {
                                let inhibit_transfer = -current_energy * link.confidence * damping / num_links.sqrt();
                                let neighbor_current = new_activation.get(&link.target_id).copied()
                                    .unwrap_or_else(|| activation.get(&link.target_id).copied().unwrap_or(0.0));
                                new_activation.insert(link.target_id, (neighbor_current + inhibit_transfer).max(0.0));
                            }
                            continue;
                        }
                        
                        // === Sheaf Contextual Gating (Phase 3.3) ===
                        // Same-dimension links get full heat transfer.
                        // Cross-dimension links get reduced transfer (domain compatibility factor).
                        let domain_factor = if let Some(target_node) = graph.get(&link.target_id) {
                            if seed_dimensions.is_empty() || seed_dimensions.contains(&target_node.dimension) {
                                1.0 // Same domain as query → full heat
                            } else if target_node.dimension == node.dimension {
                                0.9 // Same domain as source → mostly pass
                            } else {
                                0.3 // Cross-domain → heavily reduced (was 1.0 before, now 0.3)
                            }
                        } else {
                            0.5
                        };

                        // Heat transfer with Sheaf context
                        let transfer = current_energy * link.confidence * damping * domain_factor / num_links.sqrt();
                        
                        let neighbor_current = new_activation.get(&link.target_id).copied()
                            .unwrap_or_else(|| activation.get(&link.target_id).copied().unwrap_or(0.0));
                        
                        let new_val = neighbor_current + transfer;
                        new_activation.insert(link.target_id, new_val);

                        // Track for Hebbian reinforcement
                        if transfer > 0.05 {
                            hebbian_links.push((node_id, link.target_id));
                        }

                        // Track the strongest activation path (for route tracing)
                        let should_update = match activated_via.get(&link.target_id) {
                            Some(&(_, _, old_conf)) => transfer > old_conf * damping,
                            None => !seeds_found.contains(&link.target_id),
                        };
                        
                        if should_update {
                            activated_via.insert(link.target_id, (node_id, link.relation.clone(), link.confidence));
                        }
                    }

                    // === Lateral Inhibition (Phase 3.3) ===
                    // Competing nodes in the SAME dimension suppress each other slightly
                    if iter >= 2 { // Only kick in after 2 iterations (let activation spread first)
                        for (&other_id, &other_energy) in activation.iter() {
                            if other_id == node_id || other_energy < min_threshold { continue; }
                            if let Some(other_node) = graph.get(&other_id) {
                                if other_node.dimension == node.dimension && !seeds_found.contains(&other_id) {
                                    // Suppress the weaker competitor
                                    if other_energy < current_energy {
                                        let suppression = other_energy * 0.1;
                                        let current = new_activation.get(&other_id).copied().unwrap_or(other_energy);
                                        new_activation.insert(other_id, (current - suppression).max(0.0));
                                    }
                                }
                            }
                        }
                    }
                }

                // Sender decays (loses energy after spreading)
                let decayed = current_energy * decay;
                let existing = new_activation.get(&node_id).copied().unwrap_or(0.0);
                new_activation.insert(node_id, existing + decayed);
            }

            // === Winner-Take-All (Phase 3.4) ===
            // After each iteration, only keep the top-K activated nodes
            if new_activation.len() > top_k {
                let mut sorted: Vec<(u64, f64)> = new_activation.into_iter().collect();
                sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                // Always keep seeds alive
                new_activation = HashMap::new();
                for (id, score) in sorted.into_iter().take(top_k) {
                    new_activation.insert(id, score);
                }
                // Re-add seeds if they were cut
                for &seed_id in &seeds_found {
                    new_activation.entry(seed_id).or_insert(min_threshold);
                }
            }

            activation = new_activation;
        }

        // === Hebbian Learning (Phase 3.1) ===
        // Links that were traversed during this query get slightly stronger
        // (Must be done outside graph read lock)
        drop(graph);
        for (src, tgt) in hebbian_links {
            let mut g = engine.graph.write().unwrap();
            if let Some(node) = g.get_mut(&src) {
                for link in node.links.iter_mut() {
                    if link.target_id == tgt {
                        // Hebbian: conf_new = conf_old + α × (1 - conf_old), α = 0.005
                        link.confidence = (link.confidence + 0.005 * (1.0 - link.confidence)).min(0.99);
                    }
                }
            }
        }
        let graph = engine.graph.read().unwrap();

        // --- Phase 3: Serial Readout (Linearization) ---
        // Sort ALL activated nodes by score (highest first) — NO LIMIT
        let mut sorted_nodes: Vec<(u64, f64)> = activation.into_iter()
            .filter(|&(_, score)| score >= min_threshold)
            .collect();
        sorted_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Build the serial output chain
        let mut output = String::new();
        output.push_str(&format!("=== VYOUM TRAIN OF THOUGHT ({} concepts activated) ===\n\n", sorted_nodes.len()));

        for (step, (node_id, score)) in sorted_nodes.iter().enumerate() {
            if let Some(node) = graph.get(node_id) {
                // Step header
                output.push_str(&format!(
                    "Step {}: [{}] (Dimension: {}, Activation: {:.3})\n",
                    step + 1, node.concept, node.dimension, score
                ));

                // How did we get here? (Route trace)
                if let Some((parent_id, ref relation, confidence)) = activated_via.get(node_id) {
                    if let Some(parent) = graph.get(parent_id) {
                        output.push_str(&format!(
                            "  ← Reached via: [{}] {} [{}] (confidence: {:.2})\n",
                            parent.concept, relation.symbol(), node.concept, confidence
                        ));
                    }
                } else if seeds_found.contains(node_id) {
                    output.push_str("  ← [SEED: Direct match to user query]\n");
                }

                // Show this node's strongest outgoing connections
                let mut sorted_links = node.links.clone();
                sorted_links.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
                
                for link in sorted_links.iter().take(5) {
                    if let Some(target) = graph.get(&link.target_id) {
                        output.push_str(&format!(
                            "  → {} [{}] (confidence: {:.2})\n",
                            link.relation.symbol(), target.concept, link.confidence
                        ));
                    }
                }
                output.push('\n');
            }
        }

        if sorted_nodes.is_empty() {
            output.push_str("No concepts were activated strongly enough to form a coherent thought chain.\n");
        }

        output
    }
    // Relation-Filtered Spreading Activation
    // Only spreads through edges of a specific relation type.
    // None = all traversable edges (same as trace_thought_path)
    // Some("Causes")      => answers WHY questions
    // Some("Contradicts") => answers GALAT SABIT / disprove questions
    // Some("Proves")      => answers PROOF questions
    // Some("Implies")     => answers WHAT FOLLOWS questions
    pub fn trace_thought_path_filtered(
        engine: &AxiomEngine,
        seed_concepts: &[String],
        relation_filter: Option<&str>,
    ) -> String {
        let graph = engine.graph.read().unwrap();
        if graph.is_empty() {
            return "My neural graph is currently empty.".to_string();
        }

        let mut activation: HashMap<u64, f64> = HashMap::new();
        let mut activated_via: HashMap<u64, (u64, Relation, f64)> = HashMap::new();
        let mut seeds_found = Vec::new();

        for concept in seed_concepts {
            let clean = concept.to_lowercase();
            let clean = clean.trim();
            let mut found_id = None;
            for (&id, node) in graph.iter() {
                if node.concept.to_lowercase().trim() == clean {
                    found_id = Some(id); break;
                }
            }
            if found_id.is_none() {
                for (&id, node) in graph.iter() {
                    if node.concept.to_lowercase().contains(clean) || clean.contains(&node.concept.to_lowercase()) {
                        found_id = Some(id); break;
                    }
                }
            }
            if let Some(id) = found_id {
                activation.insert(id, 1.0);
                seeds_found.push(id);
            }
        }

        if seeds_found.is_empty() {
            return format!("No concept found matching: [{}].", seed_concepts.join(", "));
        }

        let damping = 0.7f64;
        let decay = 0.5f64;
        let min_threshold = 0.01f64;

        for _ in 0..8 {
            let mut new_act: HashMap<u64, f64> = HashMap::new();
            for (&nid, &energy) in activation.iter() {
                if energy < min_threshold { continue; }
                if let Some(node) = graph.get(&nid) {
                    let n = node.links.len().max(1) as f64;
                    for link in &node.links {
                        let ok = match relation_filter {
                            Some(f) => link.relation.as_str().to_lowercase() == f.to_lowercase(),
                            None    => link.relation.is_traversable(),
                        };
                        if !ok { continue; }
                        let e = (energy * damping * link.confidence) / n;
                        let slot = new_act.entry(link.target_id).or_insert(0.0);
                        if e > *slot {
                            *slot = e;
                            activated_via.entry(link.target_id)
                                .or_insert((nid, link.relation.clone(), link.confidence));
                        }
                    }
                    let r = energy * decay;
                    let s = new_act.entry(nid).or_insert(0.0);
                    if r > *s { *s = r; }
                }
            }
            for (id, e) in new_act {
                let slot = activation.entry(id).or_insert(0.0);
                if e > *slot { *slot = e; }
            }
        }

        let mut sorted: Vec<(u64, f64)> = activation.into_iter()
            .filter(|&(_, s)| s >= min_threshold).collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let flabel = relation_filter.unwrap_or("all");
        let mut out = format!(
            "=== VYOUM FILTERED THOUGHT (relation={}, {} concepts) ===\n\n",
            flabel, sorted.len()
        );
        for (step, (nid, score)) in sorted.iter().enumerate() {
            if let Some(node) = graph.get(nid) {
                out.push_str(&format!("Step {}: [{}] (Activation: {:.3})\n", step + 1, node.concept, score));
                if let Some((pid, ref rel, conf)) = activated_via.get(nid) {
                    if let Some(par) = graph.get(pid) {
                        out.push_str(&format!("  <- via: [{}] {} (conf: {:.2})\n", par.concept, rel.symbol(), conf));
                    }
                } else if seeds_found.contains(nid) {
                    out.push_str("  <- [SEED]\n");
                }
                out.push('\n');
            }
        }
        if sorted.is_empty() {
            out.push_str(&format!("No '{}' relation chains found from given seeds.\n", flabel));
        }
        out
    }
}