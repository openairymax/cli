**语言:** [English](README.md) | 简体中文

# Airymax CLI

[![Version](https://img.shields.io/badge/version-0.1.9-5a6b7e)](https://atomgit.com/openairymax/cli)
[![License](https://img.shields.io/badge/license-AGPL--3.0+Apache--2.0-4a90d9)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> **状态（0.1.9）**: 开发者运维命令行工具 —— 覆盖项目脚手架、组件创建、配置管理、
> 市场搜索与安装、部署运维等开发者工作流。经 Gateway（HTTP / JSON-RPC 2.0）
> 调用运行时服务，是独立的运行时租户。
>
> [sdk](https://atomgit.com/openairymax/sdk) 管理仓聚合的叶子仓之一。
> 独立 Rust 二进制 —— 通过 HTTP 与 Airymax Gateway 通信，不链接各语言 SDK（无 `agentrt-rs` 依赖）。

---

## 概述

**Airymax CLI**（`agentrt`）是用 Rust 构建的命令行工具，用于在终端中操作 Airymax 运行时。它覆盖 Agent 全生命周期：项目脚手架、组件创建、Agent 执行、配置管理、LLM 提供商管理、Prompt 模板管理、市场搜索与安装、部署运维和数据库迁移。

运行时相关命令通过 HTTP（JSON-RPC 2.0）与 Airymax Gateway 通信，使用 CLI 自身的 `reqwest` 客户端。CLI 是独立运行时租户 —— 不依赖、也不链接各语言 SDK（`agentrt-rs` / `agentrt-sdk-go` 等）。

## 运行时通信

CLI 通过 Gateway HTTP API（JSON-RPC 2.0）与运行时通信，直接使用自己的 HTTP 客户端（`src/client.rs`，基于 `reqwest`）。它不包装、也不消费语言 SDK：`Cargo.toml` 中没有 `agentrt-rs` 依赖。

```
agentrt <command>
   └── src/client.rs — reqwest HTTP 客户端
       ├── run      → 任务提交 / 智能体执行
       ├── llm      → LLM 提供商管理
       ├── market   → 市场搜索与安装
       └── deploy   → 部署与状态查询
```

## 目录结构

```
cli/
├── src/
│   ├── main.rs              # 入口与 CLI 路由（clap）
│   ├── client.rs            # Gateway HTTP 客户端
│   ├── templates.rs         # 项目 / 组件模板生成
│   └── commands/
│       ├── mod.rs           # 命令模块导出
│       ├── init.rs          # agentrt init — 项目脚手架
│       ├── create.rs        # agentrt create — agent/tool/plugin/prompt/skill
│       ├── run.rs           # agentrt run — 运行智能体（交互 / 单次）
│       ├── config_cmd.rs    # agentrt config — 配置管理
│       ├── llm.rs           # agentrt llm — LLM 提供商管理
│       ├── prompt.rs        # agentrt prompt — Prompt 模板管理
│       ├── market.rs        # agentrt market — 市场搜索与安装
│       ├── deploy.rs        # agentrt deploy — 部署与状态查询
│       └── db.rs            # agentrt db — 数据库迁移管理
├── Cargo.toml               # crate 清单（agentrt-cli，二进制：agentrt）
└── README.md                # 本文件
```

## 上下游依赖

### 上游

- **运行时**：通过 HTTP 和 JSON-RPC 2.0 连接到运行中的 Airymax / AgentRT 实例（`gateway_d` / Gateway HTTP API）。CLI 直接使用 `reqwest`，**无 `agentrt-rs` 依赖**（见 `Cargo.toml`）。
- **配置**：依次从 CLI 标志、环境变量（`AGENTRT_ENDPOINT`、`AGENTRT_API_KEY`）、默认值 `http://127.0.0.1:18789` 解析。

### 下游

- **开发者 / 运维人员**：从 Shell、CI 任务或运维手册驱动运行时的主要人机界面。
- **Shell 补全**：为 bash/zsh/fish 等生成补全脚本。

## 命令一览

| 命令 | 说明 | 需 Gateway |
|------|------|:----------:|
| `agentrt init <name>` | 初始化新的 Airymax 项目 | ✗ |
| `agentrt create agent\|tool\|plugin\|prompt\|skill <name>` | 创建组件脚手架 | ✗ |
| `agentrt run [prompt]` | 运行智能体（交互 / 单次） | ✓ |
| `agentrt config show\|set\|validate\|reload` | 配置管理 | △ |
| `agentrt llm list\|test\|cost` | LLM 提供商管理 | ✓ |
| `agentrt prompt list\|show\|tune\|ab-test` | Prompt 模板管理 | ✗ |
| `agentrt market search\|install\|publish` | 市场搜索与安装 | ✓ |
| `agentrt deploy deploy\|status\|logs` | 部署与运维 | ✓ |
| `agentrt db status\|migrate\|rollback\|new` | 数据库迁移管理 | ✗ |
| `agentrt completion <shell>` | 生成 Shell 补全脚本 | ✗ |

## 安装

### 从源码构建

```bash
cd cli
cargo build --release
# 二进制：./target/release/agentrt
```

### Shell 补全

```bash
source <(agentrt completion bash)
```

**环境要求：** Rust edition 2021（stable 工具链）。运行时依赖：`clap` 4.5（CLI 框架）、`reqwest` 0.12（HTTP）、`tokio` 1（异步运行时）、`serde` / `serde_json` / `serde_yaml`（序列化）、`thiserror` / `anyhow`（错误）、`colored` / `indicatif`（终端输出）、`chrono`、`dirs`、`url`、`urlencoding`、`clap_complete`。

## 使用说明

### 项目脚手架

```bash
agentrt init my-agent-project
cd my-agent-project

agentrt create agent my-agent
agentrt create tool web-scraper
agentrt create plugin logging-plugin
```

### 运行智能体

```bash
# 交互模式
agentrt run

# 单次执行
agentrt run "分析这份销售数据"

# 指定智能体配置和模型
agentrt run --agent-file agents/custom.agent.yaml --model gpt-4
```

### 配置管理

```bash
agentrt config show
agentrt config set llm.providers.openai.api_key sk-xxxx
agentrt config validate
agentrt config reload
```

### Prompt 模板管理

```bash
agentrt prompt list
agentrt prompt show intent_classify
agentrt prompt tune intent_classify --dataset ./data.jsonl
agentrt prompt ab-test intent_classify --baseline v1 --candidate v2
```

### 部署运维

```bash
agentrt deploy deploy --target docker
agentrt deploy status
agentrt deploy logs --lines 100
```

## 构建与测试

```bash
cargo build --release
cargo test
./target/release/agentrt --help
```

## 分支策略

本叶子仓在 **`develop/hubs-01`** 分支上开发，`main` 为发布快照。聚合管理仓 `sdk` 在 `main` 上直接开发。

## 许可证

采用 **AGPL v3 + Apache 2.0** 双许可证（SPDX: `AGPL-3.0-or-later OR Apache-2.0`）。详见 [LICENSE](LICENSE)。

Copyright (c) 2025-2026 **SPHARX Ltd.** All Rights Reserved.
