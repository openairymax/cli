**语言:** [English](README.md) | 简体中文

# Airymax CLI

[![Version](https://img.shields.io/badge/version-0.1.1-5a6b7e)](https://atomgit.com/openairymax/cli)
[![License](https://img.shields.io/badge/license-AGPL--3.0+Apache--2.0-4a90d9)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> [Airymax](https://atomgit.com/openairymax/airymaxhub) AI 智能体运行时平台的官方命令行工具。
> [sdk](https://atomgit.com/openairymax/sdk) 管理仓聚合的叶子仓之一。
> 基于 Airymax Rust SDK（`agentrt-rs`）构建。

---

## 概述

**Airymax CLI**（`agentrt`）是用 Rust 构建的命令行工具，用于在终端中操作 Airymax 运行时。它覆盖 Agent 全生命周期：项目脚手架、组件创建、Agent 执行、配置管理、LLM 提供商管理、Prompt 模板管理、市场搜索与安装、部署运维和数据库迁移。

运行时相关命令通过 HTTP 与 Airymax Gateway 通信；CLI 内部驱动与各语言 SDK 相同的双层 SDK 架构（Cognition / Safety / Tool / Chat），因此 CLI 是一等运行时租户，与 Agent 应用走相同的代码路径。

## 消费的双层 API 架构

CLI 是 Airymax Rust SDK 的消费者，而非新增资源客户端的提供者。其 `run`、`llm`、`market`、`deploy` 等子命令将用户意图转换为对 `AgentRTClient` 暴露的四个内嵌资源客户端的调用：

```
agentrt <command>
   └── AgentRTClient（来自 agentrt-rs）
       ├── CognitionClient   # 由 `agentrt run` 使用
       ├── SafetyClient      # 由策略 / 审计流程使用
       ├── ToolClient        # 由工具调用与市场安装使用
       └── ChatClient        # 由交互式对话会话使用
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

- **Airymax Rust SDK（`agentrt-rs`）**：提供与运行时通信的类型化 `AgentRTClient` 与四个内嵌资源客户端。
- **运行时**：通过 HTTP 和 JSON-RPC 2.0 连接到运行中的 Airymax / AgentRT 实例（`gateway_d` / Gateway HTTP API）。
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

本叶子仓在 **`feature/official-hubs-01`** 分支上开发。聚合管理仓 `sdk` 仅使用 `main` 分支。

## 许可证

采用 **AGPL v3 + Apache 2.0** 双许可证（SPDX: `AGPL-3.0-or-later OR Apache-2.0`）。详见 [LICENSE](LICENSE)。

Copyright (c) 2025-2026 **SPHARX Ltd.** All Rights Reserved.
