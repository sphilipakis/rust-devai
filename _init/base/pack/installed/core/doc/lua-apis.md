# API Documentation

The `aip` top module provides a comprehensive set of functions for interacting with files, paths, text, markdown, JSON, web services, Lua value inspection, agent control flow, command execution, semantic versioning, Handlebars templating, code formatting, Git, Rust code processing, and HTML processing within the AIPACK environment.

[Getting Started Video Tutorial](https://news.aipack.ai/p/aipack-tutorial-from-hello-world)

[AIPACK Lab Repo](https://github.com/aipack-ai/aipack-lab)

#### The available submodules are:

- [`aip.file`](#aipfile): File system operations (load, save, list, append, JSON/MD/HTML handling).
- [`aip.path`](#aippath): Path manipulation and checking (split, resolve, exists, diff, parent).
- [`aip.text`](#aiptext): Text processing utilities (trim, split, split lines, replace, truncate, escape, ensure).
- [`aip.md`](#aipmd): Markdown processing (extract blocks, extract metadata).
- [`aip.json`](#aipjson): JSON parsing and stringification.
- [`aip.web`](#aipweb): HTTP requests (GET, POST), URL parsing and resolution.
- [`aip.uuid`](#aipuuid): UUID generation and conversion.
- [`aip.hash`](#aiphash): Hashing utilities (SHA256, SHA512, Blake3) with various encodings.
- [`aip.lua`](#aiplua): Some lua helpers (for now only `.dump(data)`).
- [`aip.agent`](#aipagent): Running other AIPACK agents.
- [`aip.run`](#aiprun): Run-level pin helper (attach pins to the current run).
- [`aip.task`](#aiptask): Task-level pin helper (attach pins to the current task).
- [`aip.flow`](#aipflow): Controlling agent execution flow.
- [`aip.cmd`](#aipcmd): Executing system commands.
- [`aip.semver`](#aipsemver): Semantic versioning operations.
- [`aip.rust`](#aiprust): Rust code specific processing.
- [`aip.html`](#aiphtml): HTML processing utilities.
- [`aip.git`](#aipgit): Basic Git operations.
- [`aip.hbs`](#aiphbs): Handlebars template rendering.
- [`aip.code`](#aipcode): Code commenting utilities.
- [`aip.time`](#aiptime): Time and date utilities (now, parse/format, epoch conversions).
- [`aip.shape`](#aipshape): Record shaping utilities (rows and columns, key selection/extraction).

#### File Path supported

AIPACK supports several types of file paths:

| Type                   | Example                                  | Notes                                                                                                   |
|------------------------|------------------------------------------|---------------------------------------------------------------------------------------------------------|
| Relative               | `some/file.txt`                          | Relative to the workspace directory                                                                     |
| Absolute               | `/absolute/path/file.txt`                | Absolute path (`C:/` on Windows)                                                                        |
| Pack Ref               | `my_org@my_pack/path/file.txt`           | Finds the closest pack (in custom workspace, custom base, or install base) and uses this as a directory |
| Home Tilde             | `~/path/to/file.txt`                     | User home directory; `~` is replaced by the home directory (or `/` if no home directory is found)       |
| Session TMP            | `$tmp/some/file.txt`                     | Located in `.aipack/.sessions/_uid_/` within the workspace; unique per session (until command stops)    |
| Workspace Pack Support | `my_org@my_pack$workspace/some/file.txt` | Maps to `.aipack/support/pack/my_org/my_pack/some/file.txt` in the workspace                            |
| Base Pack Support      | `my_org@my_pack$base/some/file.txt`      | Maps to `.aipack-base/support/pack/my_org/my_pack/some/file.txt` in the base directory                  |

Important notes:

- The workspace directory is the parent directory of the `.aipack/` folder. Like `.git` or `.vscode`, the `.aipack/` folder marks a directory as the workspace.
- The pack support directory suffixes `$workspace` and `$base` must appear immediately after the pack name.
- These paths can be used in globs (e.g., `aip.file.list({"~/path/to/**/*.md", "pro@rust10x/guide/**/*.md"})`).



#### Common Data Types:

- [`FileInfo`](#filemeta) (for `aip.file..`) (FileInfo + `.content`)
- [`FileRecord`](#filerecord) (for `aip.file..`)
- [`FileStats`](#filestats) (for `aip.file..`)
- [`WebResponse`](#webresponse) (for `aip.web..`)
- [`MdSection`](#mdsection) (for `aip.md..`)
- [`MdBlock`](#mdblock) (for `aip.md..`)
- [`CmdResponse`](#cmdresponse) (for `aip.cmd..`)
- [`DestOptions`](#destoptions) (for `aip.file.save_...to_...(src_path, dest))`


#### AI Response

An `ai_response` variable will be injected into the scope in the `# Output` Lua code block if an instruction was given and an AI request occurred (otherwise, it will be `nil`).

```ts
{
  // The final text response from the AI, if available.
  content?: string,
  // A formatted string capturing essential details like usage, price, model, and duration of the request, using the fields below.
  info: string,
  // e.g., `gpt-5-mini`
  model_name: string,
  // e.g., `openai`
  adapter_kind: AdapterKind,
  // Token usage details.
  usage: {
    prompt_tokens: number,
    completion_tokens: number
  },
  // The approximate price in USD, if available.
  price_usd?: number,
  // Duration in seconds (with millisecond precision).
  duration_sec: number,
  // Reasoning content, if available (e.g., from deepseek or some groq models).
  reasoning_content?: string,
}
```

#### Global and Injected Variables:

- All stage Lua code blocks and required scripts receive a [`CTX`](#ctx) variable containing context information (e.g., `CTX.AGENT_NAME`, `CTX.TMP_DIR`, etc.).
- All stage Lua code blocks also receive `options`, which includes `.model` and `.input_concurrency`.
- `# Before All` stage Lua code blocks receive `inputs` (can be `nil` if no inputs are given).
- `# Data` stage Lua code blocks receive `input` (can be `nil` if no input is provided) and the return value from the `# Before All` stage (`before_all`, which can be `nil`).
- `# Output` stage Lua code blocks receive `input`, `ai_response` (can be `nil`), `data` (the return value from the `# Data` stage), and `before_all`.
- `# After All` stage Lua code blocks receive `outputs` (return values from each `# Output` stage), `inputs`, and `before_all`.

**NOTE**

> All of the type documentation is noted in "TypeScript style" as it is a common and concise type notation for scripting languages and works well to express Lua types.
>       However, it is important to note that there is no TypeScript support, just standard Lua. For example, Lua properties are delimited with `=` and not `:',
>       and arrays and dictionaries are denoted with `{ }`.


#### nil vs. null

Lua has the `nil` keyword which partially acts like a common `null` but not exactly.

For that reason, AIPACK also adds the global concept of `null` that behaves closer to JSON, JS, SQL, and other nulls.

Here are the key differences:

**`nil`**

- Native to `Lua`
- Means no value or property does not exist.
    - Limitation: We have no way to know if the property had a "null" value or just was not there.
- When put in an array `{"one", "two", nil, "four"}` this will actually stop the iterator (i.e., `ipairs`) on the first nil. For example:
```lua
local values = {"one", "two", nil, "four"}
for i, v in ipairs(values) do
    print("" .. i .. ": " .. v)
end
-- Will print: "1: one", "2: two"
-- NOTE: We won't see `4: four` because the iterator stop at first nil
```

**`null`**

- Added by `AIPACK` to all Lua contexts, with keyword `null` (it can be `Null` as well, but it's better to use `null` if possible)
- Behaves like a JavaScript null, and can be used in variables, property values, and array items
- In Array
```lua
local values = {"one", "two", null, "four"}
for i, v in ipairs(values) do
    print("" .. i .. ": " .. v)
end
-- Will print: "1: one", "2: two", "3: null", "4: four"
```
- Works in objects as well, and when converting to JSON
```lua
local contact = {
    name: "Jen",
    home: null,
    phone: nil, -- NOTE the lua "nil" here
    title: "Director"
}
print(contact)
-- Will print: {home = null,name = "Jen",title = "Director"}
-- Note: "home" is present with null, but phone is not

local contact_json = aip.json.stringify(contact)
print(contact_json)
-- Will print: {"home":null,"name":"Jen","title":"Director"}
-- Note: Similar to the lua print.
```

**When to use `nil` vs. `null`**

- Use `null`
    - In arrays, use `null` over `nil` since the Lua `nil` will have some unexpected side effect, it stops iterators
    - In object property values when wanting to keep the property name when the value is null. Using `nil`, the property will be virtually "erased."
- Use `nil`
    - When using simple variable initialization, e.g., `local origin_path = nil`
    - In object property values when it's okay to not preserve the property name when it's nil. 

So, if you're still not sure, use `null` in arrays, and you can use either `nil` or `null` in other scenarios.

## aip.file

File manipulation functions for loading, saving, listing, and managing files and their content, including specialized functions for JSON and Markdown.

### Functions Summary

```lua
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord

aip.file.save(rel_path: string, content: string)

aip.file.append(rel_path: string, content: string)

aip.file.delete(path: string): boolean

aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo

aip.file.exists(path: string): boolean

aip.file.list(include_globs: string | list<string>, options?: {base_dir?: string, absolute?: boolean, with_meta?: boolean}): list<FileInfo>

aip.file.list_load(include_globs: string | list<string>, options?: {base_dir?: string, absolute?: boolean}): list<FileRecord>

aip.file.first(include_globs: string | list<string>, options?: {base_dir?: string, absolute?: boolean}): FileInfo | nil

aip.file.info(path: string): FileInfo | nil

aip.file.load_json(path: string): table | value

aip.file.load_ndjson(path: string): list<table>

aip.file.append_json_line(path: string, data: value): FileInfo

aip.file.append_json_lines(path: string, data: list): FileInfo

aip.file.save_changes(path: string, changes: string): FileInfo

aip.file.load_md_sections(path: string, headings?: string | list<string>): list<MdSection>

aip.file.load_md_split_first(path: string): {before: string, first: MdSection, after: string}

aip.file.save_html_to_md(html_path: string, dest?: string | table): FileInfo

aip.file.save_html_to_slim(html_path: string, dest?: string | table): FileInfo
aip.file.load_html_as_slim(html_path: string): string
aip.file.load_html_as_md(html_path: string, options?: table): string

aip.file.save_docx_to_md(docx_path: string, dest?: string | table): FileInfo

aip.file.load_docx_as_md(docx_path: string): string

aip.file.line_spans(path: string): list<{start: number, end: number}>

aip.file.csv_row_spans(path: string): list<{start: number, end: number}>

aip.file.read_span(path: string, start: number, end: number): string

aip.file.hash_sha256(path: string): string
aip.file.hash_sha256_b64(path: string): string
aip.file.hash_sha256_b64u(path: string): string
aip.file.hash_sha256_b58u(path: string): string

aip.file.hash_sha512(path: string): string
aip.file.hash_sha512_b64(path: string): string
aip.file.hash_sha512_b64u(path: string): string
aip.file.hash_sha512_b58u(path: string): string

aip.file.hash_blake3(path: string): string
aip.file.hash_blake3_b64(path: string): string
aip.file.hash_blake3_b64u(path: string): string
aip.file.hash_blake3_b58u(path: string): string
```


> Note: All relative paths are relative to the workspace directory (parent of `.aipack/`). Unless a `base_dir` option is specified. Pack references (e.g., `ns@pack/`) can be used in paths and `base_dir`.

### aip.file.load

Load a [FileRecord](#filerecord) object with its content.

```lua
-- API Signature
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord
```

Loads the file specified by `rel_path` and returns a [FileRecord](#filerecord) object containing the file's metadata and its content.

#### Arguments

- `rel_path: string`: The path to the file, relative to the `base_dir` or workspace root.
- `options?: table`: An optional table containing:
  - `base_dir: string` (optional): The base directory from which `rel_path` is resolved. Defaults to the workspace root. Pack references (e.g., `ns@pack/`) can be used.

#### Returns

- `FileRecord`: A [FileRecord](#filerecord) table representing the file.

#### Example

```lua
local readme = aip.file.load("doc/README.md")
print(readme.path)    -- Output: "doc/README.md" (relative path used)
print(#readme.content) -- Output: <length of content>

local agent_file = aip.file.load("agent.aip", { base_dir = "ns@pack/" })
print(agent_file.path) -- Output: "agent.aip" (relative to the resolved base_dir)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the file cannot be found, read, or metadata retrieved, or if `base_dir` is invalid.

### aip.file.save

Save string content to a file at the specified path.

```lua
-- API Signature
aip.file.save(rel_path: string, content: string)
```

Writes the `content` string to the file specified by `rel_path`. Overwrites existing files. Creates directories as needed. Restricts saving outside the workspace or shared base directory for security.

#### Arguments

- `rel_path: string`: The path relative to the workspace root.
- `content: string`: The string content to write.

#### Returns

- FileInfo: Metadata ([FileInfo](#filemeta)) about the saved file.

#### Example

```lua
aip.file.save("docs/new_feature.md", "# New Feature\n\nDetails.")
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, permission issues, path restrictions, or if no workspace context.

### aip.file.append

Append string content to a file at the specified path.

```lua
-- API Signature
aip.file.append(rel_path: string, content: string)
```

Appends `content` to the end of the file at `rel_path`. Creates the file and directories if they don't exist.

#### Arguments

- `rel_path: string`: The path relative to the workspace root.
- `content: string`: The string content to append.

#### Returns

- `FileInfo`: Metadata ([FileInfo](#filemeta)) about the file.

#### Example

```lua
aip.file.append("logs/app.log", "INFO: User logged in.\n")
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, permission issues, or I/O errors.

### aip.file.delete

Deletes a file at the specified path.

```lua
-- API Signature
aip.file.delete(path: string): boolean
```

Attempts to delete the file specified by `path`. The path is resolved relative to the workspace root.

Security:
- Deleting files is only allowed within the current workspace directory.

- Deleting files under the shared base directory (`~/.aipack-base/`) is not allowed.

#### Arguments

- `path: string`  
  The path to the file to delete, relative to the workspace root.

#### Returns

- `boolean`  
  `true` if a file was deleted, `false` if the file did not exist.

#### Example

```lua
local removed = aip.file.delete("logs/app.log")
if removed then
  print("Removed logs/app.log")
else
  print("No file to remove")
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The path attempts to delete outside the allowed workspace directory.

- The target is in the `.aipack-base` folder (always forbidden).

- The file cannot be deleted due to permissions or other I/O errors.

- The operation requires a workspace context, but none is found.

### aip.file.ensure_exists

Ensure a file exists at the given path, optionally creating it or adding content if empty.

```lua
-- API Signature
aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo
```

Checks if the file exists. If not, creates it with `content`. If it exists and `options.content_when_empty` is true and the file is empty (or whitespace only), writes `content`. Intended for files, not directories.

#### Arguments

- `path: string`: The path relative to the workspace root.
- `content?: string` (optional): Content to write. Defaults to empty string.
- `options?: table` (optional):
  - `content_when_empty?: boolean` (optional): If true, the `content` will be written to the file if the file is empty (or only contains whitespace). Defaults to `false`.

#### Returns

- `FileInfo`: Metadata ([FileInfo](#filemeta)) about the file.

#### Example

```lua
-- Create config if needed, with default content
local default_config = "-- Default Settings --\nenabled=true"
local meta = aip.file.ensure_exists("config/settings.lua", default_config)
print("Ensured file:", meta.path)

-- Ensure log file exists, don't overwrite
aip.file.ensure_exists("logs/activity.log")

-- Add placeholder only if file is currently empty
aip.file.ensure_exists("src/module.lua", "-- TODO", {content_when_empty = true})
```

#### Error

Returns an error (Lua table `{ error: string }`) on creation/write failure, permission issues, or metadata retrieval failure.

### aip.file.exists

Checks if the specified path exists (file or directory).

```lua
-- API Signature
aip.file.exists(path: string): boolean
```

Checks if the file or directory specified by `path` exists. The path is resolved relative to the workspace root.
Supports relative paths, absolute paths, and pack references (`ns@pack/...`).

#### Arguments


- `path: string`: The path string to check for existence. Can be relative, absolute, or a pack reference.

#### Returns


- `boolean`: Returns `true` if a file or directory exists at the specified path, `false` otherwise.

#### Example


```lua
if aip.file.exists("README.md") then
  print("README.md exists.")
end

if aip.file.exists("ns@pack/main.aip") then
  print("Pack main agent exists.")
end
```

#### Error


Returns an error (Lua table `{ error: string }`) if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).

```ts
{
  error: string // Error message
}
```

### aip.file.list

List file metadata ([FileInfo](#filemeta)) matching glob patterns.

```lua
-- API Signature
aip.file.list(
  include_globs: string | list<string>,
  options?: {
    base_dir?: string,
    absolute?: boolean,
    with_meta?: boolean
  }
): list<FileInfo>
```

Finds files matching `include_globs` within `base_dir` (or workspace) and returns a list of [FileInfo](#filemeta) objects (metadata only, no content).

#### Arguments

- `include_globs: string | list<string>`: Glob pattern(s). Pack references supported.
- `options?: table` (optional):
  - `base_dir?: string` (optional): Base directory for globs. Defaults to workspace. Pack refs supported.
  - `absolute?: boolean` (optional): If `true`, the `path` in the returned [FileInfo](#filemeta) objects will be absolute.
    If `false` (default), the `path` will be relative to the `base_dir`. If a path resolves outside the `base_dir`
    (e.g., using `../` in globs), it will be returned as an absolute path even if `absolute` is false.
  - `with_meta?: boolean` (optional): If `false`, the function will skip fetching detailed metadata
    (`ctime`, `mtime`, `size`) for each file, potentially improving performance
    if only the path information is needed. Defaults to `true`.
  - `ctime` is creation time, `mtime` is last modification time (from the file system), both in epoch micro

#### Returns

- `list<FileInfo>`: A Lua list of [FileInfo](#filemeta) tables. Empty if no matches.

#### Example

```lua
-- List all Markdown files in the 'docs' directory (relative paths)
local doc_files = aip.file.list("*.md", { base_dir = "docs" })
for _, file in ipairs(doc_files) do
  print(file.path) -- e.g., "guide.md", "api.md"
end

-- List all '.aip' files in a specific pack (absolute paths, no detailed meta)
local agent_files = aip.file.list("**/*.aip", {
  base_dir = "ns@pack/",
  absolute = true,
  with_meta = false
})
for _, file in ipairs(agent_files) do
  print(file.path) -- e.g., "/path/to/workspace/.aipack/ns/pack/agent1.aip"
end

-- List text and config files from the workspace root
local config_files = aip.file.list({"*.txt", "*.config"})
for _, file in ipairs(config_files) do
  print(file.path, file.size) -- e.g., "notes.txt", 1024
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, glob matching error, or metadata retrieval error (if `with_meta=true`).

### aip.file.list_load

List and load files ([FileRecord](#filerecord)) matching glob patterns.

```lua
-- API Signature
aip.file.list_load(
  include_globs: string | list<string>,
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): list<FileRecord>
```

Finds files matching `include_globs` patterns within the specified `base_dir` (or workspace root),
loads the content of each matching file, and returns a list of [FileRecord](#filerecord) objects.
Each [FileRecord](#filerecord) contains both metadata and the file content.

#### Arguments

- `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): If `true`, the paths used internally and potentially the `path` in the returned [FileRecord](#filerecord)
    objects will be absolute. If `false` (default), paths will generally be relative to the `base_dir`.
    Note: The exact path stored in [FileRecord](#filerecord).path depends on internal resolution logic, especially if paths resolve outside `base_dir`.

#### Returns

- `list<FileRecord>`: A Lua list of [FileRecord](#filerecord) tables. Empty if no files match the globs.

#### Example

```lua
-- Load all Markdown files in the 'docs' directory
local doc_files = aip.file.list_load("*.md", { base_dir = "docs" })
for _, file in ipairs(doc_files) do
  print("--- File:", file.path, "---")
  print(file.content)
end

-- Load all '.aip' files in a specific pack
local agent_files = aip.file.list_load("**/*.aip", { base_dir = "ns@pack/" })
for _, file in ipairs(agent_files) do
  print("Agent Name:", file.stem)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, glob matching error, or file read/metadata error.

### aip.file.first

Find the first file matching glob patterns and return its metadata ([FileInfo](#filemeta)).

```lua
-- API Signature
aip.file.first(
  include_globs: string | list<string>,
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): FileInfo | nil
```

Searches for files matching the `include_globs` patterns within the specified `base_dir` (or workspace root).
It stops searching as soon as the first matching file is found and returns its [FileInfo](#filemeta) object (metadata only, no content).
If no matching file is found, it returns `nil`.

#### Arguments

- `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): If `true`, the `path` in the returned [FileInfo](#filemeta) object (if found) will be absolute.
    If `false` (default), the `path` will be relative to the `base_dir`. Similar to `aip.file.list`, paths outside `base_dir` become absolute.

#### Returns

- `FileInfo | nil`: If a matching file is found, returns a [FileInfo](#filemeta) table. If no matching file is found, returns `nil`.

#### Example

```lua
-- Find the first '.aip' file in a pack
local agent_meta = aip.file.first("*.aip", { base_dir = "ns@pack/" })
if agent_meta then
  print("Found agent:", agent_meta.path)
  -- To load its content:
  -- local agent_file = aip.file.load(agent_meta.path, { base_dir = "ns@pack/" })
  -- print(agent_file.content)
else
  print("No agent file found in pack.")
end

-- Find any config file in the root
local config_meta = aip.file.first({"*.toml", "*.yaml", "*.json"}, { base_dir = "." })
if config_meta then
  print("Config file:", config_meta.name)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, error during search *before* first match, or metadata retrieval error for the first match.

### aip.file.info

Retrieves file metadata ([`FileInfo`](#filemeta)) for the specified path.

```lua
-- API Signature
aip.file.info(path: string): FileInfo | nil
```

If the given `path` exists, this function returns a [`FileInfo`](#filemeta) object
containing the file metadata (no content).  
If the path cannot be resolved or the file does not exist, it returns `nil`.

#### Arguments

- `path: string` – The file or directory path. Can be relative, absolute,
  or use pack references (`ns@pack/...`, `ns@pack$workspace/...`, etc.).

#### Returns

- `FileInfo | nil`: Metadata for the file, or `nil` when not found.

#### Example

```lua
local meta = aip.file.info("README.md")
if meta then
  print("Size:", meta.size)
end
```

#### Error

Returns an error only if the path cannot be resolved (invalid pack
reference, invalid format, …). If the path resolves successfully but the
file does not exist, the function simply returns `nil`.

### aip.file.stats

Calculates aggregate statistics for a set of files matching glob patterns.

```lua
-- API Signature
aip.file.stats(
  include_globs: string | list<string> | nil,
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): FileStats | nil
```

Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root)
and returns aggregate statistics about these files in a `FileStats` object.
If `include_globs` is `nil` or no files match the patterns, returns `nil`.

#### Arguments

- `include_globs: string | list<string> | nil` - A single glob pattern string, a Lua list (table) of glob pattern strings, or `nil`.
  If `nil`, the function returns `nil`.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): Affects how files are resolved internally, but the statistics remain the same regardless.

#### Returns

- `FileStats`: A `FileStats` object containing aggregate statistics about the matching files.
- `nil` if `include_globs` is `nil`

If no files if ound a FileStats will all 0 will be returned.

#### Example

```lua
-- Get statistics for all Markdown files in the 'docs' directory
local stats = aip.file.stats("*.md", { base_dir = "docs" })
if stats then
  print("Number of files:", stats.number_of_files)
  print("Total size:", stats.total_size)
  print("First created:", stats.ctime_first)
  print("Last modified:", stats.mtime_last)
end

-- Get statistics for all '.aip' files in a specific pack
local agent_stats = aip.file.stats("**/*.aip", { base_dir = "ns@pack/" })
if agent_stats then
  print("Total agent files:", agent_stats.number_of_files)
end

-- Nil globs return nil
local nil_stats = aip.file.stats(nil)
print(nil_stats) -- Output: nil
```

#### Error

Returns an error if:
- `include_globs` is not a string, a list of strings, or `nil`.
- `base_dir` cannot be resolved (e.g., invalid pack reference).
- An error occurs during file system traversal or glob matching.

### aip.file.load_json

Load a file, parse its content as JSON, and return the corresponding Lua value.

```lua
-- API Signature
aip.file.load_json(path: string): table | value
```

Loads the file at `path` (relative to workspace), parses it as JSON, and converts it to a Lua value.

#### Arguments

- `path: string`: Path to the JSON file, relative to workspace root.

#### Returns

- `table | value`: Lua value representing the parsed JSON.

#### Example

```lua
-- Assuming 'config.json' contains {"port": 8080, "enabled": true}
local config = aip.file.load_json("config.json")
print(config.port) -- Output: 8080
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, content is invalid JSON, or conversion fails.

### aip.file.load_ndjson

Load a file containing newline-delimited JSON (NDJSON), parse each line, and return a Lua list (table) of the results.

```lua
-- API Signature
aip.file.load_ndjson(path: string): list<table>
```

Loads the file at `path` (relative to workspace), parses each non-empty line as JSON, and returns a Lua list of the parsed values. Empty lines are skipped.

#### Arguments

- `path: string`: Path to the NDJSON file, relative to workspace root.

#### Returns

- `list<table>`: Lua list containing parsed values from each line.

#### Example

```lua
-- Assuming 'logs.ndjson' contains:
-- {"level": "info", "msg": "Started"}
-- {"level": "warn", "msg": "Low space"}
local logs = aip.file.load_ndjson("logs.ndjson")
print(#logs) -- Output: 2
print(logs[1].msg) -- Output: Started
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, any line has invalid JSON, or conversion fails.

### aip.file.append_json_line

Convert a Lua value to a JSON string and append it as a new line to a file.

```lua
-- API Signature
aip.file.append_json_line(path: string, data: value)
```

Converts `data` to JSON and appends it, followed by a newline (`\n`), to the file at `path` (relative to workspace). Creates file/directories if needed.

#### Arguments

- `path: string`: Path to the target file, relative to workspace root.
- `data: value`: Lua data (table, string, number, boolean, nil) to append.

#### Returns

- `FileInfo`: Metadata ([FileInfo](#filemeta)) about the file.

#### Example

```lua
aip.file.append_json_line("output.ndjson", {user = "test", score = 100})
-- Appends '{"score":100,"user":"test"}\n' to output.ndjson
```

#### Error

Returns an error (Lua table `{ error: string }`) on conversion/serialization failure, directory creation failure, or file write/permission error.

### aip.file.append_json_lines

Convert a Lua list (table) of values to JSON strings and append them as new lines to a file.

```lua
-- API Signature
aip.file.append_json_lines(path: string, data: list)
```

Iterates through the `data` list, converts each element to JSON, and appends it followed by a newline (`\n`) to the file at `path` (relative to workspace). Creates file/directories if needed. Uses buffering.

#### Arguments

- `path: string`: Path to the target file, relative to workspace root.
- `data: list`: Lua list (table with sequential integer keys from 1) of values to append.

#### Returns

Does not return anything upon success.

#### Example

```lua
local users = { {user = "alice"}, {user = "bob"} }
aip.file.append_json_lines("users.ndjson", users)
-- Appends '{"user":"alice"}\n{"user":"bob"}\n'
```

#### Error

Returns an error (Lua table `{ error: string }`) if `data` is not a list, conversion/serialization fails for any element, directory creation fails, or file write/permission error.

### aip.file.load_md_sections

Load markdown sections from a file, optionally filtering by specific heading names.

```lua
-- API Signature
aip.file.load_md_sections(
  path: string,
  headings?: string | list<string>
): list<MdSection>
```

Reads the markdown file at `path` (relative to workspace) and splits it into sections based on headings (`#`). Returns a list of [MdSection](#mdsection) objects. Optionally filters by exact heading `name` (case-sensitive, excluding `#`).

#### Arguments

- `path: string`: Path to the markdown file, relative to workspace root.
- `headings?: string | list<string>` (optional): Heading name(s) to filter by.

#### Returns

- `list<MdSection>`: A Lua list of [MdSection](#mdsection) tables. Includes content before the first heading if no filter applied. Empty if file empty or no matching sections.

#### Example

```lua
-- Load all sections
local all_sections = aip.file.load_md_sections("doc/readme.md")

-- Load only the "Summary" section
local summary_section = aip.file.load_md_sections("doc/readme.md", "Summary")

-- Load "Summary" and "Conclusion" sections
local sections = aip.file.load_md_sections("doc/readme.md", {"Summary", "Conclusion"})
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, `headings` invalid, or parsing/conversion error.

### aip.file.load_md_split_first

Splits a markdown file into three parts based on the *first* heading encountered.

```lua
-- API Signature
aip.file.load_md_split_first(path: string): {before: string, first: MdSection, after: string}
```

Reads the file at `path` (relative to workspace) and divides it into: content before the first heading (`before`), the first heading section (`first`), and content from the second heading onwards (`after`).

#### Arguments

- `path: string`: Path to the markdown file, relative to workspace root.

#### Returns

- `table`: A table containing the three parts:
  ```ts
  {
    before: string,       // Content before first heading.
    first: MdSection,     // The first [MdSection](#mdsection) (default if no headings).
    after: string         // Content from second heading onwards.
  }
  ```

#### Example

```lua
local split = aip.file.load_md_split_first("doc/structure.md")
print("--- BEFORE ---")
print(split.before)
print("--- FIRST Heading Name ---")
print(split.first.heading.name)
print("--- AFTER ---")
print(split.after)
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, or parsing/conversion error.

### aip.file.save_html_to_md

Loads an HTML file, converts its content to Markdown, and saves it.

```lua
-- API Signature
aip.file.save_html_to_md(
  html_path: string,
  dest?: string | table
): FileInfo
```

Loads the HTML file at `html_path` (relative to workspace), converts its content to Markdown, and saves the result. The destination can be specified as a string path or a table of options ([DestOptions](#destoptions)).

#### Arguments


- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table:

  - `string`
    Path to save the `.md` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination.
      - `file_name?: string`: Custom file name for the Markdown output.
      - `suffix?: string`: Suffix appended to the source file stem before `.md`.

#### Returns


- `FileInfo`
  Metadata ([FileInfo](#filemeta)) about the created Markdown file.

#### Example


```lua
-- Default (replaces .html with .md):
aip.file.save_html_to_md("docs/page.html")
-- Result: docs/page.md

-- Using a custom string path:
aip.file.save_html_to_md("docs/page.html", "out/custom.md")

-- Using options table:
aip.file.save_html_to_md("docs/page.html", {
  base_dir = "output",
  suffix = "_v2",
})
-- Assuming source was 'docs/page.html', result might be 'output/page_v2.md'
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, parsing, conversion, or destination resolution fails.

### aip.file.save_html_to_slim

Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.), and saves the slimmed HTML.

```lua
-- API Signature
aip.file.save_html_to_slim(
  html_path: string,
  dest?: string | table
): FileInfo
```

Loads the HTML file at `html_path` (relative to workspace), removes non-content elements, and saves the cleaned HTML. The destination can be specified as a string path or a table of options ([DestOptions](#destoptions)).

#### Arguments


- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table ([DestOptions](#destoptions)) for the output `.html` file:

  - `nil`: Saves as `[original_name]-slim.html` in the same directory.
  - `string`: Path to save the slimmed `.html` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination. If provided without `file_name` or `suffix`, the output will be `[original_name].html` in this directory.
      - `file_name?: string`: Custom file name for the slimmed HTML output.
      - `suffix?: string`: Suffix appended to the source file stem (e.g., `_slimmed`).
      - `slim?: boolean`: If true (default), slims HTML before saving; if false, saves the original HTML content without slimming.

#### Returns


- `FileInfo`
  Metadata ([FileInfo](#filemeta)) about the created slimmed HTML file.

#### Example


```lua
-- Default (saves as original-slim.html):
aip.file.save_html_to_slim("web/page.html")
-- Result: web/page-slim.html

-- Using a custom string path:
aip.file.save_html_to_slim("web/page.html", "output/slim_page.html")

-- Using options table (base_dir, uses original name):
aip.file.save_html_to_slim("web/page.html", { base_dir = "slim_output" })
-- Assuming source was 'web/page.html', result might be 'slim_output/page.html'

-- Using options table (suffix):
aip.file.save_html_to_slim("web/page.html", { suffix = "_light" })
-- Assuming source was 'web/page.html', result might be 'web/page_light.html'

-- Using options table (no slimming):
aip.file.save_html_to_slim("web/page.html", { slim = false })
-- Result: web/page.html (original content saved)
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, slimming, or destination resolution fails.


### aip.file.load_html_as_slim

Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.), and returns the slimmed HTML string.

```lua
-- API Signature
aip.file.load_html_as_slim(html_path: string): string
```

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

#### Returns

- `string`
  The slimmed HTML content.

#### Example

```lua
local slim = aip.file.load_html_as_slim("web/page.html")
-- For example, ensure scripts are removed:
print(string.find(slim, "<script") == nil)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the HTML file cannot be found/read or if slimming fails.


### aip.file.load_html_as_md

Loads an HTML file, optionally "trims" (slims) its content, converts it to Markdown, and returns the Markdown string.

```lua
-- API Signature
aip.file.load_html_as_md(
  html_path: string,
  options?: { trim?: boolean } -- default true. When true, slim HTML before converting to Markdown.
): string
```

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `options?: table`
  - `trim?: boolean` (default: true)
    When `true`, trims/slims the HTML (removes scripts, styles, comments, etc.) before conversion.
    Note: For compatibility, `slim` can also be used instead of `trim`.

#### Returns

- `string`
  The Markdown content converted from the (optionally slimmed) HTML.

#### Example

```lua
-- Default (slims first, then converts)
local md1 = aip.file.load_html_as_md("docs/page.html")

-- No slimming before conversion
local md2 = aip.file.load_html_as_md("docs/page.html", { trim = false })
```

#### Error

Returns an error (Lua table `{ error: string }`) if the HTML file cannot be found/read, if slimming fails (when enabled), or if the conversion to Markdown fails.


### aip.file.save_docx_to_md

Loads a DOCX file, converts its content to Markdown, and saves it.

```lua
-- API Signature
aip.file.save_docx_to_md(
  docx_path: string,
  dest?: string | table
): FileInfo
```

#### Arguments


- `docx_path: string`
  Path to the source DOCX file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table ([DestOptions](#destoptions)):

  - `string`
    Path to save the `.md` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination.
      - `file_name?: string`: Custom file name for the Markdown output.
      - `suffix?: string`: Suffix appended to the source file stem before `.md`.

#### Returns


- `FileInfo`
  Metadata ([FileInfo](#filemeta)) about the created Markdown file.

#### Example


```lua
-- Default (replaces .docx with .md):
aip.file.save_docx_to_md("docs/spec.docx")
-- Result: docs/spec.md

-- Using a custom string path:
aip.file.save_docx_to_md("docs/spec.docx", "out/spec.md")

-- Using options table:
aip.file.save_docx_to_md("docs/spec.docx", {
  base_dir = "output",
  suffix = "_v2",
})
-- Assuming source was 'docs/spec.docx', result might be 'output/spec_v2.md'
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, parsing/conversion, or destination resolution fails.


### aip.file.line_spans

Returns the byte spans for each line in a text file.

```lua
-- API Signature
aip.file.line_spans(path: string): list<{start: number, end: number}>
```

Given a file path, computes the start and end byte offsets for every line.

#### Arguments

- `path: string`
  Path to the source file (relative, absolute, or pack-ref supported).

#### Returns

- `list<{start: number, end: number}>`
  A Lua list of tables with `start` and `end` byte offsets for each line.

#### Example

```lua
local spans = aip.file.line_spans("logs/app.log")
for i, s in ipairs(spans) do
  print(i, s.start, s.end)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved or the file cannot be read.


### aip.file.csv_row_spans

Returns the byte spans for each CSV row in a file.

```lua
-- API Signature
aip.file.csv_row_spans(path: string): list<{start: number, end: number}>
```

Parses the file as CSV and returns byte spans for each row (one span per CSV record).

#### Arguments

- `path: string`
  Path to the CSV file (relative, absolute, or pack-ref supported).

#### Returns

- `list<{start: number, end: number}>`
  A Lua list of tables with `start` and `end` byte offsets for each CSV row.

#### Example

```lua
local rows = aip.file.csv_row_spans("data/sample.csv")
-- Read the first row bytes as text:
if #rows > 0 then
  local first_row = rows[1]
  local text = aip.file.read_span("data/sample.csv", first_row.start, first_row.end)
  print(text)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved or the file cannot be read/parsed as CSV.


### aip.file.read_span

Reads and returns the file substring between the given byte offsets.

```lua
-- API Signature
aip.file.read_span(path: string, start: number, end: number): string
```

#### Arguments

- `path: string`
  Path to the source file (relative, absolute, or pack-ref supported).

- `start: number`
  Start byte offset (non-negative).

- `end: number`
  End byte offset (non-negative, must be greater than or equal to `start`).

#### Returns

- `string`
  The substring of the file content between the given byte offsets.

#### Example

```lua
local spans = aip.file.line_spans("README.md")
-- Print the first line by span:
if #spans > 0 then
  local s = spans[1]
  local line = aip.file.read_span("README.md", s.start, s.end)
  print(line)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `start`/`end` are invalid, the path cannot be resolved, or the file cannot be read.


## aip.path

Functions for path manipulation, checking, and resolution within the AIPACK workspace.

### Functions Summary

```lua
aip.path.split(path: string): (parent: string, filename: string)

aip.path.resolve(path: string): string

aip.path.exists(path: string): boolean

aip.path.is_file(path: string): boolean

aip.path.is_dir(path: string): boolean

aip.path.diff(file_path: string, base_path: string): string

aip.path.parent(path: string): string | nil

aip.path.matches_glob(path: string | nil, globs: string | list<string>): boolean | nil

aip.path.join(base: string, ...parts: string | string[]): string

aip.path.parse(path: string | nil): table | nil
```

> Note: Paths are typically relative to the workspace directory unless otherwise specified or resolved using pack references.

### aip.path.parse

Parses a path string and returns a [FileInfo](#filemeta) table representation of its components.

```lua
-- API Signature
aip.path.parse(path: string | nil): table | nil
```

Parses the given path string into a structured table containing components like `dir`, `name`, `stem`, `ext`, etc., without checking file existence or metadata.

#### Arguments


- `path: string | nil`: The path string to parse. If `nil`, the function returns `nil`.

#### Returns


- `table | nil`: A [FileInfo](#filemeta) table representing the parsed path components if `path` is a string. Returns `nil` if the input `path` was `nil`. Note that `ctime`, `mtime`, and `size` fields will be `nil` as this function only parses the string, it does not access the filesystem.

#### Example


```lua
local parsed = aip.path.parse("some/folder/file.txt")
-- parsed will be similar to { path = "some/folder/file.txt", dir = "some/folder", name = "file.txt", stem = "file", ext = "txt", ctime = nil, ... }
print(parsed.name) -- Output: "file.txt"

local nil_result = aip.path.parse(nil)
-- nil_result will be nil
```

#### Error


Returns an error (Lua table `{ error: string }`) if the path string is provided but is invalid and cannot be parsed into a valid SPath object.

```ts
{
  error: string // Error message
}
```

### aip.path.split

Split path into parent directory and filename.

```lua
-- API Signature
aip.path.split(path: string): (parent: string, filename: string)
```

Splits the given path into its parent directory and filename components.

#### Arguments

- `path: string`: The path to split.

#### Returns

- `parent: string`: The parent directory path (empty string if no parent).
- `filename: string`: The filename component (empty string if no filename).

#### Example

```lua
local parent, filename = aip.path.split("folder/file.txt")
print(parent)   -- Output: "folder"
print(filename) -- Output: "file.txt"

local parent, filename = aip.path.split("justafile.md")
print(parent)   -- Output: ""
print(filename) -- Output: "justafile.md"
```

#### Error

Does not typically error.

### aip.path.join

Joins a base path with one or more path segments.

```lua
-- API Signature
aip.path.join(base: string, ...parts: string | string[]): string
```

Constructs a new path by appending processed segments from `...parts` to the `base` path.
Each argument in `...parts` is first converted to a string:
- String arguments are used as-is.
- List (table) arguments have their string items joined by `/`.
These resulting strings are then concatenated together. Finally, this single concatenated string
is joined with `base` using system-appropriate path logic (which also normalizes separators like `//` to `/`).


#### Arguments

- `base: string`: The initial base path.
- `...parts: string | string[]` (variadic): One or more path segments to process and append. Each part can be a single string or a Lua list (table) of strings.

#### Returns

- `string`: A new string representing the combined and normalized path.

#### Example

```lua
-- Example 1: Basic join
print(aip.path.join("dir1/", "file1.txt"))             -- Output: "dir1/file1.txt"
print(aip.path.join("dir1", "file1.txt"))              -- Output: "dir1/file1.txt"

-- Example 2: Joining with a list (table)
print(aip.path.join("dir1/", {"subdir", "file2.txt"})) -- Output: "dir1/subdir/file2.txt"

-- Example 3: Multiple string arguments
-- Segments are concatenated, then joined to base.
print(aip.path.join("dir1/", "subdir/", "file3.txt"))  -- Output: "dir1/subdir/file3.txt"
print(aip.path.join("dir1/", "subdir", "file3.txt"))   -- Output: "dir1/subdirfile3.txt"

-- Example 4: Mixed arguments (strings and lists)
-- Lists are pre-joined with '/', then all resulting strings are concatenated, then joined to base.
print(aip.path.join("root/", {"user", "docs"}, "projectA", {"report", "final.pdf"}))
-- Output: "root/user/docsprojectAreport/final.pdf"

-- Example 5: Normalization
print(aip.path.join("", {"my-dir//", "///file.txt"}))  -- Output: "my-dir/file.txt"
print(aip.path.join("a", "b", "c"))                     -- Output: "a/bc"
print(aip.path.join("a/", "b/", "c/"))                  -- Output: "a/b/c/"
```

#### Error

Returns an error (Lua table `{ error: string }`) if any of the `parts` arguments cannot be converted to a string or a list of strings (e.g., passing a boolean or a function).


### aip.path.resolve

Resolves and normalizes a path relative to the workspace or pack structure.

```lua
-- API Signature
aip.path.resolve(path: string): string
```

Resolves relative paths (`.`, `..`), absolute paths, and pack references (`ns@pack/`, `ns@pack$base/`, `ns@pack$workspace/`) to a normalized, typically absolute, path.

#### Arguments

- `path: string`: The path string to resolve.

#### Returns

- `string`: The resolved and normalized path.

#### Example

```lua
local resolved_path = aip.path.resolve("./agent-script/../agent.aip")
-- Output: /path/to/workspace/agent.aip (example)

local resolved_pack_path = aip.path.resolve("ns@pack/some/file.txt")
-- Output: /path/to/aipack-base/packs/ns/pack/some/file.txt (example)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved (e.g., invalid pack reference, invalid format).

### aip.path.exists

Checks if the specified path exists (file or directory).

```lua
-- API Signature
aip.path.exists(path: string): boolean
```

Checks existence after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if exists, `false` otherwise.

#### Example

```lua
if aip.path.exists("README.md") then print("Exists") end
if aip.path.exists("ns@pack/main.aip") then print("Pack agent exists") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.is_file

Checks if the specified path points to an existing file.

```lua
-- API Signature
aip.path.is_file(path: string): boolean
```

Checks after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if it's an existing file, `false` otherwise.

#### Example

```lua
if aip.path.is_file("config.toml") then print("Is a file") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.is_dir

Checks if the specified path points to an existing directory.

```lua
-- API Signature
aip.path.is_dir(path: string): boolean
```

Checks after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if it's an existing directory, `false` otherwise.

#### Example

```lua
if aip.path.is_dir("src/") then print("Is a directory") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.diff

Computes the relative path from `base_path` to `file_path`.

```lua
-- API Signature
aip.path.diff(file_path: string, base_path: string): string
```

Calculates the relative path string that navigates from `base_path` to `file_path`.

#### Arguments

- `file_path: string`: The target path.
- `base_path: string`: The starting path.

#### Returns

- `string`: The relative path. Empty if paths are the same or cannot be computed (e.g., different drives).

#### Example

```lua
print(aip.path.diff("/a/b/c/file.txt", "/a/b/")) -- Output: "c/file.txt"
print(aip.path.diff("/a/b/", "/a/b/c/file.txt")) -- Output: "../.."
print(aip.path.diff("folder/file.txt", "folder")) -- Output: "file.txt"
```

#### Error

Returns an error (Lua table `{ error: string }`) if paths are invalid.

### aip.path.parent

Returns the parent directory path of the specified path.

```lua
-- API Signature
aip.path.parent(path: string): string | nil
```

Gets the parent directory component.

#### Arguments

- `path: string`: The path string.

#### Returns

- `string | nil`: The parent directory path, or `nil` if no parent (e.g., for ".", "/", "C:/").

#### Example

```lua
print(aip.path.parent("some/path/file.txt")) -- Output: "some/path"
print(aip.path.parent("."))                  -- Output: nil
```

#### Error

Does not typically error.


### aip.path.matches_glob

Checks if a path matches one or more glob patterns.

```lua
-- API Signature
aip.path.matches_glob(path: string | nil, globs: string | list<string>): boolean | nil
```

Determines whether the provided `path` matches any of the glob patterns given
in `globs`. The function returns `nil` when `path` is `nil`.  
If `globs` is an empty string or an empty list, the result is `false`.

#### Arguments

- `path: string | nil`  
  The path to test. If `nil`, the function returns `nil`.

- `globs: string | list<string>`  
  A single glob pattern string or a Lua list of pattern strings.  
  Standard wildcards (`*`, `?`, `**`, `[]`) are supported.

#### Returns

- `boolean | nil`  
  `true`  – when the `path` matches at least one pattern.  
  `false` – when it matches none.  
  `nil`   – when the supplied `path` was `nil`.

#### Example

```lua
-- Single pattern
print(aip.path.matches_glob("src/main.rs", "**/*.rs"))            -- true

-- Multiple patterns
print(aip.path.matches_glob("README.md", {"*.md", "*.txt"}))      -- true

-- No match
print(aip.path.matches_glob("image.png", {"*.jpg", "*.gif"}))     -- false

-- Nil path
print(aip.path.matches_glob(nil, "*.rs"))                         -- nil
```

#### Error

Returns an error (Lua table `{ error: string }`) if `globs` is not a string
or a list of strings.

## aip.text

Text manipulation functions for cleaning, splitting, modifying, and extracting text content.

### Functions Summary

```lua
aip.text.escape_decode(content: string | nil): string | nil

aip.text.escape_decode_if_needed(content: string | nil): string | nil

aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.remove_first_line(content: string | nil): string | nil

aip.text.remove_first_lines(content: string | nil, n: number): string | nil

aip.text.remove_last_line(content: string | nil): string | nil

aip.text.remove_last_lines(content: string | nil, n: number): string | nil

aip.text.trim(content: string | nil): string | nil

aip.text.trim_start(content: string | nil): string | nil

aip.text.trim_end(content: string | nil): string | nil

aip.text.remove_last_lines(content: string | nil, n: number): string | nil

aip.text.truncate(content: string | nil, max_len: number, ellipsis?: string): string | nil

aip.text.replace_markers(content: string | nil, new_sections: list): string | nil

aip.text.ensure(content: string | nil, {prefix?: string, suffix?: string}): string | nil

aip.text.ensure_single_ending_newline(content: string | nil): string | nil

aip.text.format_size(bytes: integer | nil, lowest_size_unit?: "B" | "KB" | "MB" | "GB"): string | nil -- lowest_size_unit default "B"

aip.text.extract_line_blocks(content: string | nil, options: {starts_with: string, extrude?: "content", first?: number}): (list<string> | nil, string | nil)

aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
```

### aip.text.escape_decode

HTML-decodes the entire content string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.escape_decode(content: string | nil): string | nil
```

Useful for decoding responses from LLMs that might HTML-encode output.

#### Arguments

- `content: string | nil`: The content to HTML-decode. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The decoded string, or `nil` if the input `content` was `nil`.

#### Error

Returns an error (Lua table `{ error: string }`) if decoding fails (and content was not `nil`).

### aip.text.escape_decode_if_needed

Selectively HTML-decodes content if needed (currently only decodes `&lt;`). If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.escape_decode_if_needed(content: string | nil): string | nil
```

A more conservative version of `escape_decode` for cases where only specific entities need decoding.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The potentially decoded string, or `nil` if the input `content` was `nil`.

#### Error

Returns an error (Lua table `{ error: string }`) if decoding fails (and content was not `nil`).

### aip.text.split_first

Splits a string into two parts based on the first occurrence of a separator. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)
```

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The separator string.

#### Returns

- `string | nil`: The part before the first separator. `nil` if `content` was `nil` or separator not found.
- `string | nil`: The part after the first separator. `nil` if `content` was `nil` or separator not found. Empty string if separator is at the end.

#### Example

```lua
local content = "first part===second part"
local first, second = aip.text.split_first(content, "===")
-- first = "first part"
-- second = "second part"
```

#### Error

This function does not typically error.

### aip.text.split_last

Splits a string into two parts based on the last occurrence of a separator. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)
```

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The separator string.

#### Returns

- `string | nil`: The part before the last separator. `nil` if `content` was `nil` or separator not found.
- `string | nil`: The part after the last separator. `nil` if `content` was `nil` or separator not found. Empty string if separator is at the end.

#### Example

```lua
local content = "some == text == more"
local first, second = aip.text.split_last(content, "==")
-- first = "some == text "
-- second = " more"

local content = "no separator here"
local first, second = aip.text.split_last(content, "++")
-- first = "no separator here"
-- second = nil
```

#### Error

This function does not typically error.

### aip.text.remove_first_line

Removes the first line from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_first_line(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The content with the first line removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_first_lines

Removes the first `n` lines from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_first_lines(content: string | nil, n: number): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `n: number`: The number of lines to remove.

#### Returns

- `string | nil`: The content with the first `n` lines removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_last_line

Removes the last line from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_last_line(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The content with the last line removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_last_lines

Removes the last `n` lines from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_last_lines(content: string | nil, n: number): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `n: number`: The number of lines to remove.

#### Returns

- `string | nil`: The content with the last `n` lines removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim

Trims leading and trailing whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim_start

Trims leading whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim_start(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim_end

Trims trailing whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim_end(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.truncate

Truncates content to a maximum length, optionally adding an ellipsis. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.truncate(content: string | nil, max_len: number, ellipsis?: string): string | nil
```

If `content` length exceeds `max_len`, truncates and appends `ellipsis` (if provided).

#### Arguments

- `content: string | nil`: The content to truncate. If `nil`, the function returns `nil`.
- `max_len: number`: The maximum length of the result.
- `ellipsis?: string` (optional): String to append if truncated (e.g., "...").

#### Returns

- `string | nil`: The truncated string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.replace_markers

Replaces `<<START>>...<<END>>` markers in content with corresponding sections. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.replace_markers(content: string | nil, new_sections: list): string | nil
```

Replaces occurrences of `<<START>>...<<END>>` blocks sequentially with items from `new_sections`. Items in `new_sections` can be strings or tables with a `.content` field.

#### Arguments

- `content: string | nil`: The content containing `<<START>>...<<END>>` markers. If `nil`, the function returns `nil`.
- `new_sections: list`: A Lua list of strings or tables to replace the markers.

#### Returns

- `string | nil`: The string with markers replaced, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.ensure

Ensures the content starts with `prefix` and/or ends with `suffix`. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.ensure(content: string | nil, {prefix?: string, suffix?: string}): string | nil
```

Adds the prefix/suffix only if the content doesn't already start/end with it.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `options: table`: A table with optional `prefix` and `suffix` string keys.

#### Returns

- `string | nil`: The ensured string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.ensure_single_ending_newline

Ensures the content ends with exactly one newline character (`\n`). If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.ensure_single_ending_newline(content: string | nil): string | nil
```

Removes trailing whitespace and adds a single newline if needed. Returns `\n` if content is empty. Useful for code normalization.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The string ending with a single newline, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.format_size

Formats a byte count (in bytes) into a human-readable, fixed-width string (9 characters, right-aligned).  
If `bytes` is `nil`, the function returns `nil`.

Optional lowest unit size to be used (by default "B" for Bytes)

```lua
-- API Signature
aip.text.format_size(bytes: integer | nil, lowest_size_unit?: "B" | "KB" | "MB" | "GB"): string | nil
```

### Examples

```lua
aip.text.format_size(777)          -- "   777 B "
aip.text.format_size(8_777)        -- "  8.78 KB"
aip.text.format_size(5_242_880)    -- "  5.24 MB"
aip.text.format_size(nil)          -- nil
```

### aip.text.extract_line_blocks

Extracts consecutive lines starting with a specific prefix. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.extract_line_blocks(content: string | nil, options: {starts_with: string, extrude?: "content", first?: number}): (list<string> | nil, string | nil)
```

Extracts blocks of consecutive lines from `content` where each line begins with `options.starts_with`.

#### Arguments

- `content: string | nil`: The content to extract from. If `nil`, the function returns `(nil, nil)`.
- `options: table`:
  - `starts_with: string` (required): The prefix indicating a line block.
  - `extrude?: "content"` (optional): If set, returns the remaining content after extraction as the second return value.
  - `first?: number` (optional): Limits the number of blocks extracted. Remaining lines (if any) contribute to the extruded content if `extrude` is set.

#### Returns

- `list<string> | nil`: A Lua list of strings, each element being a block of consecutive lines starting with the prefix. `nil` if input `content` was `nil`.
- `string | nil`: The remaining content if `extrude = "content"`, otherwise `nil`. `nil` if input `content` was `nil`.

#### Example

```lua
local text = "> Block 1 Line 1\n> Block 1 Line 2\nSome other text\n> Block 2"
local blocks, remain = aip.text.extract_line_blocks(text, {starts_with = ">", extrude = "content"})
-- blocks = { "> Block 1 Line 1\n> Block 1 Line 2", "> Block 2" }
-- remain = "Some other text\n"
```

#### Error

Returns an error (Lua table `{ error: string }`) if arguments are invalid (and content was not `nil`).

### aip.text.split_first_line

Splits a string into two parts based on the *first* line that exactly matches the separator. If `content` is `nil`, returns `(nil, nil)`. If no line matches, returns `(original_content, nil)`.

```lua
-- API Signature
aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)
```

The separator line itself is not included in either part.

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The exact string the line must match.

#### Returns

- `string | nil`: The part before the first matching line. `nil` if `content` was `nil`. Empty string if the first matching line was the first line.
- `string | nil`: The part after the first matching line. `nil` if `content` was `nil` or no line matched `sep`. Empty string if the first matching line was the last line.

#### Example

```lua
local text = "line one\n---\nline two\n---\nline three"
local first, second = aip.text.split_first_line(text, "---")
-- first = "line one"
-- second = "line two\n---\nline three"

local first, second = aip.text.split_first_line("START\ncontent", "START")
-- first = ""
-- second = "content"

local first, second = aip.text.split_first_line("no separator", "---")
-- first = "no separator"
-- second = nil
```

#### Error

This function does not typically error.

### aip.text.split_last_line

Splits a string into two parts based on the *last* line that exactly matches the separator. If `content` is `nil`, returns `(nil, nil)`. If no line matches, returns `(original_content, nil)`.

```lua
-- API Signature
aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
```

The separator line itself is not included in either part.

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The exact string the line must match.

#### Returns

- `string | nil`: The part before the last matching line. `nil` if `content` was `nil` or no line matched `sep`.
- `string | nil`: The part after the last matching line. `nil` if `content` was `nil` or no line matched `sep`. Empty string if the last matching line was the last line.

#### Example

```lua
local text = "line one\n---\nline two\n---\nline three"
local first, second = aip.text.split_last_line(text, "---")
-- first = "line one\n---\nline two"
-- second = "line three"

local first, second = aip.text.split_last_line("content\nEND", "END")
-- first = "content"
-- second = ""

local first, second = aip.text.split_last_line("no separator", "---")
-- first = "no separator"
-- second = nil
```

#### Error

This function does not typically error.


## aip.md

Markdown processing functions for extracting structured information like code blocks and metadata.

### Functions Summary

```lua
aip.md.extract_blocks(md_content: string): list<MdBlock>

aip.md.extract_blocks(md_content: string, lang: string): list<MdBlock>

aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): (list<MdBlock>, string)

aip.md.extract_meta(md_content: string | nil): (table | nil, string | nil)

aip.md.outer_block_content_or_raw(md_content: string): string
```

### aip.md.extract_blocks

Extracts fenced code blocks ([MdBlock](#mdblock)) from markdown content.

```lua
-- API Signatures
-- Extract all blocks:
aip.md.extract_blocks(md_content: string): list<MdBlock>
-- Extract blocks by language:
aip.md.extract_blocks(md_content: string, lang: string): list<MdBlock>
-- Extract blocks and remaining content:
aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): (list<MdBlock>, string)
```

Parses `md_content` and extracts fenced code blocks (``` ```).

#### Arguments

- `md_content: string`: The markdown content.
- `options?: string | table` (optional):
  - If string: Filter blocks by this language identifier.
  - If table:
    - `lang?: string`: Filter by language.
    - `extrude?: "content"`: If set, also return content outside the extracted blocks.

#### Returns

- If `extrude` is not set: `list<MdBlock>`: A Lua list of [MdBlock](#mdblock) objects.
- If `extrude = "content"`: `(list<MdBlock>, string)`: A tuple containing the list of [MdBlock](#mdblock) objects and the remaining content string.

#### Example

```lua
local md = "```rust\nfn main() {}\n```\nSome text.\n```lua\nprint('hi')\n```"
local rust_blocks = aip.md.extract_blocks(md, "rust")
-- rust_blocks = { { content = "fn main() {}", lang = "rust", info = "" } }

local lua_blocks, remain = aip.md.extract_blocks(md, {lang = "lua", extrude = "content"})
-- lua_blocks = { { content = "print('hi')", lang = "lua", info = "" } }
-- remain = "Some text.\n" (approx.)
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid options or parsing errors.

### aip.md.extract_meta

Extracts and merges metadata from `#!meta` TOML blocks.

```lua
-- API Signature
aip.md.extract_meta(md_content: string | nil): (table | nil, string | nil)
```

Finds all ```toml #!meta ... ``` blocks, parses their TOML content, merges them into a single Lua table, and returns the table along with the original content stripped of the meta blocks.

#### Arguments

- `md_content: string`: The markdown content.

#### Returns

- `table`: Merged metadata from all `#!meta` blocks (empty object if not found)
- `string`: Original content with meta blocks removed.

If `md_content` the return will be `(nil, nil)`

#### Example

```lua
local content = "Intro.\n```toml\n#!meta\ntitle=\"T\"\n```\nMain.\n```toml\n#!meta\nauthor=\"A\"\n```"
local meta, remain = aip.md.extract_meta(content)
-- meta = { title = "T", author = "A" }
-- remain = "Intro.\n\nMain.\n" (approx.)
```

#### Error

Returns an error (Lua table `{ error: string }`) if any meta block contains invalid TOML.

### aip.md.outer_block_content_or_raw

Extracts content from the outermost code block, or returns raw content.

```lua
-- API Signature
aip.md.outer_block_content_or_raw(md_content: string): string
```

If `md_content` starts and ends with a fenced code block (```), returns the content inside. Otherwise, returns the original `md_content`. Useful for processing LLM responses.

#### Arguments

- `md_content: string`: The markdown content.

#### Returns

- `string`: Content inside the outer block, or the original string.

#### Example

```lua
local block = "```rust\ncontent\n```"
local raw = "no block"
print(aip.md.outer_block_content_or_raw(block)) -- Output: "content\n"
print(aip.md.outer_block_content_or_raw(raw))   -- Output: "no block"
```

## aip.json

JSON parsing and stringification functions.

### Functions Summary

```lua
aip.json.parse(content: string): table | value

aip.json.parse_ndjson(content: string): list<table>

aip.json.stringify(content: table): string

aip.json.stringify_pretty(content: table): string

aip.json.stringify_to_line(content: table): string -- Deprecated alias for `stringify`
```

### aip.json.parse

Parse a JSON string into a Lua table or value.

```lua
-- API Signature
aip.json.parse(content: string): table | value
```

#### Arguments

- `content: string`: The JSON string to parse.

#### Returns

- `table | value`: A Lua value representing the parsed JSON.

#### Example

```lua
local obj = aip.json.parse('{"name": "John", "age": 30}')
print(obj.name) -- Output: John
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` is not valid JSON.

### aip.json.parse_ndjson

Parse a newline-delimited JSON (NDJSON) string into a list of tables/values.

```lua
-- API Signature
aip.json.parse_ndjson(content: string): list<table>
```

Parses each non-empty line as a separate JSON object/value.

#### Arguments

- `content: string`: The NDJSON string.

#### Returns

- `list<table>`: A Lua list containing the parsed value from each line.

#### Example

```lua
local ndjson = '{"id":1}\n{"id":2}'
local items = aip.json.parse_ndjson(ndjson)
print(items[1].id) -- Output: 1
print(items[2].id) -- Output: 2
```

#### Error

Returns an error (Lua table `{ error: string }`) if any line contains invalid JSON.

### aip.json.stringify

Stringify a Lua table/value into a compact, single-line JSON string.

```lua
-- API Signature
aip.json.stringify(content: table): string
```

#### Arguments

- `content: table`: The Lua table/value to stringify.

#### Returns

- `string`: A single-line JSON string representation.

#### Example

```lua
local obj = {name = "John", age = 30}
local json_str = aip.json.stringify(obj)
-- json_str = '{"age":30,"name":"John"}' (order may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` cannot be serialized.

### aip.json.stringify_pretty

Stringify a Lua table/value into a pretty-formatted JSON string (2-space indent).

```lua
-- API Signature
aip.json.stringify_pretty(content: table): string
```

#### Arguments

- `content: table`: The Lua table/value to stringify.

#### Returns

- `string`: A formatted JSON string with newlines and indentation.

#### Example

```lua
local obj = {name = "John", age = 30}
local json_str = aip.json.stringify_pretty(obj)
-- json_str =
-- {
--   "age": 30,
--   "name": "John"
-- } (order may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` cannot be serialized.

### aip.json.stringify_to_line (Deprecated)

Deprecated alias for `aip.json.stringify`.

```lua
-- API Signature
aip.json.stringify_to_line(content: table): string
```

## aip.web

Functions for making HTTP GET and POST requests, and for URL manipulation.

### Functions Summary

```lua
aip.web.get(url: string): WebResponse

aip.web.post(url: string, data: string | table): WebResponse

aip.web.parse_url(url: string | nil): table | nil

aip.web.resolve_href(href: string | nil, base_url: string): string | nil
```

### aip.web.get

Makes an HTTP GET request.

```lua
-- API Signature
aip.web.get(url: string): WebResponse
```

#### Arguments

- `url: string`: The URL to request.

#### Returns

- `WebResponse`: A [WebResponse](#webresponse) table containing the result.

#### Example

```lua
local response = aip.web.get("https://httpbin.org/get")
if response.success then
  print("Status:", response.status)
  -- response.content might be a string or table (if JSON)
  print("Content Type:", type(response.content))
else
  print("Error:", response.error, "Status:", response.status)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the request cannot be initiated (e.g., network error, invalid URL). Check `response.success` for HTTP-level errors (non-2xx status).

### aip.web.post

Makes an HTTP POST request.

```lua
-- API Signature
aip.web.post(url: string, data: string | table): WebResponse
```

Sends `data` in the request body. If `data` is a string, `Content-Type` is `text/plain`. If `data` is a table, it's serialized to JSON and `Content-Type` is `application/json`.

#### Arguments

- `url: string`: The URL to request.
- `data: string | table`: Data to send in the body.

#### Returns

- `WebResponse`: A [WebResponse](#webresponse) table containing the result.

#### Example

```lua
-- POST plain text
local r1 = aip.web.post("https://httpbin.org/post", "plain text data")

-- POST JSON
local r2 = aip.web.post("https://httpbin.org/post", { key = "value", num = 123 })
if r2.success and type(r2.content) == "table" then
  print("Received JSON echo:", r2.content.json.key) -- Output: value
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the request cannot be initiated or data serialization fails. Check `response.success` for HTTP-level errors.

### aip.web.parse_url

Parses a URL string and returns its components as a table.

```lua
-- API Signature
aip.web.parse_url(url: string | nil): table | nil
```

Parses the given URL string and extracts its various components.

#### Arguments


- `url: string | nil`: The URL string to parse. If `nil` is provided, the function returns `nil`.

#### Returns
 (`table | nil`)

- If the `url` is a valid string, returns a table with the following fields:
  - `scheme: string` (e.g., "http", "https")
  - `host: string | nil` (e.g., "example.com")
  - `port: number | nil` (e.g., 80, 443)
  - `path: string` (e.g., "/path/to/resource")
  - `query: table | nil` (A Lua table where keys are query parameter names and values are their corresponding string values. E.g., `{ name = "value" }`)
  - `fragment: string | nil` (The part of the URL after '#')
  - `username: string` (The username for authentication, empty string if not present)
  - `password: string | nil` (The password for authentication)
  - `url: string` (The original or normalized URL string that was parsed)
  - `page_url: string` - (The url without the query and fragment
- If the input `url` is `nil`, the function returns `nil`.

#### Example


```lua
local parsed = aip.web.parse_url("https://user:pass@example.com:8080/path/to/page.html?param1=val#fragment")
if parsed then
  print(parsed.scheme)       -- "https"
  print(parsed.host)         -- "example.com"
  print(parsed.port)         -- 8080
  print(parsed.path)         -- "/path/to/page.html"
  print(parsed.query.param1) -- "val"
  print(parsed.fragment)     -- "fragment"
  print(parsed.username)     -- "user"
  print(parsed.password)     -- "pass"
  print(parsed.url)          -- "https://user:pass@example.com:8080/path/to/page.html?query=val#fragment"
  print(parsed.page_url)     -- "https://user:pass@example.com:8080/path/to/page.html"
end

local nil_result = aip.web.parse_url(nil)
-- nil_result will be nil
```

#### Error


Returns an error (Lua table `{ error: string }`) if the `url` string is provided but is invalid and cannot be parsed.


### aip.web.resolve_href

Resolves an `href` (like one from an HTML `<a>` tag) against a `base_url`.

```lua
-- API Signature
aip.web.resolve_href(href: string | nil, base_url: string): string | nil
```

#### Arguments


- `href: string | nil`: The href string to resolve. This can be an absolute URL, a scheme-relative URL, an absolute path, or a relative path. If `nil`, the function returns `nil`.
- `base_url: string`: The base URL string against which to resolve the `href`. Must be a valid absolute URL.

#### Returns
 (`string | nil`)

- If `href` is `nil`, returns `nil`.
- If `href` is already an absolute URL (e.g., "https://example.com/page"), it's returned as is.
- Otherwise, `href` is joined with `base_url` to form an absolute URL.
- Returns the resolved absolute URL string.

#### Example


```lua
local base = "https://example.com/docs/path/"

-- Absolute href
print(aip.web.resolve_href("https://another.com/page.html", base))
-- Output: "https://another.com/page.html"

-- Relative path href
print(aip.web.resolve_href("sub/page.html", base))
-- Output: "https://example.com/docs/path/sub/page.html"

-- Absolute path href
print(aip.web.resolve_href("/other/resource.txt", base))
-- Output: "https://example.com/other/resource.txt"

-- Scheme-relative href
print(aip.web.resolve_href("//cdn.com/asset.js", base))
-- Output: "https://cdn.com/asset.js" (uses base_url's scheme)

print(aip.web.resolve_href("//cdn.com/asset.js", "http://example.com/"))
-- Output: "http://cdn.com/asset.js"

-- href is nil
print(aip.web.resolve_href(nil, base))
-- Output: nil (Lua nil)
```

#### Error


Returns an error (Lua table `{ error: string }`) if:
- `base_url` is not a valid absolute URL.
- `href` and `base_url` cannot be successfully joined (e.g., due to malformed `href`).

## aip.uuid

The `aip.uuid` module exposes functions for generating various UUIDs and converting timestamped UUIDs.

### Functions Summary

```lua
aip.uuid.new(): string
aip.uuid.new_v4(): string
aip.uuid.new_v7(): string
aip.uuid.new_v4_b64(): string
aip.uuid.new_v4_b64u(): string
aip.uuid.new_v4_b58(): string
aip.uuid.new_v7_b64(): string
aip.uuid.new_v7_b64u(): string
aip.uuid.new_v7_b58(): string
aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
```

### aip.uuid.new

Generates a new UUID version 4. This is an alias for `aip.uuid.new_v4()`.

```lua
-- API Signature
aip.uuid.new(): string
```

#### Returns


`string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").

#### Example


```lua
local id = aip.uuid.new()
print(id)
```

### aip.uuid.new_v4

Generates a new UUID version 4.

```lua
-- API Signature
aip.uuid.new_v4(): string
```

#### Returns


`string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").

#### Example


```lua
local id_v4 = aip.uuid.new_v4()
print(id_v4)
```

### aip.uuid.new_v7

Generates a new UUID version 7 (time-ordered).

```lua
-- API Signature
aip.uuid.new_v7(): string
```

#### Returns


`string`: The generated UUIDv7 as a string.

#### Example


```lua
local id_v7 = aip.uuid.new_v7()
print(id_v7)
```

### aip.uuid.new_v4_b64

Generates a new UUID version 4 and encodes it using standard Base64.

```lua
-- API Signature
aip.uuid.new_v4_b64(): string
```

#### Returns


`string`: The Base64 encoded UUIDv4 string.

#### Example


```lua
local id_v4_b64 = aip.uuid.new_v4_b64()
print(id_v4_b64)
```

### aip.uuid.new_v4_b64u

Generates a new UUID version 4 and encodes it using URL-safe Base64 without padding.

```lua
-- API Signature
aip.uuid.new_v4_b64u(): string
```

#### Returns


`string`: The URL-safe Base64 encoded (no padding) UUIDv4 string.

#### Example


```lua
local id_v4_b64u = aip.uuid.new_v4_b64u()
print(id_v4_b64u)
```

### aip.uuid.new_v4_b58

Generates a new UUID version 4 and encodes it using Base58.

```lua
-- API Signature
aip.uuid.new_v4_b58(): string
```

#### Returns


`string`: The Base58 encoded UUIDv4 string.

#### Example


```lua
local id_v4_b58 = aip.uuid.new_v4_b58()
print(id_v4_b58)
```

### aip.uuid.new_v7_b64

Generates a new UUID version 7 and encodes it using standard Base64.

```lua
-- API Signature
aip.uuid.new_v7_b64(): string
```

#### Returns


`string`: The Base64 encoded UUIDv7 string.

#### Example


```lua
local id_v7_b64 = aip.uuid.new_v7_b64()
print(id_v7_b64)
```

### aip.uuid.new_v7_b64u

Generates a new UUID version 7 and encodes it using URL-safe Base64 without padding.

```lua
-- API Signature
aip.uuid.new_v7_b64u(): string
```

#### Returns


`string`: The URL-safe Base64 encoded (no padding) UUIDv7 string.

#### Example


```lua
local id_v7_b64u = aip.uuid.new_v7_b64u()
print(id_v7_b64u)
```

### aip.uuid.new_v7_b58

Generates a new UUID version 7 and encodes it using Base58.

```lua
-- API Signature
aip.uuid.new_v7_b58(): string
```

#### Returns


`string`: The Base58 encoded UUIDv7 string.

#### Example


```lua
local id_v7_b58 = aip.uuid.new_v7_b58()
print(id_v7_b58)
```

### aip.uuid.to_time_epoch_ms

Converts a timestamped UUID string (V1, V6, V7) to milliseconds since Unix epoch.
Returns `nil` if the input is `nil`, not a valid UUID string, or if the UUID type
does not contain an extractable timestamp (e.g., V4).

```lua
-- API Signature
aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
```

#### Arguments


- `value: string | nil`: The UUID string or `nil`.

#### Returns


`integer | nil`: Milliseconds since Unix epoch, or `nil`.

#### Example


```lua
local v7_uuid_str = aip.uuid.new_v7()
local millis = aip.uuid.to_time_epoch_ms(v7_uuid_str)
if millis then
  print("Timestamp in ms: " .. millis)
else
  print("Could not extract timestamp.")
end

local v4_uuid_str = aip.uuid.new_v4()
local millis_v4 = aip.uuid.to_time_epoch_ms(v4_uuid_str)
-- millis_v4 will be nil

local invalid_millis = aip.uuid.to_time_epoch_ms("not-a-uuid")
-- invalid_millis will be nil

local nil_millis = aip.uuid.to_time_epoch_ms(nil)
-- nil_millis will be nil
```

## aip.time

```lua
aip.time.now_iso_utc(): string            -- RFC 3339 UTC (seconds precision)
-- e.g., "2025-08-23T14:35:12Z"

aip.time.now_iso_local(): string          -- RFC 3339 local time (seconds precision)
-- e.g., "2025-08-23T09:35:12-05:00"

aip.time.now_iso_utc_micro(): string      -- RFC 3339 UTC (microseconds)
-- e.g., "2025-08-23T14:35:12.123456Z"

aip.time.now_iso_local_micro(): string    -- RFC 3339 local time (microseconds)
-- e.g., "2025-08-23T09:35:12.123456-05:00"

aip.time.now_utc_micro(): integer         -- epoch microseconds (UTC)
-- e.g., 1766561712123456

aip.time.today_utc(): string              -- weekday + date (UTC)
-- e.g., "Saturday 2025-08-23"

aip.time.today_local(): string            -- weekday + date (local)
-- e.g., "Saturday 2025-08-23"

aip.time.today_iso_utc(): string          -- "YYYY-MM-DD" (UTC)
-- e.g., "2025-08-23"

aip.time.today_iso_local(): string        -- "YYYY-MM-DD" (local)
-- e.g., "2025-08-23"

aip.time.weekday_utc(): string            -- weekday name (UTC)
-- e.g., "Saturday"

aip.time.weekday_local(): string          -- weekday name (local)
-- e.g., "Saturday"

aip.time.local_tz_id(): string            -- IANA timezone id for local zone
-- e.g., "America/Los_Angeles"
```

## aip.lua

Lua value inspection functions.

### Functions Summary

```lua
aip.lua.dump(value: any): string
```

### aip.lua.dump

Dump a Lua value into its string representation.

```lua
-- API Signature
aip.lua.dump(value: any): string
```

Provides a detailed string representation of any Lua value, useful for debugging.

#### Arguments

- `value: any`: The Lua value to dump.

#### Returns

- `string`: A string representation of the value.

#### Example

```lua
local tbl = { key = "value", nested = { num = 42 } }
print(aip.lua.dump(tbl))
-- Output: Example: table: 0x... { key = "value", nested = table: 0x... { num = 42 } }
```

#### Error

Returns an error (Lua table `{ error: string }`) if the value cannot be converted to string.


## aip.agent

Functions for running other AIPACK agents from within a Lua script.

### Functions Summary

```lua
aip.agent.run(agent_name: string, options?: table): any

aip.agent.extract_options(value: any): table | nil
```

### aip.agent.run

Runs another agent and returns its response.

```lua
-- API Signature
aip.agent.run(agent_name: string, options?: table): any
```

Executes the agent specified by `agent_name`. The function waits for the called agent
to complete and returns its result. This allows for chaining agents together.

#### Arguments


- `agent_name: string`: The name of the agent to run. This can be a relative path
  (e.g., `"../my-other-agent.aip"`) or a fully qualified pack reference
  (e.g., `"my-ns@my-pack/feature/my-agent.aip"`). Relative paths are resolved
  from the directory of the calling agent.
- `options?: table`: An optional table containing input data and agent options.
  - `inputs?: string | list | table`: Input data for the agent. Can be a single string, a list of strings, or a table of structured inputs.
  - `options?: table`: Agent-specific options. These options are passed directly to the called agent's
    execution environment and can override settings defined in the called agent's `.aip` file.

##### Input Examples:

```lua
-- Run an agent with a single string input
local response = aip.agent.run("agent-name", { inputs = "hello" })

-- Run an agent with multiple string inputs
local response = aip.agent.run("agent-name", { inputs = {"input1", "input2"} })

-- Run an agent with structured inputs (e.g., file records)
local response = aip.agent.run("agent-name", {
  inputs = {
    { path = "file1.txt", content = "..." },
    { path = "file2.txt", content = "..." }
  }
})
```

#### Returns


The result of the agent execution. The type of the returned value depends on the agent's output:

- If the agent produces an AI response without a specific output script, it returns a table representing the `AiResponse` object.
- If the agent has an output script, it returns the value returned by that output script.

```ts
// Example structure of a returned AiResponse object (if no output script)
{
  action: string, // e.g., "PrintToConsole", "WriteFiles"
  outputs: any,   // Depends on the action/output
  options: table  // Options used during the run
  // ... other properties from AiResponse
}
```

#### Error


Returns an error if:
- The `agent_name` is invalid or the agent file cannot be located/loaded.
- The options table contains invalid parameters.
- The execution of the called agent fails.
- An internal communication error occurs while waiting for the agent's result.

```ts
{
  error: string // Error message
}
```

### aip.agent.extract_options

Extracts relevant agent options from a given Lua value.

```lua
-- API Signature
aip.agent.extract_options(value: any): table | nil
```

If the input `value` is a Lua table, this function creates a new table and copies
the following properties if they exist in the input table:

- `model`
- `model_aliases`
- `input_concurrency`
- `temperature`

Other properties are ignored. If the input `value` is `nil` or not a table,
the function returns `nil`.

#### Arguments


- `value: any`: The Lua value from which to extract options.

#### Returns


A new Lua table containing the extracted options, or `nil` if the input
was `nil` or not a table.


## aip.task

Functions for recording pins at the task level (attached to the current task of the current run).

### Functions Summary

```lua
aip.task.pin(iden: string, content: any)
aip.task.pin(iden: string, priority: number, content: any)
```

### aip.task.pin

Creates a pin attached to the current task. Requires that both CTX.RUN_UID and CTX.TASK_UID are available (i.e., must be called during a task cycle, not in before_all or after_all).

```lua
-- API Signatures
aip.task.pin(iden: string, content: any)
aip.task.pin(iden: string, priority: number, content: any)
```

Records a pin for the current task. When the optional priority is provided, it will be stored along with the pin.

#### Arguments

- `iden: string`
  Identifier (name) for this pin.

- `priority: number (optional)`
  Optional numeric priority to associate with the pin.

- `content: any`
  Arbitrary content to associate with the pin (string, table, etc.).

#### Returns

- Nothing. This function records the pin as a side effect.

#### Example

```lua
-- Simple pin (no priority)
aip.task.pin("review", "Needs follow-up")

-- Pin with priority
aip.task.pin("checkpoint", 0.7, { step = 3, note = "after validation" })
```

#### Error

Returns an error (Lua table `{ error: string }`) if called outside a task context (no `CTX.TASK_UID`), if there is no run context, or if arguments are invalid.


## aip.run

Functions for recording pins at the run level (attached to the overall run).

### Functions Summary

```lua
aip.run.pin(iden: string, content: any)
aip.run.pin(iden: string, priority: number, content: any)
```

### aip.run.pin

Creates a pin attached to the current run. Requires that CTX.RUN_UID is available.

```lua
-- API Signatures
aip.run.pin(iden: string, content: any)
aip.run.pin(iden: string, priority: number, content: any)
```

Records a pin for the current run. When the optional priority is provided, it will be stored along with the pin.

#### Arguments

- `iden: string`
  Identifier (name) for this pin.

- `priority: number (optional)`
  Optional numeric priority to associate with the pin.

- `content: any`
  Arbitrary content to associate with the pin (string, table, etc.).

#### Returns

- Nothing. This function records the pin as a side effect.

#### Example

```lua
-- Simple pin (no priority)
aip.run.pin("summary", "Run started successfully")

-- Pin with priority
aip.run.pin("quality-score", 0.85, { score = 0.85, rationale = "good coverage" })
```

#### Error

Returns an error (Lua table `{ error: string }`) if there is no run context (no `CTX.RUN_UID`) or if arguments are invalid.


## aip.flow

Functions for controlling the AIPACK agent execution flow from within script blocks (`before_all`, `data`).

### Functions Summary

```lua
aip.flow.before_all_response(data: BeforeAllData) -> table

aip.flow.data_response(data: DataData) -> table

aip.flow.skip(reason?: string): table
```

These functions return special tables that instruct the agent executor how to proceed. They should be the return value of the script block.

### aip.flow.before_all_response

Customizes execution flow at the 'Before All' stage (in `before_all` script block).

```lua
-- API Signature
aip.flow.before_all_response(data: BeforeAllData) -> table
```

This function is typically called within the `before_all` block of an agent script
to override the default behavior of passing all initial inputs to the agent.

#### Arguments


- `data: table` - A table defining the new inputs and options for the agent execution cycle.
  ```ts
  type BeforeAllData = {
    inputs?:  any[],        // Optional. A list of new inputs to use for the agent run cycle. Overrides initial inputs.
    options?: AgentOptions, // Optional. Partial AgentOptions to override for this run.
    before_all?: any,       // Optional. The before_all data that can be access via before_all...
  } & any // Can also include other arbitrary data fields if needed.
  ```
  related types: [AgentOptions](#agentoptions)

#### Example


```lua
local result = aip.flow.before_all_response({
  inputs = {"processed_input_1", "processed_input_2"},
  options = {
    model = "gemini-2.5-flash",
    input_concurrency = 3
  },
  before_all = {some_data = "hello world" } -- Arbitrary data is allowed
})
-- The agent executor will process this result table.
```

#### Error


This function does not directly return any errors. Errors might occur during the creation of lua table.

### aip.flow.data_response

Customizes execution flow at the 'Data' stage for a single input (in `data` script block).

```lua
-- API Signature
aip.flow.data_response(data: DataData) -> table
```

This function is typically called within the `data` block of an agent script.
It allows overriding the input and/or options for the current input cycle,
or returning additional arbitrary data.

#### Arguments


- `data: table` - A table defining the new input, options, and/or other data for the current cycle.
  ```ts
  type DataData = {
    input?: any | nil,     // Optional. The new input to use for this cycle. If nil, the original input is used.
    data?: any | nil,      // Data that will be available in the next stage. Same as returning a simple data.
    options?: AgentOptions, // Optional. Partial AgentOptions to override for this cycle.
  } & any // Can also include other arbitrary data fields (e.g., computed values, flags)
  ```
  related types: [AgentOptions](#agentoptions)

#### Example


```lua
-- Use a transformed input and override the model for this cycle
return aip.flow.data_response({
  data  = data,              -- The data that would have been returned
  input = transformed_input,
  options = { model = "gpt-4o" },
})
-- The agent executor will process this result table.
```

#### Error


This function does not directly return any errors. Errors might occur during the creation of lua table.

### aip.flow.skip

Skips processing the current input cycle (in `data` script block).

```lua
-- API Signature
aip.flow.skip(reason?: string): table
```

This function is typically called within the `data` block of an agent script
to instruct AIPACK to skip processing the current input value and move to the next one.

#### Arguments


- `reason: string (optional)`: An optional string providing the reason for skipping the input cycle.
  This reason might be logged or displayed depending on the AIPACK execution context.

#### Example


```lua
-- Skip processing if the input is nil or empty
if input == nil or input == "" then
  return aip.flow.skip("Input is empty")
end
-- Continue processing the input if not skipped
-- ... rest of data block ...
```

#### Error


This function does not directly return any errors. Errors might occur during the creation of lua table.

## aip.cmd

Functions for executing system commands.

### Functions Summary

```lua
aip.cmd.exec(cmd_name: string, args?: string | list<string>): CmdResponse | {error: string, stdout?: string, stderr?: string, exit?: number}
```

### aip.cmd.exec

Execute a system command with optional arguments.

```lua
-- API Signature
aip.cmd.exec(cmd_name: string, args?: string | list<string>): CmdResponse | {error: string, stdout?: string, stderr?: string, exit?: number}
```

Executes the command using the system shell. On Windows, wraps with `cmd /C`.

#### Arguments

- `cmd_name: string`: Command name or path.
- `args?: string | list<string>` (optional): Arguments as a single string or list of strings.

#### Returns

- `CmdResponse`: A [CmdResponse](#cmdresponse) table with stdout, stderr, and exit code, even if the exit code is non-zero.

#### Example

```lua
-- Single string argument
local r1 = aip.cmd.exec("echo", "hello world")
print("stdout:", r1.stdout) -- Output: hello world\n (or similar)
print("exit:", r1.exit)   -- Output: 0

-- Table of arguments
local r2 = aip.cmd.exec("ls", {"-l", "-a", "nonexistent"})
print("stderr:", r2.stderr) -- Output: ls: nonexistent: No such file... (or similar)
print("exit:", r2.exit)   -- Output: non-zero exit code

-- Example of potential error return (e.g., command not found)
local r3 = aip.cmd.exec("nonexistent_command")
if type(r3) == "table" and r3.error then
  print("Execution Error:", r3.error)
end
```

#### Error

Returns an error (Lua table `{ error: string, stdout?: string, stderr?: string, exit?: number }`) only if the process *fails to start* (e.g., command not found, permission issue). Non-zero exit codes from the command itself are captured in the [CmdResponse](#cmdresponse) and do not cause a Lua error by default.

## aip.semver

Functions for semantic versioning (SemVer 2.0.0) operations.

### Functions Summary

```lua
aip.semver.compare(version1: string, operator: string, version2: string): boolean | {error: string}

aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil} | {error: string}

aip.semver.is_prerelease(version: string): boolean | {error: string}

aip.semver.valid(version: string): boolean
```

### aip.semver.compare

Compares two version strings using an operator.

```lua
-- API Signature
aip.semver.compare(version1: string, operator: string, version2: string): boolean | {error: string}
```

Compares versions according to SemVer rules (prerelease < release, build metadata ignored).

#### Arguments

- `version1: string`: First version string.
- `operator: string`: Comparison operator (`<`, `<=`, `=`, `==`, `>=`, `>`, `!=`, `~=`).
- `version2: string`: Second version string.

#### Returns

- `boolean`: `true` if the comparison holds, `false` otherwise.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.semver.compare("1.2.3", ">", "1.2.0"))     -- Output: true
print(aip.semver.compare("1.0.0-alpha", "<", "1.0.0")) -- Output: true
print(aip.semver.compare("1.0.0+build", "==", "1.0.0")) -- Output: true

local r = aip.semver.compare("abc", ">", "1.0.0")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if operator is invalid or versions are not valid SemVer strings.

### aip.semver.parse

Parses a version string into its components.

```lua
-- API Signature
aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil} | {error: string}
```

#### Arguments

- `version: string`: The SemVer string to parse.

#### Returns

- `table`: A table containing `major`, `minor`, `patch`, `prerelease` (string or nil), and `build` (string or nil) components.
- `{error: string}`: An error table on failure.

#### Example

```lua
local v = aip.semver.parse("1.2.3-beta.1+build.123")
print(v.major, v.minor, v.patch) -- Output: 1 2 3
print(v.prerelease)             -- Output: beta.1
print(v.build)                  -- Output: build.123

local r = aip.semver.parse("invalid")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `version` is not a valid SemVer string.

### aip.semver.is_prerelease

Checks if a version string has a prerelease component.

```lua
-- API Signature
aip.semver.is_prerelease(version: string): boolean | {error: string}
```

#### Arguments

- `version: string`: The SemVer string to check.

#### Returns

- `boolean`: `true` if it has a prerelease component (e.g., `-alpha`), `false` otherwise.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.semver.is_prerelease("1.2.3-beta"))      -- Output: true
print(aip.semver.is_prerelease("1.2.3"))         -- Output: false
print(aip.semver.is_prerelease("1.0.0+build")) -- Output: false

local r = aip.semver.is_prerelease("invalid")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `version` is not a valid SemVer string.

### aip.semver.valid

Checks if a string is a valid SemVer 2.0.0 version.

```lua
-- API Signature
aip.semver.valid(version: string): boolean
```

#### Arguments

- `version: string`: The string to validate.

#### Returns

- `boolean`: `true` if valid, `false` otherwise.

#### Example

```lua
print(aip.semver.valid("1.2.3"))          -- Output: true
print(aip.semver.valid("1.2.3-alpha.1"))   -- Output: true
print(aip.semver.valid("1.0"))           -- Output: false
print(aip.semver.valid("invalid"))       -- Output: false
```

#### Error

This function does not typically error, returning `false` for invalid formats.

## aip.rust

Functions for processing Rust code.

### Functions Summary

```lua
aip.rust.prune_to_declarations(code: string): string | {error: string}
```

### aip.rust.prune_to_declarations

Prunes Rust code, replacing function bodies with `{ ... }`.

```lua
-- API Signature
aip.rust.prune_to_declarations(code: string): string | {error: string}
```

Replaces function bodies with `{ ... }`, preserving comments, whitespace, and non-function code structures.

#### Arguments

- `code: string`: The Rust code to prune.

#### Returns

- `string`: The pruned Rust code string on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
local rust_code = "fn greet(name: &str) {\n  println!(\"Hello, {}!\", name);\n}\n\nstruct Data;"
local pruned = aip.rust.prune_to_declarations(rust_code)
-- pruned might be: "fn greet(name: &str) { ... }\n\nstruct Data;" (exact spacing may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if pruning fails.

## aip.html

Functions for processing HTML content.

### Functions Summary

```lua
aip.html.slim(html_content: string): string | {error: string}

aip.html.to_md(html_content: string): string | {error: string}
```

### aip.html.slim

Strips non-content elements and most attributes from HTML.

```lua
-- API Signature
aip.html.slim(html_content: string): string | {error: string}
```

Removes `<script>`, `<link>`, `<style>`, `<svg>`, comments, empty lines, and most attributes (keeps `class`, `aria-label`, `href`).

#### Arguments

- `html_content: string`: The HTML content string.

#### Returns

- `string`: The cleaned HTML string on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
local html = "<script>alert('hi')</script><p class='c' style='color:red'>Hello</p>"
local cleaned = aip.html.slim(html)
-- cleaned might be: "<p class=\"c\">Hello</p>" (exact output may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if pruning fails.

### aip.html.to_md

Converts HTML content to Markdown format.

```lua
-- API Signature
aip.html.to_md(html_content: string): string | {error: string}
```

#### Arguments


- `html_content: string`: The HTML content to be converted.

#### Returns


- `string`: The Markdown representation of the HTML content.
- `{error: string}`: An error table on failure.

#### Example


```lua
local markdown_content = aip.html.to_md("<h1>Hello</h1><p>World</p>")
-- markdown_content will be "# Hello\n\nWorld\n"
```

#### Error


Returns an error (Lua table `{ error: string }`) if the HTML content fails to be converted to Markdown.

## aip.git

Functions for performing basic Git operations in the workspace.

### Functions Summary

```lua
aip.git.restore(path: string): string | {error: string, stdout?: string, stderr?: string, exit?: number}
```

### aip.git.restore

Executes `git restore <path>` in the workspace directory.

```lua
-- API Signature
aip.git.restore(path: string): string | {error: string, stdout?: string, stderr?: string, exit?: number}
```

Restores the specified file or directory path to its state from the Git index.

#### Arguments

- `path: string`: The file or directory path to restore (relative to workspace root).

#### Returns

- `string`: Standard output from the `git restore` command on success.
- `{error: string, stdout?: string, stderr?: string, exit?: number}`: An error table if the command fails (e.g., path not known to Git, non-zero exit code, stderr output). This error table is similar to a [CmdResponse](#cmdresponse) but includes an additional `error` field.

#### Example

```lua
-- Restore a modified file
local result = aip.git.restore("src/main.rs")
-- Check if result is an error table or the stdout string
if type(result) == "table" and result.error then
  print("Error restoring:", result.error)
  print("Stderr:", result.stderr) -- May contain git error message
else
  print("Restore stdout:", result)
end
```

#### Error

Returns an error (Lua table `{ error: string, stdout?: string, stderr?: string, exit?: number }`) if the `git restore` command encounters an issue, such as the path not being known to Git, insufficient permissions, or the command returning a non-zero exit code with stderr output.

## aip.hash

The `aip.hash` module exposes functions for various hashing algorithms and encodings.

### Functions Summary

```lua
aip.hash.sha256(input: string): string
aip.hash.sha256_b58(input: string): string
aip.hash.sha256_b64(input: string): string
aip.hash.sha256_b64u(input: string): string
aip.hash.sha512(input: string): string
aip.hash.sha512_b58(input: string): string
aip.hash.sha512_b64(input: string): string
aip.hash.sha512_b64u(input: string): string
aip.hash.blake3(input: string): string
aip.hash.blake3_b58(input: string): string
aip.hash.blake3_b64(input: string): string
aip.hash.blake3_b64u(input: string): string
```

### aip.hash.sha256

Computes the SHA256 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.sha256(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA256 hash, hex-encoded.

#### Example


```lua
local hex_hash = aip.hash.sha256("hello world")
-- hex_hash will be "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b58

Computes the SHA256 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.sha256_b58(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA256 hash, Base58-encoded.

#### Example


```lua
local b58_hash = aip.hash.sha256_b58("hello world")
-- b58_hash will be "CnKqR4x6r4nAv2iGk8DrZSSWp7n3W9xKRj69eZysS272"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b64

Computes the SHA256 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.sha256_b64(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA256 hash, standard Base64-encoded.

#### Example


```lua
local b64_hash = aip.hash.sha256_b64("hello world")
-- b64_hash will be "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b64u

Computes the SHA256 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.sha256_b64u(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA256 hash, URL-safe Base64-encoded without padding.

#### Example


```lua
local b64u_hash = aip.hash.sha256_b64u("hello world")
-- b64u_hash will be "uU0nuZNNPgilLlLX2n2r-sSE7-N6U4DukIj3rOLvzek"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512

Computes the SHA512 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.sha512(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA512 hash, hex-encoded.

#### Example


```lua
local hex_hash = aip.hash.sha512("hello world")
-- hex_hash will be "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b58

Computes the SHA512 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.sha512_b58(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA512 hash, Base58-encoded.

#### Example


```lua
local b58_hash = aip.hash.sha512_b58("hello world")
-- b58_hash will be "yP4cqy7jmaRDzC2bmcGNZkuQb3VdftMk6YH7ynQ2Qw4zktKsyA9fk52xghNQNAdkpF9iFmFkKh2bNVG4kDWhsok"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b64

Computes the SHA512 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.sha512_b64(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA512 hash, standard Base64-encoded.

#### Example


```lua
local b64_hash = aip.hash.sha512_b64("hello world")
-- b64_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd+5RGnx6m808qG1M2G+YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw=="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b64u

Computes the SHA512 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.sha512_b64u(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The SHA512 hash, URL-safe Base64-encoded without padding.

#### Example


```lua
local b64u_hash = aip.hash.sha512_b64u("hello world")
-- b64u_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd-5RGnx6m808qG1M2G-YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3

Computes the Blake3 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.blake3(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The Blake3 hash, hex-encoded.

#### Example


```lua
local hex_hash = aip.hash.blake3("hello world")
-- hex_hash will be "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b58

Computes the Blake3 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.blake3_b58(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The Blake3 hash, Base58-encoded.

#### Example


```lua
local b58_hash = aip.hash.blake3_b58("hello world")
-- b58_hash will be "FVPfbg9bK7mj7jnaSRXhuVcVakkXcjMPgSwxmauUofYf"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b64

Computes the Blake3 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.blake3_b64(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The Blake3 hash, standard Base64-encoded.

#### Example


```lua
local b64_hash = aip.hash.blake3_b64("hello world")
-- b64_hash will be "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b64u

Computes the Blake3 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.blake3_b64u(input: string): string
```

#### Arguments


- `input: string`: The string to hash.

#### Returns


`string`: The Blake3 hash, URL-safe Base64-encoded without padding.

#### Example


```lua
local b64u_hash = aip.hash.blake3_b64u("hello world")
-- b64u_hash will be "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.


## aip.hbs

Functions for rendering Handlebars templates.

### Functions Summary

```lua
aip.hbs.render(content: string, data: any): string | {error: string}
```

### aip.hbs.render

Renders a Handlebars template string with Lua data.

```lua
-- API Signature
aip.hbs.render(content: string, data: any): string | {error: string}
```

Converts Lua `data` to JSON internally and renders the Handlebars `content` template.

#### Arguments

- `content: string`: The Handlebars template string.
- `data: any`: The data as a Lua value (table, number, string, boolean, nil). Note that function types or userdata are not supported.

#### Returns

- `string`: The rendered template string on success.
- `{error: string}`: An error table on failure (data conversion or template rendering).

#### Example

```lua
local template = "Hello, {{name}}!"
local data = {name = "World"}
local rendered_content = aip.hbs.render(template, data)
print(rendered_content) -- Output: Hello, World!

local data_list = {
    name  = "Jen Donavan",
    todos = {"Bug Triage AIPACK", "Fix Windows Support"}
}
local template_list = [[
Hello {{name}},

Your tasks today:

{{#each todos}}
- {{this}}
{{/each}}

Have a good day (after you completed this tasks)
]]
local content_list = aip.hbs.render(template_list, data_list)
print(content_list)
```

#### Error

Returns an error (Lua table `{ error: string }`) if Lua data cannot be converted to JSON or if Handlebars rendering fails.

## aip.code

Utility functions for code formatting and manipulation.

### Functions Summary

```lua
aip.code.comment_line(lang_ext: string, comment_content: string): string | {error: string}
```

### aip.code.comment_line

Creates a single comment line appropriate for a given language extension.

```lua
-- API Signature
aip.code.comment_line(lang_ext: string, comment_content: string): string | {error: string}
```

Formats `comment_content` as a single-line comment based on `lang_ext`.

#### Arguments

- `lang_ext: string`: File extension or language identifier (e.g., "rs", "lua", "py", "js", "css", "html"). Case-insensitive.
- `comment_content: string`: The text to put inside the comment.

#### Returns

- `string`: The formatted comment line (without trailing newline) on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.code.comment_line("rs", "TODO: Refactor"))  -- Output: // TODO: Refactor
print(aip.code.comment_line("py", "Add validation"))  -- Output: # Add validation
print(aip.code.comment_line("lua", "Fix this later")) -- Output: -- Fix this later
print(aip.code.comment_line("html", "Main content"))  -- Output: <!-- Main content -->
```

#### Error

Returns an error (Lua table `{ error: string }`) on conversion or formatting issues.


## aip.shape

Functions to shape row-like Lua tables (records), convert between row- and column-oriented data, and work with record keys.

### Functions Summary

```lua
aip.shape.to_record(names: string[], values: any[]): table

aip.shape.to_records(names: string[], rows: any[][]): list<table>

aip.shape.record_to_values(record: table, names?: string[]): list<any>

aip.shape.columnar_to_records(cols: { [string]: list<any> }): list<table>

aip.shape.records_to_columnar(recs: list<table>): { [string]: list<any> }

aip.shape.select_keys(rec: table, keys: list<string>): table

aip.shape.omit_keys(rec: table, keys: list<string>): table

aip.shape.remove_keys(rec: table, keys: list<string>): integer

aip.shape.extract_keys(rec: table, keys: list<string>): table
```

### aip.shape.to_record

Build a single record (row object) from a list of column names and a list of values. Truncates to the shorter list.

```lua
-- API Signature
aip.shape.to_record(names: string[], values: any[]): table
```

#### Arguments

- `names: string[]`: Column names (all must be strings).

- `values: any[]`: Values list.

#### Returns

- `table`: A record with keys from `names` and values from `values`.

#### Example

```lua
local rec = aip.shape.to_record({ "id", "name", "email" }, { 1, "Alice", "a@x.com" })
-- rec == { id = 1, name = "Alice", email = "a@x.com" }
```

#### Error

Returns an error (Lua table `{ error: string }`) if any entry in `names` is not a string.

### aip.shape.to_records

Build multiple records from a list of column names and a list of rows (each row is a list of values). Each row is truncated to the shorter length relative to `names`.

```lua
-- API Signature
aip.shape.to_records(names: string[], rows: any[][]): list<table>
```

#### Arguments

- `names: string[]`: Column names (all must be strings).

- `rows: any[][]`: List of value lists (each must be a table).

#### Returns

- `list<table>`: A list of records.

#### Example

```lua
local names = { "id", "name" }
local rows  = { { 1, "Alice" }, { 2, "Bob" } }
local out = aip.shape.to_records(names, rows)
-- out == { { id = 1, name = "Alice" }, { id = 2, name = "Bob" } }
```

#### Error

Returns an error if a name is not a string or if any row is not a table.

### aip.shape.record_to_values

Convert a single record into an array (Lua list) of values.

```lua
-- API Signature
aip.shape.record_to_values(record: table, names?: string[]): any[]
```

- When `names` is provided, values are returned in the order of `names`.
  - Missing keys yield NA sentinel entries in the result list.
  - If `names` contains a non-string entry, an error is returned.

- When `names` is not provided, values are returned in alphabetical order of the record's string keys.
  - Non-string keys are ignored.

#### Example

```lua
local rec = { id = 1, name = "Alice", email = "a@x.com" }
local v1  = aip.shape.record_to_values(rec)
-- { 1, "a@x.com", "Alice" } (alpha by keys: email, id, name)

local v2  = aip.shape.record_to_values(rec, { "name", "id", "missing" })
-- { "Alice", 1, NA }
```

#### Error

Returns an error if `names` contains a non-string entry.

### aip.shape.columnar_to_records

Convert a column-oriented table into a list of records. All columns must be tables of the same length and keys must be strings.

```lua
-- API Signature
aip.shape.columnar_to_records(cols: { [string]: any[] }): list<table>
```

#### Arguments

- `cols: { [string]: any[] }`: Map of column name (string) to list of values (table).

#### Returns

- `list<table>`: A list of row records.

#### Example

```lua
local cols = {
  id    = { 1, 2, 3 },
  name  = { "Alice", "Bob", "Cara" },
  email = { "a@x.com", "b@x.com", "c@x.com" },
}
local recs = aip.shape.columnar_to_records(cols)
-- recs == {
--   { id = 1, name = "Alice", email = "a@x.com" },
--   { id = 2, name = "Bob",   email = "b@x.com" },
--   { id = 3, name = "Cara",  email = "c@x.com" },
-- }
```

#### Error

Returns an error if any key is not a string, any value is not a table, or columns have different lengths.

### aip.shape.records_to_columnar

Convert a list of records into a column-oriented table. Uses the intersection of string keys across all records to ensure rectangular output.

```lua
-- API Signature
aip.shape.records_to_columnar(recs: list<table>): { [string]: any[] }
```

#### Arguments

- `recs: list<table>`: List of records (each must be a table with string keys).

#### Returns

- `{ [string]: any[] }`: Columns map with values aligned by record index. Only keys present in every record are included.

#### Example

```lua
local cols = aip.shape.records_to_columnar({
  { id = 1, name = "Alice" },
  { id = 2, name = "Bob"   },
})
-- cols == { id = {1, 2}, name = {"Alice", "Bob"} }
```

#### Error

Returns an error if any record is not a table or if any key is not a string.

### aip.shape.select_keys

Return a new record with only the specified keys (original record is unchanged). Missing keys are ignored.

```lua
-- API Signature
aip.shape.select_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Source record.

- `keys: string[]`: Keys to select (all must be strings).

#### Returns

- `table`: New record with only the selected keys.

#### Example

```lua
local rec  = { id = 1, name = "Alice", email = "a@x.com" }
local out  = aip.shape.select_keys(rec, { "id", "email" })
-- out == { id = 1, email = "a@x.com" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.omit_keys

Return a new record without the specified keys (original record is unchanged). Missing keys are ignored.

```lua
-- API Signature
aip.shape.omit_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Source record.

- `keys: string[]`: Keys to omit (all must be strings).

#### Returns

- `table`: New record with keys omitted.

#### Example

```lua
local rec  = { id = 1, name = "Alice", email = "a@x.com" }
local out  = aip.shape.omit_keys(rec, { "email" })
-- out == { id = 1, name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.remove_keys

Remove the specified keys from the original record (in place) and return the number of keys actually removed. Missing keys are ignored.

```lua
-- API Signature
aip.shape.remove_keys(rec: table, keys: string[]): integer
```

#### Arguments

- `rec: table`: Record to mutate.

- `keys: string[]`: Keys to remove (all must be strings).

#### Returns

- `integer`: Count of removed keys.

#### Example

```lua
local rec = { id = 1, name = "Alice", email = "a@x.com" }
local n   = aip.shape.remove_keys(rec, { "email", "missing" })
-- n   == 1
-- rec == { id = 1, name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.extract_keys

Return a new record containing only the specified keys and remove them from the original record (in place). Missing keys are ignored.

```lua
-- API Signature
aip.shape.extract_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Record to extract from and mutate.

- `keys: string[]`: Keys to extract (all must be strings).

#### Returns

- `table`: New record containing the extracted key-value pairs.

#### Example

```lua
local rec      = { id = 1, name = "Alice", email = "a@x.com" }
local picked   = aip.shape.extract_keys(rec, { "id", "email" })
-- picked == { id = 1, email = "a@x.com" }
-- rec    == { name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

## Common Types

Common data structures returned by or used in API functions.

### FileRecord

Represents a file with its metadata and content. Returned by `aip.file.load` and `aip.file.list_load`.

```ts
{
  path : string,    // Relative or absolute path used to load/find the file
  dir: string,      // Parent directory of the path (empty if no parent)
  name : string,    // File name with extension (e.g., "main.rs")
  stem : string,    // File name without extension (e.g., "main")
  ext  : string,    // File extension (e.g., "rs")

  ctime?: number,   // Creation timestamp (microseconds since epoch), optional
  ctime?: number,   // Modification timestamp (microseconds), optional
  size?: number,    // File size in bytes, optional

  content: string   // The text content of the file
}
```

### FileInfo

Represents file metadata without content. Returned by `aip.file.list`, `aip.file.first`, `aip.file.ensure_exists`, `aip.file.save_html_to_md`, and `aip.file.save_html_to_slim`.

```ts
{
  path : string,     // Relative or absolute path
  dir: string,       // Parent directory of the path
  name : string,     // File name with extension
  stem : string,     // File name without extension
  ext  : string,     // File extension

  ctime?: number,    // Creation timestamp (microseconds), optional (if with_meta=true for list)
  mtime?: number,    // Modification timestamp (microseconds), optional (if with_meta=true for list)
  size?: number      // File size in bytes, optional (if with_meta=true for list)
}
```

### FileStats

Aggregated statistics for a collection of files. Returned by `aip.file.stats`.

```ts
{
  total_size: number,      // Total size of all matched files in bytes
  number_of_files: number, // Number of files matched
  ctime_first: number,     // Creation timestamp of the oldest file (microseconds since epoch)
  ctime_last: number,      // Creation timestamp of the newest file (microseconds since epoch)
  mtime_first: number,     // Modification timestamp of the oldest file (microseconds since epoch)
  mtime_last: number       // Modification timestamp of the newest file (microseconds since epoch)
}
```

### DestOptions

Options table used for specifying the destination path in functions like `aip.file.save_html_to_md` and `aip.file.save_html_to_slim`.

```ts
{
  base_dir?: string,  // Base directory for resolving the destination
  file_name?: string, // Custom file name for the output
  suffix?: string     // Suffix appended to the source file stem
}
```

### MdSection

Represents a section of a Markdown document, potentially associated with a heading. Returned by `aip.file.load_md_sections` and `aip.file.load_md_split_first`.

```ts
{
  content: string,    // Full content of the section (including heading line and sub-sections)
  heading?: {         // Present if the section starts with a heading
    content: string,  // The raw heading line (e.g., "## Section Title")
    level: number,    // Heading level (1-6)
    name: string      // Extracted heading name (e.g., "Section Title")
  }
}
```

### MdBlock

Represents a fenced block (usually code) in Markdown. Returned by `aip.md.extract_blocks`.

```ts
{
  content: string,     // Content inside the block (excluding fence lines)
  lang?: string,        // Language identifier (e.g., "rust", "lua"), optional
  info: string         // Full info string from the opening fence (e.g., "rust file:main.rs"), optional
}
```

### WebResponse

Represents the result of an HTTP request made by `aip.web.get` or `aip.web.post`.

```ts
{
  success: boolean,   // true if HTTP status code is 2xx, false otherwise
  status: number,     // HTTP status code (e.g., 200, 404, 500)
  url: string,        // The final URL requested (after redirects)
  content: string | table, // Response body. Decoded to a Lua table if Content-Type is application/json, otherwise a string.
  error?: string      // Error message if success is false or if request initiation failed
}
```

### CmdResponse

Represents the result of executing a system command via `aip.cmd.exec`.

```ts
{
  stdout: string,  // Standard output captured from the command
  stderr: string,  // Standard error captured from the command
  exit:   number   // Exit code returned by the command (0 usually indicates success)
}
```

### AgentOptions

Configuration options for an agent. Used in `aip.flow.before_all_response` and `aip.flow.data_response` to override settings for a run or a specific cycle.

```ts
{
  model?: string,
  temperature?: number,
  top_p?: number,
  input_concurrency?: number,
  model_aliases?: { [key: string]: string }
}
```

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
| CTX.TMP_DIR              | `.aipack/.session/0196adbf-b792-7070-a5be-eec26698c065/tmp`              | The tmp dir for this session (all redos in same session)          |
| CTX.SESSION_UID          | `0196adbf-b792-7070-a5be-eec26698c065`                                   | The Session Unique ID for this CLI Session                        |
| CTX.RUN_UID              | `0196adbf-b792-7070-a5be-ddc33698c065`                                   | The Run Unique ID                                                 |
| CTX.TASK_UID             | `0196adbf-b792-7070-a5be-aac55698c065`                                   | The Task Unique ID (when in a task stage)                         |



When running a pack. (when no packs, those will be all nil)

For `aip run acme@my_pack/my-agent`

| Key                            | Example Value                                             | Description                                                                       |
|--------------------------------|-----------------------------------------------------------|-----------------------------------------------------------------------------------|
| CTX.PACK_IDENTITY              | `acme@my_pack`                                            | Pack identity (namespace@name) (nil if not run via pack ref).                     |
| CTX.PACK_NAMESPACE             | `acme`                                                    | Namespace of the pack (nil if not run via pack reference).                        |
| CTX.PACK_NAME                  | `my_pack`                                                 | Name of the pack (nil if not run via pack reference).                             |
| CTX.PACK_REF                   | `acme@my_pack/my-agent`                                   | (Nil if not a pack) Full pack reference used (nil if not run via pack reference). |
| CTX.PACK_WORKSPACE_SUPPORT_DIR | `/Users/dev/my-project/.aipack/support/pack/acme/my_pack` | Workspace-specific support directory for this agent (if applicable).              |
| CTX.PACK_BASE_SUPPORT_DIR      | `/Users/home/john/.aipack-base/support/pack/acme/my_pack` | Base support directory for this agent (if applicable).                            |


- All paths are absolute and normalized for the OS.
- `CTX.PACK...` fields are `nil` if the agent was invoked directly via its file path rather than a pack reference (e.g., `aip run my-agent.aip`).
- The `AGENT_NAME` reflects how the agent was called, while `AGENT_FILE_PATH` is the fully resolved location.