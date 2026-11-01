//! Checked session locations, independent of named bookmarks and persistence.
mod registry;
use crate::{
    buffer::{Stamp, edit::Point},
    coordinates::Scalar,
    file::{Context, Location},
};
pub(crate) use registry::{Shared, Snapshot};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub(crate) struct Anchor {
    source: Arc<Mutex<Source>>,
}
impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
    }
}
impl Eq for Anchor {}

#[derive(Clone, Debug)]
pub(crate) struct Source {
    pub point: Point,
    pub position: Option<Scalar>,
    pub file: Option<Location>,
    pub context: Context,
    pub closed: bool,
}
impl Anchor {
    pub(crate) fn source(&self) -> Source {
        self.source.lock().expect("location owner poisoned").clone()
    }
    pub(crate) fn bind(&self, stamp: Stamp) {
        let mut source = self.source.lock().expect("location owner poisoned");
        source.point.stamp = stamp;
        source.closed = false;
    }
    pub(crate) fn same_point(&self, other: &Self) -> bool {
        let left = self.source();
        let right = other.source();
        left.position.is_some() && right.position.is_some() && left.point == right.point
    }
}
