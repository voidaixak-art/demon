use axiom_engine::engine::AxiomEngine;
use axiom_engine::reasoner::Reasoner;
use axiom_engine::relations::Relation;
use axiom_engine::storage::StorageManager;
use axiom_engine::vector_engine::VectorEngine;
use std::fs;

#[test]
fn test_seed_and_graph_initialization() {
    let test_db = "test_axiom_seed.db";
    let _ = fs::remove_file(test_db);

    let storage = StorageManager::new(test_db).expect("Failed to create storage");
    let engine = AxiomEngine::new(storage).expect("Failed to initialize engine");

    let stats = engine.get_stats();
    assert!(stats["nodes"].as_u64().unwrap() >= 5); // Node 0 + 4 foundational axioms
    assert!(stats["links"].as_u64().unwrap() >= 4);

    let self_node = engine.get_node(0).expect("Node 0 should exist");
    assert_eq!(self_node.concept, "The Axiom Engine (Self)");
    assert_eq!(self_node.dimension, "Consciousness");

    let _ = fs::remove_file(test_db);
}

#[test]
fn test_multi_step_deductive_proof() {
    let test_db = "test_axiom_proof.db";
    let _ = fs::remove_file(test_db);

    let storage = StorageManager::new(test_db).unwrap();
    let engine = AxiomEngine::new(storage).unwrap();

    // Create chain: Socrates -> IsA -> Human -> Implies -> Mortal
    let (socrates_id, _) = engine.add_or_get_node("Socrates", "Entity", None).unwrap();
    let (human_id, _) = engine.add_or_get_node("Human", "Species", None).unwrap();
    let (mortal_id, _) = engine.add_or_get_node("Mortal", "State", None).unwrap();

    engine.add_link(socrates_id, human_id, Relation::IsA, 1.0).unwrap();
    engine.add_link(human_id, mortal_id, Relation::Implies, 1.0).unwrap();

    let proof = Reasoner::prove_deduction(&engine, "Socrates", "Mortal");
    assert!(proof.proven, "Socrates -> Mortal must be proven!");
    assert_eq!(proof.steps.len(), 2);
    assert_eq!(proof.steps[0].from_node, "Socrates");
    assert_eq!(proof.steps[0].to_node, "Human");
    assert_eq!(proof.steps[1].from_node, "Human");
    assert_eq!(proof.steps[1].to_node, "Mortal");

    // Unconnected node
    let proof_unconnected = Reasoner::prove_deduction(&engine, "Socrates", "Quantum Mechanics");
    assert!(!proof_unconnected.proven);

    let _ = fs::remove_file(test_db);
}

#[test]
fn test_contradiction_detection_and_healing() {
    let test_db = "test_axiom_conflict.db";
    let _ = fs::remove_file(test_db);

    let storage = StorageManager::new(test_db).unwrap();
    let engine = AxiomEngine::new(storage).unwrap();

    let (id1, _) = engine.add_or_get_node("Free Will", "Philosophy", None).unwrap();
    let (id2, _) = engine.add_or_get_node("Hard Determinism", "Philosophy", None).unwrap();

    engine.add_link(id1, id2, Relation::Contradicts, 1.0).unwrap();

    let conflicts = engine.check_contradictions();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].0.concept, "Free Will");
    assert_eq!(conflicts[0].1.concept, "Hard Determinism");

    // Heal contradiction
    engine.resolve_contradiction(id1, id2).unwrap();
    let healed = engine.check_contradictions();
    assert_eq!(healed.len(), 0);

    let _ = fs::remove_file(test_db);
}

#[test]
fn test_vector_engine_and_mutations() {
    let v1 = VectorEngine::compute_concept_vector("Neural Networks", "Artificial Intelligence");
    let v2 = VectorEngine::compute_concept_vector("Deep Learning Neural Networks", "AI");
    let v3 = VectorEngine::compute_concept_vector("Ancient Rome Caesar", "History");

    let sim_high = VectorEngine::cosine_similarity(&v1, &v2);
    let sim_low = VectorEngine::cosine_similarity(&v1, &v3);

    assert!(sim_high > 0.4, "Similar concepts should have high cosine similarity");
    assert!(sim_high > sim_low, "Related concepts must score higher than unrelated ones");
}
