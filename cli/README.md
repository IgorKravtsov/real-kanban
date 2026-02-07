# Real Kanban CLI (`rk`)

A command-line interface for managing Kanban board tasks directly from your terminal. Link any directory to a project and create tasks without leaving your workflow.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands Reference](#commands-reference)
  - [init](#init)
  - [check](#check)
  - [projects](#projects)
  - [columns](#columns)
  - [tasks](#tasks)
  - [link](#link)
  - [unlink](#unlink)
  - [status](#status)
  - [add](#add)
  - [remove](#remove)
  - [move](#move)
  - [done](#done)
  - [describe](#describe)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Authentication](#authentication)
- [Directory Inheritance](#directory-inheritance)
- [Error Handling](#error-handling)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Installation

### From Source

Requires Rust 1.70+ and Cargo.

```bash
# Clone the repository
git clone <repository-url>
cd kanban-board/cli

# Install globally
cargo install --path .

# Verify installation
rk --help
```

### Development Build

```bash
cd cli
cargo build
./target/debug/rk --help
```

## Quick Start

```bash
# 1. Configure the CLI with your backend URL and API key
rk init http://localhost:30100 your-api-key-here

# 2. Check connection
rk check

# 3. List available projects
rk projects

# 4. Navigate to your project directory and link it
cd ~/projects/my-app
rk link 1

# 5. Create tasks from anywhere within that directory tree
rk add "Implement user authentication"
rk add "Fix navbar styling issue"

# 6. View available columns
rk columns

# 7. Create task in specific column with description
rk add "Urgent bug fix" -c "In Progress" -d "Fix the login redirect issue"
```

## Commands Reference

### init

Initialize global configuration with backend API URL and authentication key.

```
rk init <URL> <API_KEY>
rk init --url <URL>
rk init --key <API_KEY>
```

**Arguments:**
| Argument | Description | Example |
|----------|-------------|---------|
| `URL` | Backend API base URL | `http://localhost:30100` |
| `API_KEY` | Authentication key for API requests | `your-secret-api-key` |

**Options:**
| Option | Description | Example |
|--------|-------------|---------|
| `--url` | Set only the API URL (preserves existing API key) | `--url http://localhost:30100` |
| `--key` | Set only the API key (preserves existing URL) | `--key new-api-key` |

**Example:**
```bash
# Full initialization
rk init http://localhost:30100 my-super-secret-key
# Output: Configuration saved. API URL: http://localhost:30100

# Update URL only (preserves existing API key)
rk init --url http://new-backend:3001
# Output: API URL updated: http://new-backend:3001

# Update API key only (preserves existing URL)
rk init --key new-secret-key
# Output: API key updated
```

**Notes:**
- Configuration is stored globally and shared across all directories
- Re-running `init` will overwrite existing configuration
- Use `--url` to change the API URL without affecting the API key
- Use `--key` to change the API key without affecting the URL
- The API key is stored in plain text; ensure appropriate file permissions

---

### check

Verify that the CLI is configured and can connect to the backend.

```
rk check
```

**Outputs:**
- `ok` (exit 0) - Configured and can connect to backend
- `not configured` (exit 1) - Missing API URL or key
- `cannot connect` (exit 1) - Configured but backend unreachable

**Example:**
```bash
rk check
# Output: ok
```

---

### projects

List all available projects from the backend.

```
rk projects
```

**Example:**
```bash
rk projects
# Output:
# Available projects:
#   [1] My Web App
#   [2] Mobile Client
#   [3] Backend Services
```

**Notes:**
- Requires valid configuration (run `rk init` first)
- Projects are displayed with their ID (used for linking) and name

---

### columns

List all columns for the linked project. Useful for finding column names/IDs when creating tasks.

```
rk columns
```

**Example:**
```bash
rk columns
# Output:
# Columns in 'My Web App':
#   [1] Backlog
#   [2] To Do
#   [3] In Progress (default)
#   [4] Done
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Shows which column is set as default (if any)
- Column IDs and names can be used with the `add -c` option

---

### tasks

List all tasks for the linked project, grouped by column.

```
rk tasks
```

**Example:**
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
#
#   Done:
#     [3] Setup project structure
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Tasks are grouped by column
- Shows task ID and title

---

### link

Link the current working directory to a specific project.

```
rk link <PROJECT_ID> [-c <COLUMN_ID>]
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `PROJECT_ID` | Numeric ID of the project to link | Yes |
| `-c, --column` | Default column ID for new tasks | No |

**Example:**
```bash
cd ~/projects/my-web-app
rk link 1
# Output: Linked '/Users/you/projects/my-web-app' to project 'My Web App' (ID: 1)

# With specific column
rk link 1 -c 3
# Output: Linked '/Users/you/projects/my-web-app' to project 'My Web App' (ID: 1)
```

**Notes:**
- Validates that the project ID exists before linking
- Link is stored in the backend (visible in web UI)
- Subdirectories automatically inherit the parent's project link
- If no column is specified, new tasks go to the first column (typically "Backlog")

---

### unlink

Remove the project link from the current directory.

```
rk unlink
```

**Example:**
```bash
cd ~/projects/my-web-app
rk unlink
# Output: Unlinked '/Users/you/projects/my-web-app'
```

**Notes:**
- Only removes the exact directory mapping, not parent mappings
- Subdirectories will still inherit from any linked parent directory

---

### status

Display current configuration and the linked project for the current directory.

```
rk status
```

**Example:**
```bash
rk status
# Output:
# Global config:
#   API URL: http://localhost:30100
#   API Key: (set)
#
# Current directory is linked to:
#   Project: My Web App (ID: 1)
#   Default column: 3
```

**Notes:**
- Shows whether API URL and key are configured (key value is hidden)
- Fetches linked project info from the backend
- Displays inherited project link if current directory is a subdirectory of a linked directory

---

### add

Create a new task in the linked project.

```
rk add <TITLE> [-c <COLUMN>] [-d <DESCRIPTION>] [-t <TAG>]
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `TITLE` | Task title (use quotes for multi-word titles) | Yes |
| `-c, --column` | Column name or ID (e.g., 'In Progress' or '3') | No |
| `-d, --description` | Task description | No |
| `-t, --tag` | Source tag (default: 'manual') | No |

**Example:**
```bash
# Basic task (goes to default or first column)
rk add "Implement OAuth2 login flow"
# Output: Created task 'Implement OAuth2 login flow' (ID: 42) in project 'My Web App'

# Task in specific column by name (case-insensitive)
rk add "Fix critical bug" -c "In Progress"

# Task in specific column by ID
rk add "Fix critical bug" -c 3

# Task with description
rk add "Refactor auth module" -d "Split into separate files for better maintainability"

# Task with custom source tag
rk add "Auto-generated task" -t "ci"
rk add "From script" -t "automation"

# Full example with column, description, and tag
rk add "API redesign" -c "In Progress" -d "Need to restructure the endpoint handlers for v2" -t "planning"
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- If no column is specified, uses the default column (set during `link`) or the first column
- Column can be specified by name (case-insensitive matching) or by ID
- Run `rk columns` to see available columns
- Task titles and descriptions should be quoted if they contain spaces

---

### remove

Remove a task by its title.

```
rk remove <TITLE>
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `TITLE` | Task title to remove | Yes |

**Example:**
```bash
rk remove "Fix login bug"
# Output: Deleted task 'Fix login bug' (ID: 42)
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Title matching is case-insensitive
- If multiple tasks have the same title, they will be listed and you'll need to use a more specific title

---

### move

Move a task to a different column.

```
rk move <TITLE> -c <COLUMN>
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `TITLE` | Task title to move | Yes |
| `-c, --column` | Target column name or ID | Yes |

**Example:**
```bash
# Move by column name
rk move "Implement auth" -c "In Progress"
# Output: Moved task 'Implement auth' to 'In Progress'

# Move by column ID
rk move "Implement auth" -c 3
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Column can be specified by name (case-insensitive) or ID
- Run `rk columns` to see available columns

---

### done

Mark a task as done by moving it to the last column (typically "Done").

```
rk done <TITLE>
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `TITLE` | Task title to mark as done | Yes |

**Example:**
```bash
rk done "Implement OAuth"
# Output: Marked task 'Implement OAuth' as done (moved to 'Done')
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Moves the task to the last column in the project (assumes it's "Done" or equivalent)
- Shorthand for `rk move "Task" -c "Done"`

---

### describe

Append text to a task's description.

```
rk describe <TITLE> <TEXT>
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `TITLE` | Task title to update | Yes |
| `TEXT` | Text to append to the description | Yes |

**Example:**
```bash
# Add notes to a task
rk describe "Fix login bug" "Reproduced on Safari 17.2"
# Output: Updated description for task 'Fix login bug'

# Append more context
rk describe "Fix login bug" "Root cause: session cookie not set correctly"
```

**Notes:**
- Requires the current directory (or a parent) to be linked to a project
- Text is appended to the existing description with a blank line separator
- If the task has no description, the text becomes the description
- Title matching is case-insensitive

## Configuration

The CLI stores only connection configuration locally. All project/directory mappings are stored in the backend.

### Global Configuration

Stored at `~/.config/real-kanban/config.json`:

```json
{
  "api_url": "http://localhost:30100",
  "api_key": "your-secret-api-key"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `api_url` | `string` | Base URL of the Kanban backend API |
| `api_key` | `string` | Authentication key for API requests |

## Architecture

The CLI is a thin API client. **All data lives in the backend.**

```
cli/
├── Cargo.toml          # Package manifest
└── src/
    ├── main.rs         # CLI entry point and command handlers
    ├── config.rs       # Connection config management (URL + API key only)
    └── api.rs          # HTTP client for backend communication
```

### Design Principles

1. **Stateless**: CLI stores no project/task/mapping data locally
2. **Thin Client**: All business logic is in the backend
3. **API-First**: Every operation is an API call

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Command parsing (clap), dispatching to handlers, user output |
| `config.rs` | File I/O for connection config only (URL + API key) |
| `api.rs` | HTTP requests to backend, response parsing, error handling |

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | Command-line argument parsing with derive macros |
| `serde` | 1.x | Serialization/deserialization for JSON config |
| `serde_json` | 1.x | JSON parsing and formatting |
| `reqwest` | 0.12.x | HTTP client (blocking mode) |
| `dirs` | 5.x | Cross-platform config directory resolution |
| `anyhow` | 1.x | Error handling with context |
| `hostname` | 0.4.x | Get machine hostname for multi-machine identification |
| `urlencoding` | 2.x | URL encoding for query parameters |

## Authentication

The CLI uses API key authentication via the `X-API-Key` HTTP header.

```
GET /api/projects HTTP/1.1
Host: localhost:3001
X-API-Key: your-secret-api-key
```

**Security Considerations:**
- API key is stored in plain text at `~/.config/real-kanban/config.json`
- Ensure file permissions restrict access: `chmod 600 ~/.config/real-kanban/config.json`
- Do not commit config files to version control
- Consider using environment variables for CI/CD environments

## Directory Inheritance

The backend implements prefix-based directory matching, allowing subdirectories to inherit their parent's project link.

**Example:**
```bash
# Link parent directory
cd ~/projects/my-app
rk link 1

# Subdirectories inherit the link
cd ~/projects/my-app/src/components
rk status
# Output: Current directory is linked to: Project: My App (ID: 1)

cd ~/projects/my-app/tests
rk add "Add unit tests for auth module"
# Task created in 'My App' project
```

**Matching Algorithm (backend):**
```sql
WHERE ? LIKE lp.path || '%'
ORDER BY LENGTH(lp.path) DESC
```

This means:
- `/home/user/projects/app` matches `/home/user/projects/app/src`
- `/home/user/projects/app` does NOT match `/home/user/projects/app2`

## Error Handling

The CLI provides contextual error messages for common failure scenarios:

| Error | Cause | Resolution |
|-------|-------|------------|
| `API URL not configured` | Missing global config | Run `rk init <url> <api-key>` |
| `API key not configured` | Missing API key | Run `rk init <url> <api-key>` |
| `Failed to connect to API` | Backend unreachable | Check backend is running |
| `API error: 401` | Invalid API key | Verify API key with `rk status` |
| `Project with ID X not found` | Invalid project ID | Run `rk projects` to list valid IDs |
| `Column 'X' not found` | Invalid column name | Run `rk columns` to list valid columns |
| `Current directory is not linked` | No project mapping | Run `rk link <project-id>` |
| `Project has no columns` | Empty project | Create columns in the web UI |

## Examples

### Typical Workflow

```bash
# Initial setup (once)
rk init http://localhost:30100 my-api-key

# Verify connection
rk check

# Start a new project
rk projects                    # Find project ID
cd ~/code/new-feature
rk link 3                      # Link to project ID 3

# View available columns
rk columns

# Daily usage - basic tasks go to first/default column
rk add "Research OAuth providers"
rk add "Implement Google OAuth"

# Create task directly in "In Progress" column
rk add "Urgent hotfix" -c "In Progress"

# Create task with description
rk add "Write integration tests" -d "Cover auth flow and payment endpoints"

# Check current context
rk status
```

### Multi-Project Setup

```bash
# Link different directories to different projects
cd ~/projects/frontend
rk link 1

cd ~/projects/backend
rk link 2

cd ~/projects/mobile
rk link 3

# Now each directory creates tasks in its respective project
cd ~/projects/frontend && rk add "Update React to v19"
cd ~/projects/backend && rk add "Add rate limiting"
cd ~/projects/mobile && rk add "Fix iOS keyboard issue"
```

### Specifying Default Column

```bash
# Get column IDs using the columns command
rk columns

# Link with specific default column for new tasks
rk link 1 -c 3

# Now all tasks without -c flag go directly to column 3
rk add "Urgent hotfix"  # Created in column 3

# Override default with -c flag
rk add "Backlog item" -c "Backlog"  # Created in Backlog column
```

### Creating Detailed Tasks

```bash
# Task with description for context
rk add "Fix login redirect" -d "Users are redirected to 404 after OAuth callback on mobile"

# Task in specific column with full details
rk add "Implement caching" -c "In Progress" -d "Add Redis caching for API responses to reduce latency"

# Multi-line descriptions (use quotes)
rk add "Database migration" -d "Need to:
- Add new user_preferences table
- Migrate existing settings
- Update API endpoints"
```

## Troubleshooting

### "Failed to connect to API"

1. Verify backend is running: `curl http://localhost:30100/health`
2. Check URL in config: `rk status`
3. Ensure no firewall blocking the connection

### "API error: 401 Unauthorized"

1. Verify API key: `cat ~/.config/real-kanban/config.json`
2. Compare with backend's expected key
3. Re-initialize: `rk init <url> <correct-key>`

### "Current directory is not linked"

1. Check if you're in the right directory: `pwd`
2. Check status: `rk status`
3. Link current directory: `rk link <project-id>`

### Config file location

Config directory location varies by OS:
- **macOS**: `~/Library/Application Support/real-kanban/` or `~/.config/real-kanban/`
- **Linux**: `~/.config/real-kanban/`
- **Windows**: `C:\Users\<User>\AppData\Roaming\real-kanban\`

### Resetting Configuration

```bash
# Remove CLI configuration
rm -rf ~/.config/real-kanban

# Re-initialize
rk init http://localhost:30100 your-api-key
```
