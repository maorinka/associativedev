---
name: start-session
description: Start a memory session - register source and recall prior context
argument-hint: "[topic]"
allowed-tools: ["mcp__associative-memory__*"]
---

# Start Memory Session

Register this session as a source and check for relevant prior context before beginning work.

## Steps

1. **Generate session ID** using current date and optional topic:
   - Format: `session:YYYYMMDD-topic` or `session:YYYYMMDD`
   - Example: `session:20260121-auth` or `session:20260121`

2. **Register source** using `register_source`:
   ```
   register_source(
     id="session:20260121-auth",
     name="Jan 21 - Authentication Work"
   )
   ```

3. **Search for context** if topic provided:
   ```
   search("<topic>")
   ```

4. **Think through associations**:
   ```
   think("<topic>", steps=5)
   ```

5. **Report findings** to user:
   ```
   Memory check:
   - Found X contexts on <topic>
   - Key info: [summarize]
   - Sources: [list relevant sources]
   ```

   Or if nothing found:
   ```
   Memory check: No prior context found for <topic>
   ```

## Examples

### With Topic

```
User: /start-session authentication refactor

You:
1. Register source: session:20260121-auth
2. Search "authentication"
3. Think from "authentication", steps=5
4. Report:
   Memory check:
   - Found 3 contexts on authentication
   - Decided to use JWT tokens (session:20260115)
   - Known issue: token refresh needs work
   - 2 sources: session:20260115, paper:oauth-best-practices
```

### Without Topic

```
User: /start-session

You:
1. Register source: session:20260121
2. Report:
   Session registered: session:20260121
   No topic specified - proceeding without context search
```

## Notes

- Keep search broad initially
- Report key decisions and problems
- Mention relevant sources
- If no results, say so (don't guess)
