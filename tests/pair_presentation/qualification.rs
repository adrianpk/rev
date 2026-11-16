use crate::fixture::Fixture;
use rev::buffer::{Buffer, BufferId, Buffers};
use rev::pairs::{MAX_BYTES, MAX_METADATA, PresentationTicket, SLICE_TIME, SLICE_TOKENS, Status};
use rev::window::{ViewId, Windows};
use std::time::{Duration, Instant};

#[path = "qualification/facts.rs"]
mod facts;
#[path = "qualification/lifecycle.rs"]
mod lifecycle;
#[path = "qualification/mapping.rs"]
mod mapping;
#[path = "qualification/resources.rs"]
mod resources;

pub(super) struct Harness {
    fixture: Fixture,
    pub buffers: Buffers,
}
impl Harness {
    pub fn new(language: &str, colors: bool) -> Self {
        let fixture = Fixture::new();
        let root = fixture.write(
            "config.toml",
            &format!("syntax.enabled={colors}\nfiles.associations.'*.case'='{language}'"),
        );
        let startup = rev::config::prepare(&root);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        Self {
            fixture,
            buffers: Buffers::with_settings(startup.snapshot).unwrap(),
        }
    }
    pub fn source(&mut self, source: &str) -> BufferId {
        let path = self.fixture.write(
            &format!("source-{}.case", self.buffers.iter().len()),
            source,
        );
        self.buffers.visit(&path).unwrap()
    }
}

pub(super) fn view(buffer: &Buffer) -> ViewId {
    Windows::new(rev::view::View::new(buffer))
        .unwrap()
        .selected_id()
}

pub(super) fn request(buffer: &mut Buffer, view: ViewId) -> PresentationTicket {
    buffer
        .start_pair_presentation(view, buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
}

pub(super) fn complete(buffer: &mut Buffer, ticket: PresentationTicket) -> Duration {
    let end = Instant::now() + Duration::from_secs(10);
    let mut maximum = Duration::ZERO;
    while Instant::now() < end {
        let start = Instant::now();
        let status = buffer.poll_pair_presentation(ticket, start + SLICE_TIME);
        maximum = maximum.max(start.elapsed());
        assert!(buffer.retained_pair_bytes() <= MAX_METADATA);
        if status == Status::Ready {
            let result = buffer.pair_presentation(ticket).unwrap();
            assert!(result.stats.inspected_bytes <= MAX_BYTES);
            assert!(result.stats.maximum_slice_tokens <= SLICE_TOKENS);
            assert_eq!(result.stats.retained_bytes, buffer.retained_pair_bytes());
            return maximum;
        }
        assert_eq!(status, Status::Pending);
    }
    panic!("presentation request exceeded finite test deadline");
}
