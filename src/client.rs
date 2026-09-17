// SPDX-FileCopyrightText: 2025-2026 SPHARX Ltd.
// SPDX-License-Identifier: AGPL-3.0-or-later OR Apache-2.0

// Copyright (c) 2026 SPHARX Ltd. All Rights Reserved.
//
// console 前端网关客户端：非流式调用统一走 POST / JSON-RPC（rpc_call），
// 流式执行轮经 agentrt-rs run_stream_events 承担传输与帧解码。

use agentrt_rs::run_stream::{gen, run_stream_events};
use anyhow::{Context, Result};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub struct GatewayClient {
    base_url: String,
    http: HttpClient,
    rs: agentrt_rs::Client,
}

impl GatewayClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        let ua = format!("agentrt-console/{}", env!("CARGO_PKG_VERSION"));
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(ua.clone())
            .build()
            .context("Failed to create HTTP client")?;
        let rs = agentrt_rs::client::ClientBuilder::new(&base_url)
            .user_agent(&ua)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create protocol client: {e}"))?;
        Ok(Self { base_url, http, rs })
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let url = format!("{}/", self.base_url);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": method,
            "params": params,
        });
        let resp = self.http.post(&url).json(&request).send().await?;
        let body = resp.text().await?;
        let json: Value =
            serde_json::from_str(&body).context("Failed to parse JSON-RPC response")?;
        if let Some(err) = json.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            anyhow::bail!("{}: {}", method, msg);
        }
        Ok(json.get("result").cloned().unwrap_or(Value::Null))
    }

    pub async fn health_check(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let resp = tokio::time::timeout(Duration::from_secs(2), self.http.get(&url).send())
            .await
            .context("Gateway health check timed out (2s)")??;
        let body = resp.text().await?;
        serde_json::from_str(&body).context("Failed to parse health response")
    }

    pub async fn agent_run(
        &self,
        prompt: &str,
        agent_file: &str,
        model: Option<&str>,
        session_id: Option<&str>,
        messages: Option<Value>,
    ) -> Result<RunResponse> {
        let mut params = serde_json::json!({ "prompt": prompt, "agent_file": agent_file });
        if let Some(m) = model {
            if !m.is_empty() {
                params["model"] = Value::String(m.to_string());
            }
        }
        if let Some(sid) = session_id {
            if !sid.is_empty() {
                params["session_id"] = Value::String(sid.to_string());
            }
        }
        if let Some(h) = messages {
            params["messages"] = h;
        }
        let result = self.rpc_call("agent.run", params).await?;
        serde_json::from_value(result).context("Failed to parse agent.run result")
    }

    pub async fn agent_run_stream<F, E>(
        &self,
        prompt: &str,
        agent_file: &str,
        model: Option<&str>,
        session_id: &str,
        mut on_text: F,
        mut on_tool: E,
    ) -> Result<RunResponse>
    where
        F: FnMut(&str),
        E: FnMut(&str),
    {
        let mut params = serde_json::json!({
            "prompt": prompt,
            "agent_file": agent_file,
            "session_id": session_id,
        });
        if let Some(m) = model {
            if !m.is_empty() {
                params["model"] = Value::String(m.to_string());
            }
        }

        let mut final_content: Option<String> = None;
        let mut text_acc = String::new();
        let mut err_msg: Option<String> = None;
        let mut tokens: Option<u64> = None;
        let mut tool_names: HashMap<String, String> = HashMap::new();

        run_stream_events(&self.rs, params, |ev| {
            match ev.event_type.as_str() {
                gen::AIRY_RS_TYPE_TOKEN_DELTA => {
                    if let Some(d) = ev.data_str(gen::AIRY_RS_K_DELTA) {
                        if !d.is_empty() {
                            text_acc.push_str(d);
                            on_text(d);
                        }
                    }
                }
                gen::AIRY_RS_TYPE_TOOL_START => {
                    let tool = ev.data_str(gen::AIRY_RS_K_TOOL).unwrap_or("tool").to_string();
                    if let Some(tid) = ev.data_str(gen::AIRY_RS_K_TOOL_ID) {
                        tool_names.insert(tid.to_string(), tool.clone());
                    }
                    on_tool(&format!("tool: {tool}"));
                }
                gen::AIRY_RS_TYPE_TOOL_END => {
                    let tid = ev.data_str(gen::AIRY_RS_K_TOOL_ID).unwrap_or("");
                    let tool = tool_names.get(tid).cloned().unwrap_or_else(|| {
                        if tid.is_empty() {
                            "tool".to_string()
                        } else {
                            tid.to_string()
                        }
                    });
                    let ok = ev.data_str(gen::AIRY_RS_K_STATUS) == Some("ok");
                    if ok {
                        on_tool(&format!("tool: {tool} ok"));
                    } else {
                        on_tool(&format!("tool: {tool} failed"));
                    }
                }
                gen::AIRY_RS_TYPE_MESSAGE => {
                    if let Some(c) = ev.data_str(gen::AIRY_RS_K_CONTENT) {
                        if !c.is_empty() {
                            final_content = Some(c.to_string());
                        }
                    }
                }
                gen::AIRY_RS_TYPE_ERROR => {
                    if let Some(m) = ev.data_str(gen::AIRY_RS_K_MSG) {
                        if err_msg.is_none() {
                            err_msg = Some(m.to_string());
                        }
                        on_tool(&format!("error: {m}"));
                    }
                }
                gen::AIRY_RS_TYPE_RUN_END => {
                    tokens = ev.data_i64(gen::AIRY_RS_K_USE_TICKS).map(|t| t as u64);
                }
                _ => {}
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        if let Some(m) = err_msg {
            anyhow::bail!("{}", m);
        }
        let response = final_content.unwrap_or(text_acc);
        if response.trim().is_empty() {
            anyhow::bail!("agent.run_stream: empty result");
        }
        Ok(RunResponse {
            session_id: session_id.to_string(),
            response,
            tokens_used: tokens,
            cost_usd: None,
        })
    }

    pub async fn llm_list_models(&self) -> Result<LlmModelList> {
        let result = self
            .rpc_call("llm.list_models", serde_json::json!({}))
            .await?;
        serde_json::from_value(result).context("Failed to parse model list")
    }

    pub async fn llm_test(&self, model: Option<&str>) -> Result<LlmProbe> {
        let mut params = serde_json::json!({
            "messages": [{ "role": "user", "content": "ping" }],
        });
        if let Some(m) = model {
            if !m.is_empty() {
                params["model"] = Value::String(m.to_string());
            }
        }
        let result = self.rpc_call("llm.complete", params).await?;
        let content = result
            .pointer("/choices/0/content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        Ok(LlmProbe {
            model: result.get("model").and_then(|m| m.as_str()).map(String::from),
            content,
            prompt_tokens: result
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            completion_tokens: result
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_tokens: result
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            cost_usd: result
                .get("cost_usd")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    pub async fn llm_get_stats(&self) -> Result<LlmStats> {
        let result = self
            .rpc_call("llm.get_stats", serde_json::json!({}))
            .await?;
        serde_json::from_value(result).context("Failed to parse llm.get_stats result")
    }

    pub async fn market_search(&self, keyword: &str) -> Result<Vec<MarketAgent>> {
        let params = serde_json::json!({ "keyword": keyword, "offset": 0, "limit": 20 });
        let result = self.rpc_call("market.search_agents", params).await?;
        serde_json::from_value(result).context("Failed to parse market.search_agents result")
    }

    pub async fn market_install(
        &self,
        agent_id: &str,
        version: &str,
    ) -> Result<MarketInstallResult> {
        let params = serde_json::json!({ "agent_id": agent_id, "version": version });
        let result = self.rpc_call("market.install_agent", params).await?;
        serde_json::from_value(result).context("Failed to parse market.install_agent result")
    }

    pub async fn market_publish(&self, agent_id: &str, version: &str) -> Result<Value> {
        let params = serde_json::json!({ "agent": { "agent_id": agent_id, "version": version } });
        self.rpc_call("market.publish", params).await
    }

    pub async fn info_system(&self) -> Result<InfoSystem> {
        let result = self
            .rpc_call("info.system", serde_json::json!({}))
            .await?;
        serde_json::from_value(result).context("Failed to parse info.system result")
    }

    pub async fn info_health(&self) -> Result<InfoHealth> {
        let result = self
            .rpc_call("info.health", serde_json::json!({}))
            .await?;
        serde_json::from_value(result).context("Failed to parse info.health result")
    }

    pub async fn info_hardware(&self) -> Result<InfoHardware> {
        let result = self
            .rpc_call("info.hardware", serde_json::json!({}))
            .await?;
        serde_json::from_value(result).context("Failed to parse info.hardware result")
    }
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunResponse {
    pub session_id: String,
    pub response: String,
    #[serde(default)]
    pub tokens_used: Option<u64>,
    #[serde(default)]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct LlmModelEntry {
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Deserialize)]
pub struct LlmModelList {
    pub models: Vec<LlmModelEntry>,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub default_provider: String,
}

#[derive(Debug)]
pub struct LlmProbe {
    pub model: Option<String>,
    pub content: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Deserialize)]
pub struct LlmStats {
    #[serde(default)]
    pub cost: CostExport,
    #[serde(default)]
    pub llm_cache_size: u64,
    #[serde(default)]
    pub llm_cache_capacity: u64,
    #[serde(default)]
    pub llm_cache_hits: u64,
    #[serde(default)]
    pub llm_cache_misses: u64,
    #[serde(default)]
    pub llm_cache_evictions: u64,
    #[serde(default)]
    pub llm_cache_hit_rate: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct CostExport {
    #[serde(default)]
    pub models: Vec<ModelCost>,
}

#[derive(Debug, Deserialize)]
pub struct ModelCost {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    #[serde(default)]
    pub cost_usd: f64,
}

#[derive(Debug, Deserialize)]
pub struct MarketAgent {
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub installed: bool,
}

#[derive(Debug, Deserialize)]
pub struct MarketInstallResult {
    pub status: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub installed_version: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub install_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InfoSystem {
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub kernel_version: String,
    #[serde(default)]
    pub system: Option<SysSnapshot>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SysSnapshot {
    #[serde(default)]
    pub cpu_cores: u64,
    #[serde(default)]
    pub cpu_usage_pct: f64,
    #[serde(default)]
    pub total_memory_kb: f64,
    #[serde(default)]
    pub used_memory_kb: f64,
    #[serde(default)]
    pub memory_usage_pct: f64,
    #[serde(default)]
    pub disk_total_kb: f64,
    #[serde(default)]
    pub disk_free_kb: f64,
    #[serde(default)]
    pub disk_usage_pct: f64,
    #[serde(default)]
    pub uptime_sec: f64,
}

#[derive(Debug, Deserialize)]
pub struct InfoHealth {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub collecting: bool,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub staleness_sec: f64,
    #[serde(default)]
    pub uptime_s: f64,
}

#[derive(Debug, Deserialize)]
pub struct InfoHardware {
    #[serde(default)]
    pub cpu_count: Option<u64>,
    #[serde(default)]
    pub mem_total_kib: Option<f64>,
    #[serde(default)]
    pub mem_avail_kib: Option<f64>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub accel_present: Option<bool>,
    #[serde(default)]
    pub accel_count: Option<u64>,
    #[serde(default)]
    pub accel_model: Option<String>,
}
