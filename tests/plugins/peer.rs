//! Executable Rust peer, compiled by the lifecycle integration fixture against
//! the same public SDK artifacts. It has no private editor dependency.
use rev_plugin_sdk::{
    client::{Client, Event},
    identity::*,
    methods::*,
    rpc::*,
    transport::{Decoder, encode},
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

fn record(directory: &std::path::Path, name: &str, text: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(name))
        .unwrap();
    writeln!(file, "{text}").unwrap();
}
fn record_message(directory: &std::path::Path, name: &str, message: &Message) {
    let bytes = encode(message, 4 * 1024 * 1024).unwrap();
    record(
        directory,
        name,
        std::str::from_utf8(&bytes).unwrap().trim_end(),
    );
}
// Scripted raw requests retain public wire framing while allowing invalid and
// cross-session inputs which the typed SDK intentionally refuses to construct.
fn buffer_script(directory: &std::path::Path, input: &mut impl Read, output: &mut impl Write) {
    let mut decoder = Decoder::new(4 * 1024 * 1024).unwrap();
    let mut bytes = [0; 8192];
    let mut nested = std::collections::BTreeMap::new();
    for sequence in 0..1000 {
        let path = directory.join(format!("request-{sequence}"));
        let deadline = Instant::now() + Duration::from_secs(20);
        while !path.exists() {
            if Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let request = fs::read(path).unwrap();
        output.write_all(&request).unwrap();
        output.flush().unwrap();
        let mut complete = false;
        while !complete {
            let count = input.read(&mut bytes).unwrap();
            if count == 0 {
                return;
            }
            let mut offset = 0;
            while offset < count {
                let (used, message) = decoder.receive(&bytes[offset..count]).unwrap();
                offset += used;
                let Some(message) = message else {
                    continue;
                };
                match &message {
                    Message::Failure(failure)
                        if failure
                            .id
                            .as_ref()
                            .is_some_and(|id| nested.contains_key(id)) =>
                    {
                        let parent = nested.remove(failure.id.as_ref().unwrap()).unwrap();
                        let reply = Message::Failure(Failure {
                            jsonrpc: Version::V2,
                            id: Some(parent),
                            error: failure.error.clone(),
                        });
                        output
                            .write_all(&encode(&reply, 4 * 1024 * 1024).unwrap())
                            .unwrap();
                        output.flush().unwrap();
                    }
                    Message::Success(_) | Message::Failure(_) => {
                        let data = encode(&message, 4 * 1024 * 1024).unwrap();
                        let temp = directory.join("response-tmp");
                        fs::write(&temp, data).unwrap();
                        fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
                        complete = true;
                    }
                    Message::Request(request) => {
                        record_message(directory, "calls", &message);
                        if directory.join("recursive").exists() {
                            let id = OpaqueId::new(format!("nested-{}", nested.len())).unwrap();
                            writeln!(output, r#"{{"jsonrpc":"2.0","id":"{}","method":"commands.invoke","params":{{"command":{},"arguments":{{}},"context":{}}}}}"#, id, request.params["command"], request.params["context"]).unwrap();
                            nested.insert(id, request.id.clone());
                            output.flush().unwrap();
                        } else if directory.join("reply-calls").exists() {
                            writeln!(
                                output,
                                r#"{{"jsonrpc":"2.0","id":"{}","result":{{"value":{}}}}}"#,
                                request.id, request.params
                            )
                            .unwrap();
                            output.flush().unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
// Dedicated concurrent script driver: the child keeps reading notifications
// while a finite fixture sender submits requests over its real standard pipe.
fn event_script(directory: &std::path::Path, package: &Package, input: &mut impl Read) {
    use serde_json::{Value, json};
    fn send(value: Value) {
        let mut output = std::io::stdout().lock();
        writeln!(output, "{value}").unwrap();
        output.flush().unwrap();
    }
    let requests = directory.to_owned();
    std::thread::spawn(move || {
        for sequence in 0..2000 {
            let path = requests.join(format!("request-{sequence}"));
            let deadline = Instant::now() + Duration::from_secs(30);
            while !path.exists() {
                if Instant::now() >= deadline {
                    return;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            send(serde_json::from_slice(&fs::read(path).unwrap()).unwrap());
        }
    });
    let mut decoder = Decoder::new(4 * 1024 * 1024).unwrap();
    let mut bytes = [0; 8192];
    let mut received = 0;
    loop {
        while directory.join("pause").exists() {
            if !directory.join("paused").exists() {
                fs::write(directory.join("paused"), "paused").unwrap();
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let count = input.read(&mut bytes).unwrap();
        if count == 0 {
            return;
        }
        let mut offset = 0;
        while offset < count {
            let (used, message) = decoder.receive(&bytes[offset..count]).unwrap();
            offset += used;
            let Some(message) = message else {
                continue;
            };
            match &message {
                Message::Notification(notification) if notification.method == BUFFER_EVENT => {
                    let event: BufferEvent =
                        serde_json::from_value(notification.params.clone()).unwrap();
                    record_message(directory, "events", &message);
                    received += 1;
                    if directory.join("nested-react").exists() && event.kind == EventKind::Changed {
                        send(
                            json!({"jsonrpc":"2.0","id":format!("nested-event-{received}"),"method":"commands.invoke","params":{
                            "command":"other.notes.collect","arguments":{},"context":{
                                "invocation":event.invocation(),"generation":event.generation,"stamp":event.stamp,
                                "pane":null,"point":null,"selection":null,"prefix":null,"origin":event.origin}}}),
                        );
                    }
                    if directory.join("react").exists()
                        && event.kind == EventKind::Changed
                        && event.origin.package.as_ref() != Some(&package.id)
                        && (event.origin.package.is_some() || package.id.as_str() == "field.notes")
                    {
                        let mut origin = event.origin.clone();
                        origin.package = Some(package.id.clone());
                        send(
                            json!({"jsonrpc":"2.0", "id":format!("reaction-{received}"), "method":"edits.apply", "params":{
                            "operation":format!("reaction-{received}"), "origin":origin,
                            "targets":[{"stamp":event.stamp,"replacements":[{"range":{"start":0,"end":0},"text":"!"}]}]}}),
                        );
                    }
                }
                Message::Success(success) => {
                    let text = success.id.as_str();
                    if let Some(sequence) = text.strip_prefix("buffer-request-") {
                        let data = encode(&message, 4 * 1024 * 1024).unwrap();
                        let temp = directory.join("response-tmp");
                        fs::write(&temp, data).unwrap();
                        fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
                    } else {
                        record_message(directory, "reactions", &message);
                    }
                }
                Message::Failure(failure) => {
                    let id = failure.id.as_ref().unwrap().as_str();
                    if let Some(sequence) = id.strip_prefix("buffer-request-") {
                        let data = encode(&message, 4 * 1024 * 1024).unwrap();
                        let temp = directory.join("response-tmp");
                        fs::write(&temp, data).unwrap();
                        fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
                    } else {
                        record_message(directory, "reactions", &message);
                    }
                }
                Message::Request(request) => {
                    record_message(directory, "calls", &message);
                    if directory.join("hold-command").exists() {
                        let directory = directory.to_owned();
                        let request = request.clone();
                        let owner = package.id.clone();
                        std::thread::spawn(move || {
                            let deadline = Instant::now() + Duration::from_secs(20);
                            while !directory.join("release-command").exists() {
                                if Instant::now() >= deadline {
                                    return;
                                }
                                std::thread::sleep(Duration::from_millis(1));
                            }
                            let params: CommandInvokeParams =
                                serde_json::from_value(request.params).unwrap();
                            let mut origin = params.context.origin;
                            origin.package = Some(owner);
                            send(
                                json!({"jsonrpc":"2.0","id":"late-edit","method":"edits.apply","params":{
                                "operation":"late-edit","origin":origin,"targets":[{"stamp":params.context.stamp,"replacements":[{"range":{"start":0,"end":0},"text":"late"}]}]}}),
                            );
                            send(json!({"jsonrpc":"2.0","id":request.id,"result":{"value":null}}));
                        });
                        continue;
                    }
                    if directory.join("command-edit").exists() {
                        let params: CommandInvokeParams =
                            serde_json::from_value(request.params.clone()).unwrap();
                        let mut origin = params.context.origin;
                        origin.package = Some(package.id.clone());
                        send(
                            json!({"jsonrpc":"2.0","id":format!("command-edit-{}",request.id),"method":"edits.apply","params":{
                            "operation":params.context.invocation,"origin":origin,"targets":[{"stamp":params.context.stamp,"replacements":[{"range":{"start":0,"end":0},"text":"!"}]}]}}),
                        );
                    }
                    send(json!({"jsonrpc":"2.0","id":request.id,"result":{"value":null}}));
                }
                _ => {}
            }
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let directory = PathBuf::from(&args[1]);
    let package = Package {
        id: PackageId::new(&args[2]).unwrap(),
        version: Label::new("0.1.0").unwrap(),
    };
    let mode = fs::read_to_string(directory.join("mode")).unwrap_or_else(|_| "echo".into());
    if mode == "descendant-child" {
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    record(&directory, "starts", &std::process::id().to_string());
    if mode == "crash" {
        std::process::exit(17);
    }
    if mode == "noise" {
        println!("this is not JSON-RPC");
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    if mode == "truncated" {
        print!("{{");
        std::io::stdout().flush().unwrap();
        return;
    }
    if mode == "oversize" {
        std::io::stdout()
            .write_all(&vec![b'x'; 4 * 1024 * 1024 + 1])
            .unwrap();
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    if mode == "descendant" {
        let child = std::process::Command::new("/bin/sh")
            .args(["-c", "trap '' TERM; echo $$ > descendant; exec sleep 60"])
            .current_dir(&directory)
            .spawn()
            .unwrap();
        record(&directory, "owned-child", &child.id().to_string());
    }
    if mode == "stderr" {
        let mut bytes = vec![b'd'; 256 * 1024];
        bytes.extend_from_slice(b"\x1b[31mEND\n");
        std::io::stderr().write_all(&bytes).unwrap();
    }
    let handler = Handler {
        command: package.id.command(&CommandName::new("collect").unwrap()),
        handler: OpaqueId::new("collect-handler").unwrap(),
    };
    let mut client = Client::new(package.clone(), List::new(vec![handler]).unwrap()).unwrap();
    let start = Instant::now();
    let mut held = Vec::new();
    let mut automatic = std::collections::BTreeMap::new();
    let mut input = std::io::stdin();
    let mut output = std::io::stdout();
    let mut buffer = [0; 8192];
    let mut decoder = Decoder::new(4 * 1024 * 1024).unwrap();
    loop {
        let count = match input.read(&mut buffer) {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        let mut offset = 0;
        while offset < count {
            if matches!(
                mode.as_str(),
                "foreign-api" | "foreign-generation" | "undeclared"
            ) {
                let (used, message) = decoder.receive(&buffer[offset..count]).unwrap();
                offset += used;
                let Some(Message::Request(request)) = message else {
                    continue;
                };
                let mut result = request.params;
                match mode.as_str() {
                    "foreign-api" => result["api"] = "rev.plugins/0.2".into(),
                    "foreign-generation" => result["generation"] = "99999".into(),
                    _ => {}
                }
                let handlers = if mode == "undeclared" {
                    format!(
                        r#"[{{"command":"{}.ghost","handler":"ghost"}}]"#,
                        package.id
                    )
                } else {
                    "[]".into()
                };
                let body = result.to_string();
                let body = body.strip_suffix('}').unwrap();
                writeln!(
                    output,
                    r#"{{"jsonrpc":"2.0","id":"{}","result":{},"handlers":{}}}}}"#,
                    request.id, body, handlers
                )
                .unwrap();
                output.flush().unwrap();
                std::thread::sleep(Duration::from_secs(60));
                return;
            }
            let (observed, message) = decoder.receive(&buffer[offset..count]).unwrap();
            let (used, event) = client.receive(&buffer[offset..count]).unwrap();
            assert_eq!(observed, used);
            offset += used;
            let initialized = matches!(&event, Some(Event::Initialized(_)));
            match event {
                Some(Event::Initialized(offer)) => {
                    let Some(Message::Request(request)) = message else {
                        panic!("initialization request");
                    };
                    record(&directory, "offers", &request.params.to_string());
                    assert_eq!(offer.package, package);
                    if mode == "slow" || mode == "timeout" {
                        let deadline = Instant::now() + Duration::from_secs(10);
                        while !directory.join("release").exists() && Instant::now() < deadline {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                    if mode == "rpc" {
                        client
                            .request::<CommandsUnregister>(
                                CommandsUnregisterParams {
                                    registrations: List::new(vec![
                                        OpaqueId::new("unused-registration").unwrap(),
                                    ])
                                    .unwrap(),
                                },
                                start.elapsed(),
                            )
                            .unwrap();
                    }
                }
                Some(Event::Request(callback)) => {
                    record_message(
                        &directory,
                        "calls",
                        &Message::Request(callback.request().clone()),
                    );
                    if mode == "slow-call" {
                        let deadline = Instant::now() + Duration::from_secs(10);
                        while !directory.join("complete").exists() && Instant::now() < deadline {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                    if mode == "recording"
                        && (directory.join("edit").exists() || directory.join("nested").exists())
                    {
                        let params: CommandInvokeParams =
                            serde_json::from_value(callback.request().params.clone()).unwrap();
                        let request = if directory.join("nested").exists() {
                            client
                                .request::<CommandsInvoke>(
                                    CommandsInvokeParams {
                                        command: CommandTarget::Plugin(
                                            fs::read_to_string(directory.join("nested"))
                                                .ok()
                                                .filter(|id| !id.trim().is_empty())
                                                .map_or(params.command, |id| {
                                                    CommandId::new(id.trim()).unwrap()
                                                }),
                                        ),
                                        arguments: params.arguments,
                                        context: params.context,
                                    },
                                    start.elapsed(),
                                )
                                .unwrap()
                        } else {
                            let mut origin = params.context.origin;
                            origin.package = Some(package.id.clone());
                            client
                                .request::<EditsApply>(
                                    EditsApplyParams {
                                        operation: params.context.invocation.clone(),
                                        origin,
                                        targets: List::new(vec![BufferEdits {
                                            stamp: params.context.stamp.unwrap(),
                                            replacements: List::new(vec![Replacement {
                                                range: ScalarRange::new(0, 0).unwrap(),
                                                text: SourceText::new("λ").unwrap(),
                                            }])
                                            .unwrap(),
                                        }])
                                        .unwrap(),
                                    },
                                    start.elapsed(),
                                )
                                .unwrap()
                        };
                        automatic.insert(request, callback);
                    } else if mode == "hold" {
                        held.push(callback);
                    } else {
                        client
                            .respond::<CommandInvoke>(
                                &callback,
                                Ok(CommandsInvokeResult {
                                    value: Default::default(),
                                }),
                            )
                            .unwrap();
                    }
                }
                Some(Event::Notification(notification))
                    if notification.method == CommandCancel::NAME =>
                {
                    record_message(&directory, "cancels", &Message::Notification(notification));
                    for callback in held.drain(..) {
                        assert!(
                            client
                                .respond::<CommandInvoke>(
                                    &callback,
                                    Ok(CommandsInvokeResult {
                                        value: Default::default()
                                    })
                                )
                                .is_err()
                        );
                        record(&directory, "retired", "late callback rejected");
                    }
                }
                Some(Event::Response { id, result, .. }) if automatic.contains_key(&id) => {
                    let callback = automatic.remove(&id).unwrap();
                    client
                        .respond::<CommandInvoke>(
                            &callback,
                            result.map(|_| CommandsInvokeResult {
                                value: Default::default(),
                            }),
                        )
                        .unwrap();
                }
                Some(Event::Response {
                    result: Err(error), ..
                }) => {
                    record_message(
                        &directory,
                        "errors",
                        &Message::Failure(Failure {
                            jsonrpc: Version::V2,
                            id: error.data.as_ref().and_then(|d| d.request.clone()),
                            error,
                        }),
                    );
                }
                _ => {}
            }
            while !client.output().is_empty() {
                let count = output.write(client.output()).unwrap();
                client.consume_output(count).unwrap();
            }
            output.flush().unwrap();
            if mode == "events" && initialized {
                event_script(&directory, &package, &mut input);
                return;
            }
            if mode == "buffers" && initialized {
                buffer_script(&directory, &mut input, &mut output);
                return;
            }
        }
    }
}
