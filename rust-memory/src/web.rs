//! Web server for the associative memory UI
//!
//! Provides REST API and serves the web UI

use crate::memory::{Source, SourceOrigin, ThinkMode};
use crate::mcp::SharedMemory;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Web server state
#[derive(Clone)]
pub struct AppState {
    pub memory: SharedMemory,
    pub data_path: String,
}

/// Start the web server
pub async fn start_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // API routes
        .route("/api/nodes", get(get_nodes))
        .route("/api/edges", get(get_edges))
        .route("/api/stats", get(get_stats))
        .route("/api/add_path", post(add_path))
        .route("/api/add_connection", post(add_connection))
        .route("/api/think", post(think))
        .route("/api/search", get(search))
        .route("/api/find_path", get(find_path))
        .route("/api/associations", get(get_associations))
        .route("/api/decay", post(decay))
        .route("/api/save", post(save))
        // Provenance API routes
        .route("/api/sources", get(list_sources))
        .route("/api/register_source", post(register_source))
        .route("/api/edges_by_source", get(edges_by_source))
        .route("/api/source_overlap", get(source_overlap))
        .route("/api/timeline", get(concept_timeline))
        .route("/api/importance", get(get_importance))
        // Serve the UI
        .route("/", get(serve_ui))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Web server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the main UI page
async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

// === API Handlers ===

async fn get_nodes(State(state): State<AppState>) -> Json<Vec<String>> {
    let mem = state.memory.read().await;
    Json(mem.get_all_nodes())
}

#[derive(Serialize)]
struct EdgeResponse {
    from: String,
    to: String,
    conn_type: Option<String>,
    weight: f64,
    source_id: Option<String>,
    timestamp: Option<i64>,
}

async fn get_edges(State(state): State<AppState>) -> Json<Vec<EdgeResponse>> {
    let mem = state.memory.read().await;
    let edges: Vec<EdgeResponse> = mem
        .get_all_edges()
        .into_iter()
        .map(|e| EdgeResponse {
            from: e.from,
            to: e.to,
            conn_type: e.conn_type,
            weight: e.weight,
            source_id: e.source_id,
            timestamp: e.timestamp,
        })
        .collect();
    Json(edges)
}

async fn get_stats(State(state): State<AppState>) -> Json<Value> {
    let mem = state.memory.read().await;
    Json(serde_json::to_value(mem.stats()).unwrap_or_default())
}

#[derive(Deserialize)]
struct AddPathRequest {
    path: Vec<String>,
    conn_types: Option<Vec<String>>,
    weight: Option<f64>,
    bidirectional: Option<bool>,
    source_id: Option<String>,
    timestamp: Option<i64>,
}

async fn add_path(
    State(state): State<AppState>,
    Json(req): Json<AddPathRequest>,
) -> Json<Value> {
    let mut mem = state.memory.write().await;
    mem.add_path(
        &req.path,
        req.conn_types.as_deref(),
        req.weight.unwrap_or(0.5),
        req.bidirectional.unwrap_or(false),
        req.source_id.as_deref(),
        req.timestamp,
    );
    Json(json!({"success": true, "path": req.path}))
}

#[derive(Deserialize)]
struct AddConnectionRequest {
    from: String,
    to: String,
    conn_type: Option<String>,
    context: Option<Vec<String>>,
    weight: Option<f64>,
    bidirectional: Option<bool>,
    source_id: Option<String>,
    timestamp: Option<i64>,
}

async fn add_connection(
    State(state): State<AppState>,
    Json(req): Json<AddConnectionRequest>,
) -> Json<Value> {
    let mut mem = state.memory.write().await;
    mem.add_connection(
        &req.from,
        &req.to,
        req.conn_type.as_deref(),
        req.context.as_deref(),
        req.weight.unwrap_or(0.5),
        req.bidirectional.unwrap_or(false),
        req.source_id.as_deref(),
        req.timestamp,
    );
    Json(json!({"success": true}))
}

#[derive(Deserialize)]
struct ThinkRequest {
    start: String,
    context: Option<Vec<String>>,
    steps: Option<usize>,
    mode: Option<String>,
    conn_type_filter: Option<String>,
    reinforce: Option<bool>,
}

async fn think(
    State(state): State<AppState>,
    Json(req): Json<ThinkRequest>,
) -> Json<Value> {
    let mode = match req.mode.as_deref() {
        Some("strongest") => ThinkMode::Strongest,
        _ => ThinkMode::Weighted,
    };
    let steps = req.steps.unwrap_or(5).min(50);

    let result = if req.reinforce.unwrap_or(true) {
        let mut mem = state.memory.write().await;
        mem.think_and_reinforce(
            &req.start,
            req.context.as_deref(),
            steps,
            mode,
            req.conn_type_filter.as_deref(),
        )
    } else {
        let mem = state.memory.read().await;
        mem.think(
            &req.start,
            req.context.as_deref(),
            steps,
            mode,
            req.conn_type_filter.as_deref(),
        )
    };

    Json(serde_json::to_value(result).unwrap_or_default())
}

#[derive(Deserialize)]
struct SearchQuery {
    pattern: String,
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Value> {
    let mem = state.memory.read().await;
    let results = mem.search(&query.pattern);
    Json(serde_json::to_value(results).unwrap_or_default())
}

#[derive(Deserialize)]
struct FindPathQuery {
    from: String,
    to: String,
    max_depth: Option<usize>,
}

async fn find_path(
    State(state): State<AppState>,
    Query(query): Query<FindPathQuery>,
) -> Json<Value> {
    let max_depth = query.max_depth.unwrap_or(5).min(20);
    let mem = state.memory.read().await;
    let results = mem.find_path(&query.from, &query.to, max_depth);
    Json(serde_json::to_value(results).unwrap_or_default())
}

#[derive(Deserialize)]
struct AssociationsQuery {
    node: String,
    context: Option<String>, // comma-separated
    conn_type_filter: Option<String>,
}

async fn get_associations(
    State(state): State<AppState>,
    Query(query): Query<AssociationsQuery>,
) -> Json<Value> {
    let mem = state.memory.read().await;
    let context: Option<Vec<String>> = query
        .context
        .map(|c| c.split(',').map(String::from).collect());

    let assocs = mem.get_associations(
        &query.node,
        context.as_deref(),
        query.conn_type_filter.as_deref(),
    );

    let result: HashMap<String, Value> = assocs
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                json!({
                    "weight": v.weight,
                    "type": v.conn_type
                }),
            )
        })
        .collect();

    Json(serde_json::to_value(result).unwrap_or_default())
}

async fn decay(State(state): State<AppState>) -> Json<Value> {
    let mut mem = state.memory.write().await;
    mem.decay();
    Json(json!({"success": true}))
}

async fn save(State(state): State<AppState>) -> Json<Value> {
    let mem = state.memory.read().await;
    match mem.save(std::path::Path::new(&state.data_path)) {
        Ok(_) => Json(json!({"success": true, "path": state.data_path})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}

// ==================== Provenance API ====================

#[derive(Deserialize)]
struct RegisterSourceRequest {
    id: String,
    name: String,
    url: Option<String>,
    origin: Option<String>,  // "agent" or "manual"
    metadata: Option<HashMap<String, String>>,
}

async fn register_source(
    State(state): State<AppState>,
    Json(req): Json<RegisterSourceRequest>,
) -> Json<Value> {
    let origin = match req.origin.as_deref() {
        Some("manual") => SourceOrigin::Manual,
        _ => SourceOrigin::Agent,
    };

    let mut source = Source::new(req.id.clone(), req.name, origin);
    if let Some(url) = req.url {
        source = source.with_url(url);
    }
    if let Some(metadata) = req.metadata {
        for (k, v) in metadata {
            source = source.with_metadata(k, v);
        }
    }

    let mut mem = state.memory.write().await;
    mem.register_source(source);

    Json(json!({"success": true, "source_id": req.id}))
}

async fn list_sources(State(state): State<AppState>) -> Json<Value> {
    let mem = state.memory.read().await;
    let sources: Vec<&Source> = mem.list_sources();
    Json(serde_json::to_value(sources).unwrap_or_default())
}

#[derive(Deserialize)]
struct EdgesBySourceQuery {
    source_id: String,
}

async fn edges_by_source(
    State(state): State<AppState>,
    Query(params): Query<EdgesBySourceQuery>,
) -> Json<Value> {
    let mem = state.memory.read().await;
    let edges = mem.get_edges_by_source(&params.source_id);
    Json(serde_json::to_value(edges).unwrap_or_default())
}

#[derive(Deserialize)]
struct SourceOverlapQuery {
    a: String,
    b: String,
}

async fn source_overlap(
    State(state): State<AppState>,
    Query(params): Query<SourceOverlapQuery>,
) -> Json<Value> {
    let mem = state.memory.read().await;
    let overlap = mem.get_source_overlap(&params.a, &params.b);
    Json(serde_json::to_value(overlap).unwrap_or_default())
}

#[derive(Deserialize)]
struct TimelineQuery {
    node: String,
}

async fn concept_timeline(
    State(state): State<AppState>,
    Query(params): Query<TimelineQuery>,
) -> Json<Value> {
    let mem = state.memory.read().await;
    let timeline = mem.get_concept_timeline(&params.node);
    Json(serde_json::to_value(timeline).unwrap_or_default())
}

// ==================== Importance API ====================

#[derive(Deserialize)]
struct ImportanceQuery {
    iterations: Option<usize>,
    damping: Option<f64>,
}

async fn get_importance(
    State(state): State<AppState>,
    Query(params): Query<ImportanceQuery>,
) -> Json<Value> {
    let iterations = params.iterations.unwrap_or(20).min(100);
    let damping = params.damping.unwrap_or(0.85).clamp(0.0, 1.0);
    let mem = state.memory.read().await;
    let importance = mem.compute_importance(iterations, damping);
    Json(serde_json::to_value(importance).unwrap_or_default())
}
