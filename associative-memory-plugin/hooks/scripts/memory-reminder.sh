#!/bin/bash
# Hook: Remind Claude to check associative memory before every task

# Read input from stdin (contains prompt info)
INPUT=$(cat)

# Extract the prompt
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# Skip for simple queries or memory-related commands
if [[ "$PROMPT" =~ ^(hi|hello|thanks|yes|no|ok|sure)$ ]] || \
   [[ "$PROMPT" =~ (search|think|add_path|add_connection|get_stats|save|memory) ]]; then
    exit 0
fi

# Output reminder that gets injected into context
cat << 'EOF'
<memory-reminder>
BEFORE starting this task, use associative-memory MCP:
1. search("<relevant keywords from the task>")
2. think("<main concept>", steps=5)
Report what you found, then proceed.
</memory-reminder>
EOF

exit 0
