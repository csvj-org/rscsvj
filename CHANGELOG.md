# Changelog

All notable changes to this project will be documented in this file. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project does not follow Semantic Versioning until 1.0.0; before
then breaking changes may occur in any release.

## [Unreleased]

### Added

- Initial repository scaffolding: Cargo crate `rscsvj`, MIT/Apache-2.0
  dual license per Rust convention, GHA CI matrix over Rust stable and
  the MSRV (1.74) with SHA-pinned third-party actions, Dependabot config
  for cargo and github-actions, three-section README.
- Public surface: `rscsvj::parse(&str) -> Result<Table, ParseError>` and
  `rscsvj::stringify(&Table) -> Result<String, WriteError>`, with
  `Table`, `Value`, `ParseError`, and `WriteError` re-exported.
- Reader and writer implementations with strict §1 enforcement from day
  one: empty input rejected, missing trailing newline rejected, ragged
  rows rejected, duplicate header names rejected (both reader and
  writer), value tokens validated against RFC 8259 via `serde_json`,
  non-finite numbers rejected by the writer. 47 unit tests across
  `tests/parse.rs` and `tests/stringify.rs`; all 25 vectors of
  `csvj-org/conformance@master` pass when run via the
  `CSVJ_CONFORMANCE_DIR` env var.
