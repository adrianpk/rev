use std::time::Instant;

use crossterm::event::KeyCode;
use rev::command::{Command, Motion};
use rev::config::Rgb;
use rev::display::Frame;
use unicode_width::UnicodeWidthStr;

use super::fixture::{Case, Editor, SOURCES};
use super::observe::Cells;
use crate::surface::Cell;

const COLORS: [Rgb; 3] = [
    Rgb(0x10, 0x20, 0x30),
    Rgb(0x40, 0x50, 0x60),
    Rgb(0x70, 0x80, 0x90),
];

// The frozen 3 x 2 x 4 matrix observes every complete emitted update through
// all 32 insertions and 32 Backspaces, rather than sampling the settled result.
#[test]
fn frozen_bursts() {
    contract();
    let started = Instant::now();
    let mut count = 0;
    let mut frames = 0;
    for (language, source) in SOURCES {
        for profile in ["non-modal", "modal"] {
            for style in ["depth", "match", "pulse", "off"] {
                frames += burst(
                    Case {
                        language,
                        profile,
                        style,
                    },
                    source,
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 24);
    assert!(frames >= 24 * 64);
    eprintln!(
        "CONTINUITY bursts={count} completed_frames={frames} elapsed_us={}",
        started.elapsed().as_micros()
    );
}

fn contract() {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../ops/default/report/rev-delimiter-presentation/acceptance-contract.json"
    ))
    .unwrap();
    let c = &value["continuity"];
    for (language, source) in SOURCES {
        let name = if language == "plain" {
            "text"
        } else {
            language
        };
        assert_eq!(c["sources"][name], source);
    }
    assert_eq!(c["profiles"], serde_json::json!(["non-modal", "modal"]));
    assert_eq!(
        c["styles"],
        serde_json::json!(["depth", "match", "pulse", "off"])
    );
    assert_eq!(c["insert_ascii_count"], 32);
    assert_eq!(c["backspace_count"], 32);
    assert_eq!(c["native_burst_count"], 24);
    assert_eq!(c["observe_every_completed_frame"], true);
}

fn burst(case: Case, source: &str) -> usize {
    let at = source.find("leaf").unwrap() + 2;
    let mut e = Editor::new(case, source, at);
    let initial = baseline(&mut e, case);
    let cells = crate::surface::draw(&initial);
    let oracle = Oracle::new(case, &initial, &cells);
    oracle.check(source, at, &initial, &cells);
    let before = e.observer.frames;
    for step in 0..64 {
        let count = if step < 32 { step + 1 } else { 63 - step };
        let expected = format!("{}{}{}", &source[..at], "x".repeat(count), &source[at..]);
        e.handle(if step < 32 {
            KeyCode::Char('x')
        } else {
            KeyCode::Backspace
        });
        assert_eq!(e.source(), expected, "{case:?} step {step}");
        let previous = e.observer.frames;
        e.settle(|frame, cells| oracle.check(&expected, at + count, frame, cells));
        assert!(
            e.observer.frames > previous,
            "every source edit has a completed frame"
        );
        assert_eq!((e.session.view().top, e.session.view().left), (0, 0));
    }
    assert_eq!(e.source(), source);
    assert_eq!(e.session.view().point.position.0, at);
    let frames = e.observer.frames - before;
    active(&mut e, case, source);
    eprintln!("CONTINUITY {case:?} inputs=64 completed_frames={frames}");
    frames
}

fn baseline(e: &mut Editor, case: Case) -> Frame {
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let frame = e.settle(|_, _| {});
        if case.language == "plain" || frame.rows()[0].text().contains('◉') {
            return frame;
        }
        assert!(Instant::now() < deadline, "positive document baseline");
        std::thread::yield_now();
    }
}

struct Oracle {
    case: Case,
    row: usize,
    column: usize,
    prefix: Vec<Cell>,
    heading: Vec<((usize, usize), Cell)>,
}

impl Oracle {
    fn new(case: Case, frame: &Frame, cells: &Cells) -> Self {
        let row = usize::from(case.language != "plain");
        let text = frame.rows()[row].text();
        let column = text[..text.find("Body").unwrap()].width();
        let prefix = (0..column).map(|x| cells[&(row, x)]).collect();
        let heading = if row == 0 {
            Vec::new()
        } else {
            heading(frame, cells)
        };
        Self {
            case,
            row,
            column,
            prefix,
            heading,
        }
    }

    fn check(&self, source: &str, point: usize, frame: &Frame, cells: &Cells) {
        let body = source.find("Body").unwrap();
        assert!(
            frame.rows()[self.row]
                .text()
                .contains(source[body..].trim_end())
        );
        assert_eq!(frame.rows()[self.row].selection(), None);
        let cursor = frame.cursor().expect("completed source cursor");
        assert_eq!(
            (usize::from(cursor.row), usize::from(cursor.column)),
            (self.row, self.column + point - body)
        );
        for (key, expected) in &self.heading {
            assert_eq!(
                cells[key], *expected,
                "heading remains exact: {:?}",
                self.case
            );
        }
        for (x, expected) in self.prefix.iter().enumerate() {
            assert_eq!(cells[&(self.row, x)], *expected);
        }
        self.endpoints(source, frame, cells);
    }

    fn endpoints(&self, source: &str, frame: &Frame, cells: &Cells) {
        let points: Vec<_> = source
            .chars()
            .enumerate()
            .filter(|(_, c)| "()[]{}".contains(*c))
            .collect();
        let depth = [0, 1, 2, 2, 1, 0];
        assert_eq!(points.len(), depth.len());
        for ((at, character), depth) in points.iter().zip(depth) {
            let cell = cells[&(self.row, self.column + at - source.find("Body").unwrap())];
            assert_eq!(cell.character, *character);
            let expected = if self.case.style == "depth" {
                COLORS[depth]
            } else {
                Rgb(0xa1, 0xb2, 0xc3)
            };
            assert_eq!(
                cell.foreground,
                Some(expected),
                "{:?} source={at}",
                self.case
            );
            assert!(
                !cell.underline,
                "ordinary body point cannot select an enclosing pair"
            );
        }
        let actual: Vec<_> = crate::rendering::marks(frame)
            .into_iter()
            .map(|m| m.0)
            .collect();
        let expected: Vec<_> = points
            .into_iter()
            .filter_map(|(at, _)| (self.case.style == "depth").then_some(at))
            .collect();
        assert_eq!(actual, expected, "current mapped endpoint identities");
    }
}

fn heading(frame: &Frame, cells: &Cells) -> Vec<((usize, usize), Cell)> {
    assert!(
        frame.rows()[0].text().contains('◉'),
        "positive pretty-heading baseline"
    );
    let column = frame.rows()[0].text().find("Heading").unwrap();
    let column = frame.rows()[0].text()[..column].width();
    assert_eq!(cells[&(0, column)].foreground, Some(Rgb(0xca, 0x8e, 0x12)));
    cells
        .iter()
        .filter(|(key, _)| key.0 == 0)
        .map(|(key, cell)| (*key, *cell))
        .collect()
}

fn active(e: &mut Editor, case: Case, source: &str) {
    let before = e.session.view().point;
    let history = e.session.buffer().unwrap().history().len();
    e.session.execute(Command::MatchingDelimiter);
    assert_eq!(
        e.session.view().point,
        before,
        "body entry refuses an arbitrary enclosing pair"
    );
    e.opening(source);
    e.session.execute(Command::MatchingDelimiter);
    let outer = source.find('(').unwrap();
    let mate = source.find(')').unwrap();
    assert_eq!(e.session.view().point.position.0, mate);
    e.settle(|_, _| {});
    e.settle(|frame, cells| {
        let marked: Vec<_> = crate::rendering::marks(frame)
            .into_iter()
            .filter(|(_, row, column)| cells[&(*row, *column)].underline)
            .map(|m| m.0)
            .collect();
        assert_eq!(
            marked,
            if case.style == "off" {
                Vec::new()
            } else {
                vec![outer, mate]
            }
        );
    });
    e.session.execute(Command::Motion(Motion::BufferStart));
    super::advance(1000);
    e.settle(|_, _| {});
    assert_eq!(e.session.remaining_input(super::now()), None);
    assert_eq!(e.source(), source);
    assert_eq!(e.session.buffer().unwrap().stamp(), before.stamp);
    assert_eq!(e.session.buffer().unwrap().history().len(), history);
}
