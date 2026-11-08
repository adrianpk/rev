use super::editing::Selection;
use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Insertion {
    pub index: usize,
    pub replace: bool,
    pub key_only: bool,
}
pub(crate) struct Editor {
    pub target: Selection,
    pub candidate: Arc<Definition>,
    pub allowance: usize,
    pub continuing: bool,
    pub asking: bool,
    pub after: Option<Insertion>,
    pub input: Option<Insertion>,
    pub capturing: bool,
}
