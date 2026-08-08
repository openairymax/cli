// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt llm
//
// LLM provider management and cost tracking through the gateway.

use anyhow::Result;
use colored::Colorize;

use crate::client::{CostResponse, GatewayClient, LlmTestRequest};

/// List configured LLM models（经 gateway llm.list_models 转发 llm_d registry）。
pub async fn list(gateway_url: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    let list = client.llm_list_models().await?;

    if list.models.is_empty() {
        println!("{} 未配置任何模型。", "ℹ".yellow());
        println!("  编辑 $AIRY_HOME/config/model.yaml 或仓库 SSoT（ecosystem/manager/model/model.yaml）。");
        return Ok(());
    }

    println!("{} LLM 模型清单（{} 个）：", "🤖".blue().bold(), list.models.len());
    if !list.default_model.is_empty() {
        let prov = if list.default_provider.is_empty() {
            String::new()
        } else {
            format!(" [{}]", list.default_provider)
        };
        println!("  默认模型: {}{}", list.default_model.yellow().bold(), prov.green());
    }
    println!();

    let mut by_provider: std::collections::BTreeMap<&str, Vec<&crate::client::LlmModelEntry>> =
        std::collections::BTreeMap::new();
    for m in &list.models {
        by_provider.entry(m.provider.as_str()).or_default().push(m);
    }
    for (provider, models) in &by_provider {
        println!("  {}:", provider.cyan().bold());
        for m in models {
            let star = if m.default { " ★" } else { "" };
            println!("    {}{}", m.name, star.yellow());
        }
        println!();
    }

    println!(
        "  切换默认模型：编辑 {} 的 global 段（default_model / default_provider）",
        "$AIRY_HOME/config/model.yaml".yellow()
    );
    println!("  或在 agentrt run --model <模型名> 临时指定。");

    Ok(())
}

/// Test provider connectivity.
pub async fn test(gateway_url: &str, provider: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    println!("{} Testing provider: {}...", "🔍".yellow(), provider.cyan());

    let request = LlmTestRequest {
        provider: provider.to_string(),
    };

    match client
        .post::<serde_json::Value>("/api/v1/llm/test", &request)
        .await
    {
        Ok(resp) => {
            println!(
                "{} Provider '{}' is working: {}",
                "✓".green(),
                provider,
                serde_json::to_string_pretty(&resp).unwrap_or_default()
            );
        }
        Err(e) => {
            eprintln!("{} Provider '{}' test failed: {}", "✗".red(), provider, e);
        }
    }

    Ok(())
}

/// Show LLM usage costs.
pub async fn cost(gateway_url: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    let costs: CostResponse = client.get("/api/v1/llm/cost").await?;

    println!("{} LLM Usage Costs:", "💰".blue().bold());
    println!();
    println!("  Total tokens:   {}", costs.total_tokens);
    println!("  Total cost:      ${:.6}", costs.total_cost_usd);
    println!();

    if let Some(by_provider) = &costs.by_provider {
        if !by_provider.is_empty() {
            println!("  {:<20} {:<15} Cost (USD)", "Provider", "Tokens");
            println!("  {:-<20} {:-<15} {:-<15}", "", "", "");
            for pc in by_provider {
                println!(
                    "  {:<20} {:<15} ${:.6}",
                    pc.provider, pc.tokens, pc.cost_usd
                );
            }
        }
    }

    Ok(())
}