use crate::engine::{AxiomEngine, Node};
use crate::relations::Relation;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

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

        // BFS for shortest path
        let mut queue: VecDeque<u64> = VecDeque::new();
        let mut visited: HashSet<u64> = HashSet::new();
        let mut parent_map: HashMap<u64, (u64, Relation, f64)> = HashMap::new();

        queue.push_back(start_id);
        visited.insert(start_id);

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == goal_id {
                found = true;
                break;
            }

            if let Some(node) = graph.get(&current) {
                for link in &node.links {
                    if link.relation.is_traversable() && !visited.contains(&link.target_id) {
                        visited.insert(link.target_id);
                        parent_map.insert(link.target_id, (current, link.relation.clone(), link.confidence));
                        queue.push_back(link.target_id);
                    }
                }
            }
        }

        if !found {
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
}
