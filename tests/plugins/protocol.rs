use rev::plugins::protocol::Handshake;
use rev_plugin_sdk::{
    client::{Client, Event},
    identity::*,
    limits::FRAME_BYTES,
    methods::*,
    rpc::*,
    transport::*,
};
use serde_json::json;
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

fn id(value: &str) -> OpaqueId {
    OpaqueId::new(value).unwrap()
}
fn host() -> Handshake {
    Handshake::new(
        id("session-a"),
        Generation::new("7").unwrap(),
        Package {
            id: PackageId::new("field.notes").unwrap(),
            version: Label::new("0.1.0").unwrap(),
        },
    )
    .unwrap()
}

// The actual native offer travels through an OS byte stream in fragments, and
// the public client acknowledges it without importing private editor types.
#[test]
fn pipe_handshake() {
    let mut host = host();
    let (mut native_pipe, mut package_pipe) = UnixStream::pair().unwrap();
    for pipe in [&native_pipe, &package_pipe] {
        pipe.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        pipe.set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
    }
    let bytes = encode(&Message::Request(host.request().clone()), FRAME_BYTES).unwrap();
    native_pipe.write_all(&bytes).unwrap();
    let mut client = Client::new(host.offer().package.clone(), List::default()).unwrap();
    let mut remaining = bytes.len();
    let mut initialized = false;
    while remaining > 0 {
        let mut chunk = [0; 3];
        let count = package_pipe.read(&mut chunk[..remaining.min(3)]).unwrap();
        assert!(count > 0);
        remaining -= count;
        let (used, event) = client.receive(&chunk[..count]).unwrap();
        assert_eq!(used, count);
        initialized |= matches!(event, Some(Event::Initialized(_)));
    }
    assert!(initialized);
    assert!(host.offer().capabilities.is_empty());
    let output = client.output().to_vec();
    package_pipe.write_all(&output).unwrap();
    client.consume_output(output.len()).unwrap();
    let mut response = vec![0; output.len()];
    native_pipe.read_exact(&mut response).unwrap();
    let response = Decoder::new(FRAME_BYTES)
        .unwrap()
        .receive(&response)
        .unwrap()
        .1
        .unwrap();
    host.accept(&response).unwrap();
    assert!(host.accepted());
    assert!(host.accept(&response).is_err());
}

// Identity, API, units, finite profile and generation are agreement conditions;
// a failure must leave executable admission closed for every schema method.
#[test]
fn handshake_rejections() {
    let baseline = host();
    let ack = baseline
        .offer()
        .acknowledge(&baseline.offer().package, List::default())
        .unwrap();
    for (name, field, value) in [
        ("api", "api", json!("rev.plugins/0.2")),
        ("session", "session", json!("session-b")),
        ("generation", "generation", json!("8")),
        (
            "identity",
            "package",
            json!({"id":"other.notes","version":"0.1.0"}),
        ),
        ("units", "units", json!("utf16")),
        ("capabilities", "capabilities", json!(["buffers"])),
        (
            "undeclared handler",
            "handlers",
            json!([{"command":"field.notes.collect","handler":"h"}]),
        ),
    ] {
        let mut host = host();
        let mut result = serde_json::to_value(&ack).unwrap();
        result[field] = value;
        let response = Message::Success(Success {
            jsonrpc: Version::V2,
            id: host.request().id.clone(),
            result,
        });
        assert!(host.accept(&response).is_err(), "{name}");
        assert!(!host.accepted(), "{name}");
    }
    let mut host = host();
    let wrong_id = Message::Success(Success {
        jsonrpc: Version::V2,
        id: id("old-init"),
        result: serde_json::to_value(ack).unwrap(),
    });
    assert!(host.accept(&wrong_id).is_err());
    for (name, _, _) in METHODS {
        let request = Request {
            jsonrpc: Version::V2,
            id: id("request"),
            method: (*name).into(),
            params: json!({}),
        };
        assert_eq!(
            host.admit(&request).unwrap_err().kind(),
            Some(ErrorKind::UnsupportedInvocation)
        );
    }
}

// Schema publication cannot accidentally advertise executable editor methods.
// Native errors retain request context and an unsupported-method JSON-RPC code.
#[test]
fn unimplemented_methods() {
    let mut host = host();
    let ack = host
        .offer()
        .acknowledge(&host.offer().package, List::default())
        .unwrap();
    host.accept(&Message::Success(Success {
        jsonrpc: Version::V2,
        id: host.request().id.clone(),
        result: serde_json::to_value(ack).unwrap(),
    }))
    .unwrap();
    for (name, _, _) in METHODS {
        let request = Request {
            jsonrpc: Version::V2,
            id: id("request"),
            method: (*name).into(),
            params: json!({}),
        };
        let error = host.admit(&request).unwrap_err();
        assert_eq!(error.code, -32601, "{name}");
        assert_eq!(error.data.unwrap().request, Some(request.id));
    }
}
