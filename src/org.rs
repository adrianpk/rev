//! Native Org observations borrow canonical buffer text. They have no file,
//! configuration, execution or edit authority of their own.

mod form;
mod index;
mod source;

pub use form::{Form, FormKind};
pub use index::{Coverage, Index, Limit, Limits, Line, Scan};
pub use source::{Source, SourceRange};
