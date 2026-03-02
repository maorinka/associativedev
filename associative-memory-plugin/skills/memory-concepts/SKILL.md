---
name: Memory Concepts
description: This skill should be used when the user asks "what is associative memory", "explain Hebbian learning", "how does provenance tracking work", "explain memory concepts", or requests an overview of the memory system architecture and principles.
version: 0.1.0
---

# Memory Concepts

## Overview

Associative memory systems enable persistent knowledge storage across sessions by creating explicit connections between concepts, tracking where information came from, and strengthening frequently-accessed paths. This contrasts with stateless conversations where Claude forgets everything between sessions.

## Core Concepts

### Associative Memory

Store knowledge as a **directed graph** of concepts (nodes) connected by typed relationships (edges):

```
concept A --relationship-type--> concept B
```

**Example**:
```
authentication --uses--> JWT tokens --enables--> stateless architecture
```

Each connection has:
- **Type** (uses, enables, requires, causes, etc.)
- **Weight** (strengthens with traversal)
- **Provenance** (source_id tracking who said it)
- **Context** (path-aware associations)

### Hebbian Learning

"Neurons that fire together wire together" - paths that get traversed frequently become stronger.

**Mechanism**:
1. Think through a path: A → B → C
2. System reinforces weights on those edges
3. Future queries find that path more easily
4. Unused paths decay over time

**Practical effect**: Knowledge you access frequently becomes easier to recall.

### Provenance Tracking

Every edge records its origin:
- **source_id**: "conversation:jan21", "paper:smith2025", "meeting:standup"
- **timestamp**: When the connection was made

**Benefits**:
- Know who said what
- Trace claims back to sources
- Find conflicts between sources
- Trust verification

**Example queries**:
```
edges_by_source("paper:smith2025")  # What did Smith's paper contribute?
source_overlap("paper:A", "paper:B") # Where do two papers agree?
```

### Graph Structure

**Nodes**: Concepts, entities, facts
- No duplicates (case-sensitive)
- Simple strings ("authentication", "JWT tokens")

**Edges**: Typed directional connections
- Unidirectional by default (A→B doesn't create B→A)
- Optional bidirectional when relationship is symmetric
- Named types for semantic meaning

**Context**: Path-dependent associations
- A→B in context [X, Y] differs from A→B in context [Z]
- Enables disambiguation
- More precise than flat graphs

### Connection Types

Use semantic relationship names:

**Causality**: causes, prevents, triggers, enables, blocks
**Structure**: is_a, part_of, contains, includes
**Dependency**: uses, requires, depends_on
**Provenance**: created_by, authored_by, stated_by
**Logic**: contradicts, supports, extends, refines

See `references/connection-types.md` for comprehensive list with examples.

## System Architecture

### Memory Storage

**File**: `memory.json` (in working directory)
**Format**: Graph snapshot with versioning
**Updates**: In-memory during session, saved on command

### MCP Integration

**Server**: Runs via `--mcp` flag (stdio + web server)
**Tools**: search, think, add_path, add_connection, etc.
**Web UI**: Port 3001 (3D visualization)

### Web Interface

**URL**: http://localhost:3001
**Features**:
- 2D/3D graph visualization
- Search nodes
- Timeline filtering
- Source-colored edges
- Click to explore

## Key Principles

### 1. Explicit Over Implicit

**Don't**: Rely on embeddings/similarity
**Do**: Create explicit named connections

Why: You can trace relationships, not just find "similar" things.

### 2. Provenance First

**Don't**: Store facts without attribution
**Do**: Always tag with source_id

Why: Trust, verification, conflict resolution.

### 3. Unidirectional Default

**Don't**: Auto-create reverse edges
**Do**: Only create edges that make semantic sense

Why: "A uses B" doesn't mean "B uses A". Semantic correctness.

### 4. Progressive Reinforcement

**Don't**: Static weights
**Do**: Let traversal strengthen paths

Why: Frequently-accessed knowledge becomes easier to recall (mimics human memory).

### 5. Context-Aware

**Don't**: Treat all A→B connections the same
**Do**: Distinguish by context path

Why: "deploy" means different things in [CI/CD] vs [military] context.

## Memory Lifecycle

### Formation
Explicitly add connections:
```
add_path(["A", "B", "C"], conn_types=["uses", "enables"])
```

### Evolution
- Reinforcement via `think()` traversal
- Decay (configurable, not yet implemented)
- Manual updates/corrections

### Retrieval
- **search()** - Find nodes by name pattern
- **think()** - Follow associations
- **find_path()** - Discover connections between nodes
- **edges_by_source()** - Filter by provenance

## Comparison with Other Systems

### vs Vector Databases (RAG)

| Feature | Vector DB | Associative Memory |
|---------|-----------|-------------------|
| Lookup | Semantic similarity | Explicit connections |
| Relationships | Implicit | Named types |
| Provenance | Rarely built-in | First-class |
| Learning | Static | Hebbian reinforcement |
| Context | None | Path-aware |

### vs Knowledge Graphs

| Feature | Traditional KG | Associative Memory |
|---------|----------------|-------------------|
| Schema | Often required | Schema-free |
| Querying | SPARQL/Cypher | MCP tools |
| Learning | Manual updates | Auto-reinforcement |
| Integration | Database required | File-based + MCP |

### vs LLM Memory Systems

| System | Approach | Our Difference |
|--------|----------|----------------|
| Mem0, Letta | Vector + summaries | Graph + provenance |
| A-MEM | Zettelkasten + ChromaDB | Pure graph, no vectors |
| Zep | Temporal KG | Hebbian + file-based |
| Memoria | Weighted KG + summaries | MCP-first, simpler |

## Use Cases

### AI Agent Memory
Track decisions, problems, solutions across sessions

### Research Synthesis
Connect claims from multiple papers with attribution

### Decision Journal
Record choices, rationale, alternatives, outcomes

### Learning Aid
Build knowledge graphs that strengthen with review

### Institutional Memory
Capture team knowledge with provenance

See `skills/memory-workflow/` for practical usage patterns.

## Scale Considerations

**Recommended**: 500-1,000 nodes for smooth visualization

**Technical limits**:
- D3.js (2D): ~500 nodes
- 3D force graph: ~1,000 nodes
- Memory file: Grows ~150KB per 500 nodes
- MCP response: Keep under 100ms

**Large graphs**: Export to proper graph database (Neo4j, PostgreSQL)

## Additional Resources

### References

- **`references/research-papers.md`** - Academic papers (A-MEM, Memoria, Zep, etc.)
- **`references/connection-types.md`** - Comprehensive type list (in memory-workflow skill)

### Related Skills

- **memory-workflow** - How to use memory in practice
- **query-patterns** - Search and recall strategies

### External

- [A-MEM Paper](https://arxiv.org/abs/2502.12110) - NeurIPS 2025
- [Memoria Paper](https://arxiv.org/abs/2512.12686) - Weighted KG + sessions
- [Zep Architecture](https://arxiv.org/pdf/2501.13956) - Temporal KG
