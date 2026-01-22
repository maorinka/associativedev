# Research Papers on Agent Memory Systems

Comprehensive list of academic papers and industry implementations related to LLM agent memory, knowledge graphs, and associative memory systems.

## Recent Papers (2025-2026)

### A-MEM: Agentic Memory for LLM Agents
**Published**: NeurIPS 2025
**Paper**: https://arxiv.org/abs/2502.12110
**OpenReview**: https://openreview.net/forum?id=FiM0M8gcct

**Approach**: Zettelkasten-inspired with dynamic linking
**Storage**: ChromaDB (vector) + dynamic indexing
**Key insight**: Graph databases with predefined schemas limit adaptability

**Closest to our approach** in terms of dynamic association building.

### Memoria: Scalable Agentic Memory Framework
**Published**: December 2025
**Paper**: https://arxiv.org/abs/2512.12686

**Approach**: Weighted knowledge graph + session summaries
**Features**:
- Dynamic session-level summarization
- Incremental user trait capture
- Structured entities and relationships
- Hybrid short-term + long-term memory

**Similar**: Uses weighted KG like us, but more complex architecture

### Zep: Temporal Knowledge Graph Architecture
**Published**: January 2026
**Paper**: https://arxiv.org/pdf/2501.13956
**Website**: https://blog.getzep.com

**Approach**: Dual storage (episodic + semantic memory)
**Key features**:
- Mirrors psychological memory models
- Episodic: distinct events
- Semantic: associations between concepts
- Temporal evolution tracking

**Similar**: Graph-based, tracks evolution over time

### AriGraph: Episodic Memory for World Models
**Published**: IJCAI 2025
**Paper**: https://www.ijcai.org/proceedings/2025/0002.pdf

**Approach**: Working memory populated from world model graph
**Components**:
- Recent observations + actions
- Retrieved semantic knowledge
- Retrieved episodic knowledge
- Planning LLM with graph context

**Performance**: Outperforms full history, summarization, RAG, Reflexion

### MAGMA: Multi-Graph Agentic Memory
**Published**: January 2026

**Approach**: Multiple specialized graphs for different memory types

### EverMemOS: Self-Organizing Memory OS
**Published**: January 2026

**Approach**: Memory as an operating system with self-organization

## Industry Implementations

### MongoDB + LangGraph Integration
**Announced**: August 2025
**Blog**: https://www.mongodb.com/company/blog/product-release-announcements/powering-long-term-memory-for-agents-langgraph

**Features**:
- MongoDB Store for LangGraph
- Cross-session memory persistence
- Flexible, scalable storage

### Mem0: Universal Memory Layer
**GitHub**: https://github.com/mem0ai/mem0
**Stars**: 25k+

**Features**:
- Integrates with ChatGPT, Claude, etc.
- Universal memory abstraction
- Vector + metadata storage

### Letta (ex-MemGPT)
**GitHub**: https://github.com/letta-ai/letta
**Stars**: 15k+

**Features**:
- Stateful agents with self-editing memory
- Page memory in/out like virtual memory
- Self-improving capabilities

### Cognee
**GitHub**: https://github.com/topoteretes/cognee
**Stars**: 3k+

**Features**:
- Memory in 6 lines of code
- Production-ready
- Vector + graph hybrid

## Memory Taxonomies

### Functional Taxonomy
1. **Factual Memory** - Knowledge, facts, information
2. **Experiential Memory** - Insights, skills, learned patterns
3. **Working Memory** - Active context management

### Operational Lifecycle
1. **Formation** - Extraction from interactions
2. **Evolution** - Consolidation, forgetting, reinforcement
3. **Retrieval** - Access strategies, query patterns

## Key Insights from Literature

### On Graph Databases
- Predefined schemas limit adaptability (A-MEM)
- Dynamic relationship discovery is crucial
- Context-awareness improves precision

### On Memory Architecture
- Episodic vs semantic distinction matters (Zep)
- Working memory as active context (AriGraph)
- Multi-modal memory types serve different purposes (MAGMA)

### On Learning
- Self-organizing memory improves over time (EverMemOS)
- Forgetting is as important as remembering
- Reinforcement from usage improves recall

### On Integration
- Memory should be a universal layer (Mem0)
- Cross-session persistence is critical (MongoDB + LangGraph)
- Flexible storage matters for production (Cognee)

## Comparison with Our System

### What's Unique About Associative Memory

| Feature | Common Approach | Our Approach |
|---------|----------------|--------------|
| Storage | Vector DB (embeddings) | Pure graph (explicit edges) |
| Relationships | Inferred from similarity | Named types (uses, enables) |
| Provenance | Rarely built-in | First-class (source_id) |
| Reinforcement | Static or decay-only | Hebbian (traversal strengthens) |
| Context | Usually none | Path-aware associations |
| Scale | Large (production DBs) | Medium (500-1K nodes, file-based) |

### Closest Matches
1. **A-MEM** - Dynamic linking, no fixed schema
2. **Zep** - Temporal knowledge graph
3. **Memoria** - Weighted KG

### Our Advantages
- Simpler (file-based, no DB required)
- Provenance first-class
- MCP-native integration
- Hebbian reinforcement

### Our Limitations
- Smaller scale (not production-grade database)
- No vector similarity fallback
- Manual connection creation (no auto-extraction)

## ROI Data

From industry reports (2024-2025):
- 300-320% ROI for knowledge graph deployments
- Measurable impact in finance, healthcare, manufacturing
- Production maturity achieved

## Tool Landscape (2025)

**Performance-critical**: FalkorDB
**Agentic systems**: Cognee
**Dynamic schemas**: AutoSchemaKG

All mature, production-ready options exist.

## Research Directions

### Active Areas
1. **Conflict resolution** - Handling contradictory information
2. **Auto-extraction** - Deriving graphs from text automatically
3. **Forgetting strategies** - What to decay, when
4. **Multi-agent shared memory** - Coordination across agents
5. **Semantic compression** - Efficient large-graph storage

### Open Questions
1. Optimal balance between episodic and semantic memory?
2. How to handle memory at scale (millions of nodes)?
3. Best practices for provenance in multi-source systems?
4. How to validate memory accuracy over time?

## Further Reading

### Surveys
- "Memory in the Age of AI Agents: A Survey" (GitHub: Shichun-Liu/Agent-Memory-Paper-List)

### Classic Papers
- "Generative Agents" (Stanford, 2023) - Sims-like agents with memory streams
- MemoryBank - Persistent memory with forgetting
- Cognitive Architectures - SOAR, ACT-R inspired approaches

### Production Systems
- GitHub Copilot Memory (2024)
- ChatGPT Memory (OpenAI, 2024)
- Claude Projects (Anthropic, 2024)

## Using This Research

### For Understanding
- Read A-MEM for dynamic association concepts
- Read Zep for episodic/semantic split
- Read Memoria for weighted graph implementation

### For Implementation
- Study Cognee for production patterns
- Study Letta for self-improving memory
- Study MongoDB + LangGraph for persistence

### For Comparison
- Compare approaches in table format
- Identify unique features of each system
- Understand tradeoffs (scale vs simplicity, vectors vs graphs)

---

Last updated: January 2026
