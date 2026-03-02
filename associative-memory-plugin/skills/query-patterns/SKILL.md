---
name: Query Patterns
description: This skill should be used when the user asks "how to search memory", "how to find information", "recall context", "query memory", "search for decisions", "think through associations", or requests guidance on effective memory retrieval strategies.
version: 0.1.0
---

# Query Patterns

## Overview

Effective memory retrieval uses three primary tools: **search** (find nodes by name), **think** (follow associations), and **find_path** (discover connections). Each serves different recall needs.

## Core Query Tools

### search(pattern)

Find nodes by name matching a pattern.

**Usage**:
```
search("auth")          # Find all nodes containing "auth"
search("decision")      # Find all decision points
search("bug")           # Find all bugs
```

**Returns**: List of matching nodes with contexts and weights

**When to use**:
- Know part of the node name
- Broad exploration
- Finding all instances of a topic

### think(start, steps, mode)

Follow associations from a starting concept.

**Usage**:
```
think("authentication", steps=5)                    # Weighted random walk
think("auth bug", steps=7, mode="strongest")       # Follow strongest paths
```

**Modes**:
- `weighted`: Random walk weighted by edge strength (default)
- `strongest`: Always follow highest-weight edge

**When to use**:
- Explore connections from a concept
- Discover related ideas
- Retrace thought processes

### find_path(from, to)

Discover connections between two nodes.

**Usage**:
```
find_path("problem X", "solution Y")    # How are they connected?
find_path("Paper A", "Paper B")         # What links these papers?
```

**Returns**: Path if exists, empty if no connection

**When to use**:
- Know start and end points
- Verify connections exist
- Understand relationships

## Common Query Patterns

### Pattern 1: Find All Related to Topic

```
# Step 1: Search for topic
results = search("authentication")

# Step 2: Think from each result
for node in results:
    think(node, steps=3)
```

**Use case**: Comprehensive topic exploration

### Pattern 2: Recall Decisions

```
search("decision")
search("chose")
search("rejected")
```

**Use case**: Understanding why choices were made

### Pattern 3: Find Problems and Solutions

```
# Find problems
problems = search("bug")
problems += search("issue")
problems += search("error")

# For each, think to solutions
for problem in problems:
    think(problem, steps=5)
```

**Use case**: Learning from past debugging

### Pattern 4: Source Attribution

```
edges_by_source("paper:smith2025")      # What did this paper contribute?
source_overlap("paper:A", "paper:B")    # Where do papers agree?
list_sources()                           # All sources
```

**Use case**: Provenance tracking, conflict resolution

### Pattern 5: Context-Specific Search

```
# Search within specific context
search("deploy", context=["web", "hosting"])
```

**Use case**: Disambiguate terms with multiple meanings

## Advanced Techniques

### Combining Queries

```
# Find → Think → Attribute
results = search("memory leak")
for node in results:
    path = think(node, steps=5)
    # Check which source identified this
    edges = edges_by_source("session:debugging")
```

### Iterative Refinement

```
# Start broad
results = search("auth")

# Too many results? Be specific
results = search("auth bug")

# Still too many? Add context
results = search("auth", context=["production"])
```

### Weight-Based Filtering

Higher weight = more frequently accessed:

```
results = search("pattern")
# Results include total_weight
important = [r for r in results if r['total_weight'] > 1.0]
```

## Query Strategies by Use Case

### Onboarding (Learn System)

1. List all sources: `list_sources()`
2. Major topics: `search("")` (all nodes)
3. Start from key concepts: `think("architecture", steps=10)`

### Debugging (Find Related Issues)

1. Search symptoms: `search("timeout")`
2. Think from symptoms: `think("timeout", steps=5)`
3. Check past solutions: `search("fixed")`

### Research (Synthesize Sources)

1. Find claims: `search("hypothesis")`
2. Check sources: `edges_by_source("paper:A")`
3. Find overlaps: `source_overlap("paper:A", "paper:B")`

### Design (Recall Decisions)

1. Find decisions: `search("decision")`
2. Get rationale: `think("chose GraphQL", steps=3)`
3. Check tradeoffs: `search("tradeoff")`

## Think Modes Comparison

| Mode | Behavior | Best For |
|------|----------|----------|
| weighted | Random walk by weights | Natural exploration |
| strongest | Always take strongest edge | Find most-reinforced paths |

**Weighted mode**: Discovers variety of connections
**Strongest mode**: Retraces most-traveled paths

## Search Tips

### Effective Patterns

**Good**:
- Specific keywords: "auth bug"
- Domain terms: "JWT", "Redis"
- Action verbs: "decided", "fixed", "ruled out"

**Bad**:
- Too general: "system", "thing"
- Too specific: "line 42 in file.ts"
- Natural language: "the authentication thing we discussed"

### Regular Expressions

Search supports regex patterns:

```
search("bug|issue|error")           # Any of these
search("auth.*")                    # Starts with auth
search(".*decision.*")              # Contains decision
```

## Troubleshooting Queries

**No results**:
- Try broader terms: "auth" instead of "authentication system"
- Check spelling (case-sensitive)
- Use `search("")` to see all nodes

**Too many results**:
- Be more specific: "JWT token" instead of "token"
- Add context: specify the domain
- Use connection type filters

**Unexpected connections**:
- Check source: `edges_by_source(id)`
- Verify connection types
- Consider context differences

## Query Performance

**Fast** (< 10ms):
- search() with specific patterns
- find_path() between nearby nodes
- get_stats()

**Medium** (10-100ms):
- think() with 5-10 steps
- edges_by_source() on large sources
- source_overlap()

**Slow** (> 100ms):
- think() with many steps (15+)
- search("") (all nodes)
- Complex regex patterns

Optimize by starting specific and broadening as needed.

## Examples

### Example 1: Debug Recall

See `examples/debugging-recall.md`

### Example 2: Decision History

See `examples/decision-history.md`

## Additional Resources

### References

- **`references/advanced-queries.md`** - Complex query patterns

### Related Skills

- **memory-concepts** - Understanding the system
- **memory-workflow** - Capturing effectively
