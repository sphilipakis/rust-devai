# AIPACK Lua Application model

Here is the Lua application model and concepts.

The full Lua APIs (`aip.` module) can be found in [lua-apis](lua-apis)

## Convention and Flow

**aipack** injects the following modules/variables into the various script stages:

- In all Lua script stages (`# Before All`, `# Data`, `# Output`, `# After All`)
  - `aip`: The [AIPACK Lua API module](lua-apis) - A set of utility functions and submodules for convenience, common logic, and control flow.
  - `CTX`: A table containing [contextual constants](lua-apis#ctx) like paths and agent info.
    <br/>

- In the `# Before All` stage
  - `inputs`: A Lua list containing all the inputs provided to the agent run command.
    - When `-f "**/some/glob*.*"` is used, each element in `inputs` will be a [FileMeta](lua-apis#filemeta) object.
    - When `-i "some string"` is used, each element will be the corresponding string.
    <br/>

- In the `# Data` stage (runs per input)
  - `input`: The individual input item being processed for this cycle.
    - If from `-f`, it's a [FileMeta](lua-apis#filemeta) object.
    - If from `-i`, it's a string.
    - Can be modified by the return value of `# Before All` using `aip.flow.before_all_response`.
  - `before_all`: The data returned by the `# Before All` stage (if any, otherwise `nil`).
    <br/>

- In the `# Output` stage (runs per processed input)
  - `input`: The input item for this cycle (potentially modified by `# Data`).
  - `data`: The data returned by the `# Data` script for this input (if any, otherwise `nil`).
  - `before_all`: The data returned by the `# Before All` stage (if any, otherwise `nil`).
  - `ai_response`: The [AiResponse](lua-apis#ai-response) object containing the result from the AI model.
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

## aip.flow

The `aip.flow` Lua module provides special functions that return structured tables to control the agent's execution flow. These should be used as the `return` value of the corresponding script stage (`# Before All` or `# Data`).

See [aip.flow documentation in lua-apis](lua-apis#aipflow) for detailed API signatures.

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
