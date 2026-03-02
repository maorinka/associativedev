//! MCP Server implementation for associative memory
//!
//! Implements the Model Context Protocol using JSON-RPC over stdio

use crate::memory::{AssociativeMemory, Source, SourceOrigin, ThinkMode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

// Input validation limits
const MAX_PATH_LEN: usize = 100;
const MAX_NODE_NAME_LEN: usize = 500;
const MAX_STEPS: usize = 50;
const MAX_DEPTH: usize = 20;
const MIN_WEIGHT: f64 = 0.0;
const MAX_WEIGHT: f64 = 10.0;
const MAX_PATTERN_LEN: usize = 200;

fn validate_node_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Node name cannot be empty".into());
    }
    if name.len() > MAX_NODE_NAME_LEN {
        return Err(format!("Node name too long (max {} chars)", MAX_NODE_NAME_LEN));
    }
    Ok(())
}

fn validate_weight(weight: f64) -> Result<(), String> {
    if weight < MIN_WEIGHT || weight > MAX_WEIGHT {
        return Err(format!("Weight must be between {} and {}", MIN_WEIGHT, MAX_WEIGHT));
    }
    Ok(())
}

fn validate_path(path: &[String]) -> Result<(), String> {
    if path.len() < 2 {
        return Err("Path must have at least 2 nodes".into());
    }
    if path.len() > MAX_PATH_LEN {
        return Err(format!("Path too long (max {} nodes)", MAX_PATH_LEN));
    }
    for node in path {
        validate_node_name(node)?;
    }
    Ok(())
}

/// Shared memory state
pub type SharedMemory = Arc<RwLock<AssociativeMemory>>;

/// JSON-RPC request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// MCP Tool definition
#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP Server for associative memory
pub struct McpServer {
    memory: SharedMemory,
    data_path: String,
}

impl McpServer {
    pub fn new(memory: SharedMemory, data_path: String) -> Self {
        Self { memory, data_path }
    }

    /// Run the MCP server using stdio
    pub async fn run_stdio(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                        }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.unwrap_or(Value::Null);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize().await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(request.params).await,
            _ => Err(format!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(value),
                error: None,
            },
            Err(msg) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: msg,
                }),
            },
        }
    }

    async fn handle_initialize(&self) -> Result<Value, String> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "associative-memory",
                "version": "0.1.0"
            }
        }))
    }

    async fn handle_list_tools(&self) -> Result<Value, String> {
        let tools = vec![
            Tool {
                name: "add_path".into(),
                description: "Add a sequence of connected concepts to the memory".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "array", "items": {"type": "string"}, "description": "List of nodes to connect"},
                        "conn_types": {"type": "array", "items": {"type": "string"}, "description": "Connection types for each edge"},
                        "weight": {"type": "number", "description": "Initial weight (default 0.5)"},
                        "bidirectional": {"type": "boolean", "description": "Create reverse path (default false)"},
                        "source_id": {"type": "string", "description": "Optional source ID for provenance tracking"}
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "add_connection".into(),
                description: "Add a single connection between two nodes".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from": {"type": "string"},
                        "to": {"type": "string"},
                        "conn_type": {"type": "string"},
                        "weight": {"type": "number"},
                        "bidirectional": {"type": "boolean", "description": "Create reverse connection (default false)"},
                        "source_id": {"type": "string", "description": "Optional source ID for provenance tracking"}
                    },
                    "required": ["from", "to"]
                }),
            },
            Tool {
                name: "think".into(),
                description: "Start from a concept and follow associations".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "start": {"type": "string", "description": "Starting concept"},
                        "context": {"type": "array", "items": {"type": "string"}, "description": "Prior path context"},
                        "steps": {"type": "integer", "description": "Max steps (default 5, max 50)"},
                        "mode": {"type": "string", "enum": ["strongest", "weighted"]},
                        "reinforce": {"type": "boolean", "description": "Strengthen traversed edges (default true). Set false for read-only queries."}
                    },
                    "required": ["start"]
                }),
            },
            Tool {
                name: "search".into(),
                description: "Search for nodes by name pattern".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"}
                    },
                    "required": ["pattern"]
                }),
            },
            Tool {
                name: "find_path".into(),
                description: "Find paths between two nodes".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from": {"type": "string"},
                        "to": {"type": "string"},
                        "max_depth": {"type": "integer"}
                    },
                    "required": ["from", "to"]
                }),
            },
            Tool {
                name: "get_stats".into(),
                description: "Get memory statistics".into(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            Tool {
                name: "save".into(),
                description: "Save memory to disk".into(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            // Provenance tools
            Tool {
                name: "register_source".into(),
                description: "Register an information source for provenance tracking".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "Unique source ID (e.g., 'twitter:username')"},
                        "name": {"type": "string", "description": "Human-readable name"},
                        "url": {"type": "string", "description": "Optional URL"},
                        "origin": {"type": "string", "enum": ["agent", "manual"], "description": "Who added this (default: agent)"},
                        "metadata": {"type": "object", "description": "Optional metadata key-value pairs"}
                    },
                    "required": ["id", "name"]
                }),
            },
            Tool {
                name: "list_sources".into(),
                description: "List all registered information sources".into(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            Tool {
                name: "edges_by_source".into(),
                description: "Get all edges contributed by a specific source".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "source_id": {"type": "string", "description": "Source ID to query"}
                    },
                    "required": ["source_id"]
                }),
            },
            Tool {
                name: "source_overlap".into(),
                description: "Find shared concepts between two sources".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "source_a": {"type": "string"},
                        "source_b": {"type": "string"}
                    },
                    "required": ["source_a", "source_b"]
                }),
            },
            Tool {
                name: "concept_timeline".into(),
                description: "Get timeline of when different sources mentioned a concept".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "node": {"type": "string", "description": "Concept to query"}
                    },
                    "required": ["node"]
                }),
            },
        ];

        Ok(json!({ "tools": tools }))
    }

    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value, String> {
        let params = params.ok_or("Missing params")?;
        let name = params["name"].as_str().ok_or("Missing tool name")?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        let content = match name {
            "add_path" => self.tool_add_path(args).await,
            "add_connection" => self.tool_add_connection(args).await,
            "think" => self.tool_think(args).await,
            "search" => self.tool_search(args).await,
            "find_path" => self.tool_find_path(args).await,
            "get_stats" => self.tool_get_stats().await,
            "save" => self.tool_save().await,
            // Provenance tools
            "register_source" => self.tool_register_source(args).await,
            "list_sources" => self.tool_list_sources().await,
            "edges_by_source" => self.tool_edges_by_source(args).await,
            "source_overlap" => self.tool_source_overlap(args).await,
            "concept_timeline" => self.tool_concept_timeline(args).await,
            _ => Err(format!("Unknown tool: {}", name)),
        }?;

        Ok(json!({
            "content": [{"type": "text", "text": content}]
        }))
    }

    async fn tool_add_path(&self, args: Value) -> Result<String, String> {
        let path: Vec<String> = args["path"]
            .as_array()
            .ok_or("path must be array")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        validate_path(&path)?;

        let conn_types: Option<Vec<String>> = args["conn_types"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect());

        let weight = args["weight"].as_f64().unwrap_or(0.5);
        validate_weight(weight)?;

        let bidirectional = args["bidirectional"].as_bool().unwrap_or(false);
        let source_id = args["source_id"].as_str();
        let timestamp = args["timestamp"].as_i64();

        let mut mem = self.memory.write().await;
        mem.add_path(&path, conn_types.as_deref(), weight, bidirectional, source_id, timestamp);

        Ok(format!("Added path: {} ({} nodes)", path.join(" -> "), path.len()))
    }

    async fn tool_add_connection(&self, args: Value) -> Result<String, String> {
        let from = args["from"].as_str().ok_or("from required")?;
        let to = args["to"].as_str().ok_or("to required")?;
        validate_node_name(from)?;
        validate_node_name(to)?;

        let conn_type = args["conn_type"].as_str();
        let weight = args["weight"].as_f64().unwrap_or(0.5);
        validate_weight(weight)?;

        let source_id = args["source_id"].as_str();
        let timestamp = args["timestamp"].as_i64();

        let mut mem = self.memory.write().await;
        let bidirectional = args["bidirectional"].as_bool().unwrap_or(false);
        mem.add_connection(from, to, conn_type, None, weight, bidirectional, source_id, timestamp);

        Ok(format!("Added: {} -> {}", from, to))
    }

    async fn tool_think(&self, args: Value) -> Result<String, String> {
        let start = args["start"].as_str().ok_or("start required")?;
        validate_node_name(start)?;

        let context: Option<Vec<String>> = args["context"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        let steps = (args["steps"].as_u64().unwrap_or(5) as usize).min(MAX_STEPS);
        let mode = match args["mode"].as_str() {
            Some("strongest") => ThinkMode::Strongest,
            _ => ThinkMode::Weighted,
        };
        let reinforce = args["reinforce"].as_bool().unwrap_or(true);

        let result = if reinforce {
            let mut mem = self.memory.write().await;
            mem.think_and_reinforce(start, context.as_deref(), steps, mode, None)
        } else {
            let mem = self.memory.read().await;
            mem.think(start, context.as_deref(), steps, mode, None)
        };

        let path_str: String = result
            .iter()
            .enumerate()
            .map(|(i, step)| {
                if i == 0 {
                    step.node.clone()
                } else {
                    let edge = step.edge_type.as_ref().map(|t| format!("--{}-->", t)).unwrap_or("-->".into());
                    format!("{} {}", edge, step.node)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(path_str)
    }

    async fn tool_search(&self, args: Value) -> Result<String, String> {
        let pattern = args["pattern"].as_str().ok_or("pattern required")?;
        if pattern.len() > MAX_PATTERN_LEN {
            return Err(format!("Search pattern too long (max {} chars)", MAX_PATTERN_LEN));
        }
        let mem = self.memory.read().await;
        let results = mem.search(pattern);
        Ok(serde_json::to_string_pretty(&results).unwrap_or_default())
    }

    async fn tool_find_path(&self, args: Value) -> Result<String, String> {
        let from = args["from"].as_str().ok_or("from required")?;
        let to = args["to"].as_str().ok_or("to required")?;
        validate_node_name(from)?;
        validate_node_name(to)?;

        let max_depth = (args["max_depth"].as_u64().unwrap_or(5) as usize).min(MAX_DEPTH);

        let mem = self.memory.read().await;
        let results = mem.find_path(from, to, max_depth);
        Ok(serde_json::to_string_pretty(&results).unwrap_or_default())
    }

    async fn tool_get_stats(&self) -> Result<String, String> {
        let mem = self.memory.read().await;
        let stats = mem.stats();
        Ok(serde_json::to_string_pretty(&stats).unwrap_or_default())
    }

    async fn tool_save(&self) -> Result<String, String> {
        let mem = self.memory.read().await;
        mem.save(std::path::Path::new(&self.data_path))
            .map_err(|e| e.to_string())?;
        Ok(format!("Saved to {}", self.data_path))
    }

    // ==================== Provenance Tools ====================

    async fn tool_register_source(&self, args: Value) -> Result<String, String> {
        let id = args["id"].as_str().ok_or("id required")?;
        let name = args["name"].as_str().ok_or("name required")?;
        let url = args["url"].as_str();
        let origin = match args["origin"].as_str() {
            Some("manual") => SourceOrigin::Manual,
            _ => SourceOrigin::Agent,
        };

        let mut source = Source::new(id.to_string(), name.to_string(), origin);
        if let Some(u) = url {
            source = source.with_url(u.to_string());
        }
        if let Some(metadata) = args["metadata"].as_object() {
            for (k, v) in metadata {
                if let Some(val) = v.as_str() {
                    source = source.with_metadata(k.clone(), val.to_string());
                }
            }
        }

        let mut mem = self.memory.write().await;
        mem.register_source(source);

        Ok(format!("Registered source: {}", id))
    }

    async fn tool_list_sources(&self) -> Result<String, String> {
        let mem = self.memory.read().await;
        let sources = mem.list_sources();
        if sources.is_empty() {
            return Ok("No sources registered".to_string());
        }
        let list: Vec<String> = sources
            .iter()
            .map(|s| format!("- {} ({})", s.id, s.name))
            .collect();
        Ok(format!("Registered sources:\n{}", list.join("\n")))
    }

    async fn tool_edges_by_source(&self, args: Value) -> Result<String, String> {
        let source_id = args["source_id"].as_str().ok_or("source_id required")?;
        let mem = self.memory.read().await;
        let edges = mem.get_edges_by_source(source_id);
        if edges.is_empty() {
            return Ok(format!("No edges from source: {}", source_id));
        }
        let list: Vec<String> = edges
            .iter()
            .map(|e| {
                let conn = e.conn_type.as_deref().unwrap_or("--");
                format!("{} --{}-> {}", e.from, conn, e.to)
            })
            .collect();
        Ok(format!("Edges from {}:\n{}", source_id, list.join("\n")))
    }

    async fn tool_source_overlap(&self, args: Value) -> Result<String, String> {
        let source_a = args["source_a"].as_str().ok_or("source_a required")?;
        let source_b = args["source_b"].as_str().ok_or("source_b required")?;
        let mem = self.memory.read().await;
        let overlap = mem.get_source_overlap(source_a, source_b);

        let mut result = format!("Overlap between {} and {}:\n", source_a, source_b);
        result.push_str(&format!("Shared nodes ({}): {}\n",
            overlap.shared_nodes.len(),
            if overlap.shared_nodes.is_empty() { "none".to_string() } else { overlap.shared_nodes.join(", ") }
        ));
        result.push_str(&format!("Only in {}: {}\n",
            source_a,
            if overlap.only_in_a.is_empty() { "none".to_string() } else { overlap.only_in_a.join(", ") }
        ));
        result.push_str(&format!("Only in {}: {}",
            source_b,
            if overlap.only_in_b.is_empty() { "none".to_string() } else { overlap.only_in_b.join(", ") }
        ));
        Ok(result)
    }

    async fn tool_concept_timeline(&self, args: Value) -> Result<String, String> {
        let node = args["node"].as_str().ok_or("node required")?;
        let mem = self.memory.read().await;
        let timeline = mem.get_concept_timeline(node);

        if timeline.is_empty() {
            return Ok(format!("No timeline data for: {}", node));
        }

        let list: Vec<String> = timeline
            .iter()
            .map(|e| {
                let ts = e.timestamp.map(|t| {
                    chrono::DateTime::from_timestamp(t, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| t.to_string())
                }).unwrap_or_else(|| "unknown".to_string());
                let source = e.source_name.as_deref().unwrap_or(&e.source_id);
                format!("[{}] {} mentioned in {} -> {}", ts, source, e.edge_from, e.edge_to)
            })
            .collect();
        Ok(format!("Timeline for '{}':\n{}", node, list.join("\n")))
    }
}
