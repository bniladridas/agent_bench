# Agent Bench

**This is a multi-provider AI chat agent that supports OpenAI, Sambanova, and Google Gemini, with the ability to execute shell commands and perform web searches through a command-line interface. It features persistent session storage using SQLite, allowing users to save, view, and export chat histories while maintaining full conversation context across different AI providers.**

Agent Bench is a comprehensive benchmarking platform for AI agents, designed to test and compare the capabilities of different large language models across multiple providers. It supports OpenAI, Sambanova, and Google Gemini APIs while providing tool execution capabilities and persistent session management.

Agent Bench should be used together with API keys from your preferred providers to benchmark AI agent performance, test tool execution capabilities, and compare results across different LLM providers.

## Installing

If you're new, try our getting started guide below.

You can install by cloning the repository and building with Cargo.

## Running

You can run the application just by executing: `cargo run`.

If you want to start benchmarking:

1. Set up your API keys in a `.env` file
2. Run `cargo build` to compile
3. Execute `cargo run` to start
4. Select your preferred AI provider for testing
5. Begin benchmarking with shell command and web search capabilities

Our installation guide below provides a slightly more detailed introduction as well as links to more information.

## Getting Started

1. Clone the repository:
```bash
git clone https://github.com/bniladridas/agent_bench
cd agent_bench
```

2. Install Rust dependencies:
```bash
cargo build
```

3. Create a `.env` file with your API keys:
```env
OPENAI_API_KEY=your_openai_api_key_here
SAMBANOVA_API_KEY=your_sambanova_api_key_here
GEMINI_API_KEY=your_gemini_api_key_here
```

## Features

- **Multi-Provider Benchmarking**: Test OpenAI GPT-4 Turbo, Sambanova Meta-Llama-3.2-1B-Instruct, Google Gemini 2.0 Flash
- **Tool Execution Testing**: Benchmark shell commands with `[RUN_COMMAND <command>]` and web searches with `[SEARCH: query]`
- **Session Management**: SQLite database for persistent test results
- **Interactive CLI**: Colored terminal interface with session history
- **Export Functionality**: Export benchmark sessions to text files
- **Performance Metrics**: Compare response times and accuracy across providers

## API Providers

| Provider | Model | Base URL |
|----------|-------|----------|
| OpenAI | GPT-4 Turbo | `https://api.openai.com/v1/chat/completions` |
| Sambanova | Meta-Llama-3.2-1B-Instruct | `https://api.sambanova.ai/v1/chat/completions` |
| Google Gemini | Gemini 2.0 Flash | `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent` |

## Contributing

We'd love to have your help in making Agent Bench better. If you're interested, please read our guide to contributing.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Security

For information on reporting security vulnerabilities in Agent Bench, see SECURITY.md.

## FAQ

Our user FAQ has answers to many common questions about Agent Bench, from general questions to questions geared towards those that want to use.

There is also a FAQ for contributors to Agent Bench. 