# AIPACK Lua Application model

Here is the Lua application model and concepts.

The full Lua APIs (`aip.` module) can be found in [lua-api.md](lua-api.md)

## Convention and Flow

**aipack** injects the following modules/variables into the various script stages:

- In all Lua script stages (`# Before All`, `# Data`, `# Output`, `# After All`)
  - `aip`: The [AIPACK Lua API module](lua-api.md) - A set of utility functions and submodules for convenience, common logic, and control flow.
  - `CTX`: A table containing [contextual constants](#ctx) like paths and agent info.
    <br/>

- In the `# Before All` stage
  - `inputs`: A Lua list containing all the inputs provided to the agent run command.
    - When `-f "**/some/glob*.*"` is used, each element in `inputs` will be a [FileMeta](#filemeta) object.
    - When `-i "some string"` is used, each element will be the corresponding string.
    <br/>

- In the `# Data` stage (runs per input)
  - `input`: The individual input item being processed for this cycle.
    - If from `-f`, it's a [FileMeta](#filemeta) object.
    - If from `-i`, it's a string.
    - Can be modified by the return value of `# Before All` using `aip.flow.before_all_response`.
  - `before_all`: The data returned by the `# Before All` stage (if any, otherwise `nil`).
    <br/>

- In the `# Output` stage (runs per processed input)
  - `input`: The input item for this cycle (potentially modified by `# Data`).
  - `data`: The data returned by the `# Data` script for this input (if any, otherwise `nil`).
  - `before_all`: The data returned by the `# Before All` stage (if any, otherwise `nil`).
  - `ai_response`: The [AiResponse](#airesponse) object containing the result from the AI model.
  <br/>

- In the `# After All` stage (runs once at the end)
  - `inputs`: The final list of all inputs that were processed (potentially modified by `# Before All`).
  - `outputs`: A Lua list containing the values returned by the `# Output` stage for each corresponding input. Will be `nil` for skipped inputs or if `# Output` returned nothing. The order matches the `inputs` list.
  - `before_all`: The data returned by the `# Before All` stage (if any, otherwise `nil`).

Note that Lua types in the aipack documentation are expressed in a simplified TypeScript notation as it is clear and concise.

For example:

- `options?: {starts_with: string, extrude?: "content" | "fragments", first?: number | boolean}`
- Would mean:
  - The `options` property is optional, and when present, should be a "table object" (Lua Dictionary Table).
  - `starts_with` is required and must be a string.
  - `extrude` is optional and can be either the string "content" or "fragments".
  - `first` is optional and can be a number or boolean.
- Lua types map closely: tables for objects/dictionaries, tables with sequential integer keys for lists/arrays, strings, numbers, booleans, nil.
- For functions that return multiple values, a characteristic of Lua, the return will be expressed like:
  - `some_fun(name: string, options?: {...}): table, string | nil`
    - This means that the function will return two values: first a value of type table, and second a value that is either a string or nil.

## Important Lua considerations and best practices

- The scripting model is designed to pass data between stages (e.g., return data from `# Before All` to be used in `# Data`, `# Output`, `# After All`; return data from `# Data` to be used in prompt stages and `# Output`; return data from `# Output` to be aggregated in `# After All`).
- It is **not** possible to return Lua functions or userdata between stages. Only serializable data (tables, strings, numbers, booleans, nil) can be reliably passed.
- As a best practice, when processing multiple files (via `-f`):
    - The `# Before All` stage can filter or modify the `inputs` list (list of `FileMeta` objects) but should generally avoid loading file content.
    - The `# Data` stage (which runs per file) is the best place to load the content of the *current* file (`aip.file.load(input.path)`) because this allows the `input_concurrency` setting to parallelize file loading and processing.
    - The `# Before All` stage is suitable for preparing common resources (e.g., ensuring a shared output file exists) and returning common configuration or paths via `before_all`.
- You can use Lua's `require("my-module")` function to include custom Lua code. Place your `.lua` files in the `lua/` subdirectory relative to your `.aip` agent file (e.g., `my-agent.aip` can `require("utils")` if `lua/utils.lua` exists).

## CTX

All Lua scripts get the `CTX` table in scope, providing context about the current execution environment.

| Key                      | Example Value                                                            | Description                                                       |
|--------------------------|--------------------------------------------------------------------------|-------------------------------------------------------------------|
| CTX.WORKSPACE_DIR        | `/Users/dev/my-project`                                                  | Absolute path to the workspace directory (containing `.aipack/`). |
| CTX.WORKSPACE_AIPACK_DIR | `/Users/dev/my-project/.aipack`                                          | Absolute path to the `.aipack/` directory in the workspace.       |
| CTX.BASE_AIPACK_DIR      | `/Users/dev/.aipack-base`                                                | Absolute path to the user's base AIPACK directory.                |
| CTX.AGENT_NAME           | `my_pack/my-agent` or `path/to/my-agent.aip`                             | The name or path used to invoke the agent.                        |
| CTX.AGENT_FILE_PATH      | `/Users/home/john/.aipack-base/pack/installed/acme/my_pack/my-agent.aip` | Absolute path to the resolved agent `.aip` file.                  |
| CTX.AGENT_FILE_DIR       | `/Users/home/john/.aipack-base/pack/installed/acme/my_pack`              | Absolute path to the directory containing the agent file.         |
| CTX.AGENT_FILE_NAME      | `my-agent.aip`                                                           | The base name of the my-agent file.                               |
| CTX.AGENT_FILE_STEM      | `my-agent`                                                               | The base name of the agent file without extension.                |

When running a pack. (when no packs, those will be all nil)

For `aip run acme@my_pack/my-agent`

| Key                            | Example Value                                             | Description                                                                       |
|--------------------------------|-----------------------------------------------------------|-----------------------------------------------------------------------------------|
| CTX.PACK_NAMESPACE             | `acme`                                                    | Namespace of the pack (nil if not run via pack reference).                        |
| CTX.PACK_NAME                  | `my_pack`                                                 | Name of the pack (nil if not run via pack reference).                             |
| CTX.PACK_REF                   | `acme@my_pack/my-agent`                                   | (Nil if not a pack) Full pack reference used (nil if not run via pack reference). |
| CTX.PACK_IDENTITY              | `acme@my_pack`                                            | Pack identity (namespace@name) (nil if not run via pack ref).                     |
| CTX.PACK_WORKSPACE_SUPPORT_DIR | `/Users/dev/my-project/.aipack/support/pack/acme/my_pack` | Workspace-specific support directory for this agent (if applicable).              |
| CTX.PACK_BASE_SUPPORT_DIR      | `/Users/home/john/.aipack-base/support/pack/acme/my_pack` | Base support directory for this agent (if applicable).                            |


- All paths are absolute and normalized for the OS.
- `CTX.PACK...` fields are `nil` if the agent was invoked directly via its file path rather than a pack reference (e.g., `aip run my-agent.aip`).
- The `AGENT_NAME` reflects how the agent was called, while `AGENT_FILE_PATH` is the fully resolved location.

## aip.flow

The `aip.flow` Lua module provides special functions that return structured tables to control the agent's execution flow. These should be used as the `return` value of the corresponding script stage (`# Before All` or `# Data`).

See [aip.flow documentation in lua-api.md](lua-api.md#aipflow) for detailed API signatures.

### Before All Stage Flow Control

- **`aip.flow.before_all_response({ inputs?: list, options?: table, before_all?: table })`**:
    - Returned from the `# Before All` script.
    - Allows replacing the entire list of `inputs` for the run.
    - Allows overriding agent `options` (like `model`, `input_concurrency`) for the run.
    - Allows setting the `before_all` data that will be passed to subsequent stages.
    - If you just want to pass data, simply `return { my_data = "value" }`. Use `aip.flow.before_all_response` only when modifying inputs or options.

    ````lua
    -- Example (# Before All script)
    -- Filter inputs and set a global path
    local filtered_inputs = {}
    for _, input in ipairs(inputs) do
      if input.ext == "md" then
        table.insert(filtered_inputs, input)
      end
    end
    return aip.flow.before_all_response({
      inputs = filtered_inputs,
      before_all = { output_summary = "output/summary.md" }
    })
    ````

### Data Stage Flow Control

- **`aip.flow.data_response({ data?: table, input?: any, options?: table })`**:
    - Returned from the `# Data` script (runs per input).
    - Allows setting the `data` that will be passed to the prompt and `# Output` stages for *this specific input*.
    - Allows replacing the `input` value itself for *this specific input*.
    - Allows overriding agent `options` for *this specific input*.
    - If you just want to pass data, simply `return { my_data = "value" }`. Use `aip.flow.data_response` only when modifying the input or options for this cycle.

    ````lua
    -- Example (# Data script)
    -- Load content and potentially use a stronger model for large files
    local file = aip.file.load(input.path)
    local response_data = { file_content = file.content }
    if #file.content > 10000 then
      return aip.flow.data_response({
        data = response_data,
        options = { model = "gpt-4o" } -- Use a different model for this large file
      })
    else
      return response_data -- Just return the data for prompt/output stages
    end
    ````

- **`aip.flow.skip(reason?: string)`**:
    - Returned from the `# Data` script.
    - Instructs the agent to completely skip processing this input (no AI call, no `# Output` stage).
    - The corresponding entry in the `outputs` list passed to `# After All` will be `nil`.

    ````lua
    -- Example (# Data script)
    -- Skip empty files
    if not input.path then return aip.flow.skip("Input has no path") end

    local file = aip.file.load(input.path)
    -- Skip if content is only whitespace
    if not file.content:find("%S") then
        return aip.flow.skip("File is empty or whitespace only: " .. input.path)
    end

    return { file_content = file.content } -- Proceed normally
    ````

## Common Types

These are simplified descriptions of common Lua tables used in the API. See [Common Types in lua-api.md](lua-api.md#common-types) for full details.

### FileMeta

Metadata about a file, typically provided as `input` when using the `-f` flag.

```ts
{
  path : string, // Relative path from where the glob was run
  dir: string,
  name : string, // e.g., "main.rs"
  stem : string, // e.g., "main"
  ext  : string, // e.g., "rs"
  // Optional fields like created_epoch_us, modified_epoch_us, size
}
```

### FileRecord

File metadata plus its content. Returned by `aip.file.load` and `aip.file.list_load`.

```ts
{
  // Includes all FileMeta fields
  path : string, dir: string, name : string, stem : string, ext  : string,
  created_epoch_us?: number, modified_epoch_us?: number, size?: number,

  content: string // The file content as a string
}
```

### AiResponse

The response from the AI model, passed to the `# Output` stage.

```ts
{
  content: string,     // The main text content generated by the AI model
  model_name: string,  // Identifier of the model used (e.g., "gpt-4o-mini")
  // Potentially other fields like usage statistics in the future
}
```

### WebResponse

Result of `aip.web.get` or `aip.web.post`.

```ts
{
  success: boolean,   // true if HTTP status is 2xx
  status: number,     // HTTP status code
  url: string,        // Final URL after redirects
  content: string | table, // Body (parsed as table if JSON response)
  error?: string      // Error message if success is false
}
```

### CmdResponse

Result of `aip.cmd.exec`.

```ts
{
  stdout: string,  // Captured standard output
  stderr: string,  // Captured standard error
  exit:   number   // Exit code of the command
}
```