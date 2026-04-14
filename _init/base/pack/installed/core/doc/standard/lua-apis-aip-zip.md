## aip.zip

ZIP archive functions for creating archives, extracting files, reading archive text entries, and listing archive entry paths.

### Functions Summary

```lua
aip.zip.create(src_dir: string, dest_zip?: string): FileInfo
aip.zip.extract(src_zip: string, dest_dir?: string): FileInfo[]
```

### aip.zip.create

Create a ZIP archive from a directory.

```lua
-- API Signature
aip.zip.create(src_dir: string, dest_zip?: string): FileInfo
```

Creates a ZIP archive from the directory at `src_dir`.

If `dest_zip` is not provided, the destination defaults to a `.zip` file next to the source directory using the source directory name.

#### Arguments

- `src_dir: string`: The source directory to archive.
- `dest_zip?: string` (optional): The destination ZIP file path.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the created ZIP file.

#### Example

```lua
local zip_file = aip.zip.create("docs/site")
print(zip_file.path) -- e.g. "docs/site.zip"

local zip_file2 = aip.zip.create("docs/site", "build/site.zip")
print(zip_file2.name) -- e.g. "site.zip"
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
aip.zip.extract(src_zip: string, dest_dir?: string): FileInfo[]
```

Extracts the ZIP archive at `src_zip` into `dest_dir`.

If `dest_dir` is not provided, the destination defaults to a folder next to the ZIP file using the ZIP stem.

Only extracted file entries are returned, in archive order. Directory-only archive entries are not included.

#### Arguments

- `src_zip: string`: The source ZIP file path.
- `dest_dir?: string` (optional): The destination directory for extracted content.

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
```

#### Error

Returns an error if:
- The source ZIP file does not exist or cannot be read.
- The destination path is outside the allowed workspace or base directories.
- The ZIP archive contains unsafe entry paths.
- A file or directory cannot be created during extraction.

