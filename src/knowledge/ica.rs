use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::types::{AnalysisContext, IcaAnalysisResponse, LearningConfig};

static SERVICE_KEY: OnceLock<Option<String>> = OnceLock::new();

/// Get the ICA service key from setec
fn get_service_key(setec_url: &str) -> Option<String> {
    SERVICE_KEY
        .get_or_init(|| {
            let output = Command::new("setec")
                .args(["-s", setec_url, "get", "ica/service-key"])
                .output()
                .ok()?;

            if output.status.success() {
                let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !key.is_empty() {
                    log::info!("Retrieved ICA service key from setec");
                    Some(key)
                } else {
                    log::warn!("ICA service key from setec is empty");
                    None
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Failed to get ICA service key from setec: {}", stderr);
                None
            }
        })
        .clone()
}

/// ICA API client for process analysis
pub struct IcaClient {
    ica_url: String,
    setec_url: String,
}

#[derive(Serialize)]
struct ChatStatelessRequest {
    message: String,
}

#[derive(Deserialize)]
struct ChatStatelessResponse {
    response: String,
    #[allow(dead_code)]
    #[serde(rename = "sessionId")]
    session_id: String,
}

impl IcaClient {
    pub fn new(config: &LearningConfig) -> Self {
        Self {
            ica_url: config.ica_url.clone(),
            setec_url: config.setec_url.clone(),
        }
    }

    /// Check if ICA is available (has service key)
    pub fn is_available(&self) -> bool {
        get_service_key(&self.setec_url).is_some()
    }

    /// Analyze a process context using ICA
    pub fn analyze(&self, context: &AnalysisContext) -> Result<IcaAnalysisResponse> {
        let service_key = get_service_key(&self.setec_url)
            .context("ICA service key not available from setec")?;

        let prompt = build_analysis_prompt(context);

        let request = ChatStatelessRequest { message: prompt };
        let request_body =
            serde_json::to_string(&request).context("Failed to serialize request")?;

        let url = format!("{}/api/v1/chat/stateless", self.ica_url);

        log::debug!("Calling ICA at {} for: {}", url, context.command);

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .set("X-ICA-Service-Key", &service_key)
            .set("X-ICA-Service-Name", "portkiller")
            .timeout(Duration::from_secs(30))
            .send_string(&request_body)
            .context("Failed to call ICA API")?;

        let response_text = response.into_string().context("Failed to read ICA response")?;
        let response_body: ChatStatelessResponse =
            serde_json::from_str(&response_text).context("Failed to parse ICA response")?;

        // Parse the JSON response from Claude
        parse_claude_response(&response_body.response)
    }
}

fn build_analysis_prompt(context: &AnalysisContext) -> String {
    format!(
        r#"Analyze this development process and return ONLY valid JSON (no markdown, no explanation):

{}

Return a JSON object with these exact fields:
{{
  "display_name": "Human-friendly name for this process (e.g., 'DSS Backend API', 'Vite Dev Server')",
  "description": "Brief description of what this process does (1-2 sentences)",
  "category": "One of: frontend, backend, database, cache, proxy, dev_tool, infrastructure, unknown",
  "group_hint": "Optional group name if this seems related to a stack (e.g., 'DSS Stack'), or null",
  "confidence": 0.0-1.0 representing how confident you are in this analysis
}}

Focus on identifying:
- What the service does
- Whether it's part of a larger application stack
- The appropriate category

Return ONLY the JSON object, nothing else."#,
        context.to_prompt()
    )
}

fn parse_claude_response(response: &str) -> Result<IcaAnalysisResponse> {
    // Try to find JSON in the response (Claude sometimes adds extra text)
    let json_str = extract_json(response)?;

    serde_json::from_str(&json_str).context("Failed to parse Claude's JSON response")
}

fn extract_json(text: &str) -> Result<String> {
    // Try to find JSON object in response
    let trimmed = text.trim();

    // If it starts with {, try to find matching }
    if trimmed.starts_with('{') {
        if let Some(end) = find_matching_brace(trimmed) {
            return Ok(trimmed[..=end].to_string());
        }
    }

    // Look for JSON block in markdown code block
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start..].find("```\n").or(trimmed[start..].rfind("```")) {
            let json_start = start + 7;
            let json_end = start + end;
            if json_end > json_start {
                return Ok(trimmed[json_start..json_end].trim().to_string());
            }
        }
    }

    // Try to find any { } block
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = find_matching_brace(&trimmed[start..]) {
            return Ok(trimmed[start..=start + end].to_string());
        }
    }

    anyhow::bail!("No valid JSON found in response: {}", text)
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in s.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_direct() {
        let response = r#"{"display_name": "Test", "description": "A test", "category": "backend", "group_hint": null, "confidence": 0.9}"#;
        let result = extract_json(response).unwrap();
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_extract_json_with_text() {
        let response = r#"Here's the analysis:
{"display_name": "Test", "description": "A test", "category": "backend", "group_hint": null, "confidence": 0.9}
Hope this helps!"#;
        let result = extract_json(response).unwrap();
        assert!(result.contains("display_name"));
    }

    #[test]
    fn test_build_prompt() {
        let context = AnalysisContext {
            command: "node".to_string(),
            port: Some(3001),
            project_name: Some("dss".to_string()),
            container_name: None,
            container_prefix: None,
        };
        let prompt = build_analysis_prompt(&context);
        assert!(prompt.contains("node"));
        assert!(prompt.contains("3001"));
        assert!(prompt.contains("dss"));
    }
}
