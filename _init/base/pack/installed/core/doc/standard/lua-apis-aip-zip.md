## aip.zip

ZIP archive functions for creating archives, extracting files, reading archive text entries, and listing archive entry paths.

### Functions Summary

```lua
aip.zip.create(src_dir: string, dest_zip?: string, options?: ZipOptions): FileInfo
aip.zip.extract(src_zip: string, dest_dir?: string, options?: ZipOptions): FileInfo[]
aip.zip.read_text(src_zip: string, content_path: string): string | nil
aip.zip.list(src_zip: string, options?: ZipOptions): string[]
```

### ZipOptions

Options table used for ZIP entry filtering in `aip.zip.create`, `aip.zip.extract`, and `aip.zip.list`.

```ts
{
  globs?: string[] // Include-only glob patterns matched against stored relative archive paths
}
```

When `options` is omitted, or `options.globs` is omitted or empty, ZIP behavior remains unchanged.

### aip.zip.create

Create a ZIP archive from a directory.

```lua
-- API Signature
aip.zip.create(src_dir: string, dest_zip?: string, options?: ZipOptions): FileInfo
```

Creates a ZIP archive from the directory at `src_dir`.

If `dest_zip` is not provided, the destination defaults to a `.zip` file next to the source directory using the source directory name.

#### Arguments

- `src_dir: string`: The source directory to archive.
- `dest_zip?: string` (optional): The destination ZIP file path.
- `options?: ZipOptions` (optional): ZIP creation options.
  - `globs?: string[]`: Include only files whose relative archive-style paths match at least one glob.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the created ZIP file.

#### Example

```lua
local zip_file = aip.zip.create("docs/site")
print(zip_file.path) -- e.g. "docs/site.zip"

local zip_file2 = aip.zip.create("docs/site", "build/site.zip")
print(zip_file2.name) -- e.g. "site.zip"

local zip_file3 = aip.zip.create("docs/site", "build/site.zip", {
  globs = { "**/*.html", "assets/**/*.css" }
})
```

#### Error

Returns an error if:
- The source directory does not exist or is not a directory.
- The destination path is outside the allowed workspace or base directories.
- The ZIP file cannot be created.

### aip.zip.extract

Extract a ZIP archive into a directory.

```lua
-- API Signature
aip.zip.extract(src_zip: string, dest_dir?: string, options?: ZipOptions): FileInfo[]
```

Extracts the ZIP archive at `src_zip` into `dest_dir`.

If `dest_dir` is not provided, the destination defaults to a folder next to the ZIP file using the ZIP stem.

Only extracted file entries are returned, in archive order. Directory-only archive entries are not included.

#### Arguments

- `src_zip: string`: The source ZIP file path.
- `options?: ZipOptions` (optional): ZIP listing options.
  - `globs?: string[]`: Return only archive entries whose stored relative paths match at least one glob.
- `dest_dir?: string` (optional): The destination directory for extracted content.
- `options?: ZipOptions` (optional): ZIP extraction options.
  - `globs?: string[]`: Extract and return only archive entries whose stored relative paths match at least one glob.

#### Returns

- `[FileInfo](#fileinfo)[]`: Metadata for the extracted files.

#### Example

```lua
local files = aip.zip.extract("build/site.zip")
for _, file in ipairs(files) do
  print(file.path) -- e.g. "build/site/index.html"
end

local files2 = aip.zip.extract("build/site.zip", "output/site")
for _, file in ipairs(files2) do
  print(file.name, file.size)
end

local html_files = aip.zip.extract("build/site.zip", "output/site", {
  globs = { "**/*.html" }
})
```

#### Error

Returns an error if:
- The source ZIP file does not exist or cannot be read.
- The destination path is outside the allowed workspace or base directories.
- The ZIP archive contains unsafe entry paths.
- A file or directory cannot be created during extraction.

### aip.zip.read_text

Read a UTF-8 text entry from inside a ZIP archive.

```lua
-- API Signature
aip.zip.read_text(src_zip: string, content_path: string): string | nil
```

Loads the archive entry at `content_path` from the ZIP file at `src_zip`.

If the requested archive entry does not exist, this function returns `nil`.

#### Arguments

- `src_zip: string`: The source ZIP file path.
- `content_path: string`: The path of the entry inside the ZIP archive.

#### Returns

- `string | nil`: The UTF-8 text content of the archive entry, or `nil` if the entry is not found.

#### Example

```lua
local manifest = aip.zip.read_text("bundle.zip", "manifest.json")
if manifest ~= nil then
  print(manifest)
end
```

#### Error

Returns an error if:
- The source ZIP file does not exist or cannot be read.
- The ZIP archive cannot be opened.
- The requested archive entry exists but is not valid UTF-8.
- The archive entry cannot be read.

### aip.zip.list

List archive entry paths from a ZIP archive.

```lua
-- API Signature
aip.zip.list(src_zip: string, options?: ZipOptions): string[]
```

Returns archive entry paths exactly as stored in the ZIP, in archive order.

Directory entries are included as-is when present in the archive, for example with a trailing `/`.

#### Arguments

- `src_zip: string`: The source ZIP file path.

#### Returns

- `string[]`: Archive entry paths exactly as stored in the ZIP.

#### Example

```lua
local entries = aip.zip.list("bundle.zip")
for _, entry in ipairs(entries) do
  print(entry)
end

local html_entries = aip.zip.list("bundle.zip", {
  globs = { "**/*.html" }
})
```

#### Error

Returns an error if:
- The source ZIP file does not exist or cannot be read.
- The ZIP archive cannot be opened.
- The archive entries cannot be enumerated.

