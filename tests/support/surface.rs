use std::collections::BTreeMap;

use rev::config::Rgb;
use rev::display::Frame;
use unicode_width::UnicodeWidthChar;

/// Decode the actual frame writer's cursor/SGR protocol. Cells retain effective
/// colors, so an earlier correct escape cannot hide a later leaking style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub character: char,
    pub foreground: Option<Rgb>,
    pub background: Option<Rgb>,
    pub underline: bool,
}

pub fn draw(frame: &Frame) -> BTreeMap<(usize, usize), Cell> {
    crossterm::style::force_color_output(true);
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    let mut cells = BTreeMap::new();
    let mut state = Cell::default();
    let mut point = (0, 0);
    let mut text = output.as_str();
    while !text.is_empty() {
        if let Some(sequence) = text.strip_prefix("\x1b[") {
            let end = sequence.find(|c: char| ('@'..='~').contains(&c)).unwrap();
            command(&sequence[..=end], &mut point, &mut state);
            text = &sequence[end + 1..];
        } else {
            let character = text.chars().next().unwrap();
            state.character = character;
            cells.insert(point, state);
            point.1 += character.width().unwrap_or(0);
            text = &text[character.len_utf8()..];
        }
    }
    cells
}

fn command(sequence: &str, point: &mut (usize, usize), state: &mut Cell) {
    if let Some(parameters) = sequence.strip_suffix('H') {
        let (row, column) = parameters.split_once(';').unwrap();
        *point = (
            row.parse::<usize>().unwrap() - 1,
            column.parse::<usize>().unwrap() - 1,
        );
    } else if let Some(parameters) = sequence.strip_suffix('m') {
        sgr(parameters, state);
    }
}

fn sgr(parameters: &str, state: &mut Cell) {
    let codes: Vec<u16> = parameters
        .split(';')
        .map(|part| part.parse().unwrap())
        .collect();
    let mut remaining = codes.as_slice();
    while let Some((&code, tail)) = remaining.split_first() {
        remaining = tail;
        match code {
            0 => *state = Cell::default(),
            4 => state.underline = true,
            24 => state.underline = false,
            39 => state.foreground = None,
            49 => state.background = None,
            38 | 48 => remaining = rgb(tail, code, state),
            other => panic!("unexpected SGR {other}"),
        }
    }
}

fn rgb<'a>(codes: &'a [u16], code: u16, state: &mut Cell) -> &'a [u16] {
    assert_eq!(codes[0], 2);
    let channels: Vec<u8> = codes[1..4]
        .iter()
        .map(|value| u8::try_from(*value).unwrap())
        .collect();
    let color = Some(Rgb(channels[0], channels[1], channels[2]));
    if code == 38 {
        state.foreground = color;
    } else {
        state.background = color;
    }
    &codes[4..]
}

pub fn without_sgr(text: &str) -> String {
    let mut result = String::new();
    let mut tail = text;
    while let Some(start) = tail.find("\x1b[") {
        result.push_str(&tail[..start]);
        let rest = &tail[start + 2..];
        let end = rest.find(|c: char| ('@'..='~').contains(&c)).unwrap();
        if !rest[..=end].ends_with('m') {
            result.push_str(&tail[start..start + 3 + end]);
        }
        tail = &rest[end + 1..];
    }
    result.push_str(tail);
    result
}
