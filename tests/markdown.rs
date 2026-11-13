#![forbid(unsafe_code)]

#[path = "markdown/contexts.rs"]
mod contexts;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "markdown/headings.rs"]
mod headings;
#[path = "markdown/lifecycle.rs"]
mod lifecycle;
#[path = "markdown/list_controls.rs"]
mod list_controls;
#[path = "markdown/lists.rs"]
mod lists;
#[path = "markdown/palette.rs"]
mod palette;
#[path = "markdown/presentation.rs"]
mod presentation;
#[path = "support/surface.rs"]
mod surface;
#[path = "markdown/tables.rs"]
mod tables;

use std::time::{Duration, Instant};

use rev::buffer::{Buffer, Buffers};
use rev::config::prepare;
use rev::display::{Frame, Size};
use rev::session::Session;
use rev::syntax::{MarkdownRole, Style};
use rev::view::View;
use unicode_width::UnicodeWidthStr;

use fixture::Fixture;

const SIZE: Size = Size {
    width: 80,
    height: 24,
};

fn editor(config: &str, name: &str, source: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&fixture.write(name, source)).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    (fixture, Session::new(buffers, view, SIZE).unwrap())
}

fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(Instant::now() < deadline, "Markdown worker did not settle");
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn style_at(frame: &Frame, row: usize, needle: &str) -> Option<Style> {
    let row = &frame.rows()[row];
    let byte = row.text().find(needle).unwrap();
    let column = UnicodeWidthStr::width(&row.text()[..byte]);
    row.syntax()
        .iter()
        .find(|paint| {
            usize::from(paint.columns.start) <= column && column < usize::from(paint.columns.end)
        })
        .map(|paint| paint.style)
}
