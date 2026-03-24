# Unpack Command Specification

This document defines the intent, code design, and notable implementation considerations for the `aip unpack` command.

## Intent

The `aip unpack` command creates a workspace-editable copy of a repo pack inside the workspace custom packs area.

It fills the workflow gap between:

- `aip pack`, which creates a `.aipack` archive
- `aip install`, which installs a pack into the base installed cache
- `aip unpack`, which materializes a pack into the workspace for local customization

After unpacking, normal pack resolution should prefer the unpacked workspace custom copy over the installed base copy.

### Command summary

Command form:

    aip unpack namespace@name
    aip unpack namespace@name --force

Current scope:

- non-TUI CLI command
- accepts only full repo-style pack identity
- unpacks into workspace custom packs area
- may use installed pack contents or a newer downloaded archive, depending on version comparison

### User contract

Current argument shape:

- `pack_ref: String`
- `force: bool`

Behavioral contract:

- `pack_ref` must be a full `namespace@name`
- sub-path forms are rejected
- scoped variants such as `$base` or `$workspace` are rejected
- local `.aipack` files are not supported by this command

### Destination and workspace model

The unpack destination is the workspace custom pack location:

- destination root:
  - `.aipack/pack/custom/`
- destination pack path:
  - `.aipack/pack/custom/<namespace>/<name>`

`aip unpack` requires an existing workspace `.aipack/` directory.

If the current directory is not inside a valid workspace, the command must fail and instruct the user to run:

    aip init .

This is intentional because unpack is explicitly workspace-targeted and should not create workspace structure implicitly.

### Source selection intent

The unpack command chooses the best source using installed and remote state:

- if the pack is installed locally, inspect its installed version from `pack.toml`
- check the repo latest version when possible
- if the remote version is newer than the installed version, use the downloaded remote archive as the unpack source
- if the installed version is current or newer, use the installed directory as the unpack source
- if nothing is installed locally, download from the repo and unpack that archive
- if the remote version cannot be determined but the installed copy exists, use the installed copy
- if the installed copy exists but does not expose a parseable version and the remote does, prefer remote

This keeps the command practical:

- reuse local installed contents when already current
- still allow workspace unpack to pick up a newer repo version without installing it into base installed

### Overwrite behavior

Default behavior:

- if the destination already exists, fail with a clear error

Force behavior:

- `--force` allows replacement
- when forced, the whole destination directory is trashed first
- replacement is full-directory replacement, not merge or overlay

This prevents stale files and protects local edits unless the user explicitly requests replacement.

## Code Design

### Executor wiring

The executor path for this command is:

- `src/exec/cli/args.rs`
- `src/exec/event_action.rs`
- `src/exec/executor.rs`
- `src/exec/exec_cmd_unpack.rs`
- `src/exec/packer/unpacker_impl.rs`

The command is exposed as:

- `CliCommand::Unpack(UnpackArgs)` in `src/exec/cli/args.rs`
- `ExecActionEvent::CmdUnpack(UnpackArgs)` in `src/exec/event_action.rs`

Execution flow:

1. CLI parses `aip unpack ...`
2. CLI maps to `ExecActionEvent::CmdUnpack`
3. `Executor` dispatches to `exec_unpack`
4. `exec_unpack` calls `unpack_pack`
5. unpack logic performs validation, source selection, and materialization
6. command prints result details

### Thin command entrypoint

The CLI-facing command entrypoint is:

- `src/exec/exec_cmd_unpack.rs`

Responsibilities:

- receive `UnpackArgs`
- publish user-facing progress and completion messages
- call the domain unpack implementation
- return the unpack result

The command layer stays thin and does not own repository resolution or file-copy logic.

### Domain implementation

The core implementation is:

- `src/exec/packer/unpacker_impl.rs`

Primary responsibilities:

- validate unpack input
- enforce workspace presence
- compute destination path
- enforce overwrite policy
- determine source from installed and remote state
- copy installed pack contents when selected
- download and unzip repo archive when selected
- return unpack metadata to the caller

Current result model:

- `UnpackedPack`
  - `namespace`
  - `name`
  - `dest_path`
  - `source`

### Shared support

Shared repository and download logic lives in:

- `src/exec/packer/support.rs`

This allows install and unpack to share:

- repo metadata lookup
- archive download flow
- version comparison helpers
- pack archive inspection support

Relevant helpers include:

- `fetch_repo_latest_toml`
- `build_repo_pack_url`
- `download_from_repo`
- `fetch_repo_latest_version`

This separation avoids duplicating registry-fetch logic inside unpack-specific code.

## Notable Implementation Considerations and Subsystem Relations

### Validation boundaries

The command currently enforces the following validation rules:

- must be a full `namespace@name`
- must not contain `/`
- must not contain `$`
- workspace `.aipack/` must exist
- destination must not already exist unless `--force` is provided

The command does not currently support:

- local archive paths
- unpacking from arbitrary URLs
- unpacking pack sub-paths
- merge updates into an existing destination

### Relation to pack precedence and workspace behavior

After unpack, the workspace custom copy should take precedence over:

- base custom
- base installed

This is a core part of the feature intent. The command is not just extracting files, it is creating a workspace override that becomes the active version for that pack inside the current workspace.

### Relation to install and repository subsystems

Unpack is closely related to install, but it has a different end state:

- install materializes into the base installed cache
- unpack materializes into workspace custom

The two commands should share repository metadata and download logic where practical, while keeping unpack-specific destination and overwrite behavior in `unpacker_impl.rs`.

### Archive extraction safety

When unpacking from a downloaded archive, zip entry names must not be trusted blindly.

The extraction layer must reject unsafe archive paths such as:

- absolute paths
- Windows drive-prefixed absolute paths
- path traversal entries containing `..`

This safety behavior is enforced in:

- `src/support/zip.rs`

### User-visible output

The command should produce plain operational output, consistent with other CLI commands, including:

- the pack ref being unpacked
- which source was used:
  - installed copy
  - downloaded from repo
- namespace and name
- destination path
- a note that workspace custom packs now take precedence

### Current test coverage

The implemented tests cover:

- source selection behavior across installed and remote version combinations
- installed version reading from `pack.toml`
- recursive directory copy behavior
- pack identity validation expectations
- destination exists failure without `--force`
- force replacement removing old destination content

Relevant files include:

- `src/exec/packer/unpacker_impl.rs`
- `src/_tests/tests_unpacker_impl.rs`

### Non-goals and future extension options

The current version intentionally does not include:

- support for unpacking local `.aipack` files
- support for unpacking HTTP URLs directly
- support for scoped pack refs
- automatic workspace initialization
- merge-based refresh of existing unpacked directories
- automatic installation into base installed as part of unpack

Possible future enhancements, if needed:

- richer source reporting, including version numbers
- optional dry-run mode
- optional explicit source selection such as installed-only or remote-only
- local archive unpack support if a distinct workflow is desired
- smarter update messaging when remote is newer than installed
- integration with docs and README command references

