// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// CLI command: agentrt deploy status
//
// Runtime status inspection through the gateway info capabilities.

use anyhow::Result;
use colored::Colorize;

use crate::client::{GatewayClient, InfoHardware};

/// Show AgentRT runtime status（info.system + info.health + info.hardware）。
pub async fn status(gateway_url: &str) -> Result<()> {
    let client = GatewayClient::new(gateway_url)?;

    let sys = client.info_system().await?;
    let health = client.info_health().await?;
    let hw = client.info_hardware().await?;

    println!("{} AgentRT Status:", "📊".blue().bold());
    println!();
    println!("  Runtime:     {} ({})", sys.platform, sys.hostname);
    if !sys.kernel_version.is_empty() {
        println!("  Kernel:      {}", sys.kernel_version);
    }
    println!(
        "  Health:      {}{} | uptime {}",
        health.status,
        if health.running { ", running" } else { ", not running" },
        format_uptime(health.uptime_s as u64)
    );
    if health.collecting {
        println!("  Collector:   active (staleness {:.0}s)", health.staleness_sec);
    }
    if let Some(snap) = &sys.system {
        println!();
        println!("  CPU:         {:.1}% of {} cores", snap.cpu_usage_pct, snap.cpu_cores);
        println!(
            "  Memory:      {:.1}% used ({}/ {})",
            snap.memory_usage_pct,
            kb_human(snap.used_memory_kb),
            kb_human(snap.total_memory_kb)
        );
        println!(
            "  Disk:        {:.1}% used ({}/ {} free)",
            snap.disk_usage_pct,
            kb_human(snap.disk_free_kb),
            kb_human(snap.disk_total_kb)
        );
        println!("  OS uptime:   {}", format_uptime(snap.uptime_sec as u64));
    }
    print_hardware(&hw);

    Ok(())
}

fn print_hardware(hw: &InfoHardware) {
    let mut line = String::from("  Hardware:    ");
    if let Some(count) = hw.cpu_count {
        line.push_str(&format!("{count} cores"));
    }
    if let (Some(total), Some(avail)) = (hw.mem_total_kib, hw.mem_avail_kib) {
        line.push_str(&format!(
            ", {}GiB memory ({}GiB free)",
            total / 1048576.0,
            avail / 1048576.0
        ));
    }
    if let Some(profile) = &hw.profile {
        line.push_str(&format!(", profile {profile}"));
    }
    println!();
    println!("{line}");
    if hw.accel_present.unwrap_or(false) {
        let model = hw.accel_model.as_deref().unwrap_or("unknown");
        println!("  Accel:       {} x {model}", hw.accel_count.unwrap_or(0));
    }
}

fn kb_human(kb: f64) -> String {
    let gib = kb / 1048576.0;
    if gib >= 1.0 {
        format!("{gib:.1}G")
    } else {
        format!("{:.0}M", kb / 1024.0)
    }
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
