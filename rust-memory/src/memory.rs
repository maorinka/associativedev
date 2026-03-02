//! Core associative memory implementation
//!
//! Sparse tensor approach where:
//! - Full path context determines associations
//! - Edges have weights and connection types
//! - Bidirectional paths supported
//! - Reinforcement learning through traversal

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use rand::seq::SliceRandom;
use rand::Rng;

/// Origin of information - was it added by an agent or manually by a human
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceOrigin {
    Agent,
    Manual,
}

/// A source of information (Twitter profile, Reddit thread, document, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub origin: SourceOrigin,
    pub created_at: i64,
    pub metadata: HashMap<String, String>,
}

impl Source {
    pub fn new(id: String, name: String, origin: SourceOrigin) -> Self {
        Self {
            id,
            name,
            url: None,
            origin,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            metadata: HashMap::new(),
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// An edge connecting to another node with weight, type, and provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub weight: f64,
    pub conn_type: Option<String>,
    pub source_id: Option<String>,
    pub timestamp: Option<i64>,
}

impl Edge {
    pub fn new(weight: f64, conn_type: Option<String>) -> Self {
        Self {
            weight,
            conn_type,
            source_id: None,
            timestamp: None,
        }
    }

    pub fn with_provenance(mut self, source_id: String) -> Self {
        self.source_id = Some(source_id);
        self.timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        );
        self
    }

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Key for looking up associations: (path_context, current_node)
/// path_context is the full path leading to current_node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextKey {
    pub context: Vec<String>,
    pub current: String,
}

impl ContextKey {
    pub fn new(context: Vec<String>, current: String) -> Self {
        Self { context, current }
    }
}

/// A step in a thought path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStep {
    pub node: String,
    pub edge_type: Option<String>,
    pub weight: Option<f64>,
}

/// Search result for node queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: String,
    pub contexts: Vec<Vec<String>>,
    pub total_weight: f64,
}

/// Path between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    pub path: Vec<String>,
    pub edge_types: Vec<Option<String>>,
    pub total_weight: f64,
}

/// An edge with full provenance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEdge {
    pub from: String,
    pub to: String,
    pub context: Vec<String>,
    pub conn_type: Option<String>,
    pub weight: f64,
    pub source_id: Option<String>,
    pub timestamp: Option<i64>,
}

/// Information about a source mentioning a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMention {
    pub source_id: String,
    pub source_name: Option<String>,
    pub edge_count: usize,
    pub first_seen: Option<i64>,
    pub last_seen: Option<i64>,
}

/// Overlap analysis between two sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceOverlap {
    pub source_a: String,
    pub source_b: String,
    pub shared_nodes: Vec<String>,
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
}

/// Timeline entry for tracking when concepts were mentioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: Option<i64>,
    pub source_id: String,
    pub source_name: Option<String>,
    pub edge_from: String,
    pub edge_to: String,
    pub conn_type: Option<String>,
}

/// Edge information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub from: String,
    pub to: String,
    pub conn_type: Option<String>,
    pub weight: f64,
    pub source_id: Option<String>,
    pub timestamp: Option<i64>,
}

/// Serializable entry for associations (ContextKey cannot be a JSON key directly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationEntry {
    pub context_key: ContextKey,
    pub edges: HashMap<String, Edge>,
}

/// The main associative memory structure
#[derive(Debug, Clone)]
pub struct AssociativeMemory {
    /// Sparse tensor: ContextKey -> {next_node: Edge}
    associations: HashMap<ContextKey, HashMap<String, Edge>>,

    /// Registered information sources
    sources: HashMap<String, Source>,

    /// Learning rate for reinforcement
    pub learning_rate: f64,

    /// Decay rate for forgetting
    pub decay_rate: f64,
}

/// Serializable wrapper for AssociativeMemory
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableMemory {
    associations: Vec<AssociationEntry>,
    #[serde(default)]
    sources: HashMap<String, Source>,
    learning_rate: f64,
    decay_rate: f64,
}

impl Serialize for AssociativeMemory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let entries: Vec<AssociationEntry> = self
            .associations
            .iter()
            .map(|(k, v)| AssociationEntry {
                context_key: k.clone(),
                edges: v.clone(),
            })
            .collect();

        let wrapper = SerializableMemory {
            associations: entries,
            sources: self.sources.clone(),
            learning_rate: self.learning_rate,
            decay_rate: self.decay_rate,
        };
        wrapper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssociativeMemory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wrapper = SerializableMemory::deserialize(deserializer)?;
        let mut associations = HashMap::new();
        for entry in wrapper.associations {
            associations.insert(entry.context_key, entry.edges);
        }
        Ok(Self {
            associations,
            sources: wrapper.sources,
            learning_rate: wrapper.learning_rate,
            decay_rate: wrapper.decay_rate,
        })
    }
}

impl Default for AssociativeMemory {
    fn default() -> Self {
        Self::new(0.1, 0.99)
    }
}

impl AssociativeMemory {
    pub fn new(learning_rate: f64, decay_rate: f64) -> Self {
        Self {
            associations: HashMap::new(),
            sources: HashMap::new(),
            learning_rate,
            decay_rate,
        }
    }

    /// Add a path of nodes with optional connection types
    ///
    /// # Arguments
    /// * `path` - List of nodes [A, B, C, D]
    /// * `conn_types` - Optional connection types for each edge
    /// * `initial_weight` - Starting weight for edges
    /// * `bidirectional` - Also create reverse path
    pub fn add_path(
        &mut self,
        path: &[String],
        conn_types: Option<&[String]>,
        initial_weight: f64,
        bidirectional: bool,
        source_id: Option<&str>,
        timestamp: Option<i64>,
    ) {
        if path.len() < 2 {
            return;
        }

        let conn_types: Vec<Option<String>> = match conn_types {
            Some(types) => {
                let mut v: Vec<Option<String>> = types.iter().map(|s| Some(s.clone())).collect();
                // Pad with None if not enough types
                while v.len() < path.len() - 1 {
                    v.push(None);
                }
                v
            }
            None => vec![None; path.len() - 1],
        };

        // Forward path
        self.add_directed_path(path, &conn_types, initial_weight, source_id, timestamp);

        // Reverse path (if bidirectional)
        if bidirectional {
            let reverse_path: Vec<String> = path.iter().rev().cloned().collect();
            let reverse_types: Vec<Option<String>> = conn_types.iter().rev().cloned().collect();
            self.add_directed_path(&reverse_path, &reverse_types, initial_weight * 0.5, source_id, timestamp);
        }
    }

    fn add_directed_path(
        &mut self,
        path: &[String],
        conn_types: &[Option<String>],
        weight: f64,
        source_id: Option<&str>,
        timestamp: Option<i64>,
    ) {
        let ts = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });

        for i in 0..path.len() - 1 {
            let context: Vec<String> = path[..i].to_vec();
            let current = path[i].clone();
            let next_node = path[i + 1].clone();
            let conn_type = conn_types.get(i).cloned().flatten();

            let key = ContextKey::new(context, current);

            let edges = self.associations.entry(key).or_insert_with(HashMap::new);

            if let Some(edge) = edges.get_mut(&next_node) {
                edge.weight += weight;
                if conn_type.is_some() {
                    edge.conn_type = conn_type;
                }
                // Update source if provided and edge didn't have one
                if source_id.is_some() && edge.source_id.is_none() {
                    edge.source_id = source_id.map(|s| s.to_string());
                    edge.timestamp = Some(ts);
                }
            } else {
                let mut edge = Edge::new(weight, conn_type);
                if let Some(sid) = source_id {
                    edge = edge.with_provenance(sid.to_string()).with_timestamp(ts);
                }
                edges.insert(next_node, edge);
            }
        }
    }

    /// Add a single connection between two nodes
    pub fn add_connection(
        &mut self,
        from: &str,
        to: &str,
        conn_type: Option<&str>,
        context: Option<&[String]>,
        weight: f64,
        bidirectional: bool,
        source_id: Option<&str>,
        timestamp: Option<i64>,
    ) {
        let ts = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });

        let ctx = context.map(|c| c.to_vec()).unwrap_or_default();
        let key = ContextKey::new(ctx.clone(), from.to_string());

        let edges = self.associations.entry(key).or_insert_with(HashMap::new);

        if let Some(edge) = edges.get_mut(to) {
            edge.weight += weight;
            if conn_type.is_some() {
                edge.conn_type = conn_type.map(|s| s.to_string());
            }
            if source_id.is_some() && edge.source_id.is_none() {
                edge.source_id = source_id.map(|s| s.to_string());
                edge.timestamp = Some(ts);
            }
        } else {
            let mut edge = Edge::new(weight, conn_type.map(|s| s.to_string()));
            if let Some(sid) = source_id {
                edge = edge.with_provenance(sid.to_string()).with_timestamp(ts);
            }
            edges.insert(to.to_string(), edge);
        }

        if bidirectional {
            let reverse_key = ContextKey::new(ctx, to.to_string());
            let reverse_edges = self.associations.entry(reverse_key).or_insert_with(HashMap::new);

            if let Some(edge) = reverse_edges.get_mut(from) {
                edge.weight += weight * 0.5;
            } else {
                let mut edge = Edge::new(weight * 0.5, conn_type.map(|s| s.to_string()));
                if let Some(sid) = source_id {
                    edge = edge.with_provenance(sid.to_string()).with_timestamp(ts);
                }
                reverse_edges.insert(from.to_string(), edge);
            }
        }
    }

    /// Reinforce a path by increasing edge weights
    pub fn traverse(&mut self, path: &[String]) {
        if path.len() < 2 {
            return;
        }

        for i in 0..path.len() - 1 {
            let context: Vec<String> = path[..i].to_vec();
            let current = path[i].clone();
            let next_node = &path[i + 1];

            let key = ContextKey::new(context, current);

            if let Some(edges) = self.associations.get_mut(&key) {
                if let Some(edge) = edges.get_mut(next_node) {
                    edge.weight += self.learning_rate;
                }
            }
        }
    }

    /// Get associations for a node given path context
    /// Falls back to shorter contexts if exact path not found
    pub fn get_associations(
        &self,
        current: &str,
        path_context: Option<&[String]>,
        conn_type_filter: Option<&str>,
    ) -> HashMap<String, Edge> {
        let path_context = path_context.unwrap_or(&[]);

        // Try exact path first
        let key = ContextKey::new(path_context.to_vec(), current.to_string());
        if let Some(edges) = self.associations.get(&key) {
            let result = filter_by_type(edges, conn_type_filter);
            if !result.is_empty() {
                return result;
            }
        }

        // Fallback: try progressively shorter contexts
        for i in 0..path_context.len() {
            let shorter: Vec<String> = path_context[i..].to_vec();
            let key = ContextKey::new(shorter, current.to_string());
            if let Some(edges) = self.associations.get(&key) {
                let result = filter_by_type(edges, conn_type_filter);
                if !result.is_empty() {
                    return result;
                }
            }
        }

        // Final fallback: no context
        let key = ContextKey::new(vec![], current.to_string());
        if let Some(edges) = self.associations.get(&key) {
            return filter_by_type(edges, conn_type_filter);
        }

        HashMap::new()
    }

    /// Follow associations from a starting concept (read-only, no reinforcement)
    ///
    /// Use this for queries that shouldn't modify edge weights.
    /// For reinforcement, call `traverse` separately on the returned path.
    pub fn think(
        &self,
        start: &str,
        context: Option<&[String]>,
        steps: usize,
        mode: ThinkMode,
        conn_type_filter: Option<&str>,
    ) -> Vec<ThoughtStep> {
        let ctx = context.map(|c| c.to_vec()).unwrap_or_default();
        let mut path_nodes: Vec<String> = ctx.clone();
        path_nodes.push(start.to_string());

        let mut current = start.to_string();
        let mut result = vec![ThoughtStep {
            node: start.to_string(),
            edge_type: None,
            weight: None,
        }];

        let mut rng = rand::thread_rng();

        for _ in 0..steps {
            let path_context: Vec<String> = path_nodes[..path_nodes.len() - 1].to_vec();
            let assocs = self.get_associations(&current, Some(&path_context), conn_type_filter);

            if assocs.is_empty() {
                break;
            }

            let next_node = match mode {
                ThinkMode::Strongest => {
                    assocs
                        .iter()
                        .max_by(|a, b| a.1.weight.partial_cmp(&b.1.weight).unwrap())
                        .map(|(k, _)| k.clone())
                }
                ThinkMode::Weighted => {
                    let nodes: Vec<&String> = assocs.keys().collect();
                    let weights: Vec<f64> = nodes.iter().map(|n| assocs[*n].weight.max(0.0)).collect();
                    let total: f64 = weights.iter().sum();

                    if total <= 0.0 {
                        None
                    } else {
                        let mut r = rng.gen::<f64>() * total;
                        let mut selected = None;
                        for (i, w) in weights.iter().enumerate() {
                            r -= w;
                            if r <= 0.0 {
                                selected = Some(nodes[i].clone());
                                break;
                            }
                        }
                        selected.or_else(|| nodes.last().map(|n| (*n).clone()))
                    }
                }
            };

            if let Some(next) = next_node {
                let edge = &assocs[&next];
                result.push(ThoughtStep {
                    node: next.clone(),
                    edge_type: edge.conn_type.clone(),
                    weight: Some(edge.weight),
                });
                path_nodes.push(next.clone());
                current = next;
            } else {
                break;
            }
        }

        result
    }

    /// Follow associations and reinforce the traversed path (mutating)
    ///
    /// Same as `think` but also strengthens the edges along the path taken.
    pub fn think_and_reinforce(
        &mut self,
        start: &str,
        context: Option<&[String]>,
        steps: usize,
        mode: ThinkMode,
        conn_type_filter: Option<&str>,
    ) -> Vec<ThoughtStep> {
        let result = self.think(start, context, steps, mode, conn_type_filter);

        // Reinforce the path we took
        let path_nodes: Vec<String> = result.iter().map(|s| s.node.clone()).collect();
        if path_nodes.len() > 1 {
            self.traverse(&path_nodes);
        }

        result
    }

    /// Search for nodes by name pattern
    pub fn search(&self, pattern: &str) -> Vec<SearchResult> {
        let pattern_lower = pattern.to_lowercase();
        let mut results: HashMap<String, SearchResult> = HashMap::new();

        for (key, edges) in &self.associations {
            // Check current node
            if key.current.to_lowercase().contains(&pattern_lower) {
                let entry = results.entry(key.current.clone()).or_insert(SearchResult {
                    node: key.current.clone(),
                    contexts: vec![],
                    total_weight: 0.0,
                });
                entry.contexts.push(key.context.clone());
                entry.total_weight += edges.values().map(|e| e.weight).sum::<f64>();
            }

            // Check next nodes
            for (next, edge) in edges {
                if next.to_lowercase().contains(&pattern_lower) {
                    let entry = results.entry(next.clone()).or_insert(SearchResult {
                        node: next.clone(),
                        contexts: vec![],
                        total_weight: 0.0,
                    });
                    let mut ctx = key.context.clone();
                    ctx.push(key.current.clone());
                    entry.contexts.push(ctx);
                    entry.total_weight += edge.weight;
                }
            }
        }

        let mut results: Vec<SearchResult> = results.into_values().collect();
        results.sort_by(|a, b| b.total_weight.partial_cmp(&a.total_weight).unwrap());
        results
    }

    /// Find paths between two nodes using BFS
    pub fn find_path(&self, from: &str, to: &str, max_depth: usize) -> Vec<PathResult> {
        let mut results = vec![];
        let mut queue: Vec<(Vec<String>, Vec<Option<String>>, f64)> = vec![(
            vec![from.to_string()],
            vec![],
            0.0,
        )];
        let mut visited: HashMap<String, usize> = HashMap::new();

        while let Some((path, edge_types, total_weight)) = queue.pop() {
            let current = path.last().unwrap();

            if current == to {
                results.push(PathResult {
                    path: path.clone(),
                    edge_types: edge_types.clone(),
                    total_weight,
                });
                continue;
            }

            if path.len() > max_depth {
                continue;
            }

            // Check if we've visited this node with a shorter path
            if let Some(&prev_len) = visited.get(current) {
                if prev_len <= path.len() {
                    continue;
                }
            }
            visited.insert(current.clone(), path.len());

            let context: Vec<String> = path[..path.len() - 1].to_vec();
            let assocs = self.get_associations(current, Some(&context), None);

            for (next, edge) in assocs {
                if !path.contains(&next) {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    let mut new_types = edge_types.clone();
                    new_types.push(edge.conn_type);
                    queue.push((new_path, new_types, total_weight + edge.weight));
                }
            }
        }

        results.sort_by(|a, b| b.total_weight.partial_cmp(&a.total_weight).unwrap());
        results
    }

    /// Apply decay to all associations
    pub fn decay(&mut self) {
        let mut keys_to_remove = vec![];

        for (key, edges) in self.associations.iter_mut() {
            let mut nodes_to_remove = vec![];

            for (node, edge) in edges.iter_mut() {
                edge.weight *= self.decay_rate;
                if edge.weight < 0.01 {
                    nodes_to_remove.push(node.clone());
                }
            }

            for node in nodes_to_remove {
                edges.remove(&node);
            }

            if edges.is_empty() {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            self.associations.remove(&key);
        }
    }

    /// Remove associations below threshold
    pub fn prune(&mut self, threshold: f64) {
        let mut keys_to_remove = vec![];

        for (key, edges) in self.associations.iter_mut() {
            let nodes_to_remove: Vec<String> = edges
                .iter()
                .filter(|(_, e)| e.weight < threshold)
                .map(|(n, _)| n.clone())
                .collect();

            for node in nodes_to_remove {
                edges.remove(&node);
            }

            if edges.is_empty() {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            self.associations.remove(&key);
        }
    }

    /// Save to JSON file (atomic: writes to temp file then renames)
    pub fn save(&self, filepath: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let tmp = filepath.with_extension("json.tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, filepath)?;
        Ok(())
    }

    /// Load from JSON file
    pub fn load(filepath: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(filepath)?;
        let memory: Self = serde_json::from_str(&json)?;
        Ok(memory)
    }

    // ==================== Snapshot Operations ====================

    /// Convert memory to a GraphSnapshot for export/migration
    pub fn to_snapshot(&self) -> crate::storage::GraphSnapshot {
        let associations: Vec<AssociationEntry> = self
            .associations
            .iter()
            .map(|(k, v)| AssociationEntry {
                context_key: k.clone(),
                edges: v.clone(),
            })
            .collect();

        crate::storage::GraphSnapshot {
            version: crate::storage::SCHEMA_VERSION,
            associations,
            sources: self.sources.clone(),
            learning_rate: self.learning_rate,
            decay_rate: self.decay_rate,
            metadata: HashMap::new(),
        }
    }

    /// Create memory from a GraphSnapshot
    pub fn from_snapshot(snapshot: crate::storage::GraphSnapshot) -> Self {
        let mut associations = HashMap::new();
        for entry in snapshot.associations {
            associations.insert(entry.context_key, entry.edges);
        }
        Self {
            associations,
            sources: snapshot.sources,
            learning_rate: snapshot.learning_rate,
            decay_rate: snapshot.decay_rate,
        }
    }

    /// Merge another snapshot into this memory
    pub fn merge_snapshot(&mut self, snapshot: crate::storage::GraphSnapshot) -> MergeResult {
        let mut result = MergeResult {
            nodes_added: 0,
            edges_added: 0,
            edges_updated: 0,
            sources_added: 0,
        };

        // Merge associations
        for entry in snapshot.associations {
            if let Some(existing) = self.associations.get_mut(&entry.context_key) {
                for (node, edge) in entry.edges {
                    if let Some(existing_edge) = existing.get_mut(&node) {
                        existing_edge.weight += edge.weight;
                        result.edges_updated += 1;
                    } else {
                        existing.insert(node, edge);
                        result.edges_added += 1;
                    }
                }
            } else {
                result.edges_added += entry.edges.len();
                self.associations.insert(entry.context_key, entry.edges);
            }
        }

        // Merge sources
        for (id, source) in snapshot.sources {
            if !self.sources.contains_key(&id) {
                self.sources.insert(id, source);
                result.sources_added += 1;
            }
        }

        result
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let total_edges: usize = self.associations.values().map(|e| e.len()).sum();
        let max_context_len = self
            .associations
            .keys()
            .map(|k| k.context.len())
            .max()
            .unwrap_or(0);

        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for edges in self.associations.values() {
            for edge in edges.values() {
                let key = edge.conn_type.clone().unwrap_or_else(|| "untyped".to_string());
                *type_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Get all unique nodes
        let mut all_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (key, edges) in &self.associations {
            all_nodes.insert(key.current.clone());
            for node in edges.keys() {
                all_nodes.insert(node.clone());
            }
        }

        MemoryStats {
            unique_contexts: self.associations.len(),
            total_edges,
            max_context_depth: max_context_len,
            connection_types: type_counts,
            unique_nodes: all_nodes.len(),
        }
    }

    /// Get all nodes in the memory
    pub fn get_all_nodes(&self) -> Vec<String> {
        let mut nodes: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (key, edges) in &self.associations {
            nodes.insert(key.current.clone());
            for node in edges.keys() {
                nodes.insert(node.clone());
            }
        }
        let mut result: Vec<String> = nodes.into_iter().collect();
        result.sort();
        result
    }

    /// Get all edges for visualization
    pub fn get_all_edges(&self) -> Vec<EdgeInfo> {
        let mut edges = vec![];
        for (key, edge_map) in &self.associations {
            for (to, edge) in edge_map {
                edges.push(EdgeInfo {
                    from: key.current.clone(),
                    to: to.clone(),
                    conn_type: edge.conn_type.clone(),
                    weight: edge.weight,
                    source_id: edge.source_id.clone(),
                    timestamp: edge.timestamp,
                });
            }
        }
        edges
    }

    // ==================== Source Management ====================

    /// Register a new information source
    pub fn register_source(&mut self, source: Source) {
        self.sources.insert(source.id.clone(), source);
    }

    /// Get a source by ID
    pub fn get_source(&self, source_id: &str) -> Option<&Source> {
        self.sources.get(source_id)
    }

    /// List all registered sources
    pub fn list_sources(&self) -> Vec<&Source> {
        self.sources.values().collect()
    }

    /// Remove a source (does not remove edges that reference it)
    pub fn remove_source(&mut self, source_id: &str) -> Option<Source> {
        self.sources.remove(source_id)
    }

    // ==================== Provenance Queries ====================

    /// Get all edges from a specific source
    pub fn get_edges_by_source(&self, source_id: &str) -> Vec<ProvenanceEdge> {
        let mut result = vec![];
        for (key, edge_map) in &self.associations {
            for (to, edge) in edge_map {
                if edge.source_id.as_deref() == Some(source_id) {
                    result.push(ProvenanceEdge {
                        from: key.current.clone(),
                        to: to.clone(),
                        context: key.context.clone(),
                        conn_type: edge.conn_type.clone(),
                        weight: edge.weight,
                        source_id: edge.source_id.clone(),
                        timestamp: edge.timestamp,
                    });
                }
            }
        }
        result
    }

    /// Get all sources that mention a specific node
    pub fn get_sources_for_node(&self, node: &str) -> Vec<SourceMention> {
        let mut mentions: HashMap<String, SourceMention> = HashMap::new();

        for (key, edge_map) in &self.associations {
            // Check if node is the current node or a target
            let is_current = key.current == node;
            let is_target = edge_map.contains_key(node);

            if is_current || is_target {
                for (to, edge) in edge_map {
                    if let Some(ref sid) = edge.source_id {
                        let mention = mentions.entry(sid.clone()).or_insert_with(|| SourceMention {
                            source_id: sid.clone(),
                            source_name: self.sources.get(sid).map(|s| s.name.clone()),
                            edge_count: 0,
                            first_seen: edge.timestamp,
                            last_seen: edge.timestamp,
                        });
                        mention.edge_count += 1;
                        if let Some(ts) = edge.timestamp {
                            if mention.first_seen.is_none() || mention.first_seen > Some(ts) {
                                mention.first_seen = Some(ts);
                            }
                            if mention.last_seen.is_none() || mention.last_seen < Some(ts) {
                                mention.last_seen = Some(ts);
                            }
                        }
                    }
                }
            }
        }

        mentions.into_values().collect()
    }

    /// Find concepts shared between two sources
    pub fn get_source_overlap(&self, source_a: &str, source_b: &str) -> SourceOverlap {
        let mut nodes_a: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut nodes_b: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (key, edge_map) in &self.associations {
            for (to, edge) in edge_map {
                if edge.source_id.as_deref() == Some(source_a) {
                    nodes_a.insert(key.current.clone());
                    nodes_a.insert(to.clone());
                }
                if edge.source_id.as_deref() == Some(source_b) {
                    nodes_b.insert(key.current.clone());
                    nodes_b.insert(to.clone());
                }
            }
        }

        let shared: Vec<String> = nodes_a.intersection(&nodes_b).cloned().collect();
        let only_a: Vec<String> = nodes_a.difference(&nodes_b).cloned().collect();
        let only_b: Vec<String> = nodes_b.difference(&nodes_a).cloned().collect();

        SourceOverlap {
            source_a: source_a.to_string(),
            source_b: source_b.to_string(),
            shared_nodes: shared,
            only_in_a: only_a,
            only_in_b: only_b,
        }
    }

    /// Get timeline of when different sources mentioned a concept
    pub fn get_concept_timeline(&self, node: &str) -> Vec<TimelineEntry> {
        let mut entries: Vec<TimelineEntry> = vec![];

        for (key, edge_map) in &self.associations {
            let is_current = key.current == node;

            for (to, edge) in edge_map {
                let is_target = to == node;

                if (is_current || is_target) && edge.source_id.is_some() {
                    entries.push(TimelineEntry {
                        timestamp: edge.timestamp,
                        source_id: edge.source_id.clone().unwrap(),
                        source_name: edge.source_id.as_ref()
                            .and_then(|sid| self.sources.get(sid))
                            .map(|s| s.name.clone()),
                        edge_from: key.current.clone(),
                        edge_to: to.clone(),
                        conn_type: edge.conn_type.clone(),
                    });
                }
            }
        }

        // Sort by timestamp
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        entries
    }
}

/// Mode for thinking/exploration
#[derive(Debug, Clone, Copy)]
pub enum ThinkMode {
    /// Always follow the strongest association
    Strongest,
    /// Probabilistic selection weighted by edge weights
    Weighted,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub unique_contexts: usize,
    pub total_edges: usize,
    pub max_context_depth: usize,
    pub connection_types: HashMap<String, usize>,
    pub unique_nodes: usize,
}

/// Result of a merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub nodes_added: usize,
    pub edges_added: usize,
    pub edges_updated: usize,
    pub sources_added: usize,
}

/// Node importance scores based on graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeImportance {
    pub node: String,
    pub sink_score: f64,      // Importance from incoming edges (authority/fact)
    pub source_score: f64,    // Importance from outgoing edges (analyst/hub)
    pub bridge_score: f64,    // Bidirectional importance (connector)
    pub in_degree: usize,     // Number of incoming edges
    pub out_degree: usize,    // Number of outgoing edges
    pub role: String,         // "sink", "source", or "bridge"
}

impl AssociativeMemory {
    /// Compute importance scores for all nodes using weighted PageRank variants
    pub fn compute_importance(&self, iterations: usize, damping: f64) -> Vec<NodeImportance> {
        let nodes = self.get_all_nodes();
        let n = nodes.len();
        if n == 0 {
            return vec![];
        }

        // Build adjacency lists with weights
        let mut incoming: HashMap<&str, Vec<(&str, f64)>> = HashMap::new();
        let mut outgoing: HashMap<&str, Vec<(&str, f64)>> = HashMap::new();

        for node in &nodes {
            incoming.insert(node, vec![]);
            outgoing.insert(node, vec![]);
        }

        // Populate from edges (ignore context, just use current -> target)
        for (key, edges) in &self.associations {
            let from = key.current.as_str();
            for (to, edge) in edges {
                if let Some(out_list) = outgoing.get_mut(from) {
                    out_list.push((to.as_str(), edge.weight));
                }
                if let Some(in_list) = incoming.get_mut(to.as_str()) {
                    in_list.push((from, edge.weight));
                }
            }
        }

        // Initialize scores
        let initial = 1.0 / n as f64;
        let mut sink_scores: HashMap<&str, f64> = nodes.iter().map(|n| (n.as_str(), initial)).collect();
        let mut source_scores: HashMap<&str, f64> = nodes.iter().map(|n| (n.as_str(), initial)).collect();
        let mut bridge_scores: HashMap<&str, f64> = nodes.iter().map(|n| (n.as_str(), initial)).collect();

        // Iterate PageRank
        for _ in 0..iterations {
            let mut new_sink: HashMap<&str, f64> = HashMap::new();
            let mut new_source: HashMap<&str, f64> = HashMap::new();
            let mut new_bridge: HashMap<&str, f64> = HashMap::new();

            for node in &nodes {
                let node_str = node.as_str();

                // Sink score: importance flows from nodes pointing TO this node
                let mut sink_sum = 0.0;
                if let Some(in_edges) = incoming.get(node_str) {
                    for (from, weight) in in_edges {
                        let from_out_weight: f64 = outgoing.get(from)
                            .map(|e| e.iter().map(|(_, w)| w).sum())
                            .unwrap_or(1.0)
                            .max(0.001);
                        sink_sum += sink_scores.get(from).unwrap_or(&initial) * weight / from_out_weight;
                    }
                }
                new_sink.insert(node_str, (1.0 - damping) / n as f64 + damping * sink_sum);

                // Source score: importance flows from nodes this node points TO
                let mut source_sum = 0.0;
                if let Some(out_edges) = outgoing.get(node_str) {
                    for (to, weight) in out_edges {
                        let to_in_weight: f64 = incoming.get(to)
                            .map(|e| e.iter().map(|(_, w)| w).sum())
                            .unwrap_or(1.0)
                            .max(0.001);
                        source_sum += source_scores.get(to).unwrap_or(&initial) * weight / to_in_weight;
                    }
                }
                new_source.insert(node_str, (1.0 - damping) / n as f64 + damping * source_sum);

                // Bridge score: bidirectional (average both directions)
                new_bridge.insert(node_str, (new_sink[node_str] + new_source[node_str]) / 2.0);
            }

            sink_scores = new_sink;
            source_scores = new_source;
            bridge_scores = new_bridge;
        }

        // Build results with role classification
        let mut results: Vec<NodeImportance> = nodes
            .iter()
            .map(|node| {
                let node_str = node.as_str();
                let in_deg = incoming.get(node_str).map(|v| v.len()).unwrap_or(0);
                let out_deg = outgoing.get(node_str).map(|v| v.len()).unwrap_or(0);
                let sink = *sink_scores.get(node_str).unwrap_or(&0.0);
                let source = *source_scores.get(node_str).unwrap_or(&0.0);
                let bridge = *bridge_scores.get(node_str).unwrap_or(&0.0);

                // Classify role based on degree ratio
                let role = if in_deg > 0 && out_deg == 0 {
                    "sink".to_string()
                } else if out_deg > 0 && in_deg == 0 {
                    "source".to_string()
                } else if in_deg > out_deg * 2 {
                    "sink".to_string()
                } else if out_deg > in_deg * 2 {
                    "source".to_string()
                } else {
                    "bridge".to_string()
                };

                NodeImportance {
                    node: node.clone(),
                    sink_score: sink,
                    source_score: source,
                    bridge_score: bridge,
                    in_degree: in_deg,
                    out_degree: out_deg,
                    role,
                }
            })
            .collect();

        // Sort by bridge_score descending
        results.sort_by(|a, b| b.bridge_score.partial_cmp(&a.bridge_score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

fn filter_by_type(
    edges: &HashMap<String, Edge>,
    conn_type_filter: Option<&str>,
) -> HashMap<String, Edge> {
    match conn_type_filter {
        Some(filter) => edges
            .iter()
            .filter(|(_, e)| e.conn_type.as_deref() == Some(filter))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        None => edges.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_path() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["capital".into(), "Jerusalem".into(), "old_city".into()],
            Some(&["example".into(), "has".into()]),
            0.5,
            false,
            None,
            None,
        );

        let assocs = mem.get_associations("capital", None, None);
        assert!(assocs.contains_key("Jerusalem"));
        assert_eq!(assocs["Jerusalem"].conn_type, Some("example".into()));
    }

    #[test]
    fn test_context_matters() {
        let mut mem = AssociativeMemory::default();

        // food -> Jerusalem -> market
        mem.add_path(
            &["food".into(), "Jerusalem".into(), "market".into()],
            Some(&["in".into(), "has".into()]),
            0.5,
            false,
            None,
            None,
        );

        // religion -> Jerusalem -> Western_Wall
        mem.add_path(
            &["religion".into(), "Jerusalem".into(), "Western_Wall".into()],
            Some(&["in".into(), "has".into()]),
            0.5,
            false,
            None,
            None,
        );

        // With food context, should get market
        let assocs = mem.get_associations("Jerusalem", Some(&["food".into()]), None);
        assert!(assocs.contains_key("market"));
        assert!(!assocs.contains_key("Western_Wall"));

        // With religion context, should get Western_Wall
        let assocs = mem.get_associations("Jerusalem", Some(&["religion".into()]), None);
        assert!(assocs.contains_key("Western_Wall"));
        assert!(!assocs.contains_key("market"));
    }

    #[test]
    fn test_search() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["Jerusalem".into(), "old_city".into()],
            None,
            0.5,
            false,
            None,
            None,
        );

        let results = mem.search("jeru");
        assert!(!results.is_empty());
        assert_eq!(results[0].node, "Jerusalem");
    }

    #[test]
    fn test_think_readonly_does_not_mutate() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["A".into(), "B".into(), "C".into()],
            None,
            1.0,
            false,
            None,
            None,
        );

        // Get initial weight
        let initial_weight = mem.get_associations("A", None, None)["B"].weight;

        // Read-only think should NOT change weights
        let result = mem.think("A", None, 2, ThinkMode::Strongest, None);
        assert!(result.len() >= 2); // at least A -> B

        let after_weight = mem.get_associations("A", None, None)["B"].weight;
        assert_eq!(initial_weight, after_weight, "think() should not mutate weights");
    }

    #[test]
    fn test_think_and_reinforce_mutates() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["A".into(), "B".into(), "C".into()],
            None,
            1.0,
            false,
            None,
            None,
        );

        let initial_weight = mem.get_associations("A", None, None)["B"].weight;

        // think_and_reinforce SHOULD increase weights
        let result = mem.think_and_reinforce("A", None, 2, ThinkMode::Strongest, None);
        assert!(result.len() >= 2);

        let after_weight = mem.get_associations("A", None, None)["B"].weight;
        assert!(after_weight > initial_weight, "think_and_reinforce() should increase weights");
    }

    #[test]
    fn test_atomic_save_load() {
        let tmp_dir = std::env::temp_dir();
        let filepath = tmp_dir.join("test_atomic_save.json");

        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["test".into(), "save".into()],
            None,
            0.5,
            false,
            None,
            None,
        );

        // Save should work
        mem.save(&filepath).unwrap();
        assert!(filepath.exists());

        // Temp file should be cleaned up
        let tmp_file = filepath.with_extension("json.tmp");
        assert!(!tmp_file.exists(), "Temp file should be removed after rename");

        // Load should restore data
        let loaded = AssociativeMemory::load(&filepath).unwrap();
        let assocs = loaded.get_associations("test", None, None);
        assert!(assocs.contains_key("save"));

        // Cleanup
        let _ = std::fs::remove_file(&filepath);
    }

    #[test]
    fn test_bidirectional_path() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["A".into(), "B".into()],
            None,
            1.0,
            true, // bidirectional
            None,
            None,
        );

        // Forward: A -> B with weight 1.0
        let forward = mem.get_associations("A", None, None);
        assert!(forward.contains_key("B"));
        assert_eq!(forward["B"].weight, 1.0);

        // Reverse: B -> A with weight 0.5 (bidirectional = half weight)
        let reverse = mem.get_associations("B", None, None);
        assert!(reverse.contains_key("A"));
        assert_eq!(reverse["A"].weight, 0.5);
    }

    #[test]
    fn test_decay_removes_weak_edges() {
        let mut mem = AssociativeMemory::new(0.1, 0.5); // aggressive decay
        mem.add_path(
            &["A".into(), "B".into()],
            None,
            0.019, // after decay: 0.019 * 0.5 = 0.0095, which is < 0.01
            false,
            None,
            None,
        );

        assert!(!mem.get_associations("A", None, None).is_empty());

        mem.decay();
        assert!(mem.get_associations("A", None, None).is_empty(),
            "Weak edges should be pruned after decay");
    }

    #[test]
    fn test_find_path_basic() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["A".into(), "B".into(), "C".into(), "D".into()],
            None,
            1.0,
            false,
            None,
            None,
        );

        let paths = mem.find_path("A", "D", 5);
        assert!(!paths.is_empty());
        assert_eq!(paths[0].path, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_provenance_tracking() {
        let mut mem = AssociativeMemory::default();
        mem.register_source(Source::new("src1".into(), "Test Source".into(), SourceOrigin::Manual));

        mem.add_path(
            &["A".into(), "B".into()],
            None,
            1.0,
            false,
            Some("src1"),
            None,
        );

        let edges = mem.get_edges_by_source("src1");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "A");
        assert_eq!(edges[0].to, "B");
    }

    #[test]
    fn test_stats() {
        let mut mem = AssociativeMemory::default();
        mem.add_path(
            &["A".into(), "B".into(), "C".into()],
            Some(&["has".into(), "is".into()]),
            1.0,
            false,
            None,
            None,
        );

        let stats = mem.stats();
        assert_eq!(stats.unique_nodes, 3); // A, B, C
        assert_eq!(stats.total_edges, 2);  // A->B, B->C
        assert!(stats.connection_types.contains_key("has"));
        assert!(stats.connection_types.contains_key("is"));
    }
}
