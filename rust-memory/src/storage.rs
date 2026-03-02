//! Storage abstraction layer for the associative memory
//!
//! Provides a trait-based interface for different storage backends,
//! enabling easy migration between JSON, SQLite, PostgreSQL, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Re-export types that backends need
pub use crate::memory::{
    AssociationEntry, ContextKey, Edge, EdgeInfo, MemoryStats, NodeImportance,
    PathResult, ProvenanceEdge, SearchResult, Source, SourceMention, SourceOrigin,
    SourceOverlap, ThoughtStep, TimelineEntry,
};

/// Schema version for migrations
pub const SCHEMA_VERSION: u32 = 1;

/// Universal exchange format for graph data
///
/// This is the canonical format for import/export operations.
/// All storage backends must be able to produce and consume this format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    /// Schema version for forward compatibility
    pub version: u32,

    /// All associations (context -> edges)
    pub associations: Vec<AssociationEntry>,

    /// Registered information sources
    pub sources: HashMap<String, Source>,

    /// Learning rate setting
    pub learning_rate: f64,

    /// Decay rate setting
    pub decay_rate: f64,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl GraphSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            version: SCHEMA_VERSION,
            associations: Vec::new(),
            sources: HashMap::new(),
            learning_rate: 0.1,
            decay_rate: 0.99,
            metadata: HashMap::new(),
        }
    }

    /// Create snapshot with settings
    pub fn with_settings(learning_rate: f64, decay_rate: f64) -> Self {
        Self {
            version: SCHEMA_VERSION,
            associations: Vec::new(),
            sources: HashMap::new(),
            learning_rate,
            decay_rate,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Migrate from older versions
    pub fn migrate(mut self) -> Self {
        // Future migrations go here
        // if self.version < 2 {
        //     self = migrate_v1_to_v2(self);
        // }
        self.version = SCHEMA_VERSION;
        self
    }

    /// Get statistics about this snapshot
    pub fn stats(&self) -> SnapshotStats {
        let total_edges: usize = self.associations.iter().map(|a| a.edges.len()).sum();

        let mut all_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();
        for assoc in &self.associations {
            all_nodes.insert(assoc.context_key.current.clone());
            for node in assoc.edges.keys() {
                all_nodes.insert(node.clone());
            }
        }

        SnapshotStats {
            version: self.version,
            num_nodes: all_nodes.len(),
            num_edges: total_edges,
            num_sources: self.sources.len(),
            num_contexts: self.associations.len(),
        }
    }
}

impl Default for GraphSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStats {
    pub version: u32,
    pub num_nodes: usize,
    pub num_edges: usize,
    pub num_sources: usize,
    pub num_contexts: usize,
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Errors that can occur during storage operations
#[derive(Debug)]
pub enum StorageError {
    /// IO error (file not found, permission denied, etc)
    Io(std::io::Error),
    /// Serialization/deserialization error
    Serialization(String),
    /// Schema version mismatch
    VersionMismatch { expected: u32, found: u32 },
    /// Generic error
    Other(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Serialization(e) => write!(f, "Serialization error: {}", e),
            StorageError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            StorageError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Serialization(e.to_string())
    }
}

/// Storage backend trait
///
/// All storage implementations must implement this trait.
/// The trait provides a clean interface for:
/// - Exporting to/importing from GraphSnapshot
/// - Basic CRUD operations
/// - Persistence operations
pub trait StorageBackend: Send + Sync {
    /// Export all data to a GraphSnapshot
    fn export(&self) -> StorageResult<GraphSnapshot>;

    /// Import data from a GraphSnapshot (replaces existing data)
    fn import(&mut self, snapshot: GraphSnapshot) -> StorageResult<()>;

    /// Merge data from a GraphSnapshot (adds to existing data)
    fn merge(&mut self, snapshot: GraphSnapshot) -> StorageResult<MergeStats>;

    /// Save to persistent storage
    fn save(&self) -> StorageResult<()>;

    /// Load from persistent storage
    fn load(&mut self) -> StorageResult<()>;

    /// Get the storage path/location
    fn location(&self) -> &str;

    /// Optimize the storage (compact, vacuum, etc)
    fn optimize(&mut self) -> StorageResult<()>;

    /// Get backend name (e.g., "json", "sqlite", "postgres")
    fn backend_name(&self) -> &'static str;
}

/// Statistics from a merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStats {
    pub nodes_added: usize,
    pub edges_added: usize,
    pub edges_updated: usize,
    pub sources_added: usize,
}

/// JSON file storage backend
pub struct JsonStorage {
    path: String,
    snapshot: GraphSnapshot,
}

impl JsonStorage {
    /// Create a new JSON storage at the given path
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            snapshot: GraphSnapshot::new(),
        }
    }

    /// Create with initial settings
    pub fn with_settings(path: impl Into<String>, learning_rate: f64, decay_rate: f64) -> Self {
        Self {
            path: path.into(),
            snapshot: GraphSnapshot::with_settings(learning_rate, decay_rate),
        }
    }

    /// Get mutable reference to snapshot for internal use
    pub fn snapshot_mut(&mut self) -> &mut GraphSnapshot {
        &mut self.snapshot
    }

    /// Get reference to snapshot
    pub fn snapshot(&self) -> &GraphSnapshot {
        &self.snapshot
    }
}

impl StorageBackend for JsonStorage {
    fn export(&self) -> StorageResult<GraphSnapshot> {
        Ok(self.snapshot.clone())
    }

    fn import(&mut self, snapshot: GraphSnapshot) -> StorageResult<()> {
        self.snapshot = snapshot.migrate();
        Ok(())
    }

    fn merge(&mut self, snapshot: GraphSnapshot) -> StorageResult<MergeStats> {
        let snapshot = snapshot.migrate();
        let mut stats = MergeStats {
            nodes_added: 0,
            edges_added: 0,
            edges_updated: 0,
            sources_added: 0,
        };

        // Build a map of existing associations for faster lookup
        let mut existing: HashMap<ContextKey, HashMap<String, Edge>> = HashMap::new();
        for assoc in &self.snapshot.associations {
            existing.insert(assoc.context_key.clone(), assoc.edges.clone());
        }

        // Merge associations
        for assoc in snapshot.associations {
            if let Some(edges) = existing.get_mut(&assoc.context_key) {
                for (node, edge) in assoc.edges {
                    if let Some(existing_edge) = edges.get_mut(&node) {
                        // Update weight (sum them)
                        existing_edge.weight += edge.weight;
                        stats.edges_updated += 1;
                    } else {
                        edges.insert(node, edge);
                        stats.edges_added += 1;
                    }
                }
            } else {
                stats.edges_added += assoc.edges.len();
                existing.insert(assoc.context_key, assoc.edges);
            }
        }

        // Rebuild associations from merged map
        self.snapshot.associations = existing
            .into_iter()
            .map(|(k, v)| AssociationEntry {
                context_key: k,
                edges: v,
            })
            .collect();

        // Merge sources
        for (id, source) in snapshot.sources {
            if !self.snapshot.sources.contains_key(&id) {
                self.snapshot.sources.insert(id, source);
                stats.sources_added += 1;
            }
        }

        Ok(stats)
    }

    fn save(&self) -> StorageResult<()> {
        let json = serde_json::to_string_pretty(&self.snapshot)?;
        let tmp_path = format!("{}.tmp", self.path);
        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }

    fn load(&mut self) -> StorageResult<()> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(()); // Start with empty snapshot
        }
        let json = std::fs::read_to_string(path)?;
        self.snapshot = serde_json::from_str::<GraphSnapshot>(&json)
            .or_else(|_| {
                // Try loading old format (AssociativeMemory directly)
                #[derive(Deserialize)]
                struct OldFormat {
                    associations: Vec<AssociationEntry>,
                    #[serde(default)]
                    sources: HashMap<String, Source>,
                    learning_rate: f64,
                    decay_rate: f64,
                }
                let old: OldFormat = serde_json::from_str(&json)?;
                Ok::<_, serde_json::Error>(GraphSnapshot {
                    version: 1,
                    associations: old.associations,
                    sources: old.sources,
                    learning_rate: old.learning_rate,
                    decay_rate: old.decay_rate,
                    metadata: HashMap::new(),
                })
            })?
            .migrate();
        Ok(())
    }

    fn location(&self) -> &str {
        &self.path
    }

    fn optimize(&mut self) -> StorageResult<()> {
        // JSON doesn't need optimization, but we could compact the file
        self.save()
    }

    fn backend_name(&self) -> &'static str {
        "json"
    }
}

/// Convenience function to migrate between backends
pub fn migrate<From: StorageBackend, To: StorageBackend>(
    from: &From,
    to: &mut To,
) -> StorageResult<()> {
    let snapshot = from.export()?;
    to.import(snapshot)?;
    to.save()?;
    Ok(())
}

/// Export to a file in a specific format
pub fn export_to_file(backend: &impl StorageBackend, path: &Path, format: ExportFormat) -> StorageResult<()> {
    let snapshot = backend.export()?;
    let tmp = path.with_extension("tmp");

    let json = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&snapshot)?,
        ExportFormat::JsonCompact => serde_json::to_string(&snapshot)?,
    };

    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;

    Ok(())
}

/// Import from a file
pub fn import_from_file(backend: &mut impl StorageBackend, path: &Path) -> StorageResult<()> {
    let json = std::fs::read_to_string(path)?;
    let snapshot: GraphSnapshot = serde_json::from_str(&json)?;
    backend.import(snapshot.migrate())?;
    Ok(())
}

/// Export formats
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    JsonCompact,
    // Future: MessagePack, CBOR, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_roundtrip() {
        let mut snapshot = GraphSnapshot::new();
        snapshot.learning_rate = 0.2;
        snapshot.sources.insert(
            "test".into(),
            Source::new("test".into(), "Test Source".into(), SourceOrigin::Manual),
        );

        let json = serde_json::to_string(&snapshot).unwrap();
        let loaded: GraphSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.version, SCHEMA_VERSION);
        assert_eq!(loaded.learning_rate, 0.2);
        assert!(loaded.sources.contains_key("test"));
    }

    #[test]
    fn test_json_storage() {
        let mut storage = JsonStorage::new("/tmp/test_memory.json");

        // Create some test data
        let mut snapshot = GraphSnapshot::new();
        snapshot.associations.push(AssociationEntry {
            context_key: ContextKey::new(vec![], "test".into()),
            edges: HashMap::from([("target".into(), Edge::new(0.5, Some("test_type".into())))]),
        });

        storage.import(snapshot).unwrap();

        let exported = storage.export().unwrap();
        assert_eq!(exported.associations.len(), 1);
    }
}
