//! Executable Rust peer, compiled by the lifecycle integration fixture against
//! the same public SDK artifacts. It has no private editor dependency.
use rev_plugin_sdk::{
    client::{Client, Event},
    identity::*,
    methods::*,
    rpc::*,
    transport::{Decoder, encode},
};
use serde_json::{Value, json};
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
    let mut script = BufferScript {
        directory,
        decoder: Decoder::new(4 * 1024 * 1024).unwrap(),
        nested: std::collections::BTreeMap::new(),
    };
    let mut bytes = [0; 8192];
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
        if !script.receive(sequence, input, output, &mut bytes) {
            return;
        }
    }
}
struct BufferScript<'a> {
    directory: &'a std::path::Path,
    decoder: Decoder,
    nested: std::collections::BTreeMap<OpaqueId, OpaqueId>,
}
impl BufferScript<'_> {
    fn receive(
        &mut self,
        sequence: usize,
        input: &mut impl Read,
        output: &mut impl Write,
        bytes: &mut [u8],
    ) -> bool {
        let mut complete = false;
        while !complete {
            let count = input.read(bytes).unwrap();
            if count == 0 {
                return false;
            }
            let mut offset = 0;
            while offset < count {
                let (used, message) = self.decoder.receive(&bytes[offset..count]).unwrap();
                offset += used;
                let Some(message) = message else {
                    continue;
                };
                complete |= self.message(sequence, &message, output);
            }
        }
        true
    }
    fn message(&mut self, sequence: usize, message: &Message, output: &mut impl Write) -> bool {
        let directory = self.directory;
        let nested = &mut self.nested;
        match message {
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
                false
            }
            Message::Success(_) | Message::Failure(_) => {
                let data = encode(message, 4 * 1024 * 1024).unwrap();
                let temp = directory.join("response-tmp");
                fs::write(&temp, data).unwrap();
                fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
                true
            }
            Message::Request(request) => {
                self.call(request, message, output);
                false
            }
            _ => false,
        }
    }
    fn call(&mut self, request: &Request, message: &Message, output: &mut impl Write) {
        let directory = self.directory;
        let nested = &mut self.nested;
        record_message(directory, "calls", message);
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
}
// Dedicated concurrent script driver: the child keeps reading notifications
// while a finite fixture sender submits requests over its real standard pipe.
fn event_script(directory: &std::path::Path, package: &Package, input: &mut impl Read) {
    let requests = directory.to_owned();
    std::thread::spawn(move || send_requests(&requests));
    let mut decoder = Decoder::new(4 * 1024 * 1024).unwrap();
    let mut bytes = [0; 8192];
    let mut script = Events {
        directory,
        package,
        received: 0,
        journey: None,
    };
    loop {
        pause_events(directory);
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
            script.message(&message);
        }
    }
}
fn send(value: Value) {
    let mut output = std::io::stdout().lock();
    writeln!(output, "{value}").unwrap();
    output.flush().unwrap();
}
fn send_requests(directory: &std::path::Path) {
    for sequence in 0..2000 {
        let path = directory.join(format!("request-{sequence}"));
        let deadline = Instant::now() + Duration::from_secs(30);
        while !path.exists() {
            if Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        send(serde_json::from_slice(&fs::read(path).unwrap()).unwrap());
    }
}
fn pause_events(directory: &std::path::Path) {
    while directory.join("pause").exists() {
        if !directory.join("paused").exists() {
            fs::write(directory.join("paused"), "paused").unwrap();
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
struct Events<'a> {
    directory: &'a std::path::Path,
    package: &'a Package,
    received: i32,
    journey: Option<(OpaqueId, Value)>,
}
impl Events<'_> {
    fn message(&mut self, message: &Message) {
        match message {
            Message::Notification(notification) if notification.method == BUFFER_EVENT => {
                self.notification(notification, message)
            }
            Message::Success(success) if success.id.as_str().starts_with("journey-") => {
                self.journey(success)
            }
            Message::Success(success) => self.success(success, message),
            Message::Failure(failure) => self.failure(failure, message),
            Message::Request(request) => self.call(request, message),
            _ => {}
        }
    }
    fn notification(&mut self, notification: &Notification, message: &Message) {
        let directory = self.directory;
        let package = self.package;
        let event: BufferEvent = serde_json::from_value(notification.params.clone()).unwrap();
        record_message(directory, "events", message);
        self.received += 1;
        let received = self.received;
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
    fn journey(&mut self, success: &Success) {
        let directory = self.directory;
        let journey = &mut self.journey;
        match success.id.as_str() {
            "journey-create" => {
                let buffer = success.result["stamp"]["buffer"].clone();
                journey.as_mut().unwrap().1 = buffer.clone();
                let context = if fs::read_to_string(directory.join("profile")).unwrap() == "modal" {
                    "vim_normal"
                } else {
                    "editor"
                };
                send(
                    json!({"jsonrpc":"2.0","id":"journey-bind","method":"keymaps.bind","params":{"command":"field.notes.collect","keys":"F8","context":context,"scope":{"kind":"buffer","buffer":buffer}}}),
                );
            }
            "journey-bind" => {
                assert_eq!(success.result["status"], "effective");
                send(
                    json!({"jsonrpc":"2.0","id":"journey-show","method":"buffers.show","params":{"buffer":journey.as_ref().unwrap().1,"pane":null,"placement":"current"}}),
                );
            }
            "journey-show" => {
                let (callback, _) = journey.take().unwrap();
                send(json!({"jsonrpc":"2.0","id":callback,"result":{"value":null}}));
                fs::write(directory.join("generated"), "ready").unwrap();
            }
            _ => unreachable!(),
        }
    }
    fn success(&self, success: &Success, message: &Message) {
        let directory = self.directory;
        let text = success.id.as_str();
        if let Some(sequence) = text.strip_prefix("buffer-request-") {
            let data = encode(message, 4 * 1024 * 1024).unwrap();
            let temp = directory.join("response-tmp");
            fs::write(&temp, data).unwrap();
            fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
        } else {
            record_message(directory, "reactions", message);
        }
    }
    fn failure(&self, failure: &Failure, message: &Message) {
        let directory = self.directory;
        let id = failure.id.as_ref().unwrap().as_str();
        if let Some(sequence) = id.strip_prefix("buffer-request-") {
            let data = encode(message, 4 * 1024 * 1024).unwrap();
            let temp = directory.join("response-tmp");
            fs::write(&temp, data).unwrap();
            fs::rename(temp, directory.join(format!("response-{sequence}"))).unwrap();
        } else {
            record_message(directory, "reactions", message);
        }
    }
    fn call(&mut self, request: &Request, message: &Message) {
        let directory = self.directory;
        let package = self.package;
        record_message(directory, "calls", message);
        if let Ok(action) = fs::read_to_string(directory.join("action")) {
            if self.action(request, &action) {
                return;
            }
        }
        if directory.join("hold-command").exists() {
            hold_command(directory, package, request);
            return;
        }
        if directory.join("command-edit").exists() {
            command_edit(package, request);
        }
        send(json!({"jsonrpc":"2.0","id":request.id,"result":{"value":null}}));
    }
    fn action(&mut self, request: &Request, action: &str) -> bool {
        let journey = &mut self.journey;
        match action {
            "crash" => std::process::exit(17),
            "hold" => true,
            "result" => {
                *journey = Some((request.id.clone(), Value::Null));
                send(
                    json!({"jsonrpc":"2.0","id":"journey-create","method":"buffers.create","params":{"name":"Plugin result","text":"result text\n","read_only":false}}),
                );
                true
            }
            _ => panic!("unknown fixture action"),
        }
    }
}
fn hold_command(directory: &std::path::Path, package: &Package, request: &Request) {
    let directory = directory.to_owned();
    let request = request.clone();
    let owner = package.id.clone();
    std::thread::spawn(move || late_command(&directory, request, owner));
}
fn late_command(directory: &std::path::Path, request: Request, owner: PackageId) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !directory.join("release-command").exists() {
        if Instant::now() >= deadline {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let params: CommandInvokeParams = serde_json::from_value(request.params).unwrap();
    let mut origin = params.context.origin;
    origin.package = Some(owner);
    send(
        json!({"jsonrpc":"2.0","id":"late-edit","method":"edits.apply","params":{
        "operation":"late-edit","origin":origin,"targets":[{"stamp":params.context.stamp,"replacements":[{"range":{"start":0,"end":0},"text":"late"}]}]}}),
    );
    send(json!({"jsonrpc":"2.0","id":request.id,"result":{"value":null}}));
}
fn command_edit(package: &Package, request: &Request) {
    let params: CommandInvokeParams = serde_json::from_value(request.params.clone()).unwrap();
    let mut origin = params.context.origin;
    origin.package = Some(package.id.clone());
    send(
        json!({"jsonrpc":"2.0","id":format!("command-edit-{}",request.id),"method":"edits.apply","params":{
        "operation":params.context.invocation,"origin":origin,"targets":[{"stamp":params.context.stamp,"replacements":[{"range":{"start":0,"end":0},"text":"!"}]}]}}),
    );
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let directory = PathBuf::from(&args[1]);
    let package = Package {
        id: PackageId::new(&args[2]).unwrap(),
        version: Label::new("0.1.0").unwrap(),
    };
    let mode = fs::read_to_string(directory.join("mode")).unwrap_or_else(|_| "echo".into());
    if before_start(&directory, &mode) {
        return;
    }
    record_process(&directory);
    if prepare_peer(&directory, &mode) {
        return;
    }
    let handler = Handler {
        command: package.id.command(&CommandName::new("collect").unwrap()),
        handler: OpaqueId::new("collect-handler").unwrap(),
    };
    let client = Client::new(package.clone(), List::new(vec![handler]).unwrap()).unwrap();
    let mut peer = Peer {
        directory: &directory,
        package: &package,
        mode: &mode,
        client,
        start: Instant::now(),
        held: Vec::new(),
        automatic: std::collections::BTreeMap::new(),
    };
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
            let Some(used) = peer.frame(
                &mut decoder,
                &buffer[offset..count],
                &mut input,
                &mut output,
            ) else {
                return;
            };
            offset += used;
        }
    }
}
fn before_start(directory: &std::path::Path, mode: &str) -> bool {
    if mode == "state-lock" {
        let path = fs::read_to_string(directory.join("lock-path")).unwrap();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        file.try_lock().unwrap();
        fs::write(directory.join("locked"), "locked").unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        while !directory.join("release").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        return true;
    }
    if mode == "descendant-child" {
        std::thread::sleep(Duration::from_secs(60));
        return true;
    }
    false
}
fn record_process(directory: &std::path::Path) {
    record(directory, "starts", &std::process::id().to_string());
    // Inspect after closing the enumeration descriptor. Only the three wire
    // streams may survive exec; host eventfds and unrelated files stay CLOEXEC.
    let entries: Vec<_> = fs::read_dir("/proc/self/fd")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    let mut inherited: Vec<u32> = entries
        .into_iter()
        .filter(|p| fs::read_link(p).is_ok())
        .map(|p| p.file_name().unwrap().to_str().unwrap().parse().unwrap())
        .collect();
    inherited.sort_unstable();
    fs::write(
        directory.join("inherited-fds"),
        serde_json::to_vec(&inherited).unwrap(),
    )
    .unwrap();
}
fn prepare_peer(directory: &std::path::Path, mode: &str) -> bool {
    if mode == "crash" {
        std::process::exit(17);
    }
    if mode == "noise" {
        println!("this is not JSON-RPC");
        std::thread::sleep(Duration::from_secs(60));
        return true;
    }
    if mode == "truncated" {
        print!("{{");
        std::io::stdout().flush().unwrap();
        return true;
    }
    if mode == "oversize" {
        std::io::stdout()
            .write_all(&vec![b'x'; 4 * 1024 * 1024 + 1])
            .unwrap();
        std::thread::sleep(Duration::from_secs(60));
        return true;
    }
    if mode == "descendant" {
        let child = std::process::Command::new("/bin/sh")
            .args(["-c", "trap '' TERM; echo $$ > descendant; exec sleep 60"])
            .current_dir(directory)
            .spawn()
            .unwrap();
        record(directory, "owned-child", &child.id().to_string());
    }
    if mode == "stderr" {
        let mut bytes = vec![b'd'; 256 * 1024];
        bytes.extend_from_slice(b"\x1b[31mEND\n");
        std::io::stderr().write_all(&bytes).unwrap();
    }
    false
}
struct Peer<'a> {
    directory: &'a std::path::Path,
    package: &'a Package,
    mode: &'a str,
    client: Client,
    start: Instant,
    held: Vec<rev_plugin_sdk::client::Callback>,
    automatic: std::collections::BTreeMap<OpaqueId, rev_plugin_sdk::client::Callback>,
}
impl Peer<'_> {
    fn frame(
        &mut self,
        decoder: &mut Decoder,
        bytes: &[u8],
        input: &mut impl Read,
        output: &mut impl Write,
    ) -> Option<usize> {
        if matches!(
            self.mode,
            "foreign-api" | "foreign-generation" | "undeclared"
        ) {
            let (used, message) = decoder.receive(bytes).unwrap();
            let Some(Message::Request(request)) = message else {
                return Some(used);
            };
            self.reject_handshake(request, output);
            return None;
        }
        let (observed, message) = decoder.receive(bytes).unwrap();
        let (used, event) = self.client.receive(bytes).unwrap();
        assert_eq!(observed, used);
        let initialized = matches!(&event, Some(Event::Initialized(_)));
        self.event(event, message.as_ref());
        self.flush(output);
        if initialized && self.after_init(input, output) {
            return None;
        }
        Some(used)
    }
    fn reject_handshake(&self, request: Request, output: &mut impl Write) {
        let mode = self.mode;
        let package = self.package;
        let mut result = request.params;
        match mode {
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
    }
    fn flush(&mut self, output: &mut impl Write) {
        let client = &mut self.client;
        while !client.output().is_empty() {
            let count = output.write(client.output()).unwrap();
            client.consume_output(count).unwrap();
        }
        output.flush().unwrap();
    }
    fn event(&mut self, event: Option<Event>, message: Option<&Message>) {
        match event {
            Some(Event::Initialized(offer)) => self.initialized(&offer, message),
            Some(Event::Request(callback)) => self.call(callback),
            Some(Event::Notification(notification))
                if notification.method == CommandCancel::NAME =>
            {
                self.retire(notification)
            }
            Some(Event::Response { id, result, .. }) if self.automatic.contains_key(&id) => {
                self.automatic(&id, result)
            }
            Some(Event::Response {
                result: Err(error), ..
            }) => self.error(error),
            _ => {}
        }
    }
    fn initialized(&mut self, offer: &InitializeParams, message: Option<&Message>) {
        let directory = self.directory;
        let package = self.package;
        let mode = self.mode;
        let client = &mut self.client;
        let start = self.start;
        let Some(Message::Request(request)) = message else {
            panic!("initialization request");
        };
        record(directory, "offers", &request.params.to_string());
        assert_eq!(offer.package, *package);
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
    fn call(&mut self, callback: rev_plugin_sdk::client::Callback) {
        let directory = self.directory;
        let mode = self.mode;
        record_message(
            directory,
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
            self.recording(callback);
        } else if mode == "hold" {
            self.held.push(callback);
        } else {
            self.client
                .respond::<CommandInvoke>(
                    &callback,
                    Ok(CommandsInvokeResult {
                        value: Default::default(),
                    }),
                )
                .unwrap();
        }
    }
    fn recording(&mut self, callback: rev_plugin_sdk::client::Callback) {
        let directory = self.directory;
        let package = self.package;
        let client = &mut self.client;
        let start = self.start;
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
                                .map_or(params.command, |id| CommandId::new(id.trim()).unwrap()),
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
        self.automatic.insert(request, callback);
    }
    fn retire(&mut self, notification: Notification) {
        let directory = self.directory;
        let held = &mut self.held;
        let client = &mut self.client;
        record_message(directory, "cancels", &Message::Notification(notification));
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
            record(directory, "retired", "late callback rejected");
        }
    }
    fn automatic(&mut self, id: &OpaqueId, result: Result<Value, RpcError>) {
        let client = &mut self.client;
        let callback = self.automatic.remove(id).unwrap();
        client
            .respond::<CommandInvoke>(
                &callback,
                result.map(|_| CommandsInvokeResult {
                    value: Default::default(),
                }),
            )
            .unwrap();
    }
    fn error(&self, error: RpcError) {
        let directory = self.directory;
        record_message(
            directory,
            "errors",
            &Message::Failure(Failure {
                jsonrpc: Version::V2,
                id: error.data.as_ref().and_then(|d| d.request.clone()),
                error,
            }),
        );
    }
    fn after_init(&mut self, input: &mut impl Read, output: &mut impl Write) -> bool {
        let directory = self.directory;
        let package = self.package;
        let mode = self.mode;
        if mode == "eof-after-init" {
            return true;
        }
        if mode == "stall-input" {
            fs::write(directory.join("stalled"), "ready").unwrap();
            std::thread::sleep(Duration::from_secs(60));
            return true;
        }
        if mode == "flood" {
            fs::write(directory.join("flooding"), "ready").unwrap();
            // Finite real protocol pressure exceeds the 4 MiB host output
            // ceiling while this peer deliberately never drains stdin.
            for i in 0..40_000 {
                if writeln!(output, "{}", serde_json::json!({"jsonrpc":"2.0", "id":format!("flood-{i}"), "method":"buffers.list", "params":{"page":{"cursor":null,"limit":1},"kind":"text"}})).is_err() { return true; }
            }
            let _ = output.flush();
            std::thread::sleep(Duration::from_secs(60));
            return true;
        }
        if matches!(mode, "events" | "journey") {
            event_script(directory, package, input);
            return true;
        }
        if mode == "buffers" {
            buffer_script(directory, input, output);
            return true;
        }
        false
    }
}
