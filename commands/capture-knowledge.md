---
name: capture-knowledge
description: Interactively capture knowledge to memory
argument-hint: "<description>"
allowed-tools: ["mcp__associative-memory__*", "AskUserQuestion"]
---

# Capture Knowledge

Add knowledge to memory through interactive prompts.

## Steps

1. **Parse description** to understand what to capture

2. **Determine format**:
   - Chain of concepts? → Use `add_path`
   - Single connection? → Use `add_connection`
   - Ask user if ambiguous

3. **Collect information**:
   - **Nodes**: What concepts to connect?
   - **Types**: What relationships? (uses, causes, enables, etc.)
   - **Source**: Which source_id? (current session or other)
   - **Bidirectional**: Symmetric relationship?

4. **Add to memory** using appropriate tool

5. **Confirm**: Report what was added

## Interactive Mode

Use `AskUserQuestion` to collect details:

```
AskUserQuestion:
  "How should this be structured?"
  Options:
    - "Chain of concepts (A → B → C)"
    - "Single connection (A → B)"
    - "Multiple connections"
```

```
AskUserQuestion:
  "What concepts to connect?"
  [Free text input]
```

```
AskUserQuestion:
  "What relationship types?"
  [Free text, comma-separated]
```

## Examples

### Example 1: Simple Chain

```
User: /capture-knowledge We decided to use JWT for authentication

You: [Interactive prompts]
  Structure: Chain
  Path: ["authentication decision", "use JWT tokens", "stateless architecture"]
  Types: ["decided", "enables"]
  Source: session:current

Then add:
add_path(
  path=["authentication decision", "use JWT tokens", "stateless architecture"],
  conn_types=["decided", "enables"],
  source_id="session:20260121"
)

Report: Added chain: authentication decision → use JWT tokens → stateless architecture
```

### Example 2: Problem-Solution

```
User: /capture-knowledge Memory leak caused by unclosed connections, fixed with connection pooling

You: [Parse description]
add_path(
  path=["memory leak", "unclosed connections", "connection pooling", "fixed"],
  conn_types=["caused_by", "solved_with", "resulted_in"],
  source_id="session:20260121-debugging"
)

Report: Added debugging path with 4 nodes
```

## Default Behaviors

- **Source**: Use most recent registered session
- **Bidirectional**: false (unidirectional)
- **Weight**: 0.5 (default)

## Tips

- Ask clarifying questions
- Suggest connection types based on context
- Keep node names concise
- Report what was added clearly
