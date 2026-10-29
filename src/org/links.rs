//! Native link data and checked document observations. Recognition never owns
//! file visits, source edits or external process authority.
mod document;
mod external;
mod observe;
mod resolve;
mod target;
mod visit;
use crate::org::property::Properties;
use crate::{
    buffer::{Buffer, Stamp, edit::Point},
    org::{
        Cancellation, Coverage, InlineContext, InlineKind, Limits, Source, StructureKind, structure,
    },
};
use ropey::Rope;
use std::{ops::Range, path::PathBuf};
pub use target::{FileMode, Target, decode, encode};

#[derive(Clone, Debug)]
pub(crate) struct Link {
    pub bytes: Range<usize>,
    pub target: String,
    pub description: String,
}
pub(crate) struct Document {
    text: Rope,
    links: Vec<Link>,
    anchors: Vec<Anchor>,
    headings: Vec<Heading>,
    properties: Properties,
}
struct Anchor {
    name: String,
    bytes: Range<usize>,
}
struct Heading {
    byte: usize,
    end: usize,
    title: String,
}
pub(crate) struct Snapshot {
    pub stamp: Stamp,
    pub point: usize,
    pub path: Option<PathBuf>,
    directory: PathBuf,
    text: Rope,
    policy: crate::config::Org,
}
pub(crate) struct Stored {
    pub target: String,
    pub description: String,
}
pub(crate) struct Edit {
    pub bytes: Range<usize>,
    pub target: String,
    pub description: String,
}
pub(crate) enum Output {
    Store(Stored),
    Edit(Edit),
    Follow(Follow),
    Move(usize),
    Wrap,
}
impl Snapshot {
    pub(crate) fn new(
        buffer: &Buffer,
        point: Point,
        policy: &crate::config::Org,
    ) -> Result<Self, String> {
        buffer.region(point).map_err(|e| e.to_string())?;
        let text = Source::new(buffer).snapshot().map_err(|e| e.to_string())?;
        if text.len_bytes() > Limits::default().bytes {
            return Err("Link source exceeds native byte limit".into());
        }
        Ok(Self {
            stamp: buffer.stamp(),
            point: text.char_to_byte(point.position.0),
            path: buffer.file().map(|f| f.path().to_owned()),
            directory: buffer.file().and_then(|f| f.path().parent()).map_or_else(
                || std::env::current_dir().map_err(|e| e.to_string()),
                |p| Ok(p.to_owned()),
            )?,
            text,
            policy: policy.clone(),
        })
    }
    pub(crate) fn observe(
        self,
        command: crate::command::OrgCommand,
        wrap: bool,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        use crate::command::OrgCommand as C;
        let doc = Document::observe(self.text, &self.policy, cancel)?;
        match command {
            C::StoreLink => doc
                .stored(self.point, self.path.as_deref())
                .map(Output::Store),
            C::InsertLink => Ok(Output::Edit(doc.edit(self.point))),
            C::OpenLink => {
                let link = doc.at(self.point).ok_or("No supported link at point")?;
                let target = Target::parse(&link.target)?;
                doc.follow(target, self.point, &self.directory, &self.policy)
                    .map(Output::Follow)
            }
            C::NextLink | C::PreviousLink => Ok(doc
                .motion(self.point, command == C::NextLink, wrap)
                .map_or(Output::Wrap, Output::Move)),
            _ => unreachable!("qualified link observation"),
        }
    }
}

pub(crate) use resolve::{Follow, Resolve};
pub(crate) use visit::Navigation;

pub(crate) use external::viewer_argv;
pub(crate) use external::{External, Viewer};
