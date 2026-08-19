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
            
            let b_node = {
                let graph = engine.graph.read().unwrap();
                graph.get(&link_ab.target_id).cloned()
            };

            if let Some(b) = b_node {
                if b.links.is_empty() { continue; }
                
                let link_bc = b.links.choose(&mut rng).unwrap();
                let c_id = link_bc.target_id;
                
                // Don't link to self
                if c_id == a.id { continue; }
                
                // Check if A->C already exists
                let exists = a.links.iter().any(|l| l.target_id == c_id);
                
                if !exists {
                    let inferred_conf = (link_ab.confidence * link_bc.confidence * 0.9 * 100.0).round() / 100.0;
                    if inferred_conf >= 0.5 { // Only solidify somewhat strong thoughts
                        if let Ok(true) = engine.add_link(a.id, c_id, Relation::Implies, inferred_conf) {
                            let c_concept = {
                                let graph = engine.graph.read().unwrap();
                                graph.get(&c_id).map(|n| n.concept.clone()).unwrap_or_default()
                            };
                            successful.push((a.concept.clone(), c_concept, inferred_conf));
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
}
