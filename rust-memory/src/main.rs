mod memory;
mod mcp;
mod storage;
mod web;

use memory::AssociativeMemory;
use mcp::{McpServer, SharedMemory};
use storage::{export_to_file, import_from_file, ExportFormat, JsonStorage, StorageBackend};
use web::{start_server, AppState};

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_DATA_DIR: &str = ".associative-memory";
const DEFAULT_DATA_FILE: &str = "memory.json";
const WEB_PORT: u16 = 3001;

/// Resolve the data path: --data-path flag > $HOME/.associative-memory/memory.json
fn resolve_data_path(args: &[String]) -> String {
    // Check for --data-path <path> flag
    for i in 0..args.len() {
        if args[i] == "--data-path" {
            if let Some(path) = args.get(i + 1) {
                return path.clone();
            }
        }
    }

    // Default: $HOME/.associative-memory/memory.json
    if let Some(home) = std::env::var_os("HOME") {
        let dir = std::path::Path::new(&home).join(DEFAULT_DATA_DIR);
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        dir.join(DEFAULT_DATA_FILE).to_string_lossy().to_string()
    } else {
        // Fallback to CWD
        DEFAULT_DATA_FILE.to_string()
    }
}

fn print_usage() {
    eprintln!(
        r#"
Associative Memory Server

USAGE:
    associative-memory [OPTIONS] [COMMAND]

COMMANDS:
    (default)       Start web server only on port {0}
    --mcp           Start MCP server on stdio + web server on port {0}
                    (both share the same memory - recommended for Claude Code)
    --web-only      Start web server only (no MCP)
    export <file>   Export memory to file
    import <file>   Import memory from file
    stats           Show memory statistics
    migrate <from> <to>  Migrate between storage backends

OPTIONS:
    --data-path <path>  Path to memory.json (default: $HOME/.associative-memory/memory.json)
    --help              Show this help message

EXAMPLES:
    associative-memory                    # Start web server only
    associative-memory --mcp              # Start MCP + web (shared memory)
    associative-memory --data-path /path/to/memory.json --mcp
    associative-memory export backup.json # Export to file
    associative-memory import backup.json # Import from file
    associative-memory stats              # Show stats
"#,
        WEB_PORT
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Handle help
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        return Ok(());
    }

    // Check for MCP mode (runs both MCP on stdio AND web server)
    let mcp_mode = args.contains(&"--mcp".to_string());
    let web_only = args.contains(&"--web-only".to_string());
    let data_path = resolve_data_path(&args);

    // Handle CLI commands
    if args.len() > 1 && !mcp_mode && !web_only {
        // Find first positional arg (skip flags and their values)
        let mut cmd_idx = None;
        let mut i = 1;
        while i < args.len() {
            if args[i] == "--data-path" {
                i += 2; // skip flag and value
                continue;
            }
            if args[i].starts_with("--") {
                i += 1;
                continue;
            }
            cmd_idx = Some(i);
            break;
        }

        if let Some(idx) = cmd_idx {
            match args[idx].as_str() {
                "export" => {
                    if args.len() <= idx + 1 {
                        eprintln!("Usage: associative-memory export <file>");
                        std::process::exit(1);
                    }
                    return cmd_export(&args[idx + 1], &data_path).await;
                }
                "import" => {
                    if args.len() <= idx + 1 {
                        eprintln!("Usage: associative-memory import <file>");
                        std::process::exit(1);
                    }
                    return cmd_import(&args[idx + 1], &data_path).await;
                }
                "stats" => {
                    return cmd_stats(&data_path).await;
                }
                "migrate" => {
                    if args.len() <= idx + 2 {
                        eprintln!("Usage: associative-memory migrate <from> <to>");
                        std::process::exit(1);
                    }
                    return cmd_migrate(&args[idx + 1], &args[idx + 2]).await;
                }
                _ => {}
            }
        }
    }

    // Initialize logging (to stderr so it doesn't interfere with MCP on stdio)
    if !mcp_mode {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();

        tracing::info!("Starting Associative Memory Server");
    } else {
        // In MCP mode, log to stderr with minimal output
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("warn"))
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    }

    // Load or create memory
    let memory = if Path::new(&data_path).exists() {
        if !mcp_mode {
            tracing::info!("Loading memory from {}", data_path);
        }
        AssociativeMemory::load(Path::new(&data_path))?
    } else {
        if !mcp_mode {
            tracing::info!("Creating new memory at {}", data_path);
        }
        AssociativeMemory::new(0.1, 0.99)
    };

    // Create shared memory - this will be used by BOTH servers
    let shared_memory: SharedMemory = Arc::new(RwLock::new(memory));

    if mcp_mode {
        // MCP mode: Run BOTH MCP server (stdio) AND web server (port 3000)
        // They share the same memory!

        // Clone for web server
        let web_memory = shared_memory.clone();
        let web_data_path = data_path.clone();

        // Start web server in background
        let web_handle = tokio::spawn(async move {
            let state = AppState {
                memory: web_memory,
                data_path: web_data_path,
            };

            // Log to stderr so it doesn't interfere with MCP stdio
            eprintln!("[web] Starting web UI on http://localhost:{}", WEB_PORT);

            if let Err(e) = start_server(state, WEB_PORT).await {
                eprintln!("[web] Web server error: {}", e);
            }
        });

        // Run MCP server on main thread (stdio)
        let mcp_server = McpServer::new(shared_memory, data_path);
        mcp_server.run_stdio().await?;

        // If MCP exits, abort web server
        web_handle.abort();
    } else {
        // Web-only mode
        let state = AppState {
            memory: shared_memory,
            data_path: data_path,
        };

        tracing::info!("Web UI available at http://localhost:{}", WEB_PORT);
        tracing::info!("Tip: Use --mcp flag to run both MCP and web server with shared memory");
        start_server(state, WEB_PORT).await?;
    }

    Ok(())
}

/// Export memory to a file
async fn cmd_export(output_path: &str, data_path: &str) -> anyhow::Result<()> {
    println!("Loading memory from {}...", data_path);

    let mut storage = JsonStorage::new(data_path);
    storage.load()?;

    let stats = storage.snapshot().stats();
    println!(
        "Memory contains {} nodes, {} edges, {} sources",
        stats.num_nodes, stats.num_edges, stats.num_sources
    );

    let format = if output_path.ends_with(".min.json") {
        ExportFormat::JsonCompact
    } else {
        ExportFormat::Json
    };

    println!("Exporting to {}...", output_path);
    export_to_file(&storage, Path::new(output_path), format)?;

    println!("Export complete!");
    Ok(())
}

/// Import memory from a file
async fn cmd_import(input_path: &str, data_path: &str) -> anyhow::Result<()> {
    println!("Loading from {}...", input_path);

    let mut storage = JsonStorage::new(data_path);

    // Load existing if present
    if Path::new(data_path).exists() {
        storage.load()?;
        let stats = storage.snapshot().stats();
        println!(
            "Existing memory: {} nodes, {} edges",
            stats.num_nodes, stats.num_edges
        );
    }

    import_from_file(&mut storage, Path::new(input_path))?;

    let stats = storage.snapshot().stats();
    println!(
        "After import: {} nodes, {} edges, {} sources",
        stats.num_nodes, stats.num_edges, stats.num_sources
    );

    println!("Saving to {}...", data_path);
    storage.save()?;

    println!("Import complete!");
    Ok(())
}

/// Show memory statistics
async fn cmd_stats(data_path: &str) -> anyhow::Result<()> {
    if !Path::new(data_path).exists() {
        println!("No memory file found at {}", data_path);
        return Ok(());
    }

    let mut storage = JsonStorage::new(data_path);
    storage.load()?;

    let stats = storage.snapshot().stats();
    let snapshot = storage.snapshot();

    println!("=== Associative Memory Statistics ===");
    println!();
    println!("Storage: {} ({})", data_path, storage.backend_name());
    println!("Schema version: {}", stats.version);
    println!();
    println!("Nodes:    {}", stats.num_nodes);
    println!("Edges:    {}", stats.num_edges);
    println!("Contexts: {}", stats.num_contexts);
    println!("Sources:  {}", stats.num_sources);
    println!();

    if !snapshot.sources.is_empty() {
        println!("=== Sources ===");
        for (id, source) in &snapshot.sources {
            println!("  {} - {} ({:?})", id, source.name, source.origin);
        }
        println!();
    }

    // Connection type breakdown
    let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for assoc in &snapshot.associations {
        for edge in assoc.edges.values() {
            let key = edge
                .conn_type
                .clone()
                .unwrap_or_else(|| "untyped".to_string());
            *type_counts.entry(key).or_insert(0) += 1;
        }
    }

    if !type_counts.is_empty() {
        println!("=== Connection Types ===");
        let mut types: Vec<_> = type_counts.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));
        for (conn_type, count) in types.iter().take(10) {
            println!("  {}: {}", conn_type, count);
        }
        if types.len() > 10 {
            println!("  ... and {} more types", types.len() - 10);
        }
    }

    Ok(())
}

/// Migrate between storage backends
async fn cmd_migrate(from_path: &str, to_path: &str) -> anyhow::Result<()> {
    println!("Migrating from {} to {}...", from_path, to_path);

    let mut from_storage = JsonStorage::new(from_path);
    from_storage.load()?;

    let stats = from_storage.snapshot().stats();
    println!(
        "Source: {} nodes, {} edges, {} sources",
        stats.num_nodes, stats.num_edges, stats.num_sources
    );

    let mut to_storage = JsonStorage::new(to_path);
    storage::migrate(&from_storage, &mut to_storage)?;

    println!("Migration complete!");
    Ok(())
}
