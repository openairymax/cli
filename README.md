**Language:** English | [简体中文](README_zh.md)

# Airymax Console

[![Version](https://img.shields.io/badge/version-0.1.16-5a6b7e)](https://atomgit.com/openairymax/console)
[![License](https://img.shields.io/badge/license-AGPL--3.0+Apache--2.0-4a90d9)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> **Status (0.1.16)**: the user-facing console library of the Airymax runtime —
> command surface implementations plus the gateway protocol client.
>
> One of the leaf repositories aggregated by the [sdk](https://atomgit.com/openairymax/sdk) management repo.

---

## Overview

`agentrt-console` is a Rust library crate. It carries two responsibilities:

- **Command surface** — project scaffolding, component creation, agent execution,
  configuration management, LLM provider management, prompt template management,
  marketplace search & install, runtime status, and database migrations.
- **Gateway client** — JSON-RPC 2.0 over HTTP for non-streaming calls, with
  streaming runs delegated to the `agentrt-rs` protocol layer, so that transport,
  frame decoding, tool tracking and run termination share one implementation.

The crate defines no binary. Command-line entry, argument parsing and screen
rendering belong to the console front ends, not to this repository.

## Gateway

Gateway calls take an explicit base URL from the caller. Non-streaming requests
are sent as JSON-RPC 2.0 messages over HTTP POST; streaming runs are consumed
through the `agentrt-rs` event stream.

## Directory Structure

```
console/
├── src/
│   ├── lib.rs               # Crate root
│   ├── client.rs            # Gateway protocol client
│   ├── templates.rs         # Project / component template content
│   └── commands/
│       ├── mod.rs           # Command module exports
│       ├── init.rs          # project scaffolding
│       ├── create.rs        # agent / tool / plugin / prompt / skill
│       ├── run.rs           # agent execution (interactive / one-shot)
│       ├── config_cmd.rs    # configuration management
│       ├── llm.rs           # LLM provider management
│       ├── prompt.rs        # prompt template management
│       ├── market.rs        # marketplace search & install
│       ├── deploy.rs        # runtime status
│       └── db.rs            # database migrations
├── Cargo.toml               # crate manifest (agentrt-console, library)
└── README.md                # this file
```

## Command Surface

| Capability | Operations | Needs Gateway |
|------------|------------|:-------------:|
| Project scaffolding | `init` | ✗ |
| Component creation | `create agent\|tool\|plugin\|prompt\|skill` | ✗ |
| Agent execution | `run` (interactive / one-shot) | ✓ |
| Configuration | `config show\|set\|validate` | ✗ |
| LLM providers | `llm list\|test\|cost` | ✓ |
| Prompt templates | `prompt list\|show` | ✗ |
| Marketplace | `market search\|install\|publish` | ✓ |
| Runtime status | `deploy status` | ✓ |
| Database migrations | `db status\|migrate\|rollback\|new` | ✗ |

## Build & Test

```bash
cargo build --release
cargo test
```

**Requirements:** Rust edition 2021 (stable toolchain). Dependencies:
`agentrt-rs` (gateway protocol layer), `reqwest` 0.12 (HTTP), `tokio` 1 (async
runtime), `serde` / `serde_json` / `serde_yaml` (serialization), `anyhow` (error
handling), `colored` (terminal output).

## License

Dual-licensed under **AGPL v3 + Apache 2.0** (SPDX: `AGPL-3.0-or-later OR Apache-2.0`). See [LICENSE](LICENSE) for the full text.

Copyright (c) 2025-2026 **SPHARX Ltd.** All Rights Reserved.
