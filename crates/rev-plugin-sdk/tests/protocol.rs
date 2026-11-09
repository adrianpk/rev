#![forbid(unsafe_code)]

#[path = "protocol/client.rs"]
mod client;
#[path = "protocol/schemas.rs"]
mod schemas;
mod support;

use rev_plugin_sdk::{identity::*, limits::*, methods::*, rpc::*, transport::*};
use serde_json::{Value, json};
use support::*;

// Pipe boundaries may split a Unicode scalar, JSON escape or newline. A second
// frame from the same read remains caller-owned until the next receive call.
#[test]
fn fragmented_frames() {
    let message = Message::Notification(Notification {
        jsonrpc: Version::V2,
        method: "messages.show".into(),
        params: json!({"text":"first\n🦀e\u{301}", "severity":"info"}),
    });
    let bytes = frame(&message);
    for width in [1, 2, 3, 7, bytes.len()] {
        let mut decoder = Decoder::new(FRAME_BYTES).unwrap();
        let mut received = Vec::new();
        for chunk in bytes.chunks(width) {
            let (used, value) = decoder.receive(chunk).unwrap();
            assert_eq!(used, chunk.len());
            received.extend(value);
        }
        assert_eq!(received, vec![message.clone()], "width {width}");
        decoder.finish().unwrap();
    }
    let mut decoder = Decoder::new(FRAME_BYTES).unwrap();
    let joined = [bytes.as_slice(), bytes.as_slice()].concat();
    let (used, first) = decoder.receive(&joined).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(first, Some(message.clone()));
    assert_eq!(decoder.receive(&joined[used..]).unwrap().1, Some(message));
}

// Invalid envelopes retire the connection. They cannot be resynchronized by
// skipping a line, interpreted as batches, or confused with null success values.
#[test]
fn malformed_frames() {
    let cases: &[(&str, &[u8])] = &[
        ("empty", b"\n"), ("syntax", b"{\n"), ("utf8", b"{\"x\":\"\xff\"}\n"),
        ("batch", b"[]\n"), ("version", b"{\"jsonrpc\":\"1.0\",\"method\":\"x\"}\n"),
        ("numeric id", b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"x\"}\n"),
        ("null request id", b"{\"jsonrpc\":\"2.0\",\"id\":null,\"method\":\"x\"}\n"),
        ("scalar params", b"{\"jsonrpc\":\"2.0\",\"method\":\"x\",\"params\":1}\n"),
        ("both outcomes", b"{\"jsonrpc\":\"2.0\",\"id\":\"a\",\"result\":null,\"error\":{\"code\":-1,\"message\":\"bad\"}}\n"),
        ("missing outcome", b"{\"jsonrpc\":\"2.0\",\"id\":\"a\"}\n"),
        ("missing response id", b"{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"bad\"}}\n"),
        ("unknown field", b"{\"jsonrpc\":\"2.0\",\"method\":\"x\",\"force\":true}\n"),
        ("trailing object", b"{}{}\n"),
    ];
    for (name, bytes) in cases {
        let mut decoder = Decoder::new(FRAME_BYTES).unwrap();
        assert_eq!(decoder.receive(bytes), Err(WireError::Malformed), "{name}");
        assert_eq!(decoder.receive(b"{}\n"), Err(WireError::Closed), "{name}");
    }
    for bytes in [b"{\"jsonrpc\":\"2.0\",\"id\":\"a\",\"result\":null}\n".as_slice(), b"{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32700,\"message\":\"parse error\"}}\n"] {
        assert!(Decoder::new(FRAME_BYTES).unwrap().receive(bytes).is_ok());
    }
}

// The frame ceiling counts encoded bytes, including escapes; EOF only succeeds
// at a completed newline. Oversize input never accumulates beyond the ceiling.
#[test]
fn frame_bounds() {
    let message = Message::Success(Success {
        jsonrpc: Version::V2,
        id: id("a"),
        result: json!("\n🦀"),
    });
    let bytes = frame(&message);
    let limit = bytes.len() - 1;
    assert_eq!(encode(&message, limit).unwrap(), bytes);
    assert_eq!(encode(&message, limit - 1), Err(WireError::LimitExceeded));
    let mut decoder = Decoder::new(limit).unwrap();
    assert!(decoder.receive(&bytes[..limit]).unwrap().1.is_none());
    assert_eq!(decoder.buffered_bytes(), limit);
    assert!(decoder.receive(b"\n").unwrap().1.is_some());
    decoder.finish().unwrap();
    let mut decoder = Decoder::new(limit - 1).unwrap();
    assert_eq!(decoder.receive(&bytes), Err(WireError::LimitExceeded));
    assert_eq!(decoder.buffered_bytes(), 0);
    let mut decoder = Decoder::new(limit).unwrap();
    decoder.receive(&bytes[..limit]).unwrap();
    assert_eq!(decoder.finish(), Err(WireError::Truncated));
    assert!(Decoder::new(FRAME_BYTES + 1).is_err());
}

// Package and command identity are independent of forge paths and process IDs;
// untrusted revisions never pass through JSON numbers or floating-point coercion.
#[test]
fn identities() {
    for (value, valid) in [
        ("field.notes", true),
        ("rev.notes", false),
        ("Field.notes", false),
        ("a.b.c", false),
        ("a..b", false),
        ("a.1b", false),
        ("a.b/..", false),
    ] {
        assert_eq!(PackageId::new(value).is_ok(), valid, "{value}");
        assert_eq!(
            serde_json::from_value::<PackageId>(json!(value)).is_ok(),
            valid
        );
    }
    let owner = PackageId::new("field.notes").unwrap();
    assert_eq!(
        owner
            .command(&CommandName::new("collect").unwrap())
            .as_str(),
        "field.notes.collect"
    );
    assert!(
        !CommandId::new("field.notebook.collect")
            .unwrap()
            .belongs_to(&owner)
    );
    for value in ["", "-1", "01", "1.0", "1e2", "18446744073709551616"] {
        assert!(Revision::new(value).is_err(), "{value}");
    }
    for value in ["0", "9007199254740993", "18446744073709551615"] {
        let value = Revision::new(value).unwrap();
        assert_eq!(
            serde_json::from_value::<Revision>(serde_json::to_value(&value).unwrap()).unwrap(),
            value
        );
    }
    assert!(serde_json::from_value::<Revision>(json!(9)).is_err());
    assert!(serde_json::from_value::<ScalarRange>(json!({"start":4,"end":3})).is_err());
    assert!(RecordId::new("../other").is_err());
    assert!(CommandName::new("x".repeat(65)).is_err());
    let max = PackageId::new(format!("{}.{}", "a".repeat(32), "b".repeat(32))).unwrap();
    assert_eq!(
        max.command(&CommandName::new("c".repeat(64)).unwrap())
            .as_str()
            .len(),
        130
    );
}

// Whole-batch shape admission catches overlapping edits and combined payload
// bounds before a native owner attempts source/history preparation.
#[test]
fn admission() {
    let limits = Limits::default();
    let stamp = json!({"buffer":{"session":"s","id":"b"},"revision":"9007199254740993"});
    let origin = json!({"package":"field.notes","operation":"op","depth":0});
    let mut batch = json!({"operation":"op","origin":origin,"targets":[{"stamp":stamp,"replacements":[{"range":{"start":0,"end":1},"text":"🦀"}]}]});
    validate_params("edits.apply", batch.clone(), &limits).unwrap();
    batch["targets"][0]["replacements"]
        .as_array_mut()
        .unwrap()
        .push(json!({"range":{"start":0,"end":0},"text":"X"}));
    assert_eq!(
        validate_params("edits.apply", batch, &limits)
            .unwrap_err()
            .kind(),
        Some(ErrorKind::EditConflict)
    );
    assert!(serde_json::from_value::<List<Value, 2>>(json!([1, 2, 3])).is_err());
    let read = json!({"stamp":stamp,"range":{"start":0,"end":3},"text":"a🦀b"});
    validate_result("buffers.read", read.clone(), &limits).unwrap();
    let mut wrong_units = read;
    wrong_units["range"]["end"] = json!(6);
    assert_eq!(
        validate_result("buffers.read", wrong_units, &limits)
            .unwrap_err()
            .kind(),
        Some(ErrorKind::InvalidRange)
    );
    let mut raised = limits.clone();
    raised.causal_depth += 1;
    assert!(!raised.validate());
    for (name, params) in [
        (
            "buffers.save",
            json!({"buffer":{"session":"s","id":"b"},"expected_revision":"1"}),
        ),
        (
            "state.put",
            json!({"concern":"tasks","record":"one","value":null}),
        ),
        (
            "buffers.close",
            json!({"buffer":{"session":"s","id":"b"},"expected_revision":"1","force":true}),
        ),
    ] {
        assert!(validate_params(name, params, &limits).is_err(), "{name}");
    }
    let put = json!({"concern":"tasks","record":"one","expected_version":null,"value":null});
    validate_params("state.put", put, &limits).unwrap();
    validate_result(
        "state.get",
        json!({"status":"present","version":"1","value":null}),
        &limits,
    )
    .unwrap();
    validate_result("state.get", json!({"status":"absent"}), &limits).unwrap();
    assert!(validate_params("state.put", json!({"concern":"tasks","record":"one","expected_version":null,"value":"x".repeat(STATE_BYTES)}), &limits).is_err());
}

// A command's inert arguments and captured prefix obey its declared contract.
// Invalid declarations or unknown fields reject the whole registration request.
#[test]
fn command_arguments() {
    let declaration: CommandDeclaration = serde_json::from_value(json!({"name":"collect","description":"Collect task lines.","parameters":[{"name":"count","kind":"integer","required":true}],"prefix":"integer","recording":"semantic"})).unwrap();
    for (arguments, prefix, valid) in [
        (json!({"count":2}), Some(-3), true),
        (json!({"count":"2"}), None, false),
        (json!({}), None, false),
        (json!({"count":2,"code":"run"}), None, false),
        (json!([2]), None, false),
    ] {
        assert_eq!(
            declaration
                .validate_arguments(&arguments, prefix, &Limits::default())
                .is_ok(),
            valid,
            "{arguments}"
        );
    }
    let contribution = json!({"declaration":declaration,"handler":"collect"});
    assert!(
        validate_params(
            "commands.register",
            json!({"commands":[contribution,contribution]}),
            &Limits::default()
        )
        .is_err()
    );
}
