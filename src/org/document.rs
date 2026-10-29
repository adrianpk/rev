//! Explicit document actions observe real source snapshots and return checked
//! edits. Analysis has no filesystem, process, publication or configuration authority.
mod affiliated;
mod context;
mod footnotes;
mod markup;
mod observe;
mod radio;

use super::{
    Cancellation, Coverage, InlineContext, InlineKind, Limits, Source, StructureKind, inline,
    structure,
};
use crate::{
    buffer::{Buffer, Stamp, edit::Point},
    command::OrgCommand as C,
};
use ropey::Rope;
use std::ops::Range;

pub(crate) struct Snapshot {
    pub stamp: Stamp,
    text: Rope,
    point: usize,
    region: Option<Range<usize>>,
}
#[derive(Clone)]
pub(crate) enum Argument {
    None,
    Markup(String),
    Entity(String),
    Affiliated(String, String),
}
pub(crate) enum Output {
    Prompt,
    Move(usize),
    Continue,
    Message(String),
    Change {
        edits: Vec<Edit>,
        cursor: Option<usize>,
    },
}
pub(crate) struct Edit {
    pub bytes: Range<usize>,
    pub text: String,
}
struct Document {
    text: Rope,
    nodes: Vec<structure::Raw>,
    objects: Vec<inline::Raw>,
    protected: Vec<Range<usize>>,
}
impl Snapshot {
    pub fn new(buffer: &Buffer, point: Point) -> Result<Self, String> {
        let region = buffer.region(point).map_err(|e| e.to_string())?;
        let text = Source::new(buffer).snapshot().map_err(|e| e.to_string())?;
        if text.len_bytes() > Limits::default().bytes {
            return Err("Document action exceeds native source byte limit".into());
        }
        Ok(Self {
            stamp: buffer.stamp(),
            point: text.char_to_byte(point.position.0),
            region: region
                .map(|s| text.char_to_byte(s.range.start)..text.char_to_byte(s.range.end)),
            text,
        })
    }
    pub fn run(
        self,
        command: C,
        argument: Argument,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let doc = Document::observe(self.text, cancel)?;
        match command {
            C::CtrlCCtrlC | C::OpenLink => doc.context(self.point, command, cancel),
            C::FootnoteRenumber => doc.renumber(cancel),
            C::FootnoteAction => doc.footnote(self.point, false, cancel),
            C::FootnoteNew => doc.footnote(self.point, true, cancel),
            C::Emphasize => doc.emphasize(self.point, self.region, argument, cancel),
            C::InsertEntity => doc.entity(self.point, argument),
            C::EditAffiliated => doc.affiliated(self.point, argument),
            _ => unreachable!("qualified document command"),
        }
    }
}
impl Document {
    fn change(
        &self,
        mut edits: Vec<Edit>,
        cursor: Option<(usize, usize)>,
    ) -> Result<Output, String> {
        let cursor = cursor.map(|(at, offset)| {
            let delta: isize = edits
                .iter()
                .filter(|e| e.bytes.start < at)
                .map(|e| {
                    e.text.chars().count() as isize
                        - self.text.byte_slice(e.bytes.clone()).len_chars() as isize
                })
                .sum();
            (self.text.byte_to_char(at) as isize + delta) as usize + offset
        });
        edits.sort_by_key(|e| e.bytes.start);
        for pair in edits.windows(2) {
            if pair[0].bytes.end > pair[1].bytes.start || pair[0].bytes.start == pair[1].bytes.start
            {
                return Err("Document edits overlap; source unchanged".into());
            }
        }
        edits.retain(|e| self.text.byte_slice(e.bytes.clone()) != e.text.as_str());
        Ok(Output::Change { edits, cursor })
    }
    fn slice(&self, range: Range<usize>) -> String {
        self.text.byte_slice(range).to_string()
    }
    fn newline(&self) -> &'static str {
        if self.text.chars().any(|c| c == '\r') {
            "\r\n"
        } else {
            "\n"
        }
    }
}
fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Document action cancelled or timed out".into())
    } else {
        Ok(())
    }
}

pub(crate) use affiliated::valid_key as affiliated_key;
