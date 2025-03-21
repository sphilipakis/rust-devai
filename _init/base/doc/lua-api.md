# API Documentation

The `aip` top module is comprised of the following submodules.

> Note: All of the type documentation is noted in "TypeScript style" as it is a common and concise type notation for scripting languages and works well to express Lua types.
>       However, it is important to note that there is no TypeScript support, just standard Lua. For example, Lua properties are delimited with `=` and not `:`,
>       and arrays and dictionaries are denoted with `{ }`.

## aip.file

File manipulation functions for loading, saving, and managing files in the workspace.

```lua
-- Load file text content and return its FileRecord
local file = aip.file.load("doc/some-file.md")                -- FileRecord

-- Save file content (will create directories as needed)
aip.file.save("doc/some-file.md", "some new content")         -- void

-- Append content to file (create file and directories as needed)
aip.file.append("doc/some-file.md", "some new content")       -- void

-- List files matching a glob pattern
local all_doc_files = aip.file.list("doc/**/*.md")            -- {FileMeta, ...}

-- List files matching a glob pattern and options
local all_doc_files = aip.file.list("**/*.md", {base_dir = "doc/"})  -- {FileMeta, ...}

-- List files and load their content
local all_files = aip.file.list_load({"doc/**/*.md", "src/**/*.rs"}) -- {FileRecord, ...}

-- Get the first file reference matching a glob pattern
local first_doc_file = aip.file.first("doc/**/*.md")          -- FileMeta | nil

-- Ensure a file exists by creating it if not found
local file_meta = aip.file.ensure_exists("./some/file.md", "optional content") -- FileMeta

-- Load markdown sections from a file
local sections = aip.file.load_md_sections("doc/readme.md", "# Summary") -- {MdSection, ...}
```

> Note: All relative paths are relative to the workspace directory, which is the parent directory of the `.aipack/` folder.

### aip.file.load

```lua
-- API Signature
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord
```

Load a File Record object with its content.

**Arguments**

- `rel_path: string`: The relative path to the file.
- `options?: {base_dir: string}`: Optional table with `base_dir` key to specify the base directory.

**Returns**

[FileRecord](#filerecord)

```ts
{
  path    : string,  // The path to the file
  content : string,  // The text content of the file
  name    : string,  // The name of the file
  stem    : string,  // The stem of the file (name without extension)
  ext     : string   // The extension of the file
}
```

**Example**

```lua
local file = aip.file.load("doc/README.md")
-- file.content contains the text content of the file
```

**Error**

Returns an error if the file does not exist or cannot be read.

### aip.file.save

```lua
-- API Signature
aip.file.save(rel_path: string, content: string)
```

Save a File Content into a path.

**Arguments**

- `rel_path: string`: The relative path to the file.
- `content: string`: The content to write to the file.

**Returns**

Does not return anything.

**Example**

```lua
aip.file.save("doc/README.md", "Some very cool documentation")
```

**Error**

Returns an error if the file cannot be written, or if trying to save outside of workspace.

### aip.file.append

```lua
-- API Signature
aip.file.append(rel_path: string, content: string)
```

Append content to a file at a specified path.

**Arguments**

- `rel_path: string`: The relative path to the file.
- `content: string`: The content to append to the file.

**Returns**

Does not return anything.

**Example**

```lua
aip.file.append("doc/README.md", "Appended content to the file")
```

**Error**

Returns an error if the file cannot be opened or written to.

### aip.file.ensure_exists

```lua
-- API Signature
aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty: boolean}): FileMeta
```

Ensure a file exists at the given path, and if not create it with an optional content.

**Arguments**

- `path: string`: The relative path to the file.
- `content?: string`: Optional content to write to the file if it does not exist.
- `options?: {content_when_empty: boolean}`: Optional flags to set content only if the file is empty.

**Returns**

[FileMeta](#filemeta)

```ts
{
  path : string,  // The path to the file
  name : string,  // The name of the file
  stem : string,  // The stem of the file (name without extension)
  ext  : string   // The extension of the file
}
```

**Example**

```lua
local file_meta = aip.file.ensure_exists("doc/README.md", "Initial content")
```

**Error**

Returns an error if the file cannot be created or written to.

### aip.file.list

```lua
-- API Signature
aip.file.list(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): list<FileMeta>
```

List a set of file reference (no content) for a given glob.

**Arguments**

- `include_globs: string | list`: A glob pattern or a list of glob patterns to include files.
- `options?: {base_dir: string, absolute: boolean}`: Optional table with `base_dir` and `absolute` keys.

**Returns**

[FileMeta](#filemeta)

```ts
// An array/table of FileMeta
{
  path : string,  // The path to the file
  name : string,  // The name of the file
  stem : string,  // The stem of the file (name without extension)
  ext  : string   // The extension of the file
}
```

**Example**

```lua
local all_doc_files = aip.file.list("doc/**/*.md", {base_dir = "src"})
```

**Error**

Returns an error if the glob pattern is invalid or the files cannot be listed.

### aip.file.list_load

```lua
-- API Signature
aip.file.list_load(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): list<FileRecord>
```

List a set of file reference for a given glob and load their content.

**Arguments**

- `include_globs: string | list`: A glob pattern or a list of glob patterns to include files.
- `options?: {base_dir: string, absolute: boolean}`: Optional table with `base_dir` and `absolute` keys.

**Returns**

[FileRecord](#filerecord)

```ts
// An array/table of FileRecord
{
  path    : string,  // The path to the file
  name    : string,  // The name of the file
  stem    : string,  // The stem of the file (name without extension)
  ext     : string,  // The extension of the file
  content : string   // The content of the file
}
```

**Example**

```lua
local all_doc_files = aip.file.list_load("doc/**/*.md", {base_dir = "src"})
```

**Error**

Returns an error if the glob pattern is invalid or the files cannot be listed or loaded.

### aip.file.first

```lua
-- API Signature
aip.file.first(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): FileMeta | nil
```

Return the first FileMeta or nil.

**Arguments**

- `include_globs: string | list`: A glob pattern or a list of glob patterns to include files.
- `options?: {base_dir: string, absolute: boolean}`: Optional table with `base_dir` and `absolute` keys.

**Returns**

[FileMeta](#filemeta) or nil

```ts
// FileMeta or nil
{
  path : string,  // The path to the file
  name : string,  // The name of the file
  stem : string,  // The stem of the file (name without extension)
  ext  : string   // The extension of the file
}
```

**Example**

```lua
local first_doc_file = aip.file.first("doc/**/*.md", {base_dir = "src"})
if first_doc_file then
  local file = aip.file.load(first_doc_file.path)
end
```

**Error**

Returns an error if the glob pattern is invalid or the files cannot be listed.

### aip.file.load_md_sections

```lua
-- API Signature
aip.file.load_md_sections(path: string, headings?: string | list): list
```

Load markdown sections from a file, optionally filtering by headings.

**Arguments**

- `path: string`: Path to the markdown file.
- `headings?: string | list`: Optional string or list of strings representing the headings to filter by.

**Returns**

[MdSection](#mdsection)

```ts
// Array/Table of MdSection
{
  content: string,    // Content of the section
  heading?: {         // Heading information (optional)
    content: string,  // Heading content
    level: number,    // Heading level (e.g., 1 for #, 2 for ##)
    name: string      // Heading name
  }
}
```

**Example**

```lua
-- Load all sections from a file
local all_sections = aip.file.load_md_sections("doc/readme.md")

-- Load only sections with the heading "# Summary"
local summary_section = aip.file.load_md_sections("doc/readme.md", "# Summary")

-- Load sections with multiple headings
local sections = aip.file.load_md_sections("doc/readme.md", {"# Summary", "## Details"})
```

### aip.file.load_md_split_first

```lua
-- API Signature
aip.file.load_md_split_first(path: string): {before: string, first: {content: string, heading: {content: string, level: number, name: string}}, after: string}
```

Splits a markdown file into three parts: content before the first heading, the first heading and its content, and the rest of the file.

**Arguments**

- `path: string`: Path to the markdown file.

**Returns**

```ts
{
  before: string,       // Content before the first heading
  first: {              // Information about the first section
    content: string,    // Content of the first section
    heading: {          // Heading of the first section
      content: string,  // Heading content
      level: number,    // Heading level (e.g., 1 for #, 2 for ##)
      name: string      // Heading name
    }
  },
  after: string         // Content after the first section
}
```

**Example**

```lua
local split = aip.file.load_md_split_first("doc/readme.md")
print(split.before)        -- Content before the first heading
print(split.first.content) -- Content of the first section
print(split.after)         -- Content after the first section
```

## aip.path

Functions for path manipulation and checking.

```lua
-- Check if a path exists
local exists = aip.path.exists("doc/some-file.md")         -- bool

-- Check if a path is a file
local is_file = aip.path.is_file("doc/some-file.md")       -- bool

-- Check if a path is a directory
local is_dir = aip.path.is_dir("doc/")                     -- bool

-- Get the parent directory of a path
local parent_dir = aip.path.parent("doc/some-file.md")     -- string

-- Split for parent and filename
local parent_dir, file_name = aip.path.split("path/to/some-file.md") -- parent, file
-- returns "path/to", "some-file.md"

-- Join path
local path = aip.path.join("path", "to", "some-file.md")   -- string
-- "path/to/some-file.md"
```

> Note: All relative paths are relative to the workspace directory, which is the parent directory of the `.aipack/` folder.

### aip.path.exists

```lua
-- API Signature
aip.path.exists(path: string): boolean
```

Checks if the specified path exists.

**Arguments**

- `path: string`: The path to check.

**Returns**

Returns `true` if the path exists, `false` otherwise.

### aip.path.is_file

```lua
-- API Signature
aip.path.is_file(path: string): boolean
```

Checks if the specified path is a file.

**Arguments**

- `path: string`: The path to check.

**Returns**

Returns `true` if the path is a file, `false` otherwise.

### aip.path.is_dir

```lua
-- API Signature
aip.path.is_dir(path: string): boolean
```

Checks if the specified path is a directory.

**Arguments**

- `path: string`: The path to check.

**Returns**

Returns `true` if the path is a directory, `false` otherwise.

### aip.path.diff

```lua
-- API Signature
aip.path.diff(file_path: string, base_path: string): string
```

Do a diff between two paths, giving the relative path.

**Arguments**

- `file_path: string`: The file path.
- `base_path: string`: The base path.

**Returns**

Returns the relative path from the base path to the file path.

### aip.path.parent

```lua
-- API Signature
aip.path.parent(path: string): string | nil
```

Returns the parent directory of the specified path, or nil if there is no parent.

**Arguments**

- `path: string`: The path to get the parent directory from.

**Returns**

Returns the parent directory of the path, or `nil` if the path has no parent.

### aip.path.split

```lua
-- API Signature
aip.path.split(path: string): string, string
```

Split path into parent, filename.

**Arguments**

- `path: string`: The path to split.

**Returns**

```ts
{
  parent: string,
  filename: string
}
```

Returns the parent and filename of the path.

### aip.path.join

```lua
-- API Signature
aip.path.join(paths: string | table): string
```

Returns the path, with paths joined without OS normalization.

> NOTE: Currently, `aip.path.join` uses `aip.path.join_os_non_normalized`. This might change in the future.

**Arguments**

- `paths: string | table`: A string or a table of strings representing the paths to join.

**Example**

```lua
-- Table example:
local paths = {"folder", "subfolder", "file.txt"}
local full_path = aip.path.join(paths)

-- Arg example:
local full_path = aip.path.join("folder", "subfolder", "file.txt")
```

**Returns**

Returns the joined path as a string.

### aip.path.join_os_normalized

```lua
-- API Signature
aip.path.join_os_normalized(paths: string | table): string
```

Joins path components with OS normalization.

**Arguments**

- `paths: string | table`: A string or a table of strings representing the paths to join.

**Returns**

Returns the joined path as a string with OS normalization.

### aip.path.join_os_non_normalized

```lua
-- API Signature
aip.path.join_os_non_normalized(paths: string | table): string
```

Returns the path, with paths joined without OS normalization.

**Arguments**

- `paths: string | table`: A string or a table of strings representing the paths to join.

**Returns**

Returns the joined path as a string.

> NOTE: The reason why normalized is prefixed with `_os_` is because there is another type of normalization that removes the "../". There are no functions for this yet, but keeping the future open.

## aip.text

Text manipulation functions.

```lua
local trimmed = aip.text.trim(content)        -- string
local trimmed = aip.text.trim_start(content)  -- string
local trimmed = aip.text.trim_end(content)    -- string

-- Truncate content to a maximum length
local truncated_content = aip.text.truncate(content, 100, "...")  -- string

-- Ensure prefix and suffix
local ensured = aip.text.ensure(content, {prefix = "./", suffix = ".md"})  -- string

-- Ensure content ends with a single newline
local normalized_content = aip.text.ensure_single_ending_newline(content)  -- string

-- Split the first occurrence of a separator
local first, second = aip.text.split_first(content, "===\n")  -- string, string

-- Remove the first line from content
local content_without_first_line = aip.text.remove_first_line(content)  -- string

-- Remove the last line from content
local content_without_last_line = aip.text.remove_last_line(content)  -- string

-- Remove the first n lines from content
local content_without_first_lines = aip.text.remove_first_lines(content, 2)  -- string

-- Remove the last n lines from content
local content_without_last_lines = aip.text.remove_last_lines(content, 2)  -- string

-- Replace markers in content with new sections
local updated_content = aip.text.replace_markers(content, new_sections)  -- string

-- Extract line blocks
local line_blocks, remain = aip.text.extract_line_blocks(content, {starts_with = ">", extrude = "content"})  -- table, string
```

### aip.text.escape_decode

```lua
-- API Signature
aip.text.escape_decode(content: string): string
```

Some LLMs HTML-encode their responses. This function returns `content`, HTML-decoded.

**Arguments**

- `content: string`: The content to process.

**Returns**

The HTML-decoded string.

### aip.text.escape_decode_if_needed

```lua
-- API Signature
aip.text.escape_decode_if_needed(content: string): string
```

Only escape if needed. Right now, the test only tests `&lt;`.

Some LLMs HTML-encode their responses. This function returns `content` after selectively decoding certain HTML tags.

**Arguments**

- `content: string`: The content to process.

**Returns**

The HTML-decoded string.

### aip.text.split_first

```lua
-- API Signature
aip.text.split_first(content: string, sep: string): (string, string|nil)
```

Splits a string into two parts based on the first occurrence of a separator.

**Arguments**

- `content: string`: The string to split.
- `sep: string`: The separator string.

**Returns**

A tuple containing the first part and the second part (or nil if no match).

**Example**

```lua
local content = "some first content\n===\nsecond content"
local first, second = aip.text.split_first(content,"===\n")
-- first  = "some first content\n"
-- second = "second content"
-- NOTE: When no match, second is nil.
--       If matched, but nothing after, second is ""
```

### aip.text.remove_first_line

```lua
-- API Signature
aip.text.remove_first_line(content: string): string
```

Returns `content` with the first line removed.

**Arguments**

- `content: string`: The content to process.

**Returns**

The string with the first line removed.

### aip.text.remove_first_lines

```lua
-- API Signature
aip.text.remove_first_lines(content: string, n: int): string
```

Returns `content` with the first `n` lines removed.

**Arguments**

- `content: string`: The content to process.
- `n: int`: The number of lines to remove.

**Returns**

The string with the first `n` lines removed.

### aip.text.remove_last_line

```lua
-- API Signature
aip.text.remove_last_line(content: string): string
```

Returns `content` with the last line removed.

**Arguments**

- `content: string`: The content to process.

**Returns**

The string with the last line removed.

### aip.text.remove_last_lines

```lua
-- API Signature
aip.text.remove_last_lines(content: string, n: int): string
```

Returns `content` with the last `n` lines removed.

**Arguments**

- `content: string`: The content to process.
- `n: int`: The number of lines to remove.

**Returns**

The string with the last `n` lines removed.

### aip.text.trim

```lua
-- API Signature
aip.text.trim(content: string): string
```

Trims leading and trailing whitespace from a string.

**Arguments**

- `content: string`: The string to trim.

**Returns**

The trimmed string.

### aip.text.trim_start

```lua
-- API Signature
aip.text.trim_start(content: string): string
```

Trims leading whitespace from a string.

**Arguments**

- `content: string`: The content to process.

**Returns**

The trimmed string.

### aip.text.trim_end

```lua
-- API Signature
aip.text.trim_end(content: string): string
```

Trims trailing whitespace from a string.

**Arguments**

- `content: string`: The content to process.

**Returns**

The trimmed string.

### aip.text.truncate

```lua
-- API Signature
aip.text.truncate(content: string, max_len: int, ellipsis?: string): string
```

Returns `content` truncated to a maximum length of `max_len`.
If the content exceeds `max_len`, it appends the optional `ellipsis` string to indicate truncation.

**Arguments**

- `content: string`: The content to truncate.
- `max_len: int`: The maximum length of the truncated string.
- `ellipsis: string` (optional): The string to append if truncation occurs.

**Returns**

The truncated string.

### aip.text.replace_markers

```lua
-- API Signature
aip.text.replace_markers(content: string, new_sections: array): string
```

Replaces markers in `content` with corresponding sections from `new_sections`.
Each section in `new_sections` can be a string or a map containing a `.content` string.

**Arguments**

- `content: string`: The content containing markers to replace.
- `new_sections: array`: An array of strings to replace the markers.

**Returns**

The string with markers replaced by the corresponding sections.

### aip.text.ensure

```lua
-- API Signature
aip.text.ensure(content: string, {prefix?: string, suffix?: string}): string
```

Ensure the content start and/or end with the text given in the second argument dictionary.

**Arguments**

- `content: string`: The content to ensure.
- `options: table`: A table with optional `prefix` and `suffix` keys.

**Returns**

The ensured string.

### aip.text.ensure_single_ending_newline

```lua
-- API Signature
aip.text.ensure_single_ending_newline(content: string): string
```

Ensures that `content` ends with a single newline character.
If `content` is empty, it returns a newline character.

**Arguments**

- `content: string`: The content to process.

**Returns**

The string with a single ending newline.

### aip.text.extract_line_blocks

```lua
-- API Signature
aip.text.extract_line_blocks(content: string, {starts_with: string, extrude?: "content", first?: number}): table, string | nil
```

Extracts line blocks from `content` using the given options.

**Arguments**

- `content: string`: The content to extract line blocks from.
- `options: table`: A table with the following keys:
  - `starts_with: string` (required): The prefix that indicates the start of a line block.
  - `extrude: "content"` (optional): If set to `"content"`, the remaining content after extracting the blocks is returned.
  - `first: number` (optional): Limits the number of blocks returned.

**Returns**

A tuple containing:
- `blocks: table`: A table containing the extracted line blocks.
- `extruded: string | nil`: The remaining content after extracting the blocks, if `extrude` is set to `"content"`. Otherwise, `nil`.

## aip.md

Markdown handling functions.

```lua
-- Extract all blocks
local blocks = aip.md.extract_blocks(md_content)  -- Vec<MdBlock>

-- Extract blocks for specific language
local blocks = aip.md.extract_blocks(md_content, "lua")  -- Vec<MdBlock>

-- Extract blocks with options
local blocks, content = aip.md.extract_blocks(md_content, {lang = "lua", extrude = "content"})  -- Vec<MdBlock>, string

-- Extract, parse, and merge the meta, and return the value and remaining text
local meta, remain = aip.md.extract_meta(md_content)  -- table, string

-- Get content from outer code block or return raw content
local content = aip.md.outer_block_content_or_raw(content)  -- string
```

### aip.md.extract_blocks

```lua
-- API Signatures
aip.md.extract_blocks(md_content: string): Vec<MdBlock>
aip.md.extract_blocks(md_content: string, lang: string): Vec<MdBlock>
aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): Vec<MdBlock>, string
```

Extract code blocks from markdown content.

**Arguments**

- `md_content: string`: The markdown content to process.
- `options: string | table` (optional):
  - If a string, it's treated as the language filter.
  - If a table:
    - `lang: string` (optional): Filters blocks by this language.
    - `extrude: "content"` (optional): If present, extracts the content outside the blocks.

**Returns**

When `extrude = "content"` is not specified:

[MdBlock](#mdblock)

```ts
MdBlock[]
```

When `extrude = "content"` is specified:

```ts
[MdBlock[], string]
```

- `MdBlock[]`: A list of markdown blocks matching the specified criteria.
- `string`: The content of the markdown outside of the extracted blocks.

**Error**

Returns an error if the `extrude` option is not equal to `"content"`.

### aip.md.extract_meta

```lua
-- API Signature
aip.md.extract_meta(md_content: string): Table, string
```

Extract, parse, and merge the `#!meta` blocks, and return the value and the concatenated remaining text.

**Arguments**

- `md_content: string`: The markdown content to process.

**Returns**

```ts
[Table, string]
```

- `Table`: A Lua table containing the merged meta values from the meta blocks.
- `string`: The remaining content of the markdown after removing the meta blocks.

### aip.md.outer_block_content_or_raw

```lua
-- API Signature
aip.md.outer_block_content_or_raw(md_content: string): string
```

If content starts with ```, it will remove the first and last ```, and return the content in between.
Otherwise, it returns the original content.

**Arguments**

- `md_content: string`: The markdown content to process.

**Returns**

```ts
string
```

Returns the content within the outer code block if it exists; otherwise, returns the original markdown content.

> Note: This is useful in the GenAI context because often LLMs return a top block (e.g., markdown, Rust)
> And while it is better to try to handle this with the prompt, gpt-4o-mini or other models still put in markdown block

## aip.json

JSON parsing and stringification functions.

```lua
-- Parse a JSON string into a table
local obj = aip.json.parse('{"name": "John", "age": 30}')  -- Object (lua table)

-- Stringify a table into a JSON string
local json_str = aip.json.stringify(obj)  -- string

-- Stringify a table into a single-line JSON string
local json_line_str = aip.json.stringify_to_line(obj)  -- string
```

### aip.json.parse

```lua
-- API Signature
aip.json.parse(content: string): table
```

Parse a JSON string into a table that can be used in the Lua script.

**Example**

```lua
local json_str = '{"name": "John", "age": 30}'
local obj = aip.json.parse(json_str)
print(obj.name) -- prints "John"
```

**Returns**

Returns a table representing the parsed JSON.

**Error**

```ts
{
  error: string  // Error message from JSON parsing
}
```

### aip.json.stringify

```lua
-- API Signature
aip.json.stringify(content: table): string
```

Stringify a table into a JSON string with pretty formatting.
Convert a table into a JSON string with pretty formatting using tab indentation.

**Example**

```lua
local obj = {
    name = "John",
    age = 30
}
local json_str = aip.json.stringify(obj)
-- Result will be:
-- {
--     "name": "John",
--     "age": 30
-- }
```

**Returns**

Returns a formatted JSON string.

**Error**

```ts
{
  error: string  // Error message from JSON stringification
}
```

### aip.json.stringify_to_line

```lua
-- API Signature
aip.json.stringify_to_line(content: table): string
```

Stringify a table into a single line JSON string. Good for newline json.

**Example**

```lua
local obj = {
    name = "John",
    age = 30
}
local json_str = aip.json.stringify_to_line(obj)
-- Result will be:
-- {"name":"John","age":30}
```

**Returns**

Returns a single line JSON string.

**Error**

```ts
{
  error: string  // Error message from JSON stringification
}
```

## aip.lua

Lua value inspection functions.

```lua
-- Return a pretty string of a lua value
local dump = aip.lua.dump(some_var)  -- string
print(dump)
```

### aip.lua.dump

```lua
-- API Signature
aip.lua.dump(value: any): string
```

Dump a Lua value into its string representation.

**Arguments**

- `value`: The Lua value to be dumped.

**Returns**

A string representation of the Lua value.

**Example**

```lua
local tbl = { key = "value", nested = { subkey = 42 } }
print(aip.lua.dump(tbl))
```

**Error**

If the conversion fails, an error message is returned.

## aip.cmd

The `cmd` module exposes functions to execute system commands.

## Common Types

### FileRecord

```ts
{
  path    : string,  // The path to the file
  content : string,  // The text content of the file
  name    : string,  // The name of the file
  stem    : string,  // The stem of the file (name without extension)
  ext     : string   // The extension of the file
}
```

### FileMeta

```ts
{
  path : string,  // The path to the file
  name : string,  // The name of the file
  stem : string,  // The stem of the file (name without extension)
  ext  : string   // The extension of the file
}
```

### MdSection

```ts
{
  content: string,    // Content of the section
  heading?: {         // Heading information (optional)
    content: string,  // Heading content
    level: number,    // Heading level (e.g., 1 for #, 2 for ##)
    name: string      // Heading name
  }
}
```

### MdBlock

```ts
{
  content: string,  // Content of the block (without the backticks)
  lang: string,     // Language of the block (e.g., "lua", "rust", etc.)
  info: string      // Additional info (e.g., "mermaid", etc.)
}
