---
name: end-session
description: Save memory and summarize session
allowed-tools: ["mcp__associative-memory__*"]
---

# End Memory Session

Save memory to disk and summarize what was captured.

## Steps

1. **Save memory**:
   ```
   save()
   ```

2. **Get statistics**:
   ```
   get_stats()
   ```

3. **Summarize session**:
   - What was added
   - Statistics (nodes, edges, sources)
   - Key captures

## Output Format

```
## Session Summary

### Memory Saved
✓ Saved to memory.json

### Statistics
- Total nodes: X
- Total edges: Y
- Sources: Z
- Connection types: N unique types

### This Session
- Added: A new nodes
- New connections: B
- Sources used: [list]

### Key Captures
- [Summarize main things added this session]
```

## Example

```
User: /end-session

You:
## Session Summary

### Memory Saved
✓ Saved to memory.json

### Statistics
- Total nodes: 324
- Total edges: 287
- Sources: 12
- Connection types: 45 unique types

### This Session
- Added: 8 new nodes
- New connections: 12
- Sources used: session:20260121-auth

### Key Captures
- Authentication decision: chose JWT tokens
- Problem: unclosed connections → memory leak
- Solution: implemented connection pooling
- Result: 40% memory reduction
```

## Tips

- Always save before reporting
- Show session delta if possible
- Highlight key captures
- Mention if anything notable (many nodes, new sources, etc.)
