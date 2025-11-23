# CSV API Specification

This specification covers the `aip.csv` module (string-based operations) and the CSV-related functions in the `aip.file` module (file-system operations).

## Types

### CsvOptions

Configuration for parsing and saving CSV data.

```lua
type CsvOptions = {
  -- Serialization & Parsing
  delimiter?: string,         -- Default: ",". The field delimiter character.
  quote?: string,             -- Default: "\"". The quote character.
  escape?: string,            -- Default: "\"". The escape character.
  trim_fields?: boolean,      -- Default: false. If true, trims whitespace from fields.

  -- Header & Row Handling
  has_header?: boolean,       -- Parsing: First row is header (extracted). Writing: First input row is header (used).
  header_labels?: { [string]: string }, -- Map { key: label }. See below.
  skip_empty_lines?: boolean, -- Default: true. Skips empty lines during parsing.
  comment?: string,           -- Default: nil. Comment character prefix (e.g., "#").

  -- Writing specific
  skip_header_row?: boolean,  -- Default: false. Suppress header emission even if available.
}
```

**`header_labels` behavior:**
- **Parsing/Loading:** If a CSV header matches a `label` (value) in the map, it is renamed to the corresponding `key` in the returned `headers` list.
- **Writing:** If a header key matches a `key` in the map, it is written to the CSV file as `label`.

**`has_header` behavior:**
- **Parsing:** If `true`, the first row is treated as headers and separated from `rows`.
- **Writing (Matrix data):** If `true`, the first row of the matrix is treated as the header row.
- **Writing (Structured data):** Ignored if data is `{ headers: ..., rows: ... }` (headers are explicit).

---

## Module `aip.csv`

Helper functions for in-memory CSV string manipulation.

### 1. aip.csv.parse_row

Parses a single CSV row string into a list of fields.

```lua
aip.csv.parse_row(row: string, options?: CsvOptions): string[]
```

- **options**: Uses `delimiter`, `quote`, `escape`, `trim_fields`. Ignores header/comment options.

### 2. aip.csv.parse

Parses a full CSV string content.

```lua
aip.csv.parse(content: string, options?: CsvOptions): CsvContent
```

- **Returns**: `CsvContent` object: `{ _type: "CsvContent", headers: string[], rows: string[][] }`.
- **options**: Uses all parsing options (`has_header`, `skip_empty_lines`, `comment`, etc.).

### 3. aip.csv.values_to_row

Converts a list of Lua values into a single CSV row string.

```lua
aip.csv.values_to_row(values: any[], options?: CsvOptions): string
```

- **values**: List of values. `nil` becomes empty string. Tables are JSON serialized.
- **options**: Uses `delimiter`, `quote`, `escape`.

### 4. aip.csv.value_lists_to_rows

Converts a list of rows (list of values) into a list of CSV strings.

```lua
aip.csv.value_lists_to_rows(value_lists: any[][], options?: CsvOptions): string[]
```

---

## Module `aip.file` (CSV Helpers)

File system operations for CSV files. Paths are relative to the workspace root.

### 1. aip.file.load_csv_headers

Loads only the headers from a CSV file.

```lua
aip.file.load_csv_headers(path: string): string[]
```

- Useful for inspecting columns without loading the entire file.
- Implicitly assumes `has_header = true`.
- Does not support options currently.

### 2. aip.file.load_csv

Loads a CSV file into a `CsvContent` object.

```lua
aip.file.load_csv(path: string, options?: CsvOptions): CsvContent
```

- **Returns**: `{ _type: "CsvContent", headers: string[], rows: string[][] }`.
- **options**: Defaults `has_header` to `true`.

### 3. aip.file.save_as_csv

Saves data to a CSV file (overwrites).

```lua
aip.file.save_as_csv(
  path: string,
  data: any[][] | { headers: string[], rows: any[][] },
  options?: CsvOptions
): FileInfo
```

- **data**:
    - **Matrix (`any[][]`)**: If `options.has_header` is true, the first row is treated as headers.
    - **Structured (`{headers, rows}`)**: Uses explicit headers.
- **options**: `skip_header_row` can be used to omit headers in output. `header_labels` apply.

### 4. aip.file.save_records_as_csv

Saves a list of record objects (tables with keys) to CSV.

```lua
aip.file.save_records_as_csv(
  path: string,
  records: table[],
  header_keys: string[],
  options?: CsvOptions
): FileInfo
```

- **header_keys**: Defines the order of columns and which keys to extract from records.
- **options**: `header_labels` can map internal keys to output column names.

### 5. aip.file.append_csv_headers

Appends a header row to a CSV file. If the file exists, it appends the headers unless `skip_header_row` is true. If the file does not exist, it creates it and writes headers unless `skip_header_row` is true.

```lua
aip.file.append_csv_headers(
  path: string,
  headers: string[],
  options?: CsvOptions
): FileInfo
```

- **headers**: The list of header keys/names to write.
- **options**: Supports `header_labels` for mapping internal keys to output labels, and `skip_header_row` to conditionally suppress writing headers.

### 6. aip.file.append_csv_rows

Appends multiple rows of values (matrix `any[][]`) to a CSV file. Creates the file if it doesn't exist.

```lua
aip.file.append_csv_rows(
  path: string,
  value_lists: any[][],
  options?: CsvOptions
): FileInfo
```

- **value_lists**: List of lists of values (`any[][]`).

Note that this function focuses solely on appending data rows. Options related to automatic header writing (`options.has_header`, `options.header_labels`) are ignored. Headers should be managed explicitly using `aip.file.append_csv_headers`.

### 7. aip.file.append_csv_row

Appends a single row (list of values) to a CSV file. Creates the file if it doesn't exist.

```lua
aip.file.append_csv_row(
  path: string,
  values: any[],
  options?: CsvOptions
): FileInfo
```

- **values**: A list of values to append as a single CSV record.
- **Header Handling**: This function does not handle headers. Headers should be managed explicitly using `aip.file.append_csv_headers`.
