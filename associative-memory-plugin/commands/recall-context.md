---
name: recall-context
description: Search memory and follow associations for a topic
argument-hint: "<topic>"
allowed-tools: ["mcp__associative-memory__*"]
---

# Recall Context

Search for and explore associations on a topic.

## Steps

1. **Search** for topic:
   ```
   search("<topic>")
   ```

2. **Think** from most relevant results:
   ```
   think("<node>", steps=5-7)
   ```

3. **Check sources** if relevant:
   ```
   edges_by_source("<source-id>")
   ```

4. **Report findings**:
   - What was found
   - Key connections
   - Sources
   - Recommendations

## Output Format

```
## Recall: <topic>

### Nodes Found
- Node 1 (weight: X, contexts: Y)
- Node 2 (weight: X, contexts: Y)

### Associations
Following from "<strongest node>":
- Path: A → B → C → D
- Key insight: [summarize]

### Sources
- source:id1 - [description]
- source:id2 - [description]

### Recommendations
[What user should know]
```

## Examples

### Example 1: Decision Recall

```
User: /recall-context authentication decisions

You:
## Recall: authentication decisions

### Nodes Found
- "authentication decision" (weight: 1.2, 3 contexts)
- "JWT token choice" (weight: 0.8, 2 contexts)

### Associations
Following from "authentication decision":
- Path: authentication decision → JWT tokens → stateless architecture → scalability

Key insight: Chose JWT over sessions for horizontal scaling

### Sources
- session:20260115 - Initial architecture discussion
- paper:oauth-best-practices - Referenced for security

### Recommendations
Token refresh mechanism still needs implementation (marked as TODO in session:20260118)
```

## Tips

- Start with search to find entry points
- Think from highest-weight nodes
- Report both facts and connections
- Include source attribution
- Surface actionable insights
