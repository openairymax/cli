// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt llm
//
// LLM provider management and cost tracking through the gateway.

use anyhow::Result;
use colored::Colorize;

use crate::client::{GatewayClient, LlmModelEntry};

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

    let mut by_provider: std::collections::BTreeMap<&str, Vec<&LlmModelEntry>> =
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

/// Test provider connectivity（llm.list_models 定位模型 + llm.complete 探针）。
pub async fn test(gateway_url: &str, provider: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    println!("{} Testing provider: {}...", "🔍".yellow(), provider.cyan());

    let list = client.llm_list_models().await?;
    let model = list
        .models
        .iter()
        .find(|m| m.provider.eq_ignore_ascii_case(provider))
        .map(|m| m.name.clone())
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' has no configured models", provider))?;

    match client.llm_test(Some(&model)).await {
        Ok(probe) => {
            println!(
                "{} Provider '{}' is working (model: {})",
                "✓".green(),
                provider,
                probe.model.as_deref().unwrap_or(&model)
            );
            println!("  Reply: {}", probe.content);
            println!(
                "  Tokens: {} prompt / {} completion / {} total",
                probe.prompt_tokens, probe.completion_tokens, probe.total_tokens
            );
            println!("  Cost: ${:.6}", probe.cost_usd);
        }
        Err(e) => {
            eprintln!("{} Provider '{}' test failed: {}", "✗".red(), provider, e);
        }
    }

    Ok(())
}

/// Show LLM usage costs and semantic cache stats（llm.get_stats）。
pub async fn cost(gateway_url: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    let stats = client.llm_get_stats().await?;

    println!("{} LLM Usage & Cache:", "💰".blue().bold());
    println!();

    if stats.cost.models.is_empty() {
        println!("  No usage recorded yet.");
    } else {
        println!("  {:<28} {:>12} {:>12} {:>12}", "Model", "Prompt", "Completion", "Cost (USD)");
        println!("  {:-<28} {:-<12} {:-<12} {:-<12}", "", "", "", "");
        let mut total = 0.0f64;
        for mc in &stats.cost.models {
            println!(
                "  {:<28} {:>12} {:>12} {:>12.6}",
                mc.model, mc.prompt_tokens, mc.completion_tokens, mc.cost_usd
            );
            total += mc.cost_usd;
        }
        println!();
        println!("  Total cost: ${total:.6}");
    }

    println!();
    println!(
        "  Cache: {}/{} entries | hits {} / misses {} | hit rate {:.1}% | evictions {}",
        stats.llm_cache_size,
        stats.llm_cache_capacity,
        stats.llm_cache_hits,
        stats.llm_cache_misses,
        stats.llm_cache_hit_rate * 100.0,
        stats.llm_cache_evictions
    );

    Ok(())
}
