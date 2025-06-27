# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Multi-provider AI agent benchmarking support (OpenAI, Sambanova, Google Gemini)
- Tool execution testing capabilities (shell commands and web search)
- Session management with SQLite database
- Interactive CLI with colored output
- Session export functionality
- Persistent benchmark results

### Changed
- Initial release

### Deprecated
- None

### Removed
- None

### Fixed
- None

### Security
- None

## [0.1.0] - 2025-01-28

### Added
- Initial release of Agent Bench
- Support for OpenAI GPT-4 Turbo benchmarking
- Support for Sambanova Meta-Llama-3.2-1B-Instruct benchmarking
- Support for Google Gemini 2.0 Flash benchmarking
- Shell command execution testing with `[RUN_COMMAND <command>]`
- Web search functionality testing with `[SEARCH: query]`
- SQLite database for benchmark session persistence
- Session management (list, view, export)
- Colored terminal interface
- Environment variable configuration
- Comprehensive error handling

[Unreleased]: https://github.com/bniladridas/agent_bench/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bniladridas/agent_bench/releases/tag/v0.1.0 