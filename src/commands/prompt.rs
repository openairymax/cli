// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt prompt
//
// Local prompt template listing and viewing.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

/// List available Prompt templates.
pub fn list() -> Result<()> {
    let prompts_dir = Path::new("prompts");

    if !prompts_dir.exists() {
        println!(
            "{} No prompts directory found. Create one with 'agentrt init'.",
            "ℹ".yellow()
        );
        return Ok(());
    }

    println!("{} Prompt Templates:", "📝".blue().bold());
    println!();

    let entries = fs::read_dir(prompts_dir)
        .context("Failed to read prompts directory")?;

    let mut found = false;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
            let Some(stem) = path.file_stem() else {
                continue;
            };
            let name = stem.to_string_lossy();
            let description = get_description(&path);
            println!("  {} {}  {}", "•".cyan(), name.cyan().bold(), description);
            found = true;
        }
    }

    if !found {
        println!("  No prompt templates found. Create one with 'agentrt create prompt <name>'.");
    }

    Ok(())
}

/// Show a specific Prompt template.
pub fn show(name: &str) -> Result<()> {
    let path = find_prompt(name)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    println!("{} Prompt: {}", "📝".blue(), name.cyan().bold());
    println!();
    println!("{}", content);
    Ok(())
}

fn find_prompt(name: &str) -> Result<std::path::PathBuf> {
    for ext in &["yaml", "yml"] {
        let path = Path::new("prompts").join(format!("{}.{}", name, ext));
        if path.exists() {
            return Ok(path);
        }
    }
    anyhow::bail!("Prompt template '{}' not found in prompts/ directory.", name)
}

fn get_description(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("description:") || trimmed.starts_with("#") {
                    let desc = trimmed
                        .trim_start_matches("description:")
                        .trim_start_matches('#')
                        .trim();
                    if !desc.is_empty() {
                        return desc.to_string();
                    }
                }
            }
            "(no description)".dimmed().to_string()
        }
        Err(_) => "(unreadable)".red().to_string(),
    }
}
