#![forbid(unsafe_code)]
#[path = "integrated_terminal/engine.rs"]
mod engine;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "integrated_terminal/resources.rs"]
mod resources;
use fixture::Fixture;
use rev::term::{Geometry, Launch, Limits, SessionId, Sessions};
use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

struct Terminal {
    sessions: Sessions,
    id: SessionId,
    wake: Receiver<()>,
    _fixture: Fixture,
}
impl Terminal {
    fn new(script: &str, limits: Limits) -> Self {
        let fixture = Fixture::new();
        let (send, wake) = mpsc::sync_channel(1);
        let mut sessions = Sessions::new(
            limits,
            Arc::new(move || {
                let _ = send.try_send(());
            }),
        )
        .unwrap();
        let script = fixture.write("shell-fixture.sh", script);
        let launch = Launch {
            program: "/bin/sh".into(),
            args: vec![script.to_string_lossy().into_owned()],
            directory: fixture.0.clone(),
        };
        let id = sessions
            .open(
                launch,
                Geometry {
                    columns: 80,
                    rows: 24,
                },
            )
            .unwrap();
        Self {
            sessions,
            id,
            wake,
            _fixture: fixture,
        }
    }
    fn wait(&mut self, ready: impl Fn(&rev::term::Session) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let work = self.sessions.service();
            assert!(work.bytes <= rev::term::SERVICE_BYTES);
            let session = self.sessions.get(self.id).unwrap();
            assert!(
                !matches!(session.state(), rev::term::State::Failed(_)),
                "{:?}",
                session.state()
            );
            if ready(session) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "terminal did not reach condition: {:?}, {:?}",
                session.state(),
                session.diagnostic()
            );
            let remaining = deadline.saturating_duration_since(Instant::now());
            // A coalesced readiness callback drives progress. There is no
            // production polling timer or fixed fixture sleep.
            self.wake
                .recv_timeout(remaining)
                .expect("terminal readiness deadline");
        }
    }
    fn send(&mut self, bytes: &[u8]) {
        self.sessions.queue(self.id, bytes).unwrap();
    }
    fn text(&self) -> String {
        text(self.sessions.get(self.id).unwrap())
    }
}
fn text(session: &rev::term::Session) -> String {
    use alacritty_terminal::{
        grid::Dimensions,
        index::{Column, Line, Point},
    };
    let grid = session.screen().grid();
    (0..grid.screen_lines())
        .map(|row| {
            (0..grid.columns())
                .map(|column| grid[Point::new(Line(row as i32), Column(column))].c)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
