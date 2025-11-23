## API Functions Summary

### CsvOptions

The existing `CsvOptions` type is extended to support both parsing and saving configuration.

```lua
-- API Signature
type CsvOptions = {
  -- Common
  delimiter?: string,
  quote?: string,
  escape?: string,
  trim_fields?: boolean,
  has_header?: boolean,       -- Parsing: First row is header (extracted). Writing: First input row is header (used).
  header_labels?: { [string]: string }, -- Map { key: label }. Read: Maps CSV header "label" to internal "key" (applied by aip.csv.parse and aip.file.load_csv so the returned headers can be used directly with aip.shape helpers). Write: Maps internal "key" to CSV header "label" (used by all aip.file.save_* helpers to return to the label format).

  -- Parsing specific (Existing)
  skip_empty_lines?: boolean,
  comment?: string,

  -- Writing specific (New)
  skip_header_row?: boolean,            -- Suppress header emission even if available
}
```

- `header_labels`: Optional map `{ key: label }` for renaming headers.
    - **Parsing:** If a CSV header matches a `label` (value) in the map, it is renamed to the corresponding `key`.
    - **Writing:** If a record key matches a `key` in the map, it is written to the CSV header as `label`.
- `has_header`: 
    - **Parsing:** If `true`, the first row is extracted as `headers` and removed from `rows`. 
    - **Writing:** If `true`, the first entry of a bare matrix (`any[][]`) is treated as the canonical header row and consumed before writing subsequent rows. Applies to matrix helpers only and is ignored by `aip.file.save_records_as_csv` as well as by structured `{ headers, rows }` payloads.
- `skip_header_row`: **Writing only**. Suppresses header emission even if it could be written, enabling reuse of the same options record in different contexts.

Matrix helpers ignore `has_header` whenever the payload already bundles `{ headers, rows }`.

The same `header_labels` map is therefore used across `aip.csv.parse(...)`, `aip.file.load_csv(...)`, and all of the save helpers: parsing/loading remap the headers to the internal keys so they slot directly into `aip.shape` helpers, while saving remaps those keys back to the configured label format.

### 1. aip.file.save_as_csv (Matrix Data Overwrite)

Performs a full overwrite save of CSV content from a list of value lists (matrix data).

```lua
-- API Signature
aip.file.save_as_csv(
  path: string,
  data: any[][] | { headers: string[], rows: any[][] },
  options?: CsvOptions
): FileInfo
```

Saves the data in `data` to the specified `path`. The helper accepts either a bare list of value lists (matrix data) or a structured `{ headers, rows }` table. Structured payloads use their embedded `headers` as the canonical keys and their `rows` as the body, while bare matrices rely on `has_header = true` to treat their first entry as the canonical header row (otherwise no header row is emitted). Header text still flows through `header_labels`, and `skip_header_row` suppresses emission regardless of how the keys were determined.

#### Arguments

- `path: string`: Path to the target CSV file, relative to the workspace root.
- `data: any[][] | { headers: string[], rows: any[][] }`: Provide either a bare matrix (list of value lists) or a structured table containing explicit `headers` and the `rows` to write. When the structured table is used, its headers override `CsvOptions.headers` and make `has_header` unnecessary.
- `options?: CsvOptions`: When passing a bare matrix, set `has_header = true` if the first entry contains canonical header keys that should be consumed, remap the visible labels via `header_labels`, reuse the serialization switches (`delimiter`, `quote`, `escape`, `trim_fields`), and set `skip_header_row = true` if you need to suppress headers. When supplying `{ headers, rows }`, only the shared serialization and labeling fields apply because the headers are already embedded and `has_header` is ignored. The shared `header_labels` map keeps the emitted header row aligned with the labels that `aip.csv.parse`/`aip.file.load_csv` already map for use with `aip.shape` helpers.

#### Returns

- `FileInfo`: Metadata about the saved file.

#### Error

Returns an error (Lua table `{ error: string }`) on conversion failure, directory creation failure, or file write/permission error.


### 2. aip.file.save_records_as_csv (Record List Overwrite)

Saves a list of records (tables with named keys) to a CSV file.

```lua
-- API Signature
aip.file.save_records_as_csv(
  path: string,
  records: table[],
  header_keys: string[], -- Mandatory list of keys to extract from records in order.
  options?: CsvOptions
): FileInfo
```

Saves the data in `records` (a list of tables/objects) to the specified `path`. Requires `header_keys` to define the keys to extract from records and their order. The output header row uses these keys by default, but you can map them to end-user labels via `options.header_labels`, or omit the header entirely with `skip_header_row`.

#### Arguments

- `path: string`: Path to the target CSV file, relative to the workspace root.
- `records: table[]`: List of records to write.
- `header_keys: string[]`: The mandatory list of keys to extract from each record, defining the output column order.
- `options?: CsvOptions`: Use `header_labels` to rename the rendered headers, `skip_header_row` to suppress the header even though `header_keys` are provided, and the serialization switches (`delimiter`, `quote`, `escape`, `trim_fields`) for formatting. The `has_header` field inside `CsvOptions` is ignored for this helper because `header_keys` already defines the canonical order. This reuses the same `header_labels` map so the saved header row matches the label form that parsing/loading exposed for downstream `aip.shape` helpers.

#### Returns

- `FileInfo`: Metadata about the saved file.

#### Error

Returns an error (Lua table `{ error: string }`) if `header_keys` is missing or invalid, conversion fails, or file write/permission error occurs.


### 3. aip.file.append_as_csv (Matrix Data Append)

Appends new CSV content from a list of value lists (matrix data) to an existing file.

```lua
-- API Signature
aip.file.append_as_csv(
  path: string,
  data: any[][] | { headers: string[], rows: any[][] },
  options?: CsvOptions
): FileInfo
```

Appends the data in `data` to the file at `path`. If the file does not exist, it is created. The helper accepts either a bare value matrix or a structured `{ headers, rows }` table. Structured payloads use their inline `headers` when a new file is created (unless `skip_header_row` is set), while bare matrices rely on `has_header = true` to treat their first entry as the canonical header row before appending the remaining rows. Subsequent rows are appended verbatim after serialization.

#### Arguments

- `path: string`: Path to the target CSV file, relative to the workspace root.
- `data: any[][] | { headers: string[], rows: any[][] }`: Provide either a bare matrix (list of value lists) or a structured table containing explicit `headers` plus the `rows` to append. Structured payloads bypass `CsvOptions.headers`/`has_header` because they already carry the canonical keys.
- `options?: CsvOptions`: For bare matrices, set `has_header = true` when the first entry carries header keys you want consumed before appending, remap display labels via `header_labels`, reuse the serialization switches, and rely on `skip_header_row` if you need to suppress header emission in a fresh file. When the payload already includes `{ headers, rows }`, only the labeling and serialization-related fields apply and `has_header` is ignored. This keeps the output labels in sync with the header format that `aip.csv.parse`/`aip.file.load_csv` produced for `aip.shape` helpers.

#### Returns

- `FileInfo`: Metadata about the file after append operation.

#### Error

Returns an error (Lua table `{ error: string }`) on conversion failure, directory creation failure, or file write/permission error.
