**语言:** [English](README.md) | 简体中文

# Airymax Console

[![Version](https://img.shields.io/badge/version-0.1.16-5a6b7e)](https://atomgit.com/openairymax/console)
[![License](https://img.shields.io/badge/license-AGPL--3.0+Apache--2.0-4a90d9)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> **状态（0.1.16）**：Airymax 运行时的用户面控制台库 —— 命令面实现与网关协议客户端。
>
> [sdk](https://atomgit.com/openairymax/sdk) 管理仓聚合的叶子仓之一。

---

## 概述

`agentrt-console` 是一个 Rust 库 crate，承担两项职责：

- **命令面** —— 项目脚手架、组件创建、智能体执行、配置管理、LLM 提供商管理、
  Prompt 模板管理、市场搜索与安装、运行时状态、数据库迁移。
- **网关客户端** —— 非流式调用走 HTTP JSON-RPC 2.0；流式执行委派给 `agentrt-rs`
  协议层，使传输、帧解码、工具跟踪与运行终止共用同一份实现。

本 crate 不定义二进制。命令行入口、参数解析与屏幕渲染由控制台前端承担，不属于本仓。

## 网关

网关调用由调用方显式传入 base URL。非流式请求以 HTTP POST 发送 JSON-RPC 2.0 消息；
流式执行经 `agentrt-rs` 事件流消费。

## 目录结构

```
console/
├── src/
│   ├── lib.rs               # crate 根
│   ├── client.rs            # 网关协议客户端
│   ├── templates.rs         # 项目 / 组件模板内容
│   └── commands/
│       ├── mod.rs           # 命令模块导出
│       ├── init.rs          # 项目脚手架
│       ├── create.rs        # agent / tool / plugin / prompt / skill
│       ├── run.rs           # 智能体执行（交互 / 单次）
│       ├── config_cmd.rs    # 配置管理
│       ├── llm.rs           # LLM 提供商管理
│       ├── prompt.rs        # Prompt 模板管理
│       ├── market.rs        # 市场搜索与安装
│       ├── deploy.rs        # 运行时状态
│       └── db.rs            # 数据库迁移
├── Cargo.toml               # crate 清单（agentrt-console，库）
└── README.md                # 本文件
```

## 命令面

| 能力 | 操作 | 需 Gateway |
|------|------|:----------:|
| 项目脚手架 | `init` | ✗ |
| 组件创建 | `create agent\|tool\|plugin\|prompt\|skill` | ✗ |
| 智能体执行 | `run`（交互 / 单次） | ✓ |
| 配置管理 | `config show\|set\|validate` | ✗ |
| LLM 提供商 | `llm list\|test\|cost` | ✓ |
| Prompt 模板 | `prompt list\|show` | ✗ |
| 市场 | `market search\|install\|publish` | ✓ |
| 运行时状态 | `deploy status` | ✓ |
| 数据库迁移 | `db status\|migrate\|rollback\|new` | ✗ |

## 构建与测试

```bash
cargo build --release
cargo test
```

**环境要求：** Rust edition 2021（stable 工具链）。依赖：`agentrt-rs`（网关协议层）、
`reqwest` 0.12（HTTP）、`tokio` 1（异步运行时）、`serde` / `serde_json` / `serde_yaml`
（序列化）、`anyhow`（错误处理）、`colored`（终端输出）。

## 许可证

采用 **AGPL v3 + Apache 2.0** 双许可证（SPDX: `AGPL-3.0-or-later OR Apache-2.0`）。详见 [LICENSE](LICENSE)。

Copyright (c) 2025-2026 **SPHARX Ltd.** All Rights Reserved.
