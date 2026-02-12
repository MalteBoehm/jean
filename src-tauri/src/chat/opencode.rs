//! OpenCode CLI execution module
//!
//! Mirrors the claude.rs module but for the OpenCode CLI.
//! Handles building CLI arguments, spawning detached processes,
//! and tailing NDJSON output for real-time event streaming.

use super::types::{ContentBlock, EffortLevel, ThinkingLevel, ToolCall, UsageData};
use crate::http_server::EmitExt;

// =============================================================================
// OpenCode CLI response type
// =============================================================================

/// Response from OpenCode CLI execution (mirrors ClaudeResponse)
pub struct OpenCodeResponse {
    /// The text response from OpenCode
    pub content: String,
    /// The OpenCode session ID (for resuming conversations)
    pub session_id: String,
    /// Tool calls made during this response
    pub tool_calls: Vec<ToolCall>,
    /// Ordered content blocks preserving tool position in response
    pub content_blocks: Vec<ContentBlock>,
    /// Whether the response was cancelled by the user
    pub cancelled: bool,
    /// Token usage for this response
    pub usage: Option<UsageData>,
}

// =============================================================================
// Event types (reuse Claude's event types via claude module)
// =============================================================================

/// Payload for text chunk events sent to frontend
#[derive(serde::Serialize, Clone)]
struct ChunkEvent {
    session_id: String,
    worktree_id: String,
    content: String,
}

/// Payload for tool use events sent to frontend
#[derive(serde::Serialize, Clone)]
struct ToolUseEvent {
    session_id: String,
    worktree_id: String,
    id: String,
    name: String,
    input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_tool_use_id: Option<String>,
}

/// Payload for done events sent to frontend
#[derive(serde::Serialize, Clone)]
struct DoneEvent {
    session_id: String,
    worktree_id: String,
}

/// Payload for error events sent to frontend
#[derive(serde::Serialize, Clone)]
pub struct ErrorEvent {
    pub session_id: String,
    pub worktree_id: String,
    pub error: String,
}

/// Payload for cancelled events sent to frontend
#[allow(dead_code)]
#[derive(serde::Serialize, Clone)]
pub struct CancelledEvent {
    pub session_id: String,
    pub worktree_id: String,
    pub undo_send: bool,
}

/// Payload for tool block position events sent to frontend
#[derive(serde::Serialize, Clone)]
struct ToolBlockEvent {
    session_id: String,
    worktree_id: String,
    tool_call_id: String,
}

/// Payload for thinking events sent to frontend
#[derive(serde::Serialize, Clone)]
struct ThinkingEvent {
    session_id: String,
    worktree_id: String,
    content: String,
}

/// Payload for tool result events sent to frontend
#[derive(serde::Serialize, Clone)]
struct ToolResultEvent {
    session_id: String,
    worktree_id: String,
    tool_use_id: String,
    output: String,
}

// =============================================================================
// Build OpenCode CLI arguments
// =============================================================================

/// Build CLI arguments for OpenCode CLI.
///
/// Returns a tuple of (args, env_vars) where env_vars are (key, value) pairs.
#[allow(clippy::too_many_arguments)]
fn build_opencode_args(
    _app: &tauri::AppHandle,
    _session_id: &str,
    _worktree_id: &str,
    existing_opencode_session_id: Option<&str>,
    model: Option<&str>,
    execution_mode: Option<&str>,
    thinking_level: Option<&ThinkingLevel>,
    _effort_level: Option<&EffortLevel>,
    ai_language: Option<&str>,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut args = Vec::new();
    let env_vars = Vec::new();

    // Core: non-interactive mode with JSON output
    args.push("run".to_string());
    args.push("--format".to_string());
    args.push("json".to_string());

    // Model
    if let Some(m) = model {
        args.push("--model".to_string());
        args.push(m.to_string());
    }

    // Agent selection: map Jean's execution_mode to OpenCode's --agent
    // plan mode → plan agent (read-only)
    // build/yolo mode → build agent (full access)
    match execution_mode.unwrap_or("plan") {
        "plan" => {
            args.push("--agent".to_string());
            args.push("plan".to_string());
        }
        "build" | "yolo" => {
            args.push("--agent".to_string());
            args.push("build".to_string());
        }
        _ => {
            args.push("--agent".to_string());
            args.push("plan".to_string());
        }
    }

    // Thinking level → variant flag (OpenCode uses --variant for thinking modes)
    if let Some(level) = thinking_level {
        match level {
            ThinkingLevel::Think => {
                // Standard thinking - no variant override needed
            }
            ThinkingLevel::Megathink => {
                args.push("--variant".to_string());
                args.push("high".to_string());
            }
            ThinkingLevel::Ultrathink => {
                args.push("--variant".to_string());
                args.push("max".to_string());
            }
            ThinkingLevel::Off => {
                // No thinking — don't pass variant
            }
        }
    }

    // NOTE: --system-prompt does NOT exist in `opencode run`.
    // Language preference is handled by prepending to the user message in write_opencode_input_file.
    // We store it for later use when writing the input.
    let _ = ai_language; // consumed by caller, not by CLI args

    // Session resumption
    if let Some(oc_sid) = existing_opencode_session_id {
        args.push("--session".to_string());
        args.push(oc_sid.to_string());
    }

    // NOTE: --yes does NOT exist in `opencode run`.
    // OpenCode handles auto-approval differently (via config or agent behavior).

    (args, env_vars)
}

// =============================================================================
// Detached OpenCode CLI execution
// =============================================================================

/// Execute OpenCode CLI in detached mode.
///
/// Spawns OpenCode CLI as a fully detached process that survives Jean quitting.
/// Mirrors execute_claude_detached but for OpenCode CLI.
#[allow(clippy::too_many_arguments)]
pub fn execute_opencode_detached(
    app: &tauri::AppHandle,
    session_id: &str,
    worktree_id: &str,
    input_file: &std::path::Path,
    output_file: &std::path::Path,
    working_dir: &std::path::Path,
    existing_opencode_session_id: Option<&str>,
    model: Option<&str>,
    execution_mode: Option<&str>,
    thinking_level: Option<&ThinkingLevel>,
    effort_level: Option<&EffortLevel>,
    ai_language: Option<&str>,
) -> Result<(u32, OpenCodeResponse), String> {
    use super::detached::spawn_detached_claude;

    log::trace!("Executing OpenCode CLI (detached) for session: {session_id}");
    log::trace!("Input file: {input_file:?}");
    log::trace!("Output file: {output_file:?}");
    log::trace!("Working directory: {working_dir:?}");

    // Find opencode binary
    let cli_path = which::which("opencode").map_err(|e| {
        let error_msg = format!(
            "OpenCode CLI not found in PATH: {e}. Please install OpenCode: https://opencode.ai"
        );
        log::error!("{error_msg}");
        let _ = app.emit_all(
            "chat:error",
            &ErrorEvent {
                session_id: session_id.to_string(),
                worktree_id: worktree_id.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    // Build args
    let (args, env_vars) = build_opencode_args(
        app,
        session_id,
        worktree_id,
        existing_opencode_session_id,
        model,
        execution_mode,
        thinking_level,
        effort_level,
        ai_language,
    );

    log::debug!(
        "OpenCode CLI command: {} {}",
        cli_path.display(),
        args.join(" ")
    );

    // Convert env_vars to &str references
    let env_refs: Vec<(&str, &str)> = env_vars
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // Spawn detached process (reuse the same infrastructure as Claude CLI)
    let pid = spawn_detached_claude(
        &cli_path,
        &args,
        input_file,
        output_file,
        working_dir,
        &env_refs,
    )
    .map_err(|e| {
        let error_msg = format!("Failed to start OpenCode CLI: {e}");
        log::error!("{error_msg}");
        let _ = app.emit_all(
            "chat:error",
            &ErrorEvent {
                session_id: session_id.to_string(),
                worktree_id: worktree_id.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    log::trace!("Detached OpenCode CLI spawned with PID: {pid}");

    // Register the process for cancellation
    super::registry::register_process(session_id.to_string(), pid);

    // Tail the output file for real-time updates
    let response = match tail_opencode_output(app, session_id, worktree_id, output_file, pid) {
        Ok(resp) => {
            super::registry::unregister_process(session_id);
            resp
        }
        Err(e) => {
            super::registry::unregister_process(session_id);
            return Err(e);
        }
    };

    Ok((pid, response))
}

// =============================================================================
// File-based tailing for detached OpenCode CLI
// =============================================================================

/// Tail an NDJSON output file from OpenCode CLI and emit events.
///
/// OpenCode `--format json` outputs NDJSON lines. We map those to
/// Jean's event system (chat:chunk, chat:tool_use, chat:done, etc.)
pub fn tail_opencode_output(
    app: &tauri::AppHandle,
    session_id: &str,
    worktree_id: &str,
    output_file: &std::path::Path,
    pid: u32,
) -> Result<OpenCodeResponse, String> {
    use super::detached::is_process_alive;
    use super::tail::{NdjsonTailer, POLL_INTERVAL};
    use std::time::{Duration, Instant};

    log::trace!("Starting to tail OpenCode NDJSON output for session: {session_id}");

    let mut tailer = NdjsonTailer::new_from_start(output_file)?;

    let mut full_content = String::new();
    let mut opencode_session_id = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut content_blocks: Vec<ContentBlock> = Vec::new();
    let mut completed = false;
    let mut cancelled = false;
    let mut usage: Option<UsageData> = None;

    let startup_timeout = Duration::from_secs(120);
    let dead_process_timeout = Duration::from_secs(2);
    // Interactive stall detection: OpenCode's plan agent may block on stdin
    // waiting for user input. Since we run detached with piped stdin, this
    // will never arrive. Detect the stall and synthesize an AskUserQuestion.
    let interactive_stall_timeout = Duration::from_secs(30);
    let started_at = Instant::now();
    let mut last_output_time = Instant::now();
    let mut received_output = false;
    // Track last event type and text content for stall detection
    let mut last_event_type = String::new();
    let mut last_text_content = String::new();

    loop {
        let lines = tailer.poll()?;

        if !lines.is_empty() {
            last_output_time = Instant::now();
        }

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            // Skip metadata header
            if line.contains("\"_run_meta\"") {
                continue;
            }

            if !received_output {
                log::trace!("Received first OpenCode output for session: {session_id}");
                received_output = true;
            }

            // Parse the JSON line
            let msg: serde_json::Value = match serde_json::from_str(&line) {
                Ok(m) => m,
                Err(e) => {
                    log::trace!("Failed to parse OpenCode line: {e}");
                    continue;
                }
            };

            // Capture session_id from any message
            // OpenCode uses "sessionID" (capital D) at top level and inside part
            for key in ["sessionID", "sessionId", "session_id"] {
                let sid = msg
                    .get(key)
                    .or_else(|| msg.get("part").and_then(|p| p.get(key)))
                    .and_then(|v| v.as_str());
                if let Some(sid) = sid {
                    if !sid.is_empty() {
                        opencode_session_id = sid.to_string();
                        break;
                    }
                }
            }

            let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match msg_type {
                // OpenCode text content events
                "assistant" | "text" | "content" => {
                    last_event_type = msg_type.to_string();
                    // Try to get text from various possible structures
                    let text = extract_text_content(&msg);
                    if let Some(text) = text {
                        if text == "(no content)" {
                            continue;
                        }
                        full_content.push_str(&text);
                        last_text_content = text.clone();
                        content_blocks.push(ContentBlock::Text { text: text.clone() });

                        let event = ChunkEvent {
                            session_id: session_id.to_string(),
                            worktree_id: worktree_id.to_string(),
                            content: text,
                        };
                        if let Err(e) = app.emit_all("chat:chunk", &event) {
                            log::error!("Failed to emit chunk: {e}");
                        }
                    }

                    // Check for tool_use blocks inside assistant messages
                    if let Some(message) = msg.get("message") {
                        if let Some(blocks) = message.get("content").and_then(|c| c.as_array()) {
                            for block in blocks {
                                process_content_block(
                                    app,
                                    session_id,
                                    worktree_id,
                                    block,
                                    &mut full_content,
                                    &mut tool_calls,
                                    &mut content_blocks,
                                    None, // no parent tool use for root-level
                                );
                            }
                        }
                    }
                }

                // OpenCode tool use events
                // OpenCode format: part.tool (name), part.callID (id), part.state.input, part.state.output
                // When state.status="completed", the event includes both the call AND the result
                "tool_use" | "tool_call" => {
                    last_event_type = msg_type.to_string();
                    let part = msg.get("part");

                    // Extract from part (OpenCode format) or top-level (fallback)
                    let id = part
                        .and_then(|p| p.get("callID"))
                        .or_else(|| msg.get("id"))
                        .or_else(|| msg.get("tool_use_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = part
                        .and_then(|p| p.get("tool"))
                        .or_else(|| msg.get("name"))
                        .or_else(|| msg.get("tool_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Input from part.state.input or top-level
                    let state = part.and_then(|p| p.get("state"));
                    let input = state
                        .and_then(|s| s.get("input"))
                        .or_else(|| msg.get("input"))
                        .or_else(|| msg.get("arguments"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    // Check if tool already completed (OpenCode sends result in same event)
                    let status = state
                        .and_then(|s| s.get("status"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let output = if status == "completed" {
                        state.and_then(|s| s.get("output")).map(|v| {
                            if let Some(s) = v.as_str() {
                                // Truncate very long outputs for display (UTF-8 safe)
                                if s.len() > 2000 {
                                    let truncated = s.get(..2000).unwrap_or(s);
                                    format!("{}...(truncated)", truncated)
                                } else {
                                    s.to_string()
                                }
                            } else {
                                v.to_string()
                            }
                        })
                    } else {
                        None
                    };

                    tool_calls.push(ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                        output: output.clone(),
                        parent_tool_use_id: None,
                    });

                    content_blocks.push(ContentBlock::ToolUse {
                        tool_call_id: id.clone(),
                    });

                    let event = ToolUseEvent {
                        session_id: session_id.to_string(),
                        worktree_id: worktree_id.to_string(),
                        id: id.clone(),
                        name: name.clone(),
                        input,
                        parent_tool_use_id: None,
                    };
                    if let Err(e) = app.emit_all("chat:tool_use", &event) {
                        log::error!("Failed to emit tool_use: {e}");
                    }

                    let block_event = ToolBlockEvent {
                        session_id: session_id.to_string(),
                        worktree_id: worktree_id.to_string(),
                        tool_call_id: id.clone(),
                    };
                    if let Err(e) = app.emit_all("chat:tool_block", &block_event) {
                        log::error!("Failed to emit tool_block: {e}");
                    }

                    // If already completed, also emit tool_result immediately
                    if let Some(output_text) = output {
                        let result_event = ToolResultEvent {
                            session_id: session_id.to_string(),
                            worktree_id: worktree_id.to_string(),
                            tool_use_id: id,
                            output: output_text,
                        };
                        if let Err(e) = app.emit_all("chat:tool_result", &result_event) {
                            log::error!("Failed to emit tool_result: {e}");
                        }
                    }
                }

                // OpenCode tool result events
                "tool_result" => {
                    last_event_type = "tool_result".to_string();
                    let tool_id = msg
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let output = msg
                        .get("content")
                        .or_else(|| msg.get("output"))
                        .map(|v| {
                            if let Some(s) = v.as_str() {
                                s.to_string()
                            } else {
                                v.to_string()
                            }
                        })
                        .unwrap_or_default();

                    if let Some(tc) = tool_calls.iter_mut().find(|t| t.id == tool_id) {
                        tc.output = Some(output.clone());
                    }

                    let event = ToolResultEvent {
                        session_id: session_id.to_string(),
                        worktree_id: worktree_id.to_string(),
                        tool_use_id: tool_id.to_string(),
                        output,
                    };
                    if let Err(e) = app.emit_all("chat:tool_result", &event) {
                        log::error!("Failed to emit tool_result: {e}");
                    }
                }

                // OpenCode user messages (may contain tool results)
                "user" => {
                    last_event_type = "user".to_string();
                    if let Some(message) = msg.get("message") {
                        if let Some(blocks) = message.get("content").and_then(|c| c.as_array()) {
                            for block in blocks {
                                let block_type =
                                    block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                if block_type == "tool_result" {
                                    let tool_id = block
                                        .get("tool_use_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let output = extract_tool_result_content(block);

                                    if let Some(tc) =
                                        tool_calls.iter_mut().find(|t| t.id == tool_id)
                                    {
                                        tc.output = Some(output.clone());
                                    }

                                    let event = ToolResultEvent {
                                        session_id: session_id.to_string(),
                                        worktree_id: worktree_id.to_string(),
                                        tool_use_id: tool_id.to_string(),
                                        output,
                                    };
                                    if let Err(e) = app.emit_all("chat:tool_result", &event) {
                                        log::error!("Failed to emit tool_result: {e}");
                                    }
                                }
                            }
                        }
                    }
                }

                // OpenCode thinking events
                "thinking" => {
                    last_event_type = "thinking".to_string();
                    if let Some(thinking) = msg
                        .get("thinking")
                        .or_else(|| msg.get("content"))
                        .and_then(|v| v.as_str())
                    {
                        content_blocks.push(ContentBlock::Thinking {
                            thinking: thinking.to_string(),
                        });

                        let event = ThinkingEvent {
                            session_id: session_id.to_string(),
                            worktree_id: worktree_id.to_string(),
                            content: thinking.to_string(),
                        };
                        if let Err(e) = app.emit_all("chat:thinking", &event) {
                            log::error!("Failed to emit thinking: {e}");
                        }
                    }
                }

                // OpenCode completion events
                "result" | "done" | "end" => {
                    if full_content.is_empty() {
                        if let Some(result) = msg.get("result").and_then(|v| v.as_str()) {
                            full_content = result.to_string();
                        }
                    }

                    // Extract usage data
                    if let Some(usage_obj) = msg.get("usage") {
                        usage = Some(parse_usage_data(usage_obj));
                    }

                    completed = true;
                    log::trace!("Received result message - OpenCode CLI completed");
                }

                // OpenCode step_finish with stop reason
                // Reason is at msg.part.reason, tokens at msg.part.tokens
                "step_finish" => {
                    last_event_type = "step_finish".to_string();
                    let part = msg.get("part");
                    let stop_reason = part
                        .and_then(|p| p.get("reason"))
                        .or_else(|| msg.get("stop_reason"))
                        .and_then(|v| v.as_str());
                    if let Some(reason) = stop_reason {
                        if reason == "stop" || reason == "end_turn" {
                            // Try tokens from part.tokens first, then msg.usage
                            if let Some(tokens_obj) = part.and_then(|p| p.get("tokens")) {
                                usage = Some(parse_usage_data(tokens_obj));
                            } else if let Some(usage_obj) = msg.get("usage") {
                                usage = Some(parse_usage_data(usage_obj));
                            }
                            completed = true;
                        }
                    }
                }

                // OpenCode step_start events — no payload to process,
                // but we track the event type for stall detection
                "step_start" => {
                    last_event_type = "step_start".to_string();
                }

                _ => {
                    log::trace!("Unknown OpenCode message type: {msg_type}");
                }
            }
        }

        if completed {
            break;
        }

        // Check if externally cancelled
        if !super::registry::is_process_running(session_id) {
            log::trace!("Session {session_id} cancelled externally, stopping tail");
            cancelled = true;
            break;
        }

        let process_alive = is_process_alive(pid);

        if received_output {
            if !process_alive && last_output_time.elapsed() > dead_process_timeout {
                log::trace!("OpenCode process {pid} is no longer running and no new output");
                // If we received output but never got a "result", treat as completed
                if !full_content.is_empty() || !tool_calls.is_empty() {
                    // Process completed with output (completed state used by break below)
                } else {
                    cancelled = true;
                }
                break;
            }

            // Interactive stall detection: process is alive but no new output.
            // This happens when OpenCode's plan agent asks clarifying questions
            // and blocks on stdin, which never arrives in detached mode.
            if process_alive
                && last_output_time.elapsed() > interactive_stall_timeout
                && (last_event_type == "text"
                    || last_event_type == "content"
                    || last_event_type == "assistant"
                    || last_event_type == "step_start")
            {
                log::trace!(
                    "Interactive stall detected for session {session_id}: \
                     process alive, no output for {:?}, last_event={last_event_type}",
                    last_output_time.elapsed()
                );

                // Synthesize an AskUserQuestion tool call from the last text
                let ask_id = format!("opencode_ask_{}", uuid::Uuid::new_v4());
                let question_text = if last_text_content.is_empty() {
                    "OpenCode is waiting for your input.".to_string()
                } else {
                    last_text_content.clone()
                };

                let ask_input = serde_json::json!({
                    "questions": [{
                        "question": question_text,
                        "options": [],
                        "allowCustom": true
                    }]
                });

                tool_calls.push(ToolCall {
                    id: ask_id.clone(),
                    name: "AskUserQuestion".to_string(),
                    input: ask_input.clone(),
                    output: None,
                    parent_tool_use_id: None,
                });

                content_blocks.push(ContentBlock::ToolUse {
                    tool_call_id: ask_id.clone(),
                });

                // Emit tool_use event so the frontend renders the question UI
                let tool_event = ToolUseEvent {
                    session_id: session_id.to_string(),
                    worktree_id: worktree_id.to_string(),
                    id: ask_id.clone(),
                    name: "AskUserQuestion".to_string(),
                    input: ask_input,
                    parent_tool_use_id: None,
                };
                if let Err(e) = app.emit_all("chat:tool_use", &tool_event) {
                    log::error!("Failed to emit synthesized AskUserQuestion: {e}");
                }

                let block_event = ToolBlockEvent {
                    session_id: session_id.to_string(),
                    worktree_id: worktree_id.to_string(),
                    tool_call_id: ask_id,
                };
                if let Err(e) = app.emit_all("chat:tool_block", &block_event) {
                    log::error!("Failed to emit tool_block for AskUserQuestion: {e}");
                }

                // Kill the stalled process
                log::trace!("Killing stalled OpenCode process {pid}");
                #[cfg(unix)]
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output();
                }

                // Emit done event so frontend knows streaming is complete
                let done_event = DoneEvent {
                    session_id: session_id.to_string(),
                    worktree_id: worktree_id.to_string(),
                };
                if let Err(e) = app.emit_all("chat:done", &done_event) {
                    log::error!("Failed to emit done event: {e}");
                }

                // Return partial response with the synthesized AskUserQuestion
                return Ok(OpenCodeResponse {
                    content: full_content,
                    session_id: opencode_session_id,
                    tool_calls,
                    content_blocks,
                    cancelled: false,
                    usage: None,
                });
            }
        } else {
            let elapsed = started_at.elapsed();

            if elapsed > startup_timeout {
                log::warn!("Startup timeout exceeded waiting for OpenCode output");
                cancelled = true;
                break;
            }

            let secs = elapsed.as_secs();
            if secs > 0 && secs.is_multiple_of(10) && elapsed.subsec_millis() < 100 {
                log::trace!(
                    "Waiting for OpenCode output... {secs}s elapsed, process_alive: {process_alive}"
                );
            }
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    // Emit done event only if not cancelled
    if !cancelled {
        let done_event = DoneEvent {
            session_id: session_id.to_string(),
            worktree_id: worktree_id.to_string(),
        };
        if let Err(e) = app.emit_all("chat:done", &done_event) {
            log::error!("Failed to emit done event: {e}");
        }
    }

    log::trace!(
        "OpenCode tailing complete: {} chars, {} tool calls, cancelled: {cancelled}",
        full_content.len(),
        tool_calls.len()
    );

    Ok(OpenCodeResponse {
        content: full_content,
        session_id: opencode_session_id,
        tool_calls,
        content_blocks,
        cancelled,
        usage,
    })
}

// =============================================================================
// Helper functions
// =============================================================================

/// Extract text content from various OpenCode NDJSON message formats
///
/// OpenCode's JSON format wraps content in a `part` object:
/// `{"type":"text","part":{"text":"hello","sessionID":"..."}}`
fn extract_text_content(msg: &serde_json::Value) -> Option<String> {
    // OpenCode format: text nested in part.text
    if let Some(text) = msg
        .get("part")
        .and_then(|p| p.get("text"))
        .and_then(|v| v.as_str())
    {
        return Some(text.to_string());
    }

    // Direct text field (fallback)
    if let Some(text) = msg.get("text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }

    // Nested in message.content array
    if let Some(message) = msg.get("message") {
        if let Some(content) = message.get("content") {
            // Content as string
            if let Some(text) = content.as_str() {
                return Some(text.to_string());
            }
            // Content as array of blocks
            if let Some(blocks) = content.as_array() {
                let mut text_parts = Vec::new();
                for block in blocks {
                    if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                            text_parts.push(text.to_string());
                        }
                    }
                }
                if !text_parts.is_empty() {
                    return Some(text_parts.join(""));
                }
            }
        }
    }

    // Content field directly
    if let Some(content) = msg.get("content").and_then(|v| v.as_str()) {
        return Some(content.to_string());
    }

    None
}

/// Extract tool result content from a tool_result block
fn extract_tool_result_content(block: &serde_json::Value) -> String {
    block
        .get("content")
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|item| {
                        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                            item.get("text")
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            }
        })
        .unwrap_or_default()
}

/// Process a single content block from an assistant message
#[allow(clippy::too_many_arguments)]
fn process_content_block(
    app: &tauri::AppHandle,
    session_id: &str,
    worktree_id: &str,
    block: &serde_json::Value,
    full_content: &mut String,
    tool_calls: &mut Vec<ToolCall>,
    content_blocks: &mut Vec<ContentBlock>,
    parent_tool_use_id: Option<&str>,
) {
    let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match block_type {
        "text" => {
            if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                if text == "(no content)" {
                    return;
                }
                full_content.push_str(text);
                content_blocks.push(ContentBlock::Text {
                    text: text.to_string(),
                });
                let event = ChunkEvent {
                    session_id: session_id.to_string(),
                    worktree_id: worktree_id.to_string(),
                    content: text.to_string(),
                };
                if let Err(e) = app.emit_all("chat:chunk", &event) {
                    log::error!("Failed to emit chunk: {e}");
                }
            }
        }
        "tool_use" => {
            let id = block
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = block
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let input = block
                .get("input")
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            tool_calls.push(ToolCall {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
                output: None,
                parent_tool_use_id: parent_tool_use_id.map(|s| s.to_string()),
            });

            content_blocks.push(ContentBlock::ToolUse {
                tool_call_id: id.clone(),
            });

            let event = ToolUseEvent {
                session_id: session_id.to_string(),
                worktree_id: worktree_id.to_string(),
                id: id.clone(),
                name,
                input,
                parent_tool_use_id: parent_tool_use_id.map(|s| s.to_string()),
            };
            if let Err(e) = app.emit_all("chat:tool_use", &event) {
                log::error!("Failed to emit tool_use: {e}");
            }

            let block_event = ToolBlockEvent {
                session_id: session_id.to_string(),
                worktree_id: worktree_id.to_string(),
                tool_call_id: id,
            };
            if let Err(e) = app.emit_all("chat:tool_block", &block_event) {
                log::error!("Failed to emit tool_block: {e}");
            }
        }
        "thinking" => {
            if let Some(thinking) = block.get("thinking").and_then(|v| v.as_str()) {
                content_blocks.push(ContentBlock::Thinking {
                    thinking: thinking.to_string(),
                });

                let event = ThinkingEvent {
                    session_id: session_id.to_string(),
                    worktree_id: worktree_id.to_string(),
                    content: thinking.to_string(),
                };
                if let Err(e) = app.emit_all("chat:thinking", &event) {
                    log::error!("Failed to emit thinking: {e}");
                }
            }
        }
        _ => {}
    }
}

/// Parse OpenCode usage data into Jean's UsageData format
fn parse_usage_data(usage_obj: &serde_json::Value) -> UsageData {
    UsageData {
        input_tokens: usage_obj
            .get("input_tokens")
            .or_else(|| usage_obj.get("inputTokens"))
            .or_else(|| usage_obj.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        output_tokens: usage_obj
            .get("output_tokens")
            .or_else(|| usage_obj.get("outputTokens"))
            .or_else(|| usage_obj.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        cache_read_input_tokens: usage_obj
            .get("cache_read_input_tokens")
            .or_else(|| usage_obj.get("cacheReadInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        cache_creation_input_tokens: usage_obj
            .get("cache_creation_input_tokens")
            .or_else(|| usage_obj.get("cacheCreationInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_content_direct() {
        let msg = serde_json::json!({
            "type": "text",
            "text": "Hello, world!"
        });
        assert_eq!(
            extract_text_content(&msg),
            Some("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_extract_text_content_nested() {
        let msg = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    { "type": "text", "text": "Part 1 " },
                    { "type": "text", "text": "Part 2" }
                ]
            }
        });
        assert_eq!(
            extract_text_content(&msg),
            Some("Part 1 Part 2".to_string())
        );
    }

    #[test]
    fn test_extract_text_content_none() {
        let msg = serde_json::json!({
            "type": "tool_use",
            "id": "call-123"
        });
        assert_eq!(extract_text_content(&msg), None);
    }

    #[test]
    fn test_parse_usage_data() {
        let usage_json = serde_json::json!({
            "input_tokens": 100,
            "output_tokens": 200,
            "cache_read_input_tokens": 50,
            "cache_creation_input_tokens": 25
        });
        let usage = parse_usage_data(&usage_json);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 200);
        assert_eq!(usage.cache_read_input_tokens, 50);
        assert_eq!(usage.cache_creation_input_tokens, 25);
    }

    #[test]
    fn test_parse_usage_data_camel_case() {
        let usage_json = serde_json::json!({
            "inputTokens": 150,
            "outputTokens": 300,
            "cacheReadInputTokens": 75
        });
        let usage = parse_usage_data(&usage_json);
        assert_eq!(usage.input_tokens, 150);
        assert_eq!(usage.output_tokens, 300);
        assert_eq!(usage.cache_read_input_tokens, 75);
    }

    #[test]
    fn test_parse_usage_data_openai_format() {
        let usage_json = serde_json::json!({
            "prompt_tokens": 500,
            "completion_tokens": 100
        });
        let usage = parse_usage_data(&usage_json);
        assert_eq!(usage.input_tokens, 500);
        assert_eq!(usage.output_tokens, 100);
    }

    #[test]
    fn test_extract_tool_result_content_string() {
        let block = serde_json::json!({
            "type": "tool_result",
            "tool_use_id": "call-1",
            "content": "File contents here"
        });
        assert_eq!(extract_tool_result_content(&block), "File contents here");
    }

    #[test]
    fn test_extract_tool_result_content_array() {
        let block = serde_json::json!({
            "type": "tool_result",
            "tool_use_id": "call-1",
            "content": [
                { "type": "text", "text": "Line 1" },
                { "type": "text", "text": "Line 2" }
            ]
        });
        assert_eq!(extract_tool_result_content(&block), "Line 1\nLine 2");
    }
}
