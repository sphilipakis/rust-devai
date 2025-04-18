## KEY CONCEPT - ONE Markdown, ONE Agent, Multi Stages

The main **aipack** concept is to minimize the friction of creating and running an agent while providing maximum control over how those agents run and maximizing iteration speed to mature them quickly.

- **One Agent** == **One Markdown**
    - (i.e., `my-agent.aip`; a `.aip` file is a Markdown file with multi-stage sections described below)
- **Multi AI Providers / Models**: Supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come.
- **Lua** for Scripting.
- **Handlebars** for prompt templating.
- **Multi-Stage** process, where ALL steps are optional.

| Stage           | Language       | Description                                                                                                |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs and add command global data to scope (the "map" of the map/reduce capability).     |
| `# Data`        | **Lua**        | Gather additional data per input and return it for the next stages.                                        |
| `# System`      | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                               |
| `# Instruction` | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                               |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                        |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. Otherwise, `ai_response.content` will be output to the terminal. |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all inputs are completed.                     |


```sh
# Run the ./first-agent.aip on the README.md file
aip run ./my-agents/first-agent.aip -f "./README.md"
```

`./my-agents/my-first-agent.aip`

````md
# Data

```lua
-- This script runs before each instruction.
-- It has access to `input` and can fetch more data, returning it for the next stage.
-- Input originates from the command line (when using -f, this will be file metadata).

return {
    file = aip.file.load(input.path)
}
```

# Instruction

This is a Handlebars section where we can include the data generated above. For example:

Here is the content of the file to proofread:

```{{input.ext}}
{{data.file.content}}
```

- Correct the English in the content above.
- Do not modify it if it is grammatically correct.

# Output

```lua

-- This section allows processing of `ai_response.content`.
-- For example:
-- This example removes an optional leading markdown block. In certain cases, this might be useful.
local content = aip.md.outer_block_content_or_raw(ai_response.content)
-- For code output, ensuring a single trailing newline is good practice.
content = aip.text.ensure_single_ending_newline(content)
-- More processing....

-- input.path is the same as data.file.path, so we can use either
aip.file.save(input.path, content)

-- This string will be printed in the terminal if returned.
return "File saved."

```

````

- See [Complete Stages Description](#complete-stages-description) for more details on the stages.
- See [Lua doc](lua.md) for more details on the available Lua modules and functions.

## More Details

**aipack** is built on top of the [genai crate](https://crates.io/crates/genai) and therefore supports all major AI Providers and Models (OpenAI, Anthropic, Gemini, Ollama, Groq, Cohere).

You can customize the model and concurrency in `.aip/config.toml`.

New `.aip/` file structure with the `.aip` file extension. See [.aip/ folder structure](#aipack-folder-structure).

**TIP 1**: In VSCode or your editor, map the `*.aip` extension to `markdown` to benefit from markdown highlighting. `aipack` agent files are standard Markdown files.

**TIP 2**: Make sure to commit your changes before running this command so that overwritten files can be easily reverted.

_P.S. If possible, please refrain from publishing `aipack-custom` type crates on crates.io, as this might be more confusing than helpful. However, feel free to fork and code as you wish._

### API Keys

**aipack** uses the [genai crate](https://crates.io/crates/genai); therefore, the simplest way to provide the API keys for each provider is via environment variables in the terminal when running `aipack`.

Here are the names of the environment variables used:

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
GEMINI_API_KEY
XAI_API_KEY
DEEPSEEK_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

On macOS, this CLI uses the Mac keychain to store the key if it is not available in the environment variable. This feature will be extended to other operating systems as it becomes more robust.

## Complete Stages Description

Here is a full description of the complete flow:

- First, an agent receives zero or more inputs.
    - Inputs can be given through the command line via:
        - `-i` or `--input` to specify one input (multiple `-i`/`--input` flags can be used).
        - `-f some_glob` which creates one input per matched file, with the input structured as `{path, name, stem, ext}`.
    - Then the following stages occur (all are optional):
- **Stage 1**: `# Before All` (lua block) (optional)
    - The `lua` block has the following in scope:
        - `inputs`: A list of all inputs.
    - It can return:
        - Nothing.
        - Data that will be available as `before_all` in subsequent stages (e.g., `return { some = "data" }`).
        - Overridden or generated inputs via `return aipack::before_all_response({ inputs = {1, 2, 3} })`.
        - Both data and overridden inputs by passing `{ inputs = ..., before_all = ... }` to the `aipack::before_all_response` argument.
- **Stage 2**: `# Data` (lua block) (optional)
    - The `lua` block receives the following variables in scope:
        - `input`: The current input item from the command line and/or `# Before All` stage (or `nil` if no input).
        - `before_all`: Data returned from the `# Before All` stage (or `nil`).
    - It can return data that will be available as `data` in subsequent stages.
- **Stage 3**: `# Instruction` (handlebars template) (Note: `# System` and `# Assistant` stages also use Handlebars)
    - The content of this section is rendered via Handlebars, a templating engine, with the following variables in scope:
        - `input`: From Stage 1 or the command line.
        - `data`: From Stage 2 (or `nil` if Stage 2 did not run or returned nothing).
        - `before_all`: From Stage 1 (or `nil`).
- **Stage 4**: `# Output` (lua block) (optional)
    - The `lua` block receives the following scope:
        - `input`: From Stage 1 or the command line (or `nil`).
        - `data`: From Stage 2 (or `nil`).
        - `before_all`: From Stage 1 (or `nil`).
        - `ai_response`: Contains the AI's response (if an instruction was processed).
            - `.content`: The text content of the response.
            - `.model_name`: The name of the model used.
    - It can return data, which will be captured as the `output` for this input item.
- **Stage 5**: `# After All` (lua block) (optional)
    - The `lua` block receives the following scope:
        - `inputs`: The list of all inputs processed.
        - `outputs`: A list containing the return value from the `# Output` stage for each corresponding input (or `nil` if `# Output` did not run or returned nothing for an input).
        - `before_all`: From Stage 1 (or `nil`).
        - Note: The `inputs` and `outputs` arrays are kept in sync; `outputs[i]` corresponds to `inputs[i]`.
    - It can return data, which will be available to the caller of the `aipack` run function (e.g., `aipack::run(agent, inputs)`).

## Usage

Example: `aip run proof-rs-comments -f "./src/main.rs"`

(You can also use any glob pattern, like `-f "./src/**/*.rs"`)

- This command initializes the `.aip/defaults` folder if necessary, containing the "Command Agent Markdown" file `proof-rs-comments.aip` (see [.aip/defaults/proof-comments.aip](./_init/agents/proof-comments.aip)), and runs it using `genai` as follows:
    - `-f "./src/**/*.rs"`: The `-f` argument takes a glob pattern. An "input" record is created for each matching file, accessible in the `# Data` stage (each input will be of type [FileMeta](lua.md#filemeta)).
    - `# Data`: Contains a ```lua``` block executed for each `input`.
        - Provides Lua utility functions (`aip.*`) for tasks like listing files, loading content, etc. The results can be returned for use in subsequent stages.
    - `# Instruction` (and `# System`, `# Assistant`): These are Handlebars template sections with access to `input` and the `data` returned by the `# Data` stage.
        - The rendered content is sent to the AI.
    - `# Output`: Executes another ```lua``` block with access to `input`, `data`, `before_all`, and `ai_response` (including `ai_response.content`, the string returned by the AI).
        - This stage can save modified files or create new ones.
        - Future versions might allow queuing new `aipack` tasks.
- By default, this runs using the `gpt-4o-mini` model and looks for the `OPENAI_API_KEY` environment variable.
- It supports all AI providers compatible with the [genai crate](https://crates.io/crates/genai).
    - Environment variable names per provider: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `COHERE_API_KEY`, `GEMINI_API_KEY`, `GROQ_API_KEY`, `XAI_API_KEY`, `DEEPSEEK_API_KEY`.
    - On macOS, if an environment variable is not found, the tool may prompt to get/save the key from the keychain under the `aipack` group.

## `aip` command arguments

```sh
# Creates/updates the .aip/ settings folder (not strictly required, automatically runs on "run")
aip init

## -- Running a Pre-Installed Pack

# Executes the pre-installed `proof` AI Pack from the `demo` namespace (~/.aipack-base/pack/installed/demo/)
# on any file matching `content/**/*.md` (these files become 'inputs').
aip run demo@proof -f "content/**/*.md"

# Verbose mode prints details like the prompt sent, the AI response, and # Output return values to the console.
aip run demo@proof -f "content/**/*.md" --verbose

# Performs a dry run up to the request stage. Use with -v to print the rendered instruction.
# Does not call the AI or run the # Output stage. (Assumes `-w` for watch is omitted here)
aip run demo@proof -f "content/**/*.md" -v --dry req

# Performs a dry run including the AI call. Use with -v to print the rendered instruction and AI response.
# Does not run the # Output stage. (Assumes `-w` for watch is omitted here)
aip run demo@proof -f "content/**/*.md" -v --dry res

## -- Running the ask-aipack

# Creates a prompt file for interactive editing (press 'r' in the terminal to re-run).
aip run core@ask-aipack

## -- Running your own agent
aip run path/to/my-agent.aip

# Happy coding!
```

- `aip init`: Initializes the current directory as an `aipack` workspace by creating the `.aip/` folder. Optionally creates the `~/.aipack-base` folder with base AIPACK resource files if it doesn't exist.

- `aip run`: Runs an agent file (`.aip`) or a pre-installed pack.
    - The first argument is the agent file path or the pack name (e.g., `namespace@pack`).
    - `-f <path_or_glob>`: Specifies input files using a path or glob pattern. Creates one input per matched file. Can be used multiple times.
    - `--verbose` (`-v`): Prints detailed information to the console, including rendered prompts, AI responses, and `# Output` stage return values (if string-like).
    - `--dry req`: Performs a dry run, executing only the `# Before All`, `# Data`, and template rendering stages (`# System`, `# Instruction`, `# Assistant`). Use with `--verbose` to see the rendered prompt content. Does not call the AI.
    - `--dry res`: Performs a dry run including the AI call. It executes stages up to and including the AI interaction but skips the `# Output` and `# After All` stages. Use with `--verbose` to see the prompt sent and the AI response received.

- `aip init-base`: Updates the base resource folder at `~/.aipack-base`.

- `aip pack <folder_path>`: Creates an AI pack file (e.g., `my@pack-v0-1.0.aipack`) from the specified folder.

- `aip install <path/to/pack.aipack>`: Installs an AI pack from a local `.aipack` file.

- `aip install <url>`: Installs an AI pack from a URL (e.g., `https://cool-aipacks/my-aipack.aipack`).

- `aip install <pack_name>`: Installs a published AI pack from `aipack.ai` (e.g., `pro@coder`). Currently limited availability, planned to open later.

## `aipack` folder structure

(Updated in version `0.1.1` - migration from `0.1.0` is handled automatically on `aipack run` and `aipack init`)

- `.aip/` - The root folder for `aipack` configuration and custom files within your project.
    - `custom/` - Stores user-specific agents and templates. Files here take precedence over matching files in `.aip/default/`.
        - `command-agent/` - Custom agents (`.aip` files).
        - `new-template/` - Template(s) used by `aipack new my-new-cool-agent`.
            - `agent/` - Folder containing custom templates for command agents.
    - `default/` - Default command agents and templates provided by `aipack`. These files are only created if missing, preserving user customizations.
        - `command-agent/` - Default command agents.
        - `new-template/` - Default template(s) used by `aipack new`.
            - `agent/` - Folder containing default templates for command agents.

## Example of a Command Agent File

`.aip/default/proof-rs-comments.aip` (see [.aip/default/proof-rs-comments.aip](./_base/agents/proof-rs-comments.aip))

## Config

On `aipack run` or `aipack init`, a `.aip/config.toml` file will be created (if it doesn't exist) with the following structure:

```toml
[default_options]
# Required model identifier (any model supported by the Rust genai crate).
model = "gpt-4o-mini"

# Optional concurrency setting for processing inputs. Defaults to 1 if absent.
# Increasing this can speed up processing, especially with remote AI services.
input_concurrency = 1
```
