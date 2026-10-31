use super::*;

// Native named commands operate on the same session owner. Default activation
// remains inert, enabling attaches the current file, and disabling preserves text.
#[test]
fn native_controls() {
    use rev::{
        buffer::Buffers,
        command::{Command, LspCommand},
        session::Session,
        view::{Size, View},
    };
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "fn main() {}\n");
    let server = &policy(&fixture, "ready").servers["fixture"];
    let source = format!(
        "[lsp]\ninitialize_timeout_ms = 1000\nshutdown_timeout_ms = 250\n[lsp.servers.rust]\ncommand = {:?}\nargs = {:?}",
        server.command, server.args
    );
    let startup = rev::config::prepare(&fixture.write("config.toml", &source));
    assert!(startup.diagnostic.is_none());
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    session.load(&file).unwrap();
    assert_eq!(session.lsp_status(), Status::Disabled);
    session.execute(Command::Lsp(LspCommand::Mode));
    let deadline = Instant::now() + Duration::from_secs(4);
    while session.lsp_status() == Status::Starting {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(matches!(session.lsp_status(), Status::Ready(_)));
    session.execute(Command::Lsp(LspCommand::Status));
    session.execute(Command::Lsp(LspCommand::Mode));
    assert_eq!(session.lsp_status(), Status::Disabled);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .slice(
                &session
                    .buffer()
                    .unwrap()
                    .span(0..session.buffer().unwrap().len_chars())
                    .unwrap()
            )
            .unwrap()
            .to_string(),
        "fn main() {}\n"
    );
}
