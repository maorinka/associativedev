---
name: Memory Workflow
description: This skill should be used when the user asks "how to use memory", "memory workflow", "session workflow", "what to capture in memory", "memory best practices", "how to structure knowledge", or requests guidance on practical memory usage patterns.
version: 0.1.0
---

# Memory Workflow

## Overview

Effective memory usage follows a three-phase workflow: session start (recall context), during work (capture knowledge), session end (save and reflect). This workflow ensures knowledge accumulates across sessions rather than being lost.

## Three-Phase Workflow

### Phase 1: Session Start - Recall Context

**Goal**: Load prior knowledge before starting work

**Actions**:
1. Register this session as a source
2. Search for relevant prior context
3. Think through associations
4. Report findings

**Commands**:
```
/start-session [topic]
```

**Manual steps**:
```
register_source(
  id="session:jan21-auth",
  name="Jan 21 - Authentication Refactor"
)

search("authentication")
think("auth decisions", steps=5)
```

**When to search**:
- Topic keywords: "authentication", "database", "API"
- Problem domains: "bug", "performance", "security"
- Past decisions: "decision", "chosen", "rejected"

**Report template**:
```
Memory check:
- Found 3 prior contexts on authentication
- Key decision: Chose JWT over sessions (see session:jan15)
- Known issue: Token refresh needs work
- No conflicts detected
```

### Phase 2: During Work - Capture Knowledge

**Goal**: Add knowledge as you discover it

**What to Capture**:

**DO capture**:
- Decisions and rationale
- Problems encountered + solutions
- Key concepts and relationships
- Design choices and tradeoffs
- Dependencies between components
- Who/what said something (provenance)

**DON'T capture**:
- Full text/documents (summarize first)
- Temporary debugging info
- Highly volatile data
- Obvious facts already known

**Commands**:
```
/capture-knowledge <description>
```

**Manual capture**:
```
# Simple path
add_path(
  path=["authentication", "JWT tokens", "stateless architecture"],
  conn_types=["uses", "enables"],
  source_id="session:jan21-auth"
)

# Single connection
add_connection(
  from="API gateway",
  to="Redis cache",
  conn_type="depends_on",
  source_id="session:jan21-auth"
)
```

**Capture frequency**:
- After major decisions
- When solving problems
- When discovering connections
- When finishing logical chunks
- NOT after every minor action

### Phase 3: Session End - Save and Reflect

**Goal**: Persist knowledge and review what was learned

**Actions**:
1. Save memory to disk
2. Review statistics
3. Summarize session

**Commands**:
```
/end-session
```

**Manual steps**:
```
save()
get_stats()  # Review nodes, edges, sources
```

**Reflection questions**:
- What key decisions were made?
- What problems were solved?
- What connections were discovered?
- What should future sessions know?

## Connection Types

Use semantic relationship names to describe connections.

### Common Types

**Causality**:
- causes, prevents, triggers, enables, blocks
- Example: "memory leak causes performance degradation"

**Structure**:
- is_a, part_of, contains, includes
- Example: "JWT token is_a authentication mechanism"

**Dependency**:
- uses, requires, depends_on
- Example: "API gateway depends_on Redis cache"

**Provenance**:
- created_by, authored_by, stated_by
- Example: "design created_by senior architect"

**Logic**:
- contradicts, supports, extends, refines
- Example: "Paper B contradicts Paper A"

See `references/connection-types.md` for comprehensive list with examples.

### Choosing Types

**Be specific**: "depends_on" > "related_to"
**Be semantic**: Use types that convey meaning
**Be consistent**: Same relationships = same type
**Be directional**: "A uses B" ≠ "B uses A"

## Bidirectional vs Unidirectional

**Default**: Unidirectional (A→B doesn't create B→A)

**Why**: Semantic correctness
- "A uses B" doesn't mean "B uses A"
- "A causes B" doesn't mean "B causes A"

**When to use bidirectional**:
- Symmetric relationships: "related_to", "similar_to"
- Two-way dependencies: "collaborates_with"
- Equivalences: "same_as", "synonymous_with"

**How to use**:
```
add_path(
  path=["concept A", "concept B"],
  conn_types=["related_to"],
  bidirectional=true  # Explicitly opt-in
)
```

## Source Management

### Registering Sources

**Format**: `{type}:{identifier}`

**Examples**:
- `session:jan21-auth` - Conversation session
- `paper:smith2025` - Research paper
- `meeting:standup-jan21` - Team meeting
- `tweet:addyosmani-123` - Social media post
- `docs:api-reference` - Documentation

**Best practices**:
- Use consistent prefixes (session, paper, meeting, etc.)
- Include dates for time-based sources
- Use descriptive identifiers
- Keep source names short but clear

### Source Metadata

```
register_source(
  id="paper:smith2025",
  name="Smith et al. 2025 - Agent Memory Systems",
  origin="research",
  url="https://arxiv.org/abs/...",
  metadata={
    "authors": "Smith, Jones",
    "year": "2025",
    "venue": "NeurIPS"
  }
)
```

### Querying by Source

```
# What did this source contribute?
edges_by_source("paper:smith2025")

# Where do sources agree?
source_overlap("paper:smith2025", "paper:jones2025")

# Show all sources
list_sources()
```

## Capturing Patterns

### Problem → Solution

```
add_path(
  path=["auth bug", "Redis eviction", "increase memory", "fixed"],
  conn_types=["caused_by", "solved_by", "resulted_in"],
  source_id="session:debugging"
)
```

### Decision → Rationale → Tradeoff

```
add_path(
  path=["use GraphQL", "reduce over-fetching", "adds complexity"],
  conn_types=["decision", "benefit", "tradeoff"],
  source_id="session:architecture"
)
```

### Concept → Implementation → Result

```
add_path(
  path=["caching strategy", "Redis with LRU", "50% latency reduction"],
  conn_types=["implemented_as", "achieved"],
  source_id="session:performance"
)
```

### Claim → Evidence → Source

```
add_path(
  path=["Hebbian learning improves recall", "experiment data", "paper:smith2025"],
  conn_types=["supported_by", "from"],
  source_id="research:literature-review"
)
```

## Common Workflows

### Onboarding New Team Member

**Goal**: Explain architectural decisions

**Steps**:
1. Search for decisions: `search("decision")`
2. Think from topic: `think("architecture", steps=7)`
3. Show provenance: `edges_by_source("meeting:architecture-review")`

### Research Paper Synthesis

**Goal**: Connect findings from multiple papers

**Steps**:
1. Register each paper as source
2. Add key claims as paths
3. Note contradictions: `conn_type="contradicts"`
4. Find agreements: `source_overlap("paper:A", "paper:B")`

### Bug Investigation

**Goal**: Track debugging process

**Steps**:
1. Capture symptoms: `add_path(["error X", "component Y"])`
2. Record hypotheses: `add_connection("bug", "possible_cause", "hypothesis")`
3. Note what was ruled out: `conn_type="ruled_out"`
4. Capture solution: `add_path(["bug", "root cause", "fix"])`

### Feature Development

**Goal**: Track decisions and rationale

**Steps**:
1. Capture requirements: `add_path(["feature X", "requirement Y"])`
2. Record alternatives: `add_connection("approach A", "rejected", "approach B")`
3. Note chosen solution: `add_path(["chosen", "implementation", "rationale"])`
4. Track results: `add_connection("feature", "resulted_in", "outcome")`

## Memory Hygiene

### What Makes Good Memory

**Granular**: One concept per node
- Good: "JWT tokens", "stateless architecture"
- Bad: "We use JWT tokens for stateless architecture"

**Semantic**: Meaningful connection types
- Good: `uses`, `enables`, `requires`
- Bad: `related`, `connected`, `associated`

**Attributed**: Always include source_id
- Good: `source_id="session:jan21"`
- Bad: `source_id=None`

**Structured**: Use consistent patterns
- Good: Decision → Rationale → Tradeoff
- Bad: Random connections with no pattern

### When to Update vs Add

**Add new** when:
- Discovering new connections
- Learning from new sources
- Recording new decisions

**Update existing** when:
- Correcting mistakes
- Refining connection types
- Strengthening weights (via think)

### Avoiding Clutter

**Skip**:
- Overly specific: "fixed typo on line 42"
- Temporary: "testing this approach"
- Obvious: "JavaScript is a programming language"

**Capture**:
- Decisions: "Chose approach X because Y"
- Problems: "Bug caused by Z"
- Insights: "Pattern observed across A, B, C"

## Advanced Patterns

### Context-Aware Associations

Same connection in different contexts:

```
# In web context
add_path(["deploy", "production server"], context=["web", "hosting"])

# In military context
add_path(["deploy", "troops"], context=["military", "operations"])
```

### Weighted Importance

Manually set initial weights:

```
add_path(
  path=["critical bug", "production down"],
  weight=2.0  # Higher weight = more important
)
```

### Temporal Tracking

Use timestamps for versioning:

```
add_connection(
  from="API design",
  to="REST v1",
  timestamp=1640000000
)

# Later
add_connection(
  from="API design",
  to="GraphQL v2",
  timestamp=1672500000
)
```

## Integration with Other Tools

### With Claude Code

Memory hooks remind to check memory:
- UserPromptSubmit hook triggers on tasks
- Automatically suggests search/think
- Configurable via `.claude/settings.json`

### With Git

Track decisions at commit time:
```
# In commit hook
add_path(
  path=["commit:abc123", "refactored auth", "improved security"],
  source_id="git:main"
)
```

### With CI/CD

Capture build/deploy knowledge:
```
add_path(
  path=["build failure", "missing dependency", "fixed in v2.1"],
  source_id="ci:pipeline-42"
)
```

## Examples

### Example 1: Capture Conversation

See `examples/capture-conversation.md` for step-by-step walkthrough

### Example 2: Research Synthesis

See `examples/research-synthesis.md` for academic workflow

## Troubleshooting

**Memory not persisting**:
- Run `save()` explicitly
- Check file permissions on `memory.json`
- Verify working directory

**Can't find prior context**:
- Check source_id matches
- Verify nodes exist: `search("keyword")`
- Try broader search terms

**Edges not appearing**:
- Confirm unidirectional vs bidirectional
- Check connection type spelling
- Verify node names (case-sensitive)

**Graph too large**:
- Export to proper DB (Neo4j, PostgreSQL)
- Prune unused edges (future feature)
- Split into multiple graphs

## Additional Resources

### References

- **`references/connection-types.md`** - Comprehensive list of relationship types
- **`references/advanced-patterns.md`** - Complex usage patterns

### Examples

- **`examples/capture-conversation.md`** - Full conversation capture workflow
- **`examples/research-synthesis.md`** - Academic research workflow

### Related Skills

- **memory-concepts** - Understanding the system
- **query-patterns** - Effective search and recall
