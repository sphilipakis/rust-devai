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

## Important Lua considerations and best practices

- The scripting model is designed to return data from one stage to another (for example, returning data in the `# Before All`, `# Data`, or `# Output` stages).
- However, it is not possible to return functions from one stage to another.
- As a best practice, the `# Before All` stage, when needing to operate on multiple files, should only get the list of files and not load them. The `# Data` stage should handle loading the files to allow effective concurrency when set.
- The `# Before All` stage is a good place to prepare common files and return common paths.
- It is possible to use Lua `require` to include files located in the `lua/` folder of the agent file (the `.aip` file).

## CTX

All Lua scripts get the `CTX` table in scope to get the path of the runtime and agent.

| Key                            | Value                                             |
|--------------------------------|---------------------------------------------------|
| CTX.WORKSPACE_DIR              | `/absolute/path/to/workspace_dir`                 |
| CTX.WORKSPACE_AIPACK_DIR       | `/absolute/path/to/workspace_dir/.aipack`         |
| CTX.BASE_AIPACK_DIR            | `/User/john/.aipack-base`                         |
| CTX.AGENT_NAME                 | `my-agent`                                        |
| CTX.AGENT_FILE_PATH            | `/absolute/path/to/my-agent.aip`                  |
| CTX.AGENT_FILE_DIR             | `/absolute/path/to/agent`                         |
| CTX.AGENT_FILE_NAME            | `my-agent.aip`                                    |
| CTX.AGENT_FILE_STEM            | `my-agent`                                        |
| CTX.PACK_NAMESPACE             | `demo` (when `demo@craft/text`)                   |
| CTX.PACK_NAME                  | `craft` (when `demo@craft/text`)                  |
| CTX.PACK_REF                   | `demo@craft/text`                                 |
| CTX.PACK_IDENTITY              | `demo@craft` (when `demo@craft/text`)             |
| CTX.PACK_WORKSPACE_SUPPORT_DIR | `/workspace/.aipack/support/pack/craft/text`      |
| CTX.PACK_BASE_SUPPORT_DIR      | `/User/john/.aipack-base/support/pack/craft/text` |


- All paths are absolute.
- `CTX.PACK..` are nil if the agent was not referenced with a pack path (i.e., with a "@").
- The `AGENT_NAME` is the name provided that resolves to the `AGENT_FILE_PATH`.


## aip.flow

The `aip.flow` Lua modules are special functions that allow customizing the execution flow of an agent.


### Before All possible returns

#### Returning simple value

When a `# Before All` section returns a value, it will be accessible via the `before_all` global in the other lua sections. 

For example

````md

# Before All

```lua
return {some = "value"}
```

# Data 

```lua

print("Before All .some " .. before_all.some)

```

````

and `before_all` will also be accessible in the `# Output` and `# After All` Lua code blocks. 

#### Returning aip.flow.before_all_response

The `# Before All` section can take control of the inputs and even agent options, by returning a `aip.flow.before_all_response({inputs? = {..list..}, options = {..object..}})`

For example

````md
# Before All

```lua
-- some logic

-- Here has access to `inputs` array if passed

-- `before_all` is optional, and will be accessible as `before_all` global in other sections.
-- `inputs` is optional, but must be a list if present. This allows to reshape the inputs.
-- `options` is optional, and must be a dict with optionally .model, .input_concurrency, .model_aliases (dict), and what is in the default_options of config
return aip.flow.before_all_response({
  before_all = { some = "value"}
  inputs     = ["one", "two", "three"],
  options    = {model = "o3-mini-low", input_concurrency = 2}
})

```

````