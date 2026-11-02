use std::io::{self, BufRead, Read, Write};

use serde::Deserialize;
use serde_json::Value;

pub(super) const HEADER_BYTES: usize = 16 * 1024;

/// The supported JSON-RPC envelope; feature payloads stay with their owner.
#[derive(Debug, Deserialize)]
pub(super) struct Message {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: Option<String>,
    #[serde(default)]
    pub params: Value,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

pub(super) fn read(input: &mut impl BufRead, limit: usize) -> Result<(Message, usize), String> {
    let length = header(input, limit)?;
    let mut body = vec![0; length];
    input
        .read_exact(&mut body)
        .map_err(|e| format!("LSP body: {e}"))?;
    let message: Message = serde_json::from_slice(&body).map_err(|e| format!("LSP JSON: {e}"))?;
    if message.jsonrpc != "2.0" {
        return Err("LSP requires JSON-RPC 2.0".into());
    }
    Ok((message, length))
}

pub(super) fn header(input: &mut impl BufRead, limit: usize) -> Result<usize, String> {
    let mut bytes = Vec::new();
    let mut length = None;
    loop {
        let count = input
            .take((HEADER_BYTES - bytes.len() + 1) as u64)
            .read_until(b'\n', &mut bytes)
            .map_err(|e| format!("LSP header: {e}"))?;
        if count == 0 {
            return Err("LSP stdout closed".into());
        }
        if bytes.len() > HEADER_BYTES {
            return Err("LSP header exceeds 16 KiB".into());
        }
        let line = &bytes[bytes.len() - count..];
        if line == b"\r\n" {
            break;
        }
        parse_header(line, &mut length)?;
    }
    let length = length.ok_or("LSP header lacks Content-Length")?;
    if length == 0 || length > limit {
        return Err("LSP message exceeds admitted byte limit".into());
    }
    Ok(length)
}

fn parse_header(line: &[u8], length: &mut Option<usize>) -> Result<(), String> {
    let line = std::str::from_utf8(line).map_err(|_| "LSP header is not ASCII")?;
    if !line.is_ascii() || !line.ends_with("\r\n") {
        return Err("Malformed LSP header".into());
    }
    let (name, value) = line
        .trim_end()
        .split_once(':')
        .ok_or("Malformed LSP header")?;
    if name.eq_ignore_ascii_case("Content-Length") {
        if length.is_some() {
            return Err("Duplicate LSP Content-Length".into());
        }
        let value = value.trim();
        if !value.bytes().all(|v| v.is_ascii_digit()) {
            return Err("Invalid LSP Content-Length".into());
        }
        *length = Some(value.parse().map_err(|_| "Invalid LSP Content-Length")?);
    }
    Ok(())
}

pub(super) fn encode(value: &Value, limit: usize) -> Result<Vec<u8>, String> {
    let body = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if body.len() > limit {
        return Err("LSP outgoing message exceeds byte limit".into());
    }
    let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend(body);
    Ok(bytes)
}

pub(super) fn write(output: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    output.write_all(bytes)?;
    output.flush()
}
