// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// AgentRT 用户面控制台库。命令面实现与网关协议客户端由本 crate 承担；
// 命令树解析、终端装配与全屏渲染不属于本 crate。

pub mod client;
pub mod commands;
pub mod templates;
