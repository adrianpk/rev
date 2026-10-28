//! Native Org observations borrow canonical buffer text. They have no file,
//! configuration, execution or edit authority of their own.

mod color;
mod form;
mod index;
mod inline;
mod job;
mod list;
mod property;
pub(crate) use list::ListEdit;
pub(crate) use property::{PropertyValue, derived_property, property_name};
mod source;
mod structure;

pub use form::{Form, FormKind};
pub use index::{Coverage, Index, Limit, Limits, Line, Scan};
pub use inline::{InlineContext, InlineKind, InlineNode, InlineObservation, InlineScan, LinkKind};
pub use job::{Cancellation, Job, JobError};
pub use source::{Source, SourceRange};

pub use structure::{BlockKind, Structure, StructureKind, StructureNode, StructureScan};

pub use color::{ColorScan, Coloring, Highlights, Role};

mod visibility;
pub(crate) use visibility::{Insertion, Outline, SubtreeEdit, Visibility};
