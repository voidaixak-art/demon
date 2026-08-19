use crate::engine::AxiomEngine;
use crate::reasoner::Reasoner;
use crate::relations::Relation;
use crate::vector_engine::VectorEngine;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub engine: AxiomEngine,
}

#[derive(Deserialize)]
pub struct AddNodeRequest {
    pub concept: String,
    pub dimension: Option<String>,
    pub vector: Option<Vec<f64>>,
}

#[derive(Deserialize)]
pub struct AddLinkRequest {
    pub source_id: u64,
    pub target_id: u64,
    pub relation: String,
    pub confidence: Option<f64>,
}

#[derive(Deserialize)]
pub struct DeduceRequest {
    pub start: String,
    pub goal: String,
}

#[derive(Deserialize)]
pub struct DreamRequest {
    pub trials: Option<usize>,
    pub threshold: Option<f64>,
}

#[derive(Deserialize)]
pub struct HealRequest {
    pub id1: u64,
    pub id2: u64,
}

#[derive(Deserialize)]
pub struct ContextRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct AskQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct LogRequest {
    pub stage: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
}

pub fn create_router(engine: AxiomEngine) -> Router {
    let state = AppState { engine };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/stats", get(get_stats_handler))
        .route("/api/nodes", get(get_nodes_handler).post(add_node_handler))
        .route("/api/nodes/:id", get(get_node_by_id_handler))
        .route("/api/links", get(get_links_handler).post(add_link_handler))
        .route("/api/deduce", post(deduce_handler))
        .route("/api/orphans", get(get_orphans_handler))
        .route("/api/dream", post(dream_handler))
        .route("/api/contradictions", get(get_contradictions_handler))
        .route("/api/heal", post(heal_handler))
        .route("/api/logs", get(get_logs_handler).post(add_log_handler))
        .route("/api/context", post(get_context_handler))
        .route("/api/ask", get(ask_handler))
        .route("/api/centrality", get(get_centrality_handler))
        .route("/api/domains", get(get_domains_handler))
        .route("/api/attention", post(get_attention_handler))
        .route("/api/surprise", post(get_surprise_handler))
        .route("/api/intuition", post(get_intuition_handler))
        .route("/api/momentum", get(get_momentum_handler))
        .route("/api/fusion", post(post_fusion_handler))
        .fallback_service(ServeDir::new("dashboard"))
        .layer(cors)
        .with_state(state)
}

async fn get_stats_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let stats = state.engine.get_stats();
    Json(stats)
}

async fn get_nodes_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let graph = state.engine.graph.read().unwrap();
    let nodes: Vec<_> = graph.values().cloned().collect();
    Json(serde_json::json!({ "nodes": nodes }))
}

async fn get_node_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(node) = state.engine.get_node(id) {
        Ok(Json(serde_json::to_value(node).unwrap()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn add_node_handler(
    State(state): State<AppState>,
    Json(req): Json<AddNodeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let dim = req.dimension.unwrap_or_else(|| "General".to_string());
    match state.engine.add_or_get_node(&req.concept, &dim, req.vector) {
        Ok((id, is_new)) => Ok(Json(serde_json::json!({
            "id": id,
            "concept": req.concept,
            "dimension": dim,
            "is_new": is_new
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_links_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let storage = state.engine.storage.read().unwrap();
    match storage.load_all_links() {
        Ok(links) => Json(serde_json::json!({ "links": links })),
        Err(_) => Json(serde_json::json!({ "links": [] })),
    }
}

async fn add_link_handler(
    State(state): State<AppState>,
    Json(req): Json<AddLinkRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rel = Relation::from_str(&req.relation);
    let conf = req.confidence.unwrap_or(1.0);

    match state.engine.add_link(req.source_id, req.target_id, rel.clone(), conf) {
        Ok(true) => Ok(Json(serde_json::json!({
            "success": true,
            "source_id": req.source_id,
            "target_id": req.target_id,
            "relation": rel.as_str(),
            "confidence": conf
        }))),
        Ok(false) => Err(StatusCode::BAD_REQUEST),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn deduce_handler(
    State(state): State<AppState>,
    Json(req): Json<DeduceRequest>,
) -> Json<serde_json::Value> {
    let result = Reasoner::prove_deduction(&state.engine, &req.start, &req.goal);
    Json(serde_json::to_value(result).unwrap())
}

async fn get_orphans_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let orphans = state.engine.find_orphan_nodes();
    Json(serde_json::json!({ "orphans": orphans }))
}

async fn dream_handler(
    State(state): State<AppState>,
    Json(req): Json<DreamRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let trials = req.trials.unwrap_or(50);
    let threshold = req.threshold.unwrap_or(0.55);

    match VectorEngine::run_dream_mutation(&state.engine, trials, threshold) {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_contradictions_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conflicts = state.engine.check_contradictions();
    let formatted: Vec<_> = conflicts
        .into_iter()
        .map(|(n1, n2, conf)| {
            serde_json::json!({
                "source": n1,
                "target": n2,
                "confidence": conf
            })
        })
        .collect();
    Json(serde_json::json!({ "contradictions": formatted }))
}

async fn heal_handler(
    State(state): State<AppState>,
    Json(req): Json<HealRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.engine.resolve_contradiction(req.id1, req.id2) {
        Ok(success) => Ok(Json(serde_json::json!({ "success": success }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_logs_handler(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(50);
    let storage = state.engine.storage.read().unwrap();
    let logs = storage.get_recent_stream_logs(limit).unwrap_or_default();
    Json(serde_json::json!({ "logs": logs }))
}

async fn add_log_handler(
    State(state): State<AppState>,
    Json(req): Json<LogRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let storage = state.engine.storage.read().unwrap();
    match storage.log_stream(&req.stage, &req.message) {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_context_handler(
    State(state): State<AppState>,
    Json(req): Json<ContextRequest>,
) -> Json<serde_json::Value> {
    let limit = req.limit.unwrap_or(6);
    let context = Reasoner::retrieve_subgraph_context(&state.engine, &req.query, limit);
    Json(serde_json::json!({ "context": context }))
}

async fn get_centrality_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let top_nodes = state.engine.get_degree_centrality();
    let graph = state.engine.graph.read().unwrap();
    let mut results = Vec::new();

    // Return the top 10 core concepts (Wisdom)
    for (id, score) in top_nodes.into_iter().take(10) {
        if let Some(node) = graph.get(&id) {
            results.push(serde_json::json!({
                "concept": node.concept,
                "centrality_score": score
            }));
        }
    }

    Json(serde_json::json!({ "core_beliefs": results }))
}

async fn get_domains_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let domains = crate::clustering::Clustering::run_lpa(&state.engine, 10);
    let graph = state.engine.graph.read().unwrap();
    
    // Group concepts by domain ID
    let mut clustered: std::collections::HashMap<u64, Vec<String>> = std::collections::HashMap::new();
    for (node_id, domain_id) in domains {
        if let Some(node) = graph.get(&node_id) {
            clustered.entry(domain_id).or_default().push(node.concept.clone());
        }
    }

    // Convert domain_id to actual concept names for readability
    let mut named_clusters = std::collections::HashMap::new();
    for (domain_id, concepts) in clustered {
        let domain_name = graph.get(&domain_id).map(|n| n.concept.clone()).unwrap_or_else(|| format!("Domain {}", domain_id));
        named_clusters.insert(domain_name, concepts);
    }

    Json(serde_json::json!({ "domains": named_clusters }))
}

#[derive(Deserialize)]
pub struct AttentionRequest {
    pub concept: String,
}

async fn get_attention_handler(
    State(state): State<AppState>,
    Json(req): Json<AttentionRequest>,
) -> Json<serde_json::Value> {
    let memory = Reasoner::compute_attention(&state.engine, &req.concept);
    let formatted: Vec<_> = memory.into_iter().map(|(c, s)| {
        serde_json::json!({ "concept": c, "attention_score": s })
    }).collect();

    Json(serde_json::json!({ "working_memory": formatted }))
}

#[derive(Deserialize)]
pub struct SurpriseRequest {
    pub concept: String,
}

async fn get_surprise_handler(
    State(state): State<AppState>,
    Json(req): Json<SurpriseRequest>,
) -> Json<serde_json::Value> {
    let score = state.engine.calculate_surprise(&req.concept);
    Json(serde_json::json!({ "concept": req.concept, "surprise_score": (score * 1000.0).round() / 1000.0 }))
}

#[derive(Deserialize)]
pub struct IntuitionRequest {
    pub concepts: Vec<String>,
}

async fn get_intuition_handler(
    State(state): State<AppState>,
    Json(req): Json<IntuitionRequest>,
) -> Json<serde_json::Value> {
    let intuition = Reasoner::run_intuition(&state.engine, &req.concepts);
    Json(serde_json::json!({ "intuition": intuition }))
}

// Module 1 (Phase 9): Calculus - Get Momentum (Trends)
async fn get_momentum_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let graph = state.engine.graph.read().unwrap();
    let mut trends: Vec<(String, f64)> = Vec::new();
    
    for (id, node) in graph.iter() {
        if let Some(momentum) = state.engine.calculate_momentum(*id) {
            if momentum.abs() > 0.0 { // Only return concepts with some momentum
                trends.push((node.concept.clone(), momentum));
            }
        }
    }
    
    // Sort by momentum (highest positive first)
    trends.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Take top 10 trending up, top 10 trending down
    let trending_up: Vec<_> = trends.iter().filter(|(_, m)| *m > 0.0).take(10).collect();
    let trending_down: Vec<_> = trends.iter().filter(|(_, m)| *m < 0.0).rev().take(10).collect();

    Json(serde_json::json!({
        "trending_up": trending_up,
        "trending_down": trending_down
    }))
}

#[derive(Deserialize)]
pub struct FusionRequest {
    pub domain_a_id: u64,
    pub domain_b_id: u64,
}

// Module 2 (Phase 9): Group Theory - Fuse Domains
async fn post_fusion_handler(State(state): State<AppState>, Json(req): Json<FusionRequest>) -> Json<serde_json::Value> {
    let result = Reasoner::fuse_domains(&state.engine, req.domain_a_id, req.domain_b_id);
    
    if let Some((a, b, is_iso)) = result {
        if is_iso {
            Json(serde_json::json!({
                "success": true,
                "message": format!("Group Isomorphism found! Fused [{}] with [{}]", a, b)
            }))
        } else {
            Json(serde_json::json!({
                "success": true,
                "message": format!("Functor Mapped! [{}] mapped to [{}] via Category Theory", a, b)
            }))
        }
    } else {
        Json(serde_json::json!({
            "success": false,
            "message": "No structural symmetry found between domains."
        }))
    }
}

pub async fn run_server(engine: AxiomEngine, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Spawn the cognitive mind loop in the background!
    let engine_clone = engine.clone();
    tokio::spawn(async move {
        crate::mind::start_cognitive_loop(engine_clone).await;
    });

    let app = create_router(engine);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🌌 AXIOM ENGINE DAEMON ONLINE at http://localhost:{}", port);
    println!("📡 Neural Dashboard: http://localhost:{}/index.html", port);
    println!("⚡ SQLite WAL Persistence Active (zero JSON bottlenecks)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ask_handler(
    State(state): State<AppState>,
    Query(params): Query<AskQuery>,
) -> Json<serde_json::Value> {
    if let Some(node) = state.engine.get_node_by_concept(&params.q) {
        let graph = state.engine.graph.read().unwrap();
        
        if node.links.is_empty() {
            return Json(serde_json::json!({
                "answer": format!("I have processed the concept of '{}', but it currently exists as an isolated thought with no synaptic connections.", node.concept)
            }));
        }

        // Sort links by confidence descending
        let mut sorted_links = node.links.clone();
        sorted_links.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut implies = Vec::new();
        let mut causes = Vec::new();
        let mut equivalent = Vec::new();
        let mut others = Vec::new();

        // Limit to top 20 strongest synapses
        for link in sorted_links.iter().take(20) {
            if let Some(target) = graph.get(&link.target_id) {
                match link.relation {
                    Relation::Implies => implies.push(target.concept.clone()),
                    Relation::Causes => causes.push(target.concept.clone()),
                    Relation::Equivalent => equivalent.push(target.concept.clone()),
                    _ => others.push(format!("{} (via {:?})", target.concept, link.relation)),
                }
            }
        }

        let mut parts = vec![];
        if !implies.is_empty() {
            parts.push(format!("Through logical deduction, '{}' inherently implies: {}.", node.concept, implies.join(", ")));
        }
        if !causes.is_empty() {
            parts.push(format!("It fundamentally causes or leads to: {}.", causes.join(", ")));
        }
        if !equivalent.is_empty() {
            parts.push(format!("It is conceptually equivalent to: {}.", equivalent.join(", ")));
        }
        if !others.is_empty() {
            parts.push(format!("I've also found abstract connections to: {}.", others.join(", ")));
        }

        // Module 4: Analogical Reasoning
        if let Some((analog, conf)) = Reasoner::find_analogy(&state.engine, &node.concept) {
            parts.push(format!("(💡 Structural Analogy: Mathematically, '{}' behaves very similarly to '{}' in my neural graph. Structural similarity: {:.0}%)", node.concept, analog, conf * 100.0));
        }

        Json(serde_json::json!({
            "answer": parts.join("\n\n")
        }))
    } else {
        Json(serde_json::json!({
            "answer": format!("My subconscious has not yet encountered or analyzed the concept of '{}'. Try letting the crawlers read more.", params.q)
        }))
    }
}
