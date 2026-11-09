use super::*;
use rev_plugin_sdk::client::{Client, Event};
use std::time::Duration;

fn list_params() -> BuffersListParams {
    BuffersListParams {
        page: Page {
            cursor: None,
            limit: 10,
        },
        kind: Some(ResourceKind::Text),
    }
}
fn response(request: OpaqueId, result: Value) -> Vec<u8> {
    frame(Message::Success(Success {
        jsonrpc: Version::V2,
        id: request,
        result,
    }))
}

// Initialization must precede operations. Foreign packages, API revisions,
// elevated limits and unsupported capabilities cannot silently change the offer.
#[test]
fn negotiation() {
    let mut pending = Client::new(package(), List::default()).unwrap();
    assert!(
        pending
            .request::<BuffersList>(list_params(), Duration::ZERO)
            .is_err()
    );
    let baseline = offer(vec![]);
    let mut client = client(&baseline);
    assert!(client.initialized());
    assert!(
        client
            .request::<BuffersList>(list_params(), Duration::ZERO)
            .is_err()
    );
    for name in ["api", "package", "limit", "duplicate capabilities"] {
        let mut offer = offer(vec![Capability::Buffers]);
        match name {
            "api" => offer.api = Label::new("rev.plugins/9.0").unwrap(),
            "package" => offer.package.id = PackageId::new("other.notes").unwrap(),
            "limit" => offer.limits.pending_per_package = 17,
            _ => {
                offer.capabilities =
                    List::new(vec![Capability::Buffers, Capability::Buffers]).unwrap()
            }
        }
        let mut client = Client::new(package(), List::default()).unwrap();
        assert!(
            client.receive(&frame(initialize(&offer))).is_err(),
            "{name}"
        );
        assert!(!client.initialized(), "{name}");
        assert!(
            client
                .request::<BuffersList>(list_params(), Duration::ZERO)
                .is_err()
        );
        let reply = drain(&mut client);
        assert!(matches!(
            Decoder::new(FRAME_BYTES)
                .unwrap()
                .receive(&reply)
                .unwrap()
                .1,
            Some(Message::Failure(_))
        ));
    }
    for name in ["frame", "queue"] {
        let mut offer = offer(vec![]);
        if name == "frame" {
            offer.limits.frame_bytes = 64;
        } else {
            offer.limits.outbound_per_package = 1;
        }
        let mut client = Client::new(package(), List::default()).unwrap();
        assert!(
            client.receive(&frame(initialize(&offer))).is_err(),
            "{name}"
        );
        assert!(!client.initialized());
        assert!(client.receive(&frame(initialize(&offer))).is_err());
    }
}

// Several requests may complete out of order. ID, result shape and generation
// qualification prevent a wrong or duplicate response from resolving a slot.
#[test]
fn correlation() {
    let mut client = client(&offer(vec![Capability::Buffers]));
    let first = client
        .request::<BuffersList>(list_params(), Duration::ZERO)
        .unwrap();
    let second = client
        .request::<BuffersList>(list_params(), Duration::ZERO)
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(client.pending_len(), 2);
    for request in [&second, &first] {
        let event = client
            .receive(&response(
                request.clone(),
                json!({"buffers":[],"next":null}),
            ))
            .unwrap()
            .1
            .unwrap();
        assert!(matches!(event, Event::Response { id, result: Ok(_), .. } if &id == request));
    }
    assert_eq!(client.pending_len(), 0);
    assert!(matches!(
        client.receive(&response(first, json!({}))).unwrap().1,
        Some(Event::StaleResponse(_))
    ));
    assert!(matches!(
        client.receive(&response(id("6:1"), json!({}))).unwrap().1,
        Some(Event::StaleResponse(_))
    ));
    let wrong = client
        .request::<BuffersList>(list_params(), Duration::ZERO)
        .unwrap();
    assert!(matches!(
        client
            .receive(&response(wrong, json!({"text":"wrong result"})))
            .unwrap()
            .1,
        Some(Event::Response { result: Err(_), .. })
    ));
    let failed = client
        .request::<BuffersList>(list_params(), Duration::ZERO)
        .unwrap();
    let failure = frame(Message::Failure(Failure {
        jsonrpc: Version::V2,
        id: Some(failed),
        error: RpcError::application(ErrorKind::Busy),
    }));
    assert!(
        matches!(client.receive(&failure).unwrap().1, Some(Event::Response { result: Err(error), .. }) if error.kind() == Some(ErrorKind::Busy))
    );
}

// Admission is transactional at both limits: a full correlation set or output
// queue rejects a new call without leaving a pending slot or losing queued data.
#[test]
fn queue_bounds() {
    let mut client = client(&offer(vec![Capability::Buffers]));
    for _ in 0..16 {
        client
            .request::<BuffersList>(list_params(), Duration::ZERO)
            .unwrap();
    }
    let queued = client.queued_bytes();
    assert!(
        client
            .request::<BuffersList>(list_params(), Duration::ZERO)
            .is_err()
    );
    assert_eq!(client.pending_len(), 16);
    assert_eq!(client.queued_bytes(), queued);
    let prefix = client.output()[..3].to_vec();
    client.consume_output(3).unwrap();
    assert_eq!(client.queued_bytes(), queued - 3);
    let mut bytes = prefix;
    bytes.extend(drain(&mut client));
    assert_eq!(bytes.len(), queued);
    assert!(client.consume_output(1).is_err());
    client.retire();
    assert_eq!(client.pending_len(), 0);
    assert_eq!(client.queued_bytes(), 0);
    assert!(client.receive(b"{}\n").is_err());

    let mut full = support::client(&offer(vec![Capability::Buffers]));
    let params = || BuffersCreateParams {
        name: Label::new("result").unwrap(),
        text: SourceText::new("x".repeat(TEXT_BYTES)).unwrap(),
        read_only: false,
    };
    for _ in 0..7 {
        full.request::<BuffersCreate>(params(), Duration::ZERO)
            .unwrap();
    }
    let queued = full.queued_bytes();
    assert!(
        full.request::<BuffersCreate>(params(), Duration::ZERO)
            .is_err()
    );
    assert_eq!(full.queued_bytes(), queued);
    assert_eq!(full.pending_len(), 7);
    assert!(queued <= FRAME_BYTES);
}

// Deadlines are driven by a monotonic caller clock. Canceled/expired replies
// have no correlation authority, and EOF clears every owned connection resource.
#[test]
fn deadlines() {
    let mut client = client(&offer(vec![Capability::Buffers]));
    let first = client
        .request::<BuffersList>(list_params(), Duration::from_secs(1))
        .unwrap();
    let second = client
        .request::<BuffersList>(list_params(), Duration::from_secs(2))
        .unwrap();
    assert_eq!(client.next_deadline(), Some(Duration::from_secs(61)));
    assert!(client.expire(Duration::from_secs(60)).is_empty());
    assert_eq!(client.expire(Duration::from_secs(61)), vec![first.clone()]);
    assert!(client.cancel(&second));
    assert!(!client.cancel(&second));
    for request in [first, second] {
        assert!(matches!(
            client
                .receive(&response(request, json!({"buffers":[],"next":null})))
                .unwrap()
                .1,
            Some(Event::StaleResponse(_))
        ));
    }
    client.finish().unwrap();
    assert!(!client.initialized());
    assert_eq!(client.queued_bytes(), 0);
}

// Host callbacks belong to one session/generation and registered handler. A
// cancellation retires callback authority before any late result can be queued.
#[test]
fn callbacks() {
    let mut client = client(&offer(vec![Capability::Commands, Capability::Events]));
    let handler = Handler {
        command: CommandId::new("field.notes.collect").unwrap(),
        handler: id("collect"),
    };
    client.add_handler(handler.clone()).unwrap();
    assert!(client.add_handler(handler.clone()).is_err());
    let context = json!({"invocation":"invoke-1","generation":"7","stamp":{"buffer":{"session":"session-a","id":"buffer-1"},"revision":"1"},"pane":null,"point":0,"selection":null,"prefix":null,"origin":{"package":null,"operation":"op","depth":0}});
    let call = Request {
        jsonrpc: Version::V2,
        id: id("host-1"),
        method: "command.invoke".into(),
        params: json!({"command":"field.notes.collect","handler":"collect","arguments":{},"context":context}),
    };
    for (name, field, value) in [
        ("generation", "generation", json!("6")),
        (
            "session",
            "stamp",
            json!({"buffer":{"session":"session-b","id":"buffer-1"},"revision":"1"}),
        ),
    ] {
        let mut invalid = call.clone();
        invalid.params["context"][field] = value;
        assert!(
            client.receive(&frame(Message::Request(invalid))).is_err(),
            "{name}"
        );
        drain(&mut client);
        assert_eq!(client.pending_len(), 0);
    }
    let Some(Event::Request(first_callback)) = client
        .receive(&frame(Message::Request(call.clone())))
        .unwrap()
        .1
    else {
        panic!("expected admitted callback");
    };
    assert_eq!(client.pending_len(), 1);
    let cancel = frame(Message::Notification(Notification {
        jsonrpc: Version::V2,
        method: "command.cancel".into(),
        params: json!({"invocation":"invoke-1","generation":"7"}),
    }));
    assert!(matches!(
        client.receive(&cancel).unwrap().1,
        Some(Event::Notification(_))
    ));
    assert_eq!(client.pending_len(), 0);
    assert!(
        client
            .respond::<CommandInvoke>(
                &first_callback,
                Ok(CommandsInvokeResult { value: json!(null) })
            )
            .is_err()
    );
    let canceled = drain(&mut client);
    assert!(
        matches!(Decoder::new(FRAME_BYTES).unwrap().receive(&canceled).unwrap().1, Some(Message::Failure(failure)) if failure.error.kind() == Some(ErrorKind::Cancelled))
    );
    let Some(Event::Request(next_callback)) = client
        .receive(&frame(Message::Request(call.clone())))
        .unwrap()
        .1
    else {
        panic!("expected replacement callback");
    };
    assert!(
        client
            .respond::<CommandInvoke>(
                &first_callback,
                Ok(CommandsInvokeResult {
                    value: json!("late")
                })
            )
            .is_err()
    );
    client
        .respond::<CommandInvoke>(
            &next_callback,
            Ok(CommandsInvokeResult {
                value: json!("done"),
            }),
        )
        .unwrap();
    assert_eq!(client.pending_len(), 0);
    drain(&mut client);
    client.remove_handler(&handler.command);
    assert!(client.receive(&frame(Message::Request(call))).is_err());
    drain(&mut client);
    let event = |generation| {
        frame(Message::Notification(Notification {
            jsonrpc: Version::V2,
            method: BUFFER_EVENT.into(),
            params: json!({"subscription":"sub","sequence":"2","generation":generation,"kind":"resnapshot","stamp":null,"origin":{"package":null,"operation":"op","depth":0}}),
        }))
    };
    assert!(client.receive(&event("6")).is_err());
    assert!(matches!(
        client.receive(&event("7")).unwrap().1,
        Some(Event::Notification(_))
    ));
    assert!(client.output().is_empty());
    let mut duplicate = support::client(&offer(vec![Capability::Commands]));
    duplicate.add_handler(handler).unwrap();
    let request = Request {
        jsonrpc: Version::V2,
        id: id("duplicate"),
        method: "command.cancel".into(),
        params: json!({"invocation":"other","generation":"7"}),
    };
    duplicate
        .receive(&frame(Message::Request(request.clone())))
        .unwrap();
    assert!(
        duplicate
            .receive(&frame(Message::Request(request)))
            .is_err()
    );
    assert!(!duplicate.initialized());
    assert_eq!(duplicate.pending_len(), 0);
}
