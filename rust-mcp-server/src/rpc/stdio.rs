use serde_json::Value;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use crate::AppState;
use super::{handle_jsonrpc, jsonrpc_error, RpcMode};

pub async fn run_stdio(state: AppState) -> Result<(), String> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        let mut content_length: Option<usize> = None;
        let mut inline_payload: Option<Value> = None;

        loop {
            line.clear();
            let read = reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Failed reading stdio header: {e}"))?;

            if read == 0 {
                return Ok(());
            }

            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }

            let lower = trimmed.to_ascii_lowercase();
            if let Some(value) = lower.strip_prefix("content-length:") {
                let parsed = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("Invalid Content-Length header: {e}"))?;
                content_length = Some(parsed);
                continue;
            }

            // Compatibility fallback for clients that send line-delimited JSON-RPC
            // instead of Content-Length framed stdio messages.
            if trimmed.starts_with('{') {
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(payload) => {
                        inline_payload = Some(payload);
                        break;
                    }
                    Err(e) => {
                        let err = jsonrpc_error(Value::Null, -32700, &format!("Parse error: {e}"));
                        write_stdio_message(&mut stdout, &err).await?;
                        inline_payload = None;
                        break;
                    }
                }
            }
        }

        if let Some(payload) = inline_payload {
            handle_stdio_payload(&mut stdout, &state, payload, false).await?;
            continue;
        }

        let Some(length) = content_length else {
            continue;
        };

        let mut body = vec![0u8; length];
        reader
            .read_exact(&mut body)
            .await
            .map_err(|e| format!("Failed reading stdio body: {e}"))?;

        let payload: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                let err = jsonrpc_error(Value::Null, -32700, &format!("Parse error: {e}"));
                write_stdio_message(&mut stdout, &err).await?;
                continue;
            }
        };

        handle_stdio_payload(&mut stdout, &state, payload, true).await?;
    }
}

async fn handle_stdio_payload(
    stdout: &mut io::Stdout,
    state: &AppState,
    payload: Value,
    use_content_length_framing: bool,
) -> Result<(), String> {
    let response = handle_jsonrpc(&payload, state, RpcMode::StdioMcp).await;
    if payload.get("id").is_some() {
        if use_content_length_framing {
            write_stdio_message(stdout, &response).await?;
        } else {
            write_stdio_jsonline(stdout, &response).await?;
        }
    }
    Ok(())
}

async fn write_stdio_message(stdout: &mut io::Stdout, payload: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(payload).map_err(|e| format!("Failed to encode JSON: {e}"))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());

    stdout
        .write_all(header.as_bytes())
        .await
        .map_err(|e| format!("Failed writing stdio header: {e}"))?;
    stdout
        .write_all(&body)
        .await
        .map_err(|e| format!("Failed writing stdio body: {e}"))?;
    stdout
        .flush()
        .await
        .map_err(|e| format!("Failed flushing stdio output: {e}"))?;
    Ok(())
}

async fn write_stdio_jsonline(stdout: &mut io::Stdout, payload: &Value) -> Result<(), String> {
    let mut body = serde_json::to_vec(payload).map_err(|e| format!("Failed to encode JSON: {e}"))?;
    body.push(b'\n');

    stdout
        .write_all(&body)
        .await
        .map_err(|e| format!("Failed writing stdio JSON line: {e}"))?;
    stdout
        .flush()
        .await
        .map_err(|e| format!("Failed flushing stdio output: {e}"))?;
    Ok(())
}
