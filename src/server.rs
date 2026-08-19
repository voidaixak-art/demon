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

        Json(serde_json::json!({
            "answer": parts.join("\n\n")
        }))
    } else {
        Json(serde_json::json!({
            "answer": format!("My subconscious has not yet encountered or analyzed the concept of '{}'. Try letting the crawlers read more.", params.q)
        }))
    }
}
