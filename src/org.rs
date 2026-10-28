//! Native Org observations borrow canonical buffer text. They have no file,
//! configuration, execution or edit authority of their own.

mod form;
mod source;

pub use form::{Form, FormKind};
pub use source::{Source, SourceRange};
