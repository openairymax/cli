// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt run
//
// Sends agent execution requests to the gateway.

use anyhow::Result;
use colored::Colorize;

use crate::client::{GatewayClient, RunResponse};

/// Execute an agent run through the gateway.
pub async fn execute(
    gateway_url: &str,
    prompt: Option<String>,
    agent_file: &str,
    model: Option<String>,
) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    match client.health_check().await {
        Ok(health) => {
            println!(
                "{} Gateway: {} (v{})",
                "✓".green(),
                health.status,
                health.version.as_deref().unwrap_or("unknown")
            );
        }
        Err(_) => {
            anyhow::bail!(
                "Cannot connect to AgentRT gateway at {}. Is it running?",
                gateway_url
            );
        }
    }

    match prompt {
        Some(text) => {
            println!("{} Sending prompt: {}", "▶".blue().bold(), text);
            println!();

            let response = client
                .agent_run(&text, agent_file, model.as_deref(), None, None)
                .await?;

            println!("{}", response.response);
            println!();
            print_run_meta(&response);
        }
        None => {
            println!("{} Starting interactive session...", "▶".blue().bold());
            println!("  Agent: {}", agent_file);
            println!("  Type your prompts or Ctrl+C to exit.");
            println!();
            run_interactive(&client, agent_file, model.as_deref()).await?;
        }
    }

    Ok(())
}

/// Run in interactive mode, reading user input in a loop.
async fn run_interactive(
    client: &GatewayClient,
    agent_file: &str,
    model: Option<&str>,
) -> Result<()> {
    let session_id = new_session_id();
    loop {
        print!("{} ", ">".cyan().bold());
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} Error reading input: {}", "✗".red(), e);
                break;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("{} Goodbye.", "✋".yellow());
            break;
        }

        let result = client
            .agent_run_stream(
                input,
                agent_file,
                model,
                &session_id,
                |delta| {
                    print!("{delta}");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                },
                |note| println!("\n  {} {}", "⚙".dimmed(), note),
            )
            .await;

        println!();
        match result {
            Ok(response) => {
                println!();
                print_run_meta(&response);
            }
            Err(e) => {
                eprintln!("{} Gateway error: {}", "✗".red(), e);
            }
        }
    }

    Ok(())
}

fn print_run_meta(response: &RunResponse) {
    let mut parts = Vec::new();
    if let Some(tokens) = response.tokens_used {
        parts.push(format!("Tokens: {tokens}"));
    }
    if let Some(cost) = response.cost_usd {
        parts.push(format!("Cost: ${cost:.6}"));
    }
    parts.push(format!("Session: {}", response.session_id));
    println!("{} {}", "ℹ".dimmed(), parts.join(" | "));
}

/// 引擎契约：调用方自带会话 id 须带 "sess_" 前缀，引擎原样采用。
fn new_session_id() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("sess_cli_{ms:x}")
}
