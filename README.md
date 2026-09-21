# Flockwell

A sheep-management application under development. The current repository provides
a pure Rust domain library and a read-only Google Sheets audit CLI. Google Sheets
is the current data source; an HTTP API and frontend are not implemented yet.

## Workspace

- [`flockwell-domain`](flockwell-domain/src/lib.rs): animal types, tag validation,
  duplicate audits, and checked in-memory registry operations. No networking or
  persistence; it can compile for WebAssembly.
- [`flockwell-audit`](flockwell-audit/src/main.rs): reads Sheets, parses rows, and
  reports domain findings and input errors.
- [`registry tests`](flockwell-domain/tests/registry.rs): examples of creation,
  updates, validation previews, and rejected-operation behavior.

## Current tag rules

All tag types trim surrounding whitespace and reject blank strings. Animal tag
fields are optional: `None` is allowed, and the Sheets parser maps blank cells to
`None`. Leading zeros are preserved; identifiers are stored as strings.

| Field | Present-value rule | Collection uniqueness |
| --- | --- | --- |
| `tag` | Exactly 15 ASCII digits (`0`–`9`), e.g. `250029228122437` | Unique |
| `uhf_tag` | Nonblank, converted to uppercase; no other format constraint | Unique after normalization |
| `tip_tag` | Nonblank, converted to uppercase; no other format constraint | Duplicates allowed for now |
| `uhf_tag_visual` | Nonblank, converted to uppercase; no other format constraint | Duplicates allowed |

Uppercase normalization uses Rust's `str::to_uppercase`; original casing is not
retained. Equality, ordering, and hashing use that stored value. Tag namespaces
are separate: a `tag` value may equal another animal's `uhf_tag` value. All supplied
animals, including animals with a disposition, participate in uniqueness checks.
The proposed 12-month tip-tag reuse rule awaits assignment dates and a defined
time-window policy.

## What the types guarantee

- **IDs:** `AnimalId`, `LambingId`, and `DispositionId` are distinct types. Their
  constructors require UUID version 7 and the RFC variant. They do not generate
  IDs or prove uniqueness or the existence of a referenced record.
- **`Animal`:** contains typed IDs and validated tag values. Its private fields
  prevent unrestricted mutation. It does not prove collection uniqueness,
  lambing/disposition existence, or chronological/business consistency. Comments
  are unrestricted strings, and life stage is currently only an optional override.
- **`AnimalRegistry`:** owns a collection with unique IDs, tags, and UHF tags.
  Loading audits the complete input before building indexes. Create/update
  operations check before changing state; rejected commands leave animals and
  indexes unchanged. Updates cannot change an animal's ID. A preview describes
  current state only, so applying a command validates again.

Registry guarantees apply only to its in-memory collection. It neither saves to
Sheets nor coordinates concurrent writers or checks data it has not loaded.
Lambing and disposition existence checks await their domain record types.

Auditing does not require a valid registry. The CLI can report duplicate records,
but currently rejects an entire row after its first parsing error. Duplicate
checks therefore cover only successfully parsed animals; rejected rows make the
audit incomplete.

## Development checks

CI uses Rust **1.90.0** and the workspace `Cargo.lock`. With rustup installed,
prepare the same toolchain:

```sh
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
```

Run these commands from the repository root:

```sh
cargo +1.90.0 test --workspace --locked
cargo +1.90.0 fmt --all --check
cargo +1.90.0 clippy --workspace --all-targets --locked -- -D clippy::all
cargo +1.90.0 build -p flockwell-domain --target wasm32-unknown-unknown --locked
```

Tests include unit, integration, and documentation tests. Clippy findings fail
CI; compiler warnings such as the currently unused `fetch_animal_headers` remain
visible without failing that check. The WASM build checks domain compilation,
not browser execution or JavaScript bindings. No Google credentials are required.
The [CI workflow](.github/workflows/ci.yml) runs on pull requests and pushes to
`main`, and can also be started manually. Update its toolchain pin and these
commands together when upgrading Rust.

## Run the Sheets audit

Copy `.env.example` to `.env`. Set `FLOCKWELL_GOOGLE_CREDENTIALS_JSON` to the service
account JSON **content**, not a file path, using valid Bash quoting. Share the
spreadsheet with that service account. The spreadsheet ID and `Animals` tab are
currently configured in [`google_sheets.rs`](flockwell-audit/src/google_sheets.rs).
Then run `bash run.sh`. The script sources `.env` as local Bash configuration.

Exit codes: `0` = audit passed, `1` = duplicate findings, `2` = configuration,
fetching, header, or row errors (including incomplete audits).

The CLI currently reads Sheets only. `json_input.rs` is not wired into a compiled
module, and `animals.json` contains old five-digit sample tags; neither is a
working offline CLI input path yet.
