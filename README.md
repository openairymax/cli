**Language:** English | [简体中文](README_zh.md)

# Airymax CLI

[![Version](https://img.shields.io/badge/version-0.1.1-5a6b7e)](https://atomgit.com/openairymax/cli)
[![License](https://img.shields.io/badge/license-AGPL--3.0+Apache--2.0-4a90d9)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> Official command-line interface for the [Airymax](https://atomgit.com/openairymax/airymaxhub) AI Agent Runtime Platform.
> One of the leaf repositories aggregated by the [sdk](https://atomgit.com/openairymax/sdk) management repo.
> Standalone Rust binary — talks to the Airymax Gateway over HTTP and does not link the language SDKs (no `agentrt-rs` dependency).

---

## Overview

The **Airymax CLI** (`agentrt`) is a Rust-built command-line tool for operating the Airymax runtime from the terminal. It covers the full agent lifecycle: project scaffolding, component creation, agent execution, configuration management, LLM provider management, prompt template management, marketplace search & install, deployment operations, and database migrations.

Runtime commands talk to the Airymax Gateway over HTTP (JSON-RPC 2.0) using the CLI's own `reqwest` client. The CLI is a standalone runtime tenant — it does not depend on or link the language SDKs (`agentrt-rs` / `agentrt-sdk-go` / etc.).

## Runtime Communication

The CLI talks to the runtime over the Gateway HTTP API (JSON-RPC 2.0) directly from its own HTTP client (`src/client.rs`, built on `reqwest`). It does not wrap or consume the language SDKs: there is no `agentrt-rs` dependency in `Cargo.toml`.

```
agentrt <command>
   └── src/client.rs — reqwest HTTP client
       ├── run      → task submission / agent execution
       ├── llm      → LLM provider management
       ├── market   → marketplace search & install
       └── deploy   → deployment & status
```

## Directory Structure

```
cli/
├── src/
│   ├── main.rs              # Entry point + CLI routing (clap)
│   ├── client.rs            # Gateway HTTP client
│   ├── templates.rs         # Project / component template generation
│   └── commands/
│       ├── mod.rs           # Command module exports
│       ├── init.rs          # agentrt init — project scaffolding
│       ├── create.rs        # agentrt create — agent/tool/plugin/prompt/skill
│       ├── run.rs           # agentrt run — run an agent (interactive / one-shot)
│       ├── config_cmd.rs    # agentrt config — configuration management
│       ├── llm.rs           # agentrt llm — LLM provider management
│       ├── prompt.rs        # agentrt prompt — prompt template management
│       ├── market.rs        # agentrt market — marketplace search & install
│       ├── deploy.rs        # agentrt deploy — deployment & status
│       └── db.rs            # agentrt db — database migration management
├── Cargo.toml               # Crate manifest (agentrt-cli, binary: agentrt)
└── README.md                # This file
```

## Upstream & Downstream Dependencies

### Upstream

- **Runtime**: Connects to a running Airymax / AgentRT instance (`gateway_d` / Gateway HTTP API) over HTTP and JSON-RPC 2.0. The CLI uses `reqwest` directly and has **no `agentrt-rs` dependency** (see `Cargo.toml`).
- **Configuration**: Resolved from CLI flags, then environment variables (`AGENTRT_ENDPOINT`, `AGENTRT_API_KEY`), then a `http://127.0.0.1:18789` default.

### Downstream

- **Developers / operators**: The primary human-facing surface for driving the runtime from a shell, CI job, or ops runbook.
- **Shell completion**: Generates completion scripts for bash/zsh/fish/etc.

## Command Reference

| Command | Description | Needs Gateway |
|---------|-------------|:-------------:|
| `agentrt init <name>` | Initialize a new Airymax project | ✗ |
| `agentrt create agent\|tool\|plugin\|prompt\|skill <name>` | Scaffold a component | ✗ |
| `agentrt run [prompt]` | Run an agent (interactive / one-shot) | ✓ |
| `agentrt config show\|set\|validate\|reload` | Configuration management | △ |
| `agentrt llm list\|test\|cost` | LLM provider management | ✓ |
| `agentrt prompt list\|show\|tune\|ab-test` | Prompt template management | ✗ |
| `agentrt market search\|install\|publish` | Marketplace search & install | ✓ |
| `agentrt deploy deploy\|status\|logs` | Deployment & operations | ✓ |
| `agentrt db status\|migrate\|rollback\|new` | Database migration management | ✗ |
| `agentrt completion <shell>` | Generate shell completion | ✗ |

## Installation

### From source

```bash
cd cli
cargo build --release
# Binary: ./target/release/agentrt
```

### Shell completion

```bash
source <(agentrt completion bash)
```

**Requirements:** Rust edition 2021 (stable toolchain). Runtime dependencies: `clap` 4.5 (CLI framework), `reqwest` 0.12 (HTTP), `tokio` 1 (async runtime), `serde` / `serde_json` / `serde_yaml` (serialization), `thiserror` / `anyhow` (errors), `colored` / `indicatif` (terminal output), `chrono`, `dirs`, `url`, `urlencoding`, `clap_complete`.

## Usage

### Project scaffolding

```bash
agentrt init my-agent-project
cd my-agent-project

agentrt create agent my-agent
agentrt create tool web-scraper
agentrt create plugin logging-plugin
```

### Run an agent

```bash
# Interactive mode
agentrt run

# One-shot
agentrt run "Analyze this sales data"

# With explicit agent config and model
agentrt run --agent-file agents/custom.agent.yaml --model gpt-4
```

### Configuration management

```bash
agentrt config show
agentrt config set llm.providers.openai.api_key sk-xxxx
agentrt config validate
agentrt config reload
```

### Prompt templates

```bash
agentrt prompt list
agentrt prompt show intent_classify
agentrt prompt tune intent_classify --dataset ./data.jsonl
agentrt prompt ab-test intent_classify --baseline v1 --candidate v2
```

### Deployment & operations

```bash
agentrt deploy deploy --target docker
agentrt deploy status
agentrt deploy logs --lines 100
```

## Build & Test

```bash
cargo build --release
cargo test
./target/release/agentrt --help
```

## Branch Strategy

This leaf repository is developed on **`feature/official-hubs-01`**. The aggregating `sdk` management repo stays on `main`.

## License

Dual-licensed under **AGPL v3 + Apache 2.0** (SPDX: `AGPL-3.0-or-later OR Apache-2.0`). See [LICENSE](LICENSE) for the full text.

Copyright (c) 2025-2026 **SPHARX Ltd.** All Rights Reserved.
