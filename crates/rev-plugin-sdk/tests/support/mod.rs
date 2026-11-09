use rev_plugin_sdk::{
    API,
    client::{Client, Event},
    identity::*,
    limits::Limits,
    methods::*,
    rpc::*,
    transport,
};

pub fn id(value: &str) -> OpaqueId {
    OpaqueId::new(value).unwrap()
}
pub fn package() -> Package {
    Package {
        id: PackageId::new("field.notes").unwrap(),
        version: Label::new("0.1.0").unwrap(),
    }
}
pub fn offer(capabilities: Vec<Capability>) -> InitializeParams {
    InitializeParams {
        session: id("session-a"),
        generation: Generation::new("7").unwrap(),
        package: package(),
        api: Label::new(API).unwrap(),
        capabilities: List::new(capabilities).unwrap(),
        units: Units::UnicodeScalar,
        limits: Limits::default(),
    }
}
pub fn frame(message: &Message) -> Vec<u8> {
    transport::encode(message, Limits::default().frame_bytes as usize).unwrap()
}
pub fn initialize(offer: &InitializeParams) -> Message {
    Message::Request(Request {
        jsonrpc: Version::V2,
        id: id("init"),
        method: "initialize".into(),
        params: serde_json::to_value(offer).unwrap(),
    })
}
pub fn client(offer: &InitializeParams) -> Client {
    let mut client = Client::new(package(), List::default()).unwrap();
    assert!(matches!(
        client.receive(&frame(&initialize(offer))).unwrap().1,
        Some(Event::Initialized(_))
    ));
    drain(&mut client);
    client
}
pub fn drain(client: &mut Client) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !client.output().is_empty() {
        let count = client.output().len();
        bytes.extend_from_slice(client.output());
        client.consume_output(count).unwrap();
    }
    bytes
}
