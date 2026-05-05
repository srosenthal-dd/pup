use anyhow::Result;
use std::io::Write;

const DOCS_AI_URL: &str = "https://app.datadoghq.com/api/unstable/docs-ai/chat";
// Public key embedded in the Datadog docs site JS — not a user credential.
const DOCS_AI_API_KEY: &str = "ddpub_docsai_nkbIDfPWw4pKuRlLef8aDs2onVqdimFI";

// Allows test overrides without changing the function signature (mirrors PUP_MOCK_SERVER).
fn endpoint_url() -> String {
    std::env::var("PUP_DOCS_AI_URL").unwrap_or_else(|_| DOCS_AI_URL.to_string())
}

/// Ask the Datadog Docs AI a question and stream the response to `out`.
///
/// `out` is injected so callers can redirect output (pass `&mut std::io::stdout()`
/// for normal use) and tests can capture or control write failures.
#[cfg(not(target_arch = "wasm32"))]
pub async fn ask(question: &str, out: &mut impl Write) -> Result<()> {
    use futures::StreamExt;

    if question.trim().is_empty() {
        anyhow::bail!("question cannot be empty");
    }

    let conversation_id = format!("dd_docsai_{}", uuid::Uuid::new_v4());

    let body = serde_json::json!({
        "data": {
            "attributes": {
                "query": question,
                "conversation_id": conversation_id,
                "anchor_url": "https://docs.datadoghq.com/",
                "rewrite_query": true
            }
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .post(endpoint_url())
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .header("x-docs-ai-api-key", DOCS_AI_API_KEY)
        .header("User-Agent", crate::useragent::get())
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Docs AI request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = match resp.text().await {
            Ok(b) => b,
            Err(e) => format!("<failed to read error body: {e}>"),
        };
        anyhow::bail!("Docs AI error (HTTP {status}): {err_body}");
    }

    let mut buffer = String::new();
    let mut bytes_stream = resp.bytes_stream();
    let mut saw_done = false;

    // SSE framing follows the same pattern as bits.rs: accumulate chunks,
    // split on \n\n event boundaries, strip "data: " prefix per line.
    'outer: while let Some(chunk_result) = bytes_stream.next().await {
        let chunk = chunk_result.map_err(|e| anyhow::anyhow!("Stream read error: {e}"))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(end) = buffer.find("\n\n") {
            let event_block = buffer[..end].to_string();
            buffer = buffer[end + 2..].to_string();

            for line in event_block.lines() {
                let Some(data_str) = line.strip_prefix("data: ") else {
                    continue;
                };
                if data_str.trim() == "[DONE]" {
                    saw_done = true;
                    break 'outer;
                }
                if let Some(text) = extract_sse_content(data_str) {
                    if !emit(out, &text)? {
                        return Ok(());
                    }
                }
            }
        }
    }

    // Drain any remaining buffer when the stream ends without [DONE].
    for line in buffer.lines() {
        let Some(data_str) = line.strip_prefix("data: ") else {
            continue;
        };
        if data_str.trim() == "[DONE]" {
            saw_done = true;
        } else if let Some(text) = extract_sse_content(data_str) {
            if !emit(out, &text)? {
                return Ok(());
            }
        }
    }

    if !saw_done {
        eprintln!(
            "Warning: Docs AI stream ended without a completion signal — the response may be truncated."
        );
    }
    writeln!(out).map_err(|e| anyhow::anyhow!("Failed to write Docs AI response: {e}"))?;
    Ok(())
}

/// Write `text` to `out` and flush. Returns `Ok(false)` on `BrokenPipe` (caller
/// should stop and return `Ok(())`), `Ok(true)` on success, `Err` on other I/O errors.
fn emit(out: &mut impl Write, text: &str) -> Result<bool> {
    if let Err(e) = writeln!(out, "{text}") {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            return Ok(false);
        }
        return Err(anyhow::anyhow!("Failed to write Docs AI response: {e}"));
    }
    if let Err(e) = out.flush() {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            return Ok(false);
        }
        return Err(anyhow::anyhow!("Failed to write Docs AI response: {e}"));
    }
    Ok(true)
}

/// Extract text content from a single SSE data line.
///
/// Tries multiple envelope shapes in priority order. Returns None for non-JSON
/// frames, metadata events, or shapes that carry no printable content.
pub(crate) fn extract_sse_content(data_str: &str) -> Option<String> {
    // Non-JSON lines are intentionally skipped — SSE streams include comment lines,
    // heartbeat pings, and control frames (e.g. "event: ping") that are not JSON.
    let val = serde_json::from_str::<serde_json::Value>(data_str).ok()?;
    let text = val
        .pointer("/data/attributes/content")
        .or_else(|| val.pointer("/attributes/content"))
        .or_else(|| val.get("content"))
        .or_else(|| val.get("text"))
        .and_then(|v| v.as_str())?;
    Some(text.to_string())
}

// ---------------------------------------------------------------------------
// WASM stub
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub async fn ask(_question: &str, _out: &mut impl Write) -> Result<()> {
    anyhow::bail!("docs ask is not supported in WASM builds")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_sse_content unit tests ---

    #[test]
    fn extract_nested_data_attributes_content() {
        let json = r#"{"data":{"attributes":{"content":"hello world"}}}"#;
        assert_eq!(extract_sse_content(json).as_deref(), Some("hello world"));
    }

    #[test]
    fn extract_attributes_content() {
        let json = r#"{"attributes":{"content":"from attributes"}}"#;
        assert_eq!(
            extract_sse_content(json).as_deref(),
            Some("from attributes")
        );
    }

    #[test]
    fn extract_flat_content_field() {
        let json = r#"{"content":"flat content"}"#;
        assert_eq!(extract_sse_content(json).as_deref(), Some("flat content"));
    }

    #[test]
    fn extract_flat_text_field() {
        let json = r#"{"text":"flat text"}"#;
        assert_eq!(extract_sse_content(json).as_deref(), Some("flat text"));
    }

    #[test]
    fn extract_returns_none_for_non_json() {
        assert_eq!(extract_sse_content("not json"), None);
        assert_eq!(extract_sse_content(""), None);
    }

    #[test]
    fn extract_returns_none_when_no_content_field() {
        assert_eq!(extract_sse_content(r#"{"data":{"attributes":{}}}"#), None);
        assert_eq!(extract_sse_content(r#"{"other":"value"}"#), None);
    }

    // --- ask() integration tests via mockito ---

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn ask_streams_sse_content_to_out() {
        let _guard = crate::test_utils::ENV_LOCK.lock().await;

        let mut server = mockito::Server::new_async().await;
        let sse_body = concat!(
            "data: {\"content\":\"Hello\"}\n\n",
            "data: {\"content\":\", world\"}\n\n",
            "data: [DONE]\n\n",
        );
        let _mock = server
            .mock("POST", "/api/unstable/docs-ai/chat")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(sse_body)
            .create_async()
            .await;

        std::env::set_var(
            "PUP_DOCS_AI_URL",
            format!("{}/api/unstable/docs-ai/chat", server.url()),
        );
        let mut buf = Vec::new();
        let result = ask("what is a monitor?", &mut buf).await;
        std::env::remove_var("PUP_DOCS_AI_URL");

        result.expect("ask() should succeed");
        assert!(
            buf.starts_with(b"Hello\n, world\n"),
            "output should contain streamed content, one event per line"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn ask_returns_error_on_non_200() {
        let _guard = crate::test_utils::ENV_LOCK.lock().await;

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/api/unstable/docs-ai/chat")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":"rate limited"}"#)
            .create_async()
            .await;

        std::env::set_var(
            "PUP_DOCS_AI_URL",
            format!("{}/api/unstable/docs-ai/chat", server.url()),
        );
        let result = ask("what is a monitor?", &mut std::io::sink()).await;
        std::env::remove_var("PUP_DOCS_AI_URL");

        let err = result.expect_err("ask() should fail on HTTP 429");
        let msg = err.to_string();
        assert!(msg.contains("429"), "error should mention HTTP status");
        assert!(
            msg.contains("rate limited"),
            "error should include the response body"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn ask_rejects_empty_question() {
        let result = ask("", &mut std::io::sink()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("question cannot be empty"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn ask_rejects_whitespace_only_question() {
        let result = ask("   ", &mut std::io::sink()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("question cannot be empty"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn ask_handles_broken_pipe_gracefully() {
        let _guard = crate::test_utils::ENV_LOCK.lock().await;

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/api/unstable/docs-ai/chat")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body("data: {\"content\":\"Hello\"}\n\ndata: [DONE]\n\n")
            .create_async()
            .await;

        std::env::set_var(
            "PUP_DOCS_AI_URL",
            format!("{}/api/unstable/docs-ai/chat", server.url()),
        );
        let result = ask("what is a monitor?", &mut BrokenPipeWriter).await;
        std::env::remove_var("PUP_DOCS_AI_URL");

        assert!(result.is_ok(), "broken pipe should be silently handled");
    }

    /// A writer whose flush always returns BrokenPipe, simulating a closed pipe.
    struct BrokenPipeWriter;

    impl Write for BrokenPipeWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "broken pipe",
            ))
        }
    }
}
