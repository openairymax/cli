// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt market
//
// Search, install, and publish through the gateway market capabilities.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::client::GatewayClient;

/// Search the agent marketplace.
pub async fn search(gateway_url: &str, keyword: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    println!("{} Searching marketplace for: {}", "🔍".blue(), keyword.cyan());
    println!();

    let results = client.market_search(keyword).await?;

    if results.is_empty() {
        println!("  No results found for '{keyword}'.");
        return Ok(());
    }

    println!("  {:<28} {:<10} {:<16} Installed", "ID", "Version", "Author");
    println!("  {:-<28} {:-<10} {:-<16} {:-<10}", "", "", "", "");

    for r in &results {
        let installed = if r.installed { "✓".green() } else { "-".dimmed() };
        println!(
            "  {:<28} {:<10} {:<16} {}",
            r.agent_id.cyan(),
            r.version,
            r.author,
            installed
        );
        if !r.description.is_empty() {
            println!("    {} — {}", r.name.dimmed(), r.description);
        }
    }

    println!();
    println!("  To install: agentrt market install <agent_id>");

    Ok(())
}

/// Install from the marketplace.
pub async fn install(gateway_url: &str, package: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    println!("{} Installing: {}", "📦".blue(), package.cyan());

    let result = client.market_install(package, "latest").await?;

    if result.status == "installed" {
        println!(
            "{} Installed {}",
            "✓".green(),
            result.installed_version.as_deref().unwrap_or(package)
        );
        if let Some(id) = &result.agent_id {
            println!("  Agent ID: {}", id.cyan());
        }
        if let Some(path) = &result.install_path {
            println!("  Installed to: {}", path.cyan());
        }
    } else {
        anyhow::bail!(
            "Failed to install '{package}': {}",
            result.message.as_deref().unwrap_or("unknown error")
        );
    }

    Ok(())
}

/// Publish the local agent manifest to the marketplace.
pub async fn publish(gateway_url: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    let manifest = std::fs::read_to_string("agents/main.agent.yaml")
        .context("No agents/main.agent.yaml found. Run 'agentrt init' first.")?;
    let spec: serde_yaml::Value = serde_yaml::from_str(&manifest)
        .context("Failed to parse agents/main.agent.yaml")?;
    let agent_id = spec
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let version = spec
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if agent_id.is_empty() {
        anyhow::bail!("agents/main.agent.yaml is missing the 'name' field");
    }

    println!("{} Publishing {} (v{})...", "📤".blue(), agent_id.cyan(), version);

    let resp = client.market_publish(agent_id, version).await?;

    println!("{} Published successfully", "✓".green());
    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());

    Ok(())
}
