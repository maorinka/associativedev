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

1. **Associative Memory MCP Server** - Build from `rust-memory/` directory (see MCP Server Setup below)
2. **Claude Code** installed

## Installation

### Option 1: Temporary (Development)

```bash
cc --plugin-dir /path/to/associativedev
```

This loads the plugin for the current session only.

### Option 2: Permanent Installation

#### Step 1: Install Skills

Copy skills to your user skills directory:

```bash
# Linux/macOS
cp -r skills/* ~/.claude/skills/

# Windows (PowerShell)
Copy-Item -Recurse skills\* $env:USERPROFILE\.claude\skills\
```

#### Step 2: Install Plugin (for commands)

Create the plugin directory structure:

```bash
# Linux/macOS
mkdir -p ~/.claude/plugins/cache/user/associative-memory/0.1.0
cp -r .claude-plugin ~/.claude/plugins/cache/user/associative-memory/0.1.0/
cp -r commands ~/.claude/plugins/cache/user/associative-memory/0.1.0/

# Windows (PowerShell)
$pluginPath = "$env:USERPROFILE\.claude\plugins\cache\user\associative-memory\0.1.0"
New-Item -ItemType Directory -Force -Path $pluginPath
Copy-Item -Recurse .claude-plugin $pluginPath\
Copy-Item -Recurse commands $pluginPath\
```

#### Step 3: Register the Plugin

Add to `~/.claude/plugins/installed_plugins.json` inside the `"plugins"` object:

```json
"associative-memory@user": [
  {
    "scope": "user",
    "installPath": "C:\\Users\\YOUR_USERNAME\\.claude\\plugins\\cache\\user\\associative-memory\\0.1.0",
    "version": "0.1.0",
    "installedAt": "2026-01-23T00:00:00.000Z",
    "lastUpdated": "2026-01-23T00:00:00.000Z"
  }
]
```

> **Note:** Adjust `installPath` for your OS:
> - Windows: `C:\\Users\\USERNAME\\.claude\\plugins\\cache\\user\\associative-memory\\0.1.0`
> - Linux/macOS: `/home/USERNAME/.claude/plugins/cache/user/associative-memory/0.1.0`

#### Step 4: Restart Claude Code

Restart Claude Code for changes to take effect.

## MCP Server Setup

The commands require the associative-memory MCP server. Build from `rust-memory/`:

```bash
cd rust-memory
cargo build --release
```

Then add to `~/.claude/.mcp.json`:

```json
{
  "mcpServers": {
    "associative-memory": {
      "command": "/path/to/rust-memory/target/release/associative-memory",
      "args": ["--mcp"]
    }
  }
}
```

Or add `.mcp.json` to the plugin directory (`~/.claude/plugins/cache/user/associative-memory/0.1.0/.mcp.json`):

```json
{
  "associative-memory": {
    "command": "/path/to/rust-memory/target/release/associative-memory",
    "args": ["--mcp"]
  }
}
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
