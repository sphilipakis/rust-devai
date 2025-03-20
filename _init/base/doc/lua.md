# AIPACK Lua Application model

Here is the Lua application model and concepts.

The full Lua APIs (`aip.` module) can be found in [lua-aip](lua-api.md)

## Convention and Flow

**aipack** injects the following modules/variables into the various script stages:

- In all scripts (`# Before All`, `# Data`, `# Output`, `# After All`)
  - [aip](lua-api.md) - A set of utility functions and submodules for convenience and common logic, as well as control flow for the agent executions.
  - [CTX](#ctx) - A set of constants mostly related to the various paths used for this execution (e.g., `CTX.AGENT_FILE_PATH`)
    <br/>

- In the `# Before All` stage
  - `inputs` - The list of inputs given to the run command.
    - When `-f "**/some/glob*.*"` is used, each input will be the matching `FileMeta` object.
    <br/>

- In the `# Data` stage
  - `input` - The individual input given from the aipack run.
    - In the case of `-f ...`, it will be the [FileMeta](#filemeta) structure for each file.
    <br/>

- In the `# Output` stage
  - `data` - Whatever is returned by the `# Data` script.
  - `ai_response` - The [AiResponse](#airesponse)
  <br/>

- In the `# After All` stage
  - `inputs` - The inputs sent or modified by `# Before All`.
  - `outputs` - The outputs returned by the `# Output` stage.
    - The same order as `inputs`, and `nil` when an item has been skipped or the output did not return anything.

Note that Lua types in the aipack documentation are expressed in a simplified TypeScript notation as it is clear and concise.

For example:

- `options?: {starts_with: string, extrude?: "content" | "fragments", first?: number | boolean}`
- Would mean:
  - The `options` property is optional, and when present, should be a "table object" (Lua Dictionary Table).
  - `starts_with` is required and can only be a string.
  - `extrude` is optional and can be either "content" or "fragments".
  - `first` is optional and can be a number or boolean.
- Obviously, all of those will map to the appropriate Lua type which has a good mapping.
- For functions that return multiple values, a characteristic of Lua, the return will be expressed like:
  - `some_fun(name: string, options?: {...}): table, string | nil`
    - This means that the function will return a value of type table, and then a string or nil.


## CTX

All Lua scripts get the `CTX` table in scope to get the path of the runtime and agent.

| Key                      | Value                                     |
|--------------------------|-------------------------------------------|
| CTX.WORKSPACE_DIR        | `/absolute/path/to/workspace_dir`         |
| CTX.WORKSPACE_AIPACK_DIR | `/absolute/path/to/workspace_dir/.aipack` |
| CTX.BASE_AIPACK_DIR      | `/absolute/path/to/home/.aipack-base`     |
| CTX.AGENT_NAME           | `my-agent`                                |
| CTX.AGENT_FILE_PATH      | `/absolute/path/to/my-agent.aip`          |
| CTX.AGENT_FILE_DIR       | `/absolute/path/to/agent`                 |
| CTX.AGENT_FILE_NAME      | `my-agent.aip`                            |
| CTX.AGENT_FILE_STEM      | `my-agent`                                |
| CTX.PACK_NAMESPACE       | `demo` (when `demo@craft/text`)           |
| CTX.PACK_NAME            | `craft` (when `demo@craft/text`)          |
| CTX.PACK_REF             | `demo@craft/text`                         |
| CTX.PACK_IDENTITY        | `demo@craft` (when `demo@craft/text`)     |

- All paths are absolute.
- `CTX.PACK..` are nil if the agent was not referenced with a pack path (i.e., with a "@").
- The `AGENT_NAME` is the name provided that resolves to the `AGENT_FILE_PATH`.


## aip.flow

The `aip.flow` Lua modules are special functions that allow customizing the execution flow of an agent.

For example,

In a `# Before All` Lua code block

```lua
-- To reshape the inputs and even the agent options with those data, and we can still return the `before_all` data if desired.
return aip.flow.before_all_response({
    before_all = "Some before all data",
    inputs = {"one", "two", "three", 4, "five"},
    options: {model: "o3-mini-high"}
})
```

In a `# Before All` or `# Data` code block, doing

```lua
-- Will stop and "skip" the current execution
aipack.skip("File already contains the documentation")
```

