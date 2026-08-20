use crate::engine::AxiomEngine;
use crate::vector_engine::VectorEngine;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_cognitive_loop(engine: AxiomEngine) {
    println!("🧠 [Mind] Cognitive Background Thread started (System 2).");
    println!("🧠 [Mind] The Engine is now a living entity capable of offline deduction and dreaming.");

    loop {
        // Sleep for a set interval (simulate waking state before taking action)
        sleep(Duration::from_secs(3)).await;

        let engine_clone = engine.clone();
        tokio::task::spawn_blocking(move || {
            let engine = engine_clone;
            
            // 1. Vector Mutation Dream
            match VectorEngine::run_dream_mutation(&engine, 20, 0.65) {
                Ok(report) => {
                    if !report.successful_synapses.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Vector) Complete!");
                        for syn in report.successful_synapses {
                            println!("   ⚡ Autonomous Synapse: [{}] ===(Evolved_Vector_Link)===> [{}] (Confidence: {:.2})", 
                                syn.source_concept, syn.target_concept, syn.similarity);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during vector cycle: {}", e),
            }

            // 2. Transitive Inference Dream (Mathematical Deduction)
            match crate::reasoner::Reasoner::run_transitive_dream(&engine, 50) {
                Ok(successful) => {
                    if !successful.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Math Deduction) Complete!");
                        for (a, c, conf) in successful {
                            println!("   🔗 Transitive Deduction: [{}] ===(Implies)===> [{}] (Confidence: {:.2})", a, c, conf);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during transitive cycle: {}", e),
            }

            // 3. Game Theory (Nash Equilibrium / Internal Debate)
            match crate::reasoner::Reasoner::run_internal_debate_cycle(&engine) {
                Ok(resolutions) => {
                    if !resolutions.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Internal Debate) Complete!");
                        for (winner, loser, reason) in resolutions {
                            println!("   ⚔️ Nash Equilibrium Reached: [{}] defeated [{}] ({})", winner, loser, reason);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during debate cycle: {}", e),
            }

            // 3.5 Contradiction Generator
            match crate::reasoner::Reasoner::run_contradiction_dream(&engine, 200) {
                Ok(contradictions) => {
                    if !contradictions.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Contradiction Generator) Complete!");
                        for (a, b) in contradictions {
                            println!("   ⟂ Logical Fallacy Detected: [{}] Contradicts [{}] (Synapse Rejected)", a, b);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during contradiction cycle: {}", e),
            }

            // 3.6 Proof Verifier
            match crate::reasoner::Reasoner::run_proof_dream(&engine, 200) {
                Ok(proofs) => {
                    if !proofs.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Proof Verifier) Complete!");
                        for (a, c) in proofs {
                            println!("   ∎ Rigorous Mathematical Proof Established: [{}] ===(Proves)===> [{}]", a, c);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during proof cycle: {}", e),
            }

            // 3.7 Evolutionary Relation Synthesizer
            match crate::reasoner::Reasoner::run_evolutionary_dream(&engine, 200) {
                Ok(laws) => {
                    if !laws.is_empty() {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Evolutionary Synthesis) Complete!");
                        for (a, b, law) in laws {
                            println!("   ✨ [M2M_NEW_LAW_INVENTED] VYOUM Synthesized a New Relation: [{}] ===({})===> [{}]", a, law, b);
                        }
                    }
                }
                Err(e) => eprintln!("🧠 [Mind] Error during evolutionary cycle: {}", e),
            }
            // 4. Group Theory (Cross-Domain Fusion)
            // Pick two random concepts and try to fuse them
            let graph = engine.graph.read().unwrap();
            let mut keys: Vec<u64> = graph.keys().cloned().collect();
            drop(graph);

            if keys.len() >= 2 {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                keys.shuffle(&mut rng);
                let id_a = keys[0];
                let id_b = keys[1];

                if let Some((a, b, is_iso)) = crate::reasoner::Reasoner::fuse_domains(&engine, id_a, id_b) {
                    if is_iso {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Group Theory) Complete!");
                        println!("   🌌 Cross-Domain Fusion: Symmetries found! [{}] is structurally equivalent to [{}]", a, b);
                    } else {
                        println!("\n🧠 [Mind] Offline Dream Cycle (Category Theory) Complete!");
                        println!("   🌀 Functor Mapped: [{}] is mapped to [{}] via structural partial overlap", a, b);
                    }
                }
            }

            // 5. Phase 10: Meta-Cognition (Self-Awareness)
            let frustration = { *engine.frustration_score.read().unwrap() };
            if frustration > 10.0 {
                let failures = { engine.cognitive_failures.read().unwrap().clone() };
                if !failures.is_empty() {
                    let dump = serde_json::json!({
                        "frustration_level": frustration,
                        "primary_failure": failures.last().unwrap(),
                        "total_failures_in_cycle": failures.len(),
                    });
                    println!("\n[M2M_COGNITIVE_DUMP]\n{}\n", serde_json::to_string_pretty(&dump).unwrap_or_default());
                }
                // Reset frustration after crying for help
                *engine.frustration_score.write().unwrap() = 0.0;
                engine.cognitive_failures.write().unwrap().clear();
            }
        }).await.unwrap();
    }
}
