---
name: rk-kanban
description: Manage Kanban board tasks using the rk CLI. Use when creating tasks, linking directories to projects, checking project status, or managing Kanban workflow from the terminal.
allowed-tools: Bash(rk *)
---

# Real Kanban CLI (`rk`)

The `rk` command-line tool manages Kanban board tasks directly from the terminal. It connects to a Real Kanban backend API and allows creating, moving, and removing tasks from any directory.

## Available Commands

| Command | Description |
|---------|-------------|
| `rk check` | Check if rk is initialized and can connect to backend |
| `rk init <url> <api-key>` | Configure backend URL and API key (one-time setup) |
| `rk projects` | List all available projects |
| `rk columns` | List columns for the linked project |
| `rk tasks` | List all tasks for the linked project |
| `rk link <project-id> [-c column-id]` | Link current directory to a project |
| `rk unlink` | Remove directory-project link |
| `rk status` | Show current config and linked project |
| `rk add "<title>"` | Create a new task in the linked project |
| `rk remove "<title>"` | Remove a task by title |
| `rk move "<title>" -c <column>` | Move a task to a different column |
| `rk done "<title>"` | Mark a task as done (move to last column) |
| `rk describe "<title>" "<text>"` | Append text to a task's description |

## First Step: Check Initialization

Before using any rk commands, always check if it's initialized:

```bash
rk check
```

**Outputs:**
- `ok` (exit 0) - Configured and can connect to backend
- `not configured` (exit 1) - Missing API URL or key
- `cannot connect` (exit 1) - Configured but backend unreachable

**IMPORTANT**: If `rk check` fails, inform the user they need to run `rk init` manually. Do not attempt to initialize on their behalf.

### Linking Directories

Each directory can be linked to a specific project. Subdirectories inherit the parent's link:

```bash
cd ~/projects/my-app
rk link 1           # Link to project ID 1
rk link 1 -c 3      # Link with specific default column
```

### Creating Tasks

Once a directory is linked, create tasks with:

```bash
rk add "Implement user authentication"
rk add "Fix bug in payment flow"
```

Tasks are created in:
1. The default column specified during `rk link -c <id>`, OR
2. The first column of the project (typically "Backlog")

## Command Details

### `rk check`

```bash
rk check
# Output: "ok", "not configured", or "cannot connect"
```

- Returns exit code 0 if initialized and connected
- Returns exit code 1 if not configured or cannot connect
- Use this before any other rk command

### `rk init`

```bash
rk init <URL> <API_KEY>
```

- Stores configuration at `~/.config/real-kanban/config.json`
- Only needs to be run once per machine
- Re-running overwrites existing configuration

### `rk projects`

```bash
rk projects
# Output:
# Available projects:
#   [1] My Web App
#   [2] Mobile Client
```

- Requires valid configuration
- Shows project ID (for linking) and name

### `rk columns`

```bash
rk columns
# Output:
# Columns in project "My Web App":
#   [1] Backlog
#   [2] To Do
#   [3] In Progress
#   [4] Done
```

- Requires current directory to be linked to a project
- Shows column ID and name for the linked project
- Use column IDs or names with `rk move` command

### `rk tasks`

```bash
rk tasks
# Output:
# Tasks in 'My Web App':
#
#   Backlog:
#     [1] Implement OAuth2 login
#     [2] Add password reset flow
#
#   In Progress:
#     [5] Fix navbar styling
```

- Lists all tasks in the linked project
- Tasks are grouped by column
- Shows task ID and title
- Requires directory to be linked

### `rk link`

```bash
rk link <PROJECT_ID> [-c <COLUMN_ID>]
```

- Validates project exists before linking
- Stores link in the backend (visible in web UI header)
- Includes hostname for multi-machine identification
- Subdirectories automatically inherit parent links

### `rk unlink`

```bash
rk unlink
```

- Removes link for current directory only
- Removes from backend
- Parent directory links still apply to subdirectories

### `rk status`

```bash
rk status
# Output:
# Global config:
#   API URL: http://localhost:3001
#   API Key: (set)
#
# Current directory is linked to:
#   Project: My Web App (ID: 1)
#   Default column: 3
```

- Shows if API is configured
- Fetches linked project from backend
- Shows linked project for current directory (including inherited links)

### `rk add`

```bash
rk add "<TITLE>"
```

- Requires directory to be linked (or inherit a link)
- Creates task in default column or first column
- Returns created task ID

### `rk remove`

```bash
rk remove "<TITLE>"
```

- Removes a task by its title
- Requires directory to be linked
- Matches task title exactly

### `rk move`

```bash
rk move "<TITLE>" -c <COLUMN>
rk move "<TITLE>" --column <COLUMN>
```

- Moves a task to a different column
- `<COLUMN>` can be column name or column ID
- Requires directory to be linked

Examples:
```bash
rk move "Fix login bug" -c "In Progress"
rk move "Add tests" -c 3
```

### `rk done`

```bash
rk done "<TITLE>"
```

- Marks a task as done by moving it to the last column
- Shortcut for `rk move "<TITLE>" -c <last-column>`
- Requires directory to be linked

### `rk describe`

```bash
rk describe "<TITLE>" "<TEXT>"
```

- Appends text to a task's existing description
- If task has no description, the text becomes the description
- Text is appended with a blank line separator
- Requires directory to be linked

Examples:
```bash
rk describe "Fix login bug" "Reproduced on Safari 17.2"
rk describe "API redesign" "Added notes from meeting with team"
```

## Common Patterns

### Task Workflow

```bash
rk add "Implement feature X"      # Create task in Backlog
rk describe "Implement feature X" "Requirements from product team"  # Add context
rk move "Implement feature X" -c "In Progress"  # Start working
rk describe "Implement feature X" "Found edge case with empty inputs"  # Add notes
rk done "Implement feature X"     # Mark complete
```

### Quick Task Creation

When working on a feature, quickly add tasks:

```bash
rk add "TODO: Handle edge case for empty input"
rk add "FIXME: Memory leak in connection pool"
rk add "Add unit tests for auth module"
```

### Multi-Project Setup

Link different directories to different projects:

```bash
cd ~/projects/frontend && rk link 1
cd ~/projects/backend && rk link 2
cd ~/projects/mobile && rk link 3
```

### Check Before Adding

Always verify the current context:

```bash
rk status  # Check which project is linked
rk add "New task goes to correct project"
```

### View Available Columns

Before moving tasks, check available columns:

```bash
rk columns  # List columns for current project
rk move "My task" -c "Testing"
```

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| "API URL not configured" | Missing init | Run `rk init <url> <key>` |
| "Failed to connect to API" | Backend down | Start the backend server |
| "API error: 401" | Invalid API key | Re-run `rk init` with correct key |
| "Project with ID X not found" | Wrong ID | Run `rk projects` to list valid IDs |
| "Current directory is not linked" | No mapping | Run `rk link <project-id>` |
| "Task not found" | Wrong title | Check exact task title |

## Integration Tips

### With Git Hooks

Add tasks on commit:

```bash
# In .git/hooks/post-commit
rk add "Review: $(git log -1 --pretty=%s)"
```

### With Shell Aliases

```bash
alias task='rk add'
alias todo='rk add "TODO: $1"'
alias wip='rk move "$1" -c "In Progress"'
```

### From Scripts

```bash
#!/bin/bash
# Create tasks from a file
while IFS= read -r line; do
  rk add "$line"
done < tasks.txt
```

### Complete a Task After PR Merge

```bash
# In .git/hooks/post-merge
rk done "$(git log -1 --pretty=%s | sed 's/Merge.*: //')"
```
