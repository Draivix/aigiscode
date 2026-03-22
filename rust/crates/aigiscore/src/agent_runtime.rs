use crate::agentic::{
    focus_agentic_review_artifact, AgenticAdapterId, AgenticAdapterInvocation,
    AgenticReviewArtifact, AgenticStructuredReviewResponse,
};
use crate::artifacts::{default_agent_run_paths, default_agent_spider_report_path, AgentRunPaths};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentRunResult {
    pub adapter: AgenticAdapterId,
    pub review_json: PathBuf,
    pub review_markdown: PathBuf,
    pub output_schema: PathBuf,
    pub execution_events: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSpiderPacketResult {
    pub packet_id: String,
    pub primary_target_file: String,
    pub output_dir: PathBuf,
    pub adapter: AgenticAdapterId,
    pub review_json: Option<PathBuf>,
    pub review_markdown: Option<PathBuf>,
    pub output_schema: Option<PathBuf>,
    pub execution_events: Option<PathBuf>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSpiderRunResult {
    pub adapter: AgenticAdapterId,
    pub aggregate_report: PathBuf,
    pub packet_limit: usize,
    pub completed_packets: usize,
    pub failed_packets: usize,
    pub packets: Vec<AgentSpiderPacketResult>,
}

#[derive(Debug, Error)]
pub enum AgentRunError {
    #[error("unsupported agent adapter: {0}")]
    UnsupportedAdapter(String),
    #[error("requested adapter is not present in agentic-review.json: {0}")]
    AdapterNotAvailable(String),
    #[error("required environment variable `{0}` is not set")]
    MissingEnvVar(&'static str),
    #[error("failed to create agent-run output directory: {0}")]
    CreateOutputDir(#[source] std::io::Error),
    #[error("failed to write agent output schema `{path}`: {source}")]
    WriteSchema {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write execution event log `{path}`: {source}")]
    WriteEvents {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write agent structured review `{path}`: {source}")]
    WriteReview {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read agent structured review `{path}`: {source}")]
    ReadReview {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse agent structured review `{path}`: {source}")]
    ParseReview {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write agent markdown report `{path}`: {source}")]
    WriteMarkdown {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to build OpenAI HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),
    #[error("failed to send OpenAI Responses request: {0}")]
    SendHttpRequest(#[source] reqwest::Error),
    #[error("OpenAI Responses request failed with status {status}: {body}")]
    OpenAiRequestFailed { status: u16, body: String },
    #[error("failed to parse OpenAI response JSON: {0}")]
    ParseHttpResponse(#[source] serde_json::Error),
    #[error("OpenAI response did not include a response id for background polling")]
    MissingResponseId,
    #[error("OpenAI response did not include any assistant output text")]
    MissingOutputText,
    #[error("OpenAI response ended with status `{status}`")]
    IncompleteOpenAiResponse { status: String },
    #[error("failed to start codex exec: {0}")]
    StartCodex(#[source] std::io::Error),
    #[error("codex exec failed with status {status}: {stderr}")]
    CodexExecFailed { status: i32, stderr: String },
}

pub fn parse_agent_adapter_id(value: &str) -> Result<AgenticAdapterId, AgentRunError> {
    match value {
        "codex-exec" | "codex_exec" | "codex_exec_cli" => Ok(AgenticAdapterId::CodexExecCli),
        "responses-http" | "responses_http" | "openai-responses" | "openai_responses_http" => {
            Ok(AgenticAdapterId::OpenAiResponsesHttp)
        }
        "codex-sdk" | "codex_sdk" | "codex_sdk_typescript" => {
            Ok(AgenticAdapterId::CodexSdkTypeScript)
        }
        other => Err(AgentRunError::UnsupportedAdapter(other.to_owned())),
    }
}

pub fn run_agent_review(
    review: &AgenticReviewArtifact,
    repo_root: &Path,
    output_dir: Option<&Path>,
    adapter: AgenticAdapterId,
    model_override: Option<&str>,
) -> Result<AgentRunResult, AgentRunError> {
    let paths = default_agent_run_paths(repo_root, output_dir);
    fs::create_dir_all(&paths.output_dir).map_err(AgentRunError::CreateOutputDir)?;
    let adapter_plan = review
        .execution
        .adapters
        .iter()
        .find(|plan| plan.id == adapter)
        .ok_or_else(|| AgentRunError::AdapterNotAvailable(format!("{adapter:?}")))?;

    match &adapter_plan.invocation {
        AgenticAdapterInvocation::CodexExecCli {
            binary,
            default_model,
            schema_flag,
            output_file_flag,
            json_events_flag,
            default_sandbox,
            ..
        } => run_codex_exec(
            review,
            repo_root,
            &paths,
            binary,
            model_override.unwrap_or(default_model),
            schema_flag,
            output_file_flag,
            json_events_flag,
            default_sandbox,
        ),
        AgenticAdapterInvocation::OpenAiResponsesHttp {
            endpoint,
            method,
            model,
            reasoning_effort,
            background,
            ..
        } => run_openai_responses_http(
            review,
            &paths,
            endpoint,
            method,
            model_override.unwrap_or(model),
            reasoning_effort,
            *background,
        ),
        AgenticAdapterInvocation::CodexSdkTypeScript { .. } => Err(
            AgentRunError::UnsupportedAdapter(String::from(
                "codex_sdk_typescript is not implemented yet; use codex-exec today or build a Node sidecar",
            )),
        ),
    }
}

pub fn run_agent_spider(
    review: &AgenticReviewArtifact,
    repo_root: &Path,
    output_dir: Option<&Path>,
    adapter: AgenticAdapterId,
    model_override: Option<&str>,
    packet_limit: usize,
) -> Result<AgentSpiderRunResult, AgentRunError> {
    let aggregate_report = default_agent_spider_report_path(repo_root, output_dir);
    let spider_root = aggregate_report
        .parent()
        .expect("aggregate spider report must have a parent")
        .join("agent-spider");
    fs::create_dir_all(&spider_root).map_err(AgentRunError::CreateOutputDir)?;

    let mut packets = Vec::new();
    for (index, packet) in review.task_packets.iter().take(packet_limit).enumerate() {
        let packet_output_dir = spider_root.join(format!(
            "{:02}-{}",
            index + 1,
            sanitize_packet_id(&packet.id)
        ));
        let focused = match focus_agentic_review_artifact(review, &packet.id) {
            Some(review) => review,
            None => continue,
        };
        let packet_result = match run_agent_review(
            &focused,
            repo_root,
            Some(&packet_output_dir),
            adapter,
            model_override,
        ) {
            Ok(result) => AgentSpiderPacketResult {
                packet_id: packet.id.clone(),
                primary_target_file: packet.primary_target_file.clone(),
                output_dir: packet_output_dir,
                adapter: result.adapter,
                review_json: Some(result.review_json),
                review_markdown: Some(result.review_markdown),
                output_schema: Some(result.output_schema),
                execution_events: Some(result.execution_events),
                error: None,
            },
            Err(error) => AgentSpiderPacketResult {
                packet_id: packet.id.clone(),
                primary_target_file: packet.primary_target_file.clone(),
                output_dir: packet_output_dir,
                adapter,
                review_json: None,
                review_markdown: None,
                output_schema: None,
                execution_events: None,
                error: Some(error.to_string()),
            },
        };
        packets.push(packet_result);
    }

    let result = AgentSpiderRunResult {
        adapter,
        aggregate_report: aggregate_report.clone(),
        packet_limit,
        completed_packets: packets
            .iter()
            .filter(|packet| packet.error.is_none())
            .count(),
        failed_packets: packets
            .iter()
            .filter(|packet| packet.error.is_some())
            .count(),
        packets,
    };
    let report_bytes =
        serde_json::to_vec_pretty(&result).expect("failed to serialize aggregate spider report");
    fs::write(&aggregate_report, report_bytes).map_err(|source| AgentRunError::WriteEvents {
        path: aggregate_report,
        source,
    })?;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn run_codex_exec(
    review: &AgenticReviewArtifact,
    repo_root: &Path,
    paths: &AgentRunPaths,
    binary: &str,
    model: &str,
    schema_flag: &str,
    output_file_flag: &str,
    json_events_flag: &str,
    sandbox: &str,
) -> Result<AgentRunResult, AgentRunError> {
    let schema_json = serde_json::to_vec_pretty(&review.execution.structured_output.json_schema)
        .expect("failed to serialize existing agent schema");
    fs::write(&paths.output_schema, schema_json).map_err(|source| AgentRunError::WriteSchema {
        path: paths.output_schema.clone(),
        source,
    })?;

    let artifact_dir = artifact_dir_for_prompt(repo_root, &paths.output_dir);
    let prompt = format!(
        "{}\n\n{}\n\nRead the required AigisCode context artifacts from `{artifact_dir}` before judging the repository. Produce the final answer strictly as JSON matching the provided schema. Every claim must cite the relevant `task_packet_id`, and `report_markdown` must contain the complete human-readable report for `{}`.",
        review.system_prompt,
        review.user_prompt,
        paths.review_markdown.display(),
    );

    let mut command = Command::new(binary);
    command
        .arg("exec")
        .arg("--cd")
        .arg(repo_root)
        .arg("--ephemeral")
        .arg("--sandbox")
        .arg(sandbox)
        .arg("--model")
        .arg(model)
        .arg(json_events_flag)
        .arg(schema_flag)
        .arg(&paths.output_schema)
        .arg(output_file_flag)
        .arg(&paths.review_json)
        .arg(prompt);

    if paths.output_dir.strip_prefix(repo_root).is_err() {
        command.arg("--add-dir").arg(&paths.output_dir);
    }

    let output = command.output().map_err(AgentRunError::StartCodex)?;
    fs::write(&paths.execution_events, &output.stdout).map_err(|source| {
        AgentRunError::WriteEvents {
            path: paths.execution_events.clone(),
            source,
        }
    })?;
    if !output.status.success() {
        return Err(AgentRunError::CodexExecFailed {
            status: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let review_json =
        fs::read_to_string(&paths.review_json).map_err(|source| AgentRunError::ReadReview {
            path: paths.review_json.clone(),
            source,
        })?;
    let structured: AgenticStructuredReviewResponse =
        serde_json::from_str(&review_json).map_err(|source| AgentRunError::ParseReview {
            path: paths.review_json.clone(),
            source,
        })?;
    fs::write(&paths.review_markdown, structured.report_markdown).map_err(|source| {
        AgentRunError::WriteMarkdown {
            path: paths.review_markdown.clone(),
            source,
        }
    })?;

    Ok(AgentRunResult {
        adapter: AgenticAdapterId::CodexExecCli,
        review_json: paths.review_json.clone(),
        review_markdown: paths.review_markdown.clone(),
        output_schema: paths.output_schema.clone(),
        execution_events: paths.execution_events.clone(),
    })
}

fn run_openai_responses_http(
    review: &AgenticReviewArtifact,
    paths: &AgentRunPaths,
    endpoint: &str,
    method: &str,
    model: &str,
    reasoning_effort: &str,
    background: bool,
) -> Result<AgentRunResult, AgentRunError> {
    if !method.eq_ignore_ascii_case("POST") {
        return Err(AgentRunError::UnsupportedAdapter(format!(
            "unsupported responses-http method: {method}"
        )));
    }
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| AgentRunError::MissingEnvVar("OPENAI_API_KEY"))?;
    let schema_json = serde_json::to_vec_pretty(&review.execution.structured_output.json_schema)
        .expect("failed to serialize existing agent schema");
    fs::write(&paths.output_schema, schema_json).map_err(|source| AgentRunError::WriteSchema {
        path: paths.output_schema.clone(),
        source,
    })?;

    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(AgentRunError::BuildHttpClient)?;
    let request_body =
        build_openai_responses_request_body(review, model, reasoning_effort, background);
    let initial = send_openai_json_request(&client, endpoint, &api_key, &request_body)?;
    let mut events = vec![initial.clone()];
    let final_response = if background {
        let response_id = initial
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or(AgentRunError::MissingResponseId)?;
        poll_openai_background_response(&client, endpoint, response_id, &api_key, &mut events)?
    } else {
        initial
    };

    let mut events_jsonl = String::new();
    for event in &events {
        events_jsonl.push_str(
            &serde_json::to_string(event).expect("failed to serialize HTTP execution event"),
        );
        events_jsonl.push('\n');
    }
    fs::write(&paths.execution_events, events_jsonl).map_err(|source| {
        AgentRunError::WriteEvents {
            path: paths.execution_events.clone(),
            source,
        }
    })?;

    let output_text = extract_openai_output_text(&final_response)?;
    fs::write(&paths.review_json, output_text.as_bytes()).map_err(|source| {
        AgentRunError::WriteReview {
            path: paths.review_json.clone(),
            source,
        }
    })?;
    let structured: AgenticStructuredReviewResponse =
        serde_json::from_str(&output_text).map_err(|source| AgentRunError::ParseReview {
            path: paths.review_json.clone(),
            source,
        })?;
    fs::write(&paths.review_markdown, structured.report_markdown).map_err(|source| {
        AgentRunError::WriteMarkdown {
            path: paths.review_markdown.clone(),
            source,
        }
    })?;

    Ok(AgentRunResult {
        adapter: AgenticAdapterId::OpenAiResponsesHttp,
        review_json: paths.review_json.clone(),
        review_markdown: paths.review_markdown.clone(),
        output_schema: paths.output_schema.clone(),
        execution_events: paths.execution_events.clone(),
    })
}

fn send_openai_json_request(
    client: &Client,
    endpoint: &str,
    api_key: &str,
    body: &JsonValue,
) -> Result<JsonValue, AgentRunError> {
    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .send()
        .map_err(AgentRunError::SendHttpRequest)?;
    let status = response.status();
    let body_text = response.text().map_err(AgentRunError::SendHttpRequest)?;
    if !status.is_success() {
        return Err(AgentRunError::OpenAiRequestFailed {
            status: status.as_u16(),
            body: redact_sensitive_error_text(&body_text),
        });
    }
    serde_json::from_str(&body_text).map_err(AgentRunError::ParseHttpResponse)
}

fn send_openai_get_request(
    client: &Client,
    endpoint: &str,
    api_key: &str,
) -> Result<JsonValue, AgentRunError> {
    let response = client
        .get(endpoint)
        .bearer_auth(api_key)
        .header("Accept", "application/json")
        .send()
        .map_err(AgentRunError::SendHttpRequest)?;
    let status = response.status();
    let body_text = response.text().map_err(AgentRunError::SendHttpRequest)?;
    if !status.is_success() {
        return Err(AgentRunError::OpenAiRequestFailed {
            status: status.as_u16(),
            body: redact_sensitive_error_text(&body_text),
        });
    }
    serde_json::from_str(&body_text).map_err(AgentRunError::ParseHttpResponse)
}

fn build_openai_responses_request_body(
    review: &AgenticReviewArtifact,
    model: &str,
    reasoning_effort: &str,
    background: bool,
) -> JsonValue {
    let context_payload =
        serde_json::to_string_pretty(review).expect("failed to serialize agentic review context");
    json!({
        "model": model,
        "background": background,
        "reasoning": {
            "effort": reasoning_effort
        },
        "input": [
            {
                "role": "system",
                "content": [
                    {
                        "type": "input_text",
                        "text": review.system_prompt
                    }
                ]
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_text",
                        "text": review.user_prompt
                    },
                    {
                        "type": "input_text",
                        "text": format!(
                            "Use this AigisCode review contract as the source of truth for graph, doctrine, convergence, and task-packet context. Produce only JSON that matches the requested schema.\n\n{}",
                            context_payload
                        )
                    }
                ]
            }
        ],
        "text": {
            "format": {
                "type": "json_schema",
                "name": review.execution.structured_output.schema_name,
                "schema": review.execution.structured_output.json_schema,
                "strict": true
            }
        }
    })
}

fn poll_openai_background_response(
    client: &Client,
    endpoint: &str,
    response_id: &str,
    api_key: &str,
    events: &mut Vec<JsonValue>,
) -> Result<JsonValue, AgentRunError> {
    let poll_endpoint = format!("{endpoint}/{response_id}");
    for _ in 0..120 {
        let response = send_openai_get_request(client, &poll_endpoint, api_key)?;
        let status = response
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown");
        events.push(response.clone());
        match status {
            "completed" => return Ok(response),
            "queued" | "in_progress" => {
                thread::sleep(Duration::from_secs(2));
            }
            other => {
                return Err(AgentRunError::IncompleteOpenAiResponse {
                    status: other.to_owned(),
                });
            }
        }
    }
    Err(AgentRunError::IncompleteOpenAiResponse {
        status: String::from("timeout"),
    })
}

fn extract_openai_output_text(response: &JsonValue) -> Result<String, AgentRunError> {
    if let Some(text) = response.get("output_text").and_then(JsonValue::as_str) {
        return Ok(text.to_owned());
    }
    if let Some(outputs) = response.get("output").and_then(JsonValue::as_array) {
        let mut chunks = Vec::new();
        for output in outputs {
            if output.get("type").and_then(JsonValue::as_str) != Some("message") {
                continue;
            }
            if let Some(contents) = output.get("content").and_then(JsonValue::as_array) {
                for content in contents {
                    if content.get("type").and_then(JsonValue::as_str) == Some("output_text") {
                        if let Some(text) = content.get("text").and_then(JsonValue::as_str) {
                            chunks.push(text.to_owned());
                        }
                    }
                }
            }
        }
        if !chunks.is_empty() {
            return Ok(chunks.join("\n"));
        }
    }
    Err(AgentRunError::MissingOutputText)
}

fn artifact_dir_for_prompt(repo_root: &Path, output_dir: &Path) -> String {
    if let Ok(relative) = output_dir.strip_prefix(repo_root) {
        relative.display().to_string()
    } else {
        output_dir.display().to_string()
    }
}

fn sanitize_packet_id(packet_id: &str) -> String {
    let sanitized = packet_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>();
    sanitized
        .trim_matches('-')
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn redact_sensitive_error_text(text: &str) -> String {
    let mut redacted = text.to_owned();
    for prefix in ["sk-proj-", "sk-"] {
        while let Some(index) = redacted.find(prefix) {
            let rest = &redacted[index + prefix.len()..];
            let token_len = rest
                .chars()
                .take_while(|ch| {
                    !ch.is_whitespace()
                        && *ch != '"'
                        && *ch != '\''
                        && *ch != ','
                        && *ch != '}'
                        && *ch != ']'
                })
                .count();
            if token_len == 0 {
                break;
            }
            let end = index + prefix.len() + token_len;
            redacted.replace_range(index..end, "[REDACTED_API_KEY]");
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::parse_agent_adapter_id;
    use crate::agentic::{build_agentic_review_artifact, AgenticAdapterId};
    use crate::artifacts::{build_agent_handoff_artifact, build_guard_decision_artifact};
    use crate::doctrine::built_in_doctrine_registry;
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use crate::policy::PolicyBundle;
    use crate::review::build_review_surface;
    use crate::surface::build_architecture_surface;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_supported_agent_adapters() {
        assert_eq!(
            parse_agent_adapter_id("codex-exec").unwrap(),
            AgenticAdapterId::CodexExecCli
        );
        assert_eq!(
            parse_agent_adapter_id("openai_responses_http").unwrap(),
            AgenticAdapterId::OpenAiResponsesHttp
        );
        assert_eq!(
            parse_agent_adapter_id("codex-sdk").unwrap(),
            AgenticAdapterId::CodexSdkTypeScript
        );
    }

    #[test]
    fn agentic_review_exposes_multiple_adapter_plans() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);
        let doctrine = built_in_doctrine_registry();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let convergence = crate::artifacts::build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);
        let artifact =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        assert_eq!(
            artifact.execution.preferred_local_adapter,
            AgenticAdapterId::CodexExecCli
        );
        assert_eq!(
            artifact.execution.preferred_service_adapter,
            AgenticAdapterId::OpenAiResponsesHttp
        );
        assert!(artifact
            .execution
            .adapters
            .iter()
            .any(|adapter| { adapter.id == AgenticAdapterId::CodexSdkTypeScript }));
    }

    #[test]
    fn extracts_output_text_from_responses_message_shape() {
        let response = json!({
            "status": "completed",
            "output": [
                {
                    "type": "message",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "{\"verdict\":\"warn\"}"
                        }
                    ]
                }
            ]
        });
        assert_eq!(
            super::extract_openai_output_text(&response).unwrap(),
            "{\"verdict\":\"warn\"}"
        );
    }

    #[test]
    fn builds_openai_request_body_with_strict_json_schema() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);
        let doctrine = built_in_doctrine_registry();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let convergence = crate::artifacts::build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);
        let artifact =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        let body =
            super::build_openai_responses_request_body(&artifact, "gpt-5.4", "medium", false);
        assert_eq!(body["model"], "gpt-5.4");
        assert_eq!(body["background"], false);
        assert_eq!(body["text"]["format"]["strict"], true);
        assert_eq!(
            body["text"]["format"]["name"],
            artifact.execution.structured_output.schema_name
        );
    }

    #[test]
    fn sanitizes_packet_ids_for_spider_directories() {
        assert_eq!(
            super::sanitize_packet_id("guardian:compatibility-scar:EntityUiConfigService.php"),
            "guardian-compatibility-scar-EntityUiConfigService-php"
        );
    }

    #[test]
    fn redacts_sensitive_api_keys_from_error_text() {
        let message =
            "Incorrect API key provided: sk-proj-abc123******xyz. You can find your API key.";
        let redacted = super::redact_sensitive_error_text(message);
        assert!(!redacted.contains("sk-proj-abc123******xyz"));
        assert!(redacted.contains("[REDACTED_API_KEY]"));
    }

    fn create_fixture() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscode-agent-runtime-fixture-{nonce}"));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        path
    }
}
