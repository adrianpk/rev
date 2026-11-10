use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::config::Server;
use crate::lsp::{frame::Message, protocol, roots};
use crate::org::Cancellation;

use super::helper::{Helper, Streams, checkpoint};
use super::process::Process;

/// Reuse the client's actual capability admission and request responses. The
/// exchange sends no didOpen/didChange and has no reference to user buffers.
pub(super) fn verify(server: &Server, root: &Path, cancel: &Cancellation) -> Result<(), String> {
    let helper = Helper {
        executable: server.command.clone().into(),
        args: server.args.clone(),
        directory: root.to_owned(),
        env: server.env.clone(),
        seconds: 30,
        watch: vec![(root.to_owned(), super::PAYLOAD_BYTES)],
    };
    let (mut streams, [stdin, stdout, stderr]) = Streams::new()?;
    let mut command = helper.command();
    command.stdin(stdin).stdout(stdout).stderr(stderr);
    checkpoint(cancel, Instant::now() + Duration::from_secs(30))?;
    let mut process = Process::start(&mut command)?;
    // Release the parent copies of the child descriptors before polling EOF.
    drop(command);
    let result = exchange(&helper, &mut streams, server, cancel);
    process.finish(result)
}

fn exchange(
    helper: &Helper,
    streams: &mut Streams,
    server: &Server,
    cancel: &Cancellation,
) -> Result<(), String> {
    let root = roots::uri(&helper.directory)?;
    streams.send(&protocol::initialize(server, &root))?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut phase = 0;
    let mut account_at = Instant::now();
    loop {
        checkpoint(cancel, deadline)?;
        streams.step()?;
        helper.monitor(&mut account_at)?;
        phase = drain(streams, server, (&root, phase), cancel)?;
        if completed(streams, phase)? {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn drain(
    streams: &mut Streams,
    server: &Server,
    state: (&str, u8),
    cancel: &Cancellation,
) -> Result<u8, String> {
    let (root, mut phase) = state;
    while let Some(message) = streams.message()? {
        if cancel.is_cancelled() {
            return Err("installation probe cancelled".into());
        }
        phase = response(message, streams, server, (root, phase))?;
    }
    Ok(phase)
}

fn completed(streams: &Streams, phase: u8) -> Result<bool, String> {
    if !streams.closed {
        return Ok(false);
    }
    if phase == 2 && streams.output.is_empty() && streams.flushed() {
        return Ok(true);
    }
    Err("probe closed before initialize/shutdown completed".into())
}

fn response(
    message: Message,
    streams: &mut Streams,
    server: &Server,
    state: (&str, u8),
) -> Result<u8, String> {
    let (root, phase) = state;
    if message.method.is_some() {
        if message.id.is_some() {
            streams.send(&protocol::request(&message, server, root))?;
        }
        return Ok(phase);
    }
    if phase == 0 && message.id == Some(json!(1)) {
        return initialized(message, streams);
    }
    if phase == 1 && message.id == Some(json!(2)) {
        return shutdown(&message, streams);
    }
    Ok(phase)
}

fn initialized(message: Message, streams: &mut Streams) -> Result<u8, String> {
    if message.error.is_some() {
        return Err("probe initialize rejected".into());
    }
    protocol::capabilities(message.result.ok_or("probe initialize lacks result")?)?;
    streams.send(&json!({"jsonrpc":"2.0","method":"initialized","params":{}}))?;
    streams.send(&json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}))?;
    Ok(1)
}

fn shutdown(message: &Message, streams: &mut Streams) -> Result<u8, String> {
    if message.error.is_some() {
        return Err("probe shutdown rejected".into());
    }
    streams.send(&json!({"jsonrpc":"2.0","method":"exit"}))?;
    Ok(2)
}
