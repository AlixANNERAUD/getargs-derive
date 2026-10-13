# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0](https://github.com/AlixANNERAUD/getargs-derive/releases/tag/v0.1.0) - 2026-10-13

### Added

- add argument relations ([#31](https://github.com/AlixANNERAUD/getargs-derive/pull/31))
- support argument aliases ([#30](https://github.com/AlixANNERAUD/getargs-derive/pull/30))
- support environment variable fallbacks ([#32](https://github.com/AlixANNERAUD/getargs-derive/pull/32))
- add ValueEnum derive ([#33](https://github.com/AlixANNERAUD/getargs-derive/pull/33))
- add value validation ([#35](https://github.com/AlixANNERAUD/getargs-derive/pull/35))
- extend generated help customization ([#36](https://github.com/AlixANNERAUD/getargs-derive/pull/36))
- [**breaking**] support hyphen values, negative numbers and last positionals ([#37](https://github.com/AlixANNERAUD/getargs-derive/pull/37))
- support flattened argument structs ([#38](https://github.com/AlixANNERAUD/getargs-derive/pull/38))
- generate shell completions and man pages ([#39](https://github.com/AlixANNERAUD/getargs-derive/pull/39))
- support repeated values ([#40](https://github.com/AlixANNERAUD/getargs-derive/pull/40))
- add subcommands ([#41](https://github.com/AlixANNERAUD/getargs-derive/pull/41))
- [**breaking**] support non-UTF-8 arguments ([#42](https://github.com/AlixANNERAUD/getargs-derive/pull/42))

### Changed

- [**breaking**] remove legacy attribute syntax ([#34](https://github.com/AlixANNERAUD/getargs-derive/pull/34))

### Documentation

- rewrite README and crate metadata for discoverability ([#43](https://github.com/AlixANNERAUD/getargs-derive/pull/43))

### Miscellaneous

- check pull request titles follow Conventional Commits ([#29](https://github.com/AlixANNERAUD/getargs-derive/pull/29))

### Other

- Initial commit
- Initialize project structure with Cargo configuration and macros
- Add error handling and update dependencies in Cargo files
- Improve error documentation in Error enum for clarity on argument parsing issues
- Update README and lib.rs for improved documentation and structure
- Add comprehensive tests for argument parsing and error handling
- Refactor error handling in Error enum for improved clarity and detail on invalid options
- Enhance error handling in tests and enforce documentation requirements
- Update Cargo.toml with project metadata and configuration
- Update README.md to enhance error handling and improve documentation clarity
- Add GitHub Actions workflow for CI checks, including Clippy, documentation, and coverage
- Update README.md to add Codecov badge and improve error handling descriptions
- Add `from_args` convenience method for CLI argument parsing and update tests
- Add authors field to Cargo.toml files for package metadata
- Add formatting check step to CI workflow
- Refactor error handling in CLI argument parsing to improve clarity and add new error variants
- Align GetArgs attributes with Clap conventions
- Add automated release workflow with release-plz
- Add generated help for GetArgs
- Support Option<T> fields
- Add version flag support
- Move unit tests into per-topic integration tests
- Split README into per-topic guide pages
- Dispatch arg and command keys through sorted registries
- Add Count and SetFalse flag actions ([#26](https://github.com/AlixANNERAUD/getargs-derive/pull/26))
- Report offending arguments in parse errors ([#27](https://github.com/AlixANNERAUD/getargs-derive/pull/27))
