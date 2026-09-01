use crate::context::Contextpayload;
use crate::trender::StreamRenderer;
use anyhow::{Context, Result};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

const SYSTEM_PROMPT: &str = "You are a terse, expert debugging assistant. \
You are given a program's raw stderr/stack trace (or, if signal_name starts \
with 'TIMEOUT', a note that the process hung and was killed after a timeout \
instead of crashing) plus source snippets around any referenced lines. \
Explain concisely, in markdown, what most likely went wrong and how to fix \
it — for a real crash, focus on the fault; for a TIMEOUT, focus on infinite \
loops, deadlocks, or blocking waits. Prefer bullet points. Do not restate \
the whole stack trace.";

pub async fn explain_crash(payload: &Contextpayload, mock: bool) -> Result<String> {
    let api_key = std::env::var("GEMINI_API_KEY").ok();

    let mut renderer = StreamRenderer::new();
    let mut raw = String::new();

    if mock || api_key.is_none() {
        if !mock {
            eprintln!(
                "(no GEMINI_API_KEY set — streaming a mock explanation; pass --mock to silence this notice)\n"
            );
        }
        stream_mock(payload, &mut renderer, &mut raw).await?;
        return Ok(raw);
    }

    let api_key = api_key.unwrap();
    let user_content = serde_json::to_string_pretty(payload)?;

    let body = serde_json::json!({
        "system_instruction": { "parts": [{ "text": SYSTEM_PROMPT }] },
        "contents": [{ "role": "user", "parts": [{ "text": user_content }] }]
    });

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:streamGenerateContent?alt=sse&key={api_key}"
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .context("request to LLM gateway failed")?
        .error_for_status()
        .context("LLM gateway returned an error status")?;

    let mut stream = response.bytes_stream().eventsource();

    while let Some(event) = stream.next().await {
        let event = event.context("malformed SSE event from LLM gateway")?;
        if event.data == "[DONE]" {
            break;
        }
        // Each SSE frame is a JSON chunk; pull the text delta out of it.
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&event.data) {
            if let Some(text) = v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                raw.push_str(text);
                renderer.push(text);
            }
        }
    }
    renderer.finish();
    Ok(raw)
}

async fn stream_mock(
    payload: &Contextpayload,
    renderer: &mut StreamRenderer,
    raw: &mut String,
) -> Result<()> {
    let is_timeout = payload
        .signal_name
        .as_deref()
        .map(|s| s.starts_with("TIMEOUT"))
        .unwrap_or(false);

    let explanation = if is_timeout {
        format!(
            "## Summary\n\n\
            The program did not exit on its own and was killed after a timeout \
            (**{}**). This means it hung rather than crashed — no segfault, no \
            abort, just no forward progress.\n\n\
            ## Likely cause\n\n\
            - An unconditional or incorrectly-bounded loop (e.g. a `while` \
            condition that never flips false).\n\
            - A blocking call waiting on something that never arrives — a lock \
            that's never released, a read on a socket/pipe with nothing sent, \
            or a channel recv with no matching send.\n\n\
            ## Suggested fix\n\n\
            - Re-run under a debugger (`gdb -p <pid>` while it's still hung, \
            or attach then `Ctrl-C` + `bt`) to see exactly which line it's stuck on.\n\
            - Add a loop-invariant assertion or iteration cap while debugging \
            to fail fast instead of hanging.\n\
            - If it's blocked on I/O or a lock, check for a missing signal/notify \
            or a channel that's never closed.\n",
            payload.signal_name.as_deref().unwrap_or("timeout"),
        )
    } else {
        format!(
            "## Crash summary\n\n\
            The program was killed by **{}** at `{}:{}`.\n\n\
            ## Likely cause\n\n\
            - The snippet around the crash site suggests a null or out-of-bounds \
            pointer dereference — double check any pointer arithmetic or array \
            indexing right before this line.\n\
            - If this is user input driven, add a bounds check before the access.\n\n\
            ## Suggested fix\n\n\
            - Add a guard clause validating the pointer/index is in range.\n\
            - Recompile with `-fsanitize=address` to pinpoint the exact faulting access.\n",
            payload.signal_name.as_deref().unwrap_or("an unknown signal"),
            payload.file.as_deref().unwrap_or("<unknown file>"),
            payload.line.map(|l| l.to_string()).unwrap_or_else(|| "?".into()),
        )
    };

    for chunk in chunk_words(&explanation, 4) {
        raw.push_str(&chunk);
        renderer.push(&chunk);
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    renderer.finish();
    Ok(())
}

fn chunk_words(text: &str, per_chunk: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_inclusive(' ').collect();
    words
        .chunks(per_chunk)
        .map(|c| c.concat())
        .collect()
}
