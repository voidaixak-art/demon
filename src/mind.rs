use crate::engine::AxiomEngine;
use crate::vector_engine::VectorEngine;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_cognitive_loop(engine: AxiomEngine) {
    println!("🧠 [Mind] Cognitive Background Thread started (System 2).");
    println!("🧠 [Mind] The Engine is now a living entity capable of offline deduction and dreaming.");

    loop {
        // Sleep for a set interval (simulate waking state before taking action)
        sleep(Duration::from_secs(10)).await;

        // Perform a Dream Mutation (offline vector similarity mapping)
        // This simulates the 'CONNECT' and 'DREAM' cycles without LLMs.
        match VectorEngine::run_dream_mutation(&engine, 20, 0.65) {
            Ok(report) => {
                if !report.successful_synapses.is_empty() {
                    println!("\n🧠 [Mind] Offline Dream Cycle Complete!");
                    for syn in report.successful_synapses {
                        println!("   ⚡ Autonomous Synapse: [{}] ===(Evolved_Vector_Link)===> [{}] (Confidence: {:.2})", 
                            syn.source_concept, syn.target_concept, syn.similarity);
                    }
                }
                
                if report.consolidated_orphans > 0 {
                    println!("   🧹 Consolidated {} orphan nodes.", report.consolidated_orphans);
                }
            }
            Err(e) => {
                eprintln!("🧠 [Mind] Error during cognitive cycle: {}", e);
            }
        }
        
        // TODO: In the future, we can add the Dijkstra Reasoner here to deduce A -> C paths.
    }
}
