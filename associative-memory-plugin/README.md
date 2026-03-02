# Associative Memory Plugin

Comprehensive guide for using the associative-memory MCP to build persistent knowledge graphs across Claude Code sessions.

## Overview

This plugin teaches effective use of the associative-memory MCP system for:
- Building persistent context that survives across sessions
- Tracking provenance (who said what, when)
- Creating knowledge graphs with semantic relationships
- Following associations through Hebbian-reinforced paths

## Features

### Skills
- **memory-concepts** - Core concepts: associative memory, Hebbian learning, provenance
- **memory-workflow** - Session workflows: what to capture, when, how
- **query-patterns** - Search and recall strategies

### Commands
- `/start-session [topic]` - Begin session with context recall
- `/capture-knowledge <description>` - Add knowledge interactively
- `/recall-context <topic>` - Search and follow associations
- `/end-session` - Save and summarize

### Hook
- **UserPromptSubmit** - Reminds to check memory before tasks

## Prerequisites

1. **Associative Memory MCP Server** running
2. **Configuration** in `.mcp.json`:
   ```json
   {
     "mcpServers": {
       "associative-memory": {
         "command": "/path/to/associative-memory",
         "args": ["--mcp"]
       }
     }
   }
   ```

## Installation

### From Marketplace
```bash
# Coming soon
```

### Local Development
```bash
cc --plugin-dir /path/to/associative-memory-plugin
```

## Quick Start

1. **Start a session**:
   ```
   /start-session authentication refactor
   ```

2. **Capture as you work**:
   ```
   /capture-knowledge We decided to use JWT tokens because of stateless requirements
   ```

3. **Recall context later**:
   ```
   /recall-context authentication decisions
   ```

4. **End session**:
   ```
   /end-session
   ```

## Usage

### Skills Trigger Automatically

Ask questions like:
- "What is associative memory?" → Loads **memory-concepts**
- "How should I use memory in my workflow?" → Loads **memory-workflow**
- "How do I search for prior decisions?" → Loads **query-patterns**

### Commands Are Explicit

Run commands directly:
- `/start-session` - Check memory before starting
- `/capture-knowledge` - Add knowledge interactively
- `/recall-context` - Search and think
- `/end-session` - Save and summarize

### Hook Runs Automatically

The UserPromptSubmit hook reminds you to check memory before tasks (only in projects with associative-memory MCP configured).

## Configuration

Create `.claude/associative-memory.local.md` (optional):

```yaml
---
default_source_prefix: session
auto_save: true
reminder_enabled: true
---

# Associative Memory Settings

Customize memory behavior for this project.
```

### Settings

- `default_source_prefix` - Prefix for auto-generated source IDs (default: "session")
- `auto_save` - Auto-save after captures (default: true)
- `reminder_enabled` - Show memory reminder hook (default: true)

## Examples

See `skills/*/examples/` for:
- Capturing conversations
- Research synthesis
- Decision history tracking
- Debugging with memory

## Architecture

```
associative-memory-plugin/
├── .claude-plugin/
│   └── plugin.json
├── skills/
│   ├── memory-concepts/
│   ├── memory-workflow/
│   └── query-patterns/
├── commands/
│   ├── start-session.md
│   ├── capture-knowledge.md
│   ├── recall-context.md
│   └── end-session.md
└── hooks/
    ├── hooks.json
    └── scripts/
        └── memory-reminder.sh
```

## License

MIT
