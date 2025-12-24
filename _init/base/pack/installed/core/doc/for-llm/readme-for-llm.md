# AIPack Framework Overview for LLMs

AIPack is a framework for creating and running AI agents defined as multi-stage Markdown files (`.aip`). It bridges Lua scripting, Handlebars templating, and various AI providers.

## Core Concepts

- **Agent = Markdown**: One `.aip` file represents one agent.
- **Multi-Stage**: Execution is divided into optional stages (Lua or Handlebars).
- **Concurrency**: Inputs can be processed in parallel using the `input_concurrency` option.
- **Workspaces**: A directory with a `.aipack/` folder is a workspace. Global resources live in `~/.aipack-base/`.

## The .aip File Structure

An agent file consists of Markdown headings representing stages.

### Execution Stages

| Stage           | Language    | Frequency    | Scope / Purpose                                                                 |
|-----------------|-------------|--------------|---------------------------------------------------------------------------------|
| `# Before All`  | Lua         | Once         | Setup global data, filter/modify `inputs`, override `options`.                  |
| `# Data`        | Lua         | Per Input    | Gather input-specific data (e.g., file content), return `data` or `aip.flow`.   |
| `# System`      | Handlebars  | Per Input    | Render the system prompt.                                                       |
| `# Instruction` | Handlebars  | Per Input    | Render user prompt (Aliases: `# User`, `# Inst`).                               |
| `# Assistant`   | Handlebars  | Per Input    | Render assistant priming (Aliases: `# Model`, `# Jedi Trick`).                  |
| `# Output`      | Lua         | Per Response | Process `ai_response`, perform side effects (saving files), return `output`.    |
| `# After All`   | Lua         | Once         | Final processing using `inputs` and `outputs` lists.                            |

## Input Handling

- **Strings**: Provided via `-i "content"`.
- **Files**: Provided via `-f "glob/path/**/*"`. Each matched file becomes a `FileInfo` input.
- **Custom**: `# Before All` can generate or replace the `inputs` list entirely.

## Variable Injection

Stages receive specific variables in their scope:

- **All Lua Stages**: `aip` (API), `CTX` (Constants).
- **# Before All**: `inputs` (Original list).
- **# Data**: `input`, `before_all` (Return value from Before All).
- **Handlebars**: `input`, `data` (Return value from Data), `before_all`.
- **# Output**: `input`, `data`, `before_all`, `ai_response`.
- **# After All**: `inputs`, `outputs` (Aligned list of Output returns), `before_all`.

## Path Resolution Logic

AIPack uses specific path prefixes:

- `relative/path`: Relative to the workspace root.
- `~/path`: User home directory.
- `ns@pack/path`: Resolved to the installed pack directory.
- `$tmp/path`: Session-specific temporary directory.
- `ns@pack$workspace/path`: Workspace support directory for a pack.
- `ns@pack$base/path`: Global base support directory for a pack.

## CLI Execution Patterns

- **Standard Run**: `aip run agent.aip -f "src/**/*.rs"`
- **Pack Run**: `aip run namespace@pack/agent`
- **Dry Run (Render Only)**: `aip run agent.aip -f file.txt -v --dry req`
- **Dry Run (With AI, No Output)**: `aip run agent.aip -f file.txt -v --dry res`

## Configuration Files

- **Workspace**: `.aipack/config.toml` (Project-specific models, aliases, concurrency).
- **Global**: `~/.aipack-base/config.toml` (Default model, API key source).

Refer to `api-reference-for-llm.md` for the complete `aip.*` Lua API documentation.
