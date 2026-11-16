use crate::{surface, terminal::row};
use rev::config::Rgb;

pub const NORMAL: Rgb = Rgb(0xa1, 0xb2, 0xc3);
const DEPTH: [Rgb; 2] = [Rgb(0x10, 0x20, 0x30), Rgb(0x40, 0x50, 0x60)];
const MATCH: Rgb = Rgb(0xab, 0xcd, 0xef);
const BACKGROUND: Rgb = Rgb(0x20, 0x30, 0x40);

#[derive(Default)]
pub struct Observer {
    pub frames: usize,
}

impl Observer {
    // Decode every completed update, including earlier updates in a read. A
    // later correct frame cannot conceal lost source text or spurious marks.
    pub fn record(&mut self, frame: &str) {
        self.frames += 1;
        let cells = surface::decode(frame.as_bytes().to_vec());
        let marks = cells
            .iter()
            .filter(|((r, _), cell)| *r < 22 && cell.underline)
            .count();
        assert!(marks <= 2, "only active endpoints may be underlined");
        assert!(!frame.contains("[5m") && !frame.contains("[6m"));
        for number in 0..22 {
            let text: String = cells
                .range((number, 0)..(number + 1, 0))
                .map(|(_, cell)| cell.character)
                .collect();
            if text.starts_with("Line ") {
                let index: usize = text[5..9].parse().unwrap();
                assert!(index < 1000);
                assert!(text.starts_with(&format!("Line {index:04} (value)")));
            }
        }
    }
}

// Independent palette and literal source positions, rather than colors read
// back from the editor settings or its presentation facts.
pub fn form(frame: &str, style: &str, active: bool) -> bool {
    if !row(frame, 1).starts_with("Body (outer [inner]) tail") {
        return false;
    }
    let cells = surface::decode(frame.as_bytes().to_vec());
    [(5, 0), (12, 1), (18, 1), (19, 0)]
        .into_iter()
        .all(|(column, depth)| {
            let marked = active && matches!(column, 5 | 19);
            cells.get(&(0, column)).is_some_and(|cell| {
                cell.foreground == Some(foreground(style, depth, marked))
                    && cell.underline == marked
                    && (cell.background == Some(BACKGROUND)) == marked
            })
        })
}

fn foreground(style: &str, depth: usize, marked: bool) -> Rgb {
    match style {
        "depth" => DEPTH[depth],
        "match" | "pulse" if marked => MATCH,
        _ => NORMAL,
    }
}

pub fn position(frame: &str, line: usize, column: usize) -> bool {
    frame.contains(&format!("L{line} C{column} "))
}

pub fn navigation(frame: &str, line: usize) -> bool {
    position(frame, line, 11)
        && (1..=22).any(|r| {
            row(frame, r).starts_with(&format!("Line {:04} (value)", line - 1))
                && caret(frame) == Some((r, 11))
        })
}

pub fn caret(frame: &str) -> Option<(usize, usize)> {
    let tail = frame.strip_suffix("\x1b[?25h")?.rsplit_once("\x1b[")?.1;
    let (row, column) = tail.strip_suffix('H')?.split_once(';')?;
    Some((row.parse().ok()?, column.parse().ok()?))
}

pub fn unmarked(frame: &str) -> bool {
    let cells = surface::decode(frame.as_bytes().to_vec());
    (0..22)
        .filter(|row| {
            cells
                .get(&(*row, 0))
                .is_some_and(|cell| cell.character == 'L')
        })
        .all(|row| {
            cells.range((row, 0)..(row + 1, 0)).all(|(_, cell)| {
                !cell.underline
                    && cell.background != Some(BACKGROUND)
                    && (cell.character.is_whitespace() || cell.foreground == Some(NORMAL))
            })
        })
}

pub fn pulsing(frame: &str) -> bool {
    surface::decode(frame.as_bytes().to_vec())
        .iter()
        .filter(|((row, _), cell)| {
            *row < 22
                && cell.underline
                && cell.foreground == Some(MATCH)
                && cell.background == Some(BACKGROUND)
        })
        .count()
        == 2
}
