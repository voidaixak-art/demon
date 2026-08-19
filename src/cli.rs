use crate::engine::AxiomEngine;
use crate::reasoner::Reasoner;
use crate::vector_engine::VectorEngine;
use std::io::{self, Write};

pub fn run_cli(engine: AxiomEngine) {
    println!("\n╔═════════════════════════════════════════════════════════════════════════╗");
    println!("║       👁️  AXIOM ENGINE 2.0: THE NEURO-SYMBOLIC COGNITIVE MIND            ║");
    println!("║       Deterministic Graph Core • Zero Hallucination • SQLite WAL        ║");
    println!("╚═════════════════════════════════════════════════════════════════════════╝\n");
    print_help();

    loop {
        print!("\naxiom> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "help" | "?" => print_help(),
            "stats" => {
                let stats = engine.get_stats();
                println!("\n📊 NEURAL COGNITION STATS:");
                println!("  • Total Concepts (Nodes)     : {}", stats["nodes"]);
                println!("  • Active Synapses (Links)    : {}", stats["links"]);
                println!("  • Synaptic Density           : {} links/node", stats["density"]);
                println!("  • Node 0 (Self) Connections  : {}", stats["self_connections"]);
            }
            "list" => {
                let filter_dim = if parts.len() > 1 { Some(parts[1].to_lowercase()) } else { None };
                let graph = engine.graph.read().unwrap();
                println!("\n🧠 MEMORY CONCEPTS ({} total):", graph.len());
                for (id, node) in graph.iter() {
                    if let Some(ref dim) = filter_dim {
                        if !node.dimension.to_lowercase().contains(dim) {
                            continue;
                        }
                    }
                    println!("  [#{:03}] {} ({}) [links: {}]", id, node.concept, node.dimension, node.links.len());
                }
            }
            "links" => {
                let graph = engine.graph.read().unwrap();
                println!("\n🔗 ACTIVE SYNAPTIC PATHWAYS:");
                let mut count = 0;
                for (src_id, node) in graph.iter() {
                    for link in &node.links {
                        let target_name = graph.get(&link.target_id).map(|n| n.concept.as_str()).unwrap_or("Unknown");
                        println!(
                            "  [#{}] {}  {}  [#{}] {} (conf: {:.2})",
                            src_id,
                            node.concept,
                            link.relation.symbol(),
                            link.target_id,
                            target_name,
                            link.confidence
                        );
                        count += 1;
                    }
                }
                println!("Total Synapses: {}", count);
            }
            "deduce" | "prove" => {
                let remainder = input.replacen("deduce", "", 1).replacen("prove", "", 1);
                let parts: Vec<&str> = remainder.split("->").collect();
                if parts.len() == 2 {
                    let start = parts[0].trim();
                    let goal = parts[1].trim();
                    let proof = Reasoner::prove_deduction(&engine, start, goal);
                    println!("\n{}", proof.explanation);
                } else {
                    println!("Usage: deduce <Start Concept> -> <Goal Concept>");
                    println!("Example: deduce The Axiom Engine (Self) -> Causality");
                }
            }
            "dream" | "mutate" => {
                println!("\n🌙 Initiating Dream / Synaptic Mutation Cycle...");
                match VectorEngine::run_dream_mutation(&engine, 100, 0.50) {
                    Ok(report) => {
                        println!("⚡ Synaptic Trials: {}", report.total_trials);
                        println!("✨ New Synapses Formed: {}", report.successful_synapses.len());
                        for s in report.successful_synapses {
                            println!("   Synapse Wired: [{}] ⚡ [{}] (score: {:.3})", s.source_concept, s.target_concept, s.similarity);
                        }
                        println!("🕸️ Remaining Orphan Nodes: {}", report.consolidated_orphans);
                    }
                    Err(e) => println!("Error in dream cycle: {:?}", e),
                }
            }
            "orphans" => {
                let orphans = engine.find_orphan_nodes();
                println!("\n🕸️ ISOLATED BLIND SPOTS (Orphan Nodes: {}):", orphans.len());
                for o in orphans {
                    println!("  - [#{}] {} ({})", o.id, o.concept, o.dimension);
                }
            }
            "contradictions" => {
                let conflicts = engine.check_contradictions();
                println!("\n⚠️ LOGICAL CONTRADICTIONS DETECTED ({}):", conflicts.len());
                for (n1, n2, conf) in conflicts {
                    println!("  [#{}] {} ⟂ [#{}] {} (conf: {:.2})", n1.id, n1.concept, n2.id, n2.concept, conf);
                }
            }
            "heal" => {
                if parts.len() == 3 {
                    if let (Ok(id1), Ok(id2)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                        match engine.resolve_contradiction(id1, id2) {
                            Ok(_) => println!("✅ Contradiction healed between #{} and #{}", id1, id2),
                            Err(e) => println!("Error: {:?}", e),
                        }
                    }
                } else {
                    println!("Usage: heal <id1> <id2>");
                }
            }
            "logs" => {
                let storage = engine.storage.read().unwrap();
                let logs = storage.get_recent_stream_logs(15).unwrap_or_default();
                println!("\n👁️ STREAM OF CONSCIOUSNESS (Recent):");
                for l in logs.iter().rev() {
                    println!("  [{}] [{}] {}", l.timestamp, l.stage, l.message);
                }
            }
            "context" | "query" => {
                let query = input.replacen("context", "", 1).replacen("query", "", 1);
                let ctx = Reasoner::retrieve_subgraph_context(&engine, query.trim(), 5);
                println!("\n{}", ctx);
            }
            "exit" | "quit" => {
                println!("Shutting down cognitive terminal. Memory preserved in SQLite.");
                break;
            }
            _ => {
                println!("Unknown command: '{}'. Type 'help' for options.", command);
            }
        }
    }
}

fn print_help() {
    println!("Available Commands:");
    println!("  stats                  - View neural graph stats and density");
    println!("  list [dimension]       - List all concepts in memory");
    println!("  links                  - Display all active causal synapses");
    println!("  deduce <A> -> <B>      - Deduce & prove strict mathematical causal path");
    println!("  dream                  - Trigger instant semantic vector mutation cycle");
    println!("  orphans                - View isolated concepts (subconscious blind spots)");
    println!("  contradictions         - Detect logical conflicts");
    println!("  heal <id1> <id2>       - Resolve contradiction between two nodes");
    println!("  logs                   - View real-time Stream of Consciousness logs");
    println!("  context <query>        - Extract relevant axiomatic subgraph for a query");
    println!("  help                   - Show this menu");
    println!("  exit                   - Exit terminal");
}
