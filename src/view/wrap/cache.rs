use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::Scalar;

use super::Fragment;

const LIMIT: usize = 8192;
const TEXT_BYTES: usize = 1024 * 1024;

type Key = (usize, Scalar);

#[derive(Clone, Debug, Default)]
struct Entries {
    identity: Option<(Stamp, u16, usize, bool)>,
    fragments: BTreeMap<Key, Fragment>,
    order: VecDeque<Key>,
    bytes: usize,
}

/// Candidate snapshots share disposable observations; a real split forks them.
/// Derived work is excluded from semantic View equality and retained requests.
#[derive(Clone, Default)]
pub(in crate::view) struct Cache(Arc<Mutex<Entries>>);
impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entries = self.0.lock().expect("fragment cache poisoned");
        f.debug_struct("Cache")
            .field("fragments", &entries.fragments.len())
            .field("text_bytes", &entries.bytes)
            .finish()
    }
}
impl PartialEq for Cache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for Cache {}

impl Cache {
    fn observe(&self, buffer: &Buffer, width: u16) -> MutexGuard<'_, Entries> {
        let identity = (
            buffer.stamp(),
            width,
            buffer.policy().tab_width,
            buffer.soft_wrap(),
        );
        let mut entries = self.0.lock().expect("fragment cache poisoned");
        if entries.identity != Some(identity) {
            let prefix = entries.identity.and_then(|(stamp, old_width, tab, wrap)| {
                (old_width == width && tab == identity.2 && wrap == identity.3)
                    .then(|| buffer.unchanged_prefix(stamp))
                    .flatten()
            });
            if let Some(prefix) = prefix {
                entries.fragments.retain(|_, f| f.observed_end < prefix);
                let mut order = std::mem::take(&mut entries.order);
                order.retain(|key| entries.fragments.contains_key(key));
                entries.order = order;
                entries.bytes = 0;
                for f in entries.fragments.values_mut() {
                    f.stamp = identity.0;
                }
                entries.bytes = entries.fragments.values().map(|f| f.text.len()).sum();
                entries.identity = Some(identity);
            } else {
                *entries = Entries {
                    identity: Some(identity),
                    ..Entries::default()
                };
            }
        }
        entries
    }

    pub(in crate::view) fn before(
        &self,
        buffer: &Buffer,
        width: u16,
        row: usize,
        point: Scalar,
    ) -> Option<Fragment> {
        self.observe(buffer, width)
            .fragments
            .range((row, Scalar(0))..=(row, point))
            .next_back()
            .map(|(_, f)| f.clone())
    }

    pub(in crate::view) fn exact(
        &self,
        buffer: &Buffer,
        width: u16,
        row: usize,
        point: Scalar,
    ) -> Option<Fragment> {
        self.observe(buffer, width)
            .fragments
            .get(&(row, point))
            .cloned()
    }

    pub(in crate::view) fn previous(
        &self,
        buffer: &Buffer,
        width: u16,
        current: &Fragment,
    ) -> Option<Fragment> {
        self.observe(buffer, width)
            .fragments
            .range((current.line, Scalar(0))..(current.line, current.source.start))
            .next_back()
            .map(|(_, f)| f)
            .filter(|f| {
                f.source.end == current.source.start && f.columns.end == current.columns.start
            })
            .cloned()
    }

    pub(in crate::view) fn remember(&self, buffer: &Buffer, width: u16, fragment: Fragment) {
        let mut entries = self.observe(buffer, width);
        let key = (fragment.line, fragment.source.start);
        if entries.fragments.contains_key(&key) {
            return;
        }
        entries.bytes += fragment.text.len();
        entries.order.push_back(key);
        entries.fragments.insert(key, fragment);
        while entries.fragments.len() > LIMIT || entries.bytes > TEXT_BYTES {
            let oldest = entries
                .order
                .pop_front()
                .expect("fragment retention exceeds its cap");
            let fragment = entries
                .fragments
                .remove(&oldest)
                .expect("retained fragment exists");
            entries.bytes -= fragment.text.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffers;
    use crate::view::View;

    // Eviction above the cap loses no source and a later revisit recomputes it.
    // A new split owns independent observations; candidate snapshots retain the
    // same semantic view even when shared derived entries change.
    #[test]
    fn eviction_and_clone() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&"x".repeat(LIMIT + 30)).unwrap();
        let b = buffers.get(id).unwrap();
        let view = View::new(b);
        let retained = view.clone();
        let fragments = view.wrapped_rows(b, 1, (LIMIT + 40) as u16).unwrap();
        assert_eq!(fragments.last().unwrap().source.end, Scalar(b.len_chars()));
        assert_eq!(view.wrap_cache.0.lock().unwrap().fragments.len(), LIMIT);
        assert!(view.wrap_cache.0.lock().unwrap().bytes <= TEXT_BYTES);
        assert_eq!(view, retained);
        let peer = view.clone_for_split();
        assert!(!Arc::ptr_eq(&view.wrap_cache.0, &peer.wrap_cache.0));
        peer.wrapped_rows(b, 13, 3).unwrap();
        assert_eq!(peer.wrap_cache.0.lock().unwrap().identity.unwrap().1, 13);
        assert_eq!(view.wrap_cache.0.lock().unwrap().identity.unwrap().1, 1);
        assert_eq!(
            view.wrapped_rows(b, 1, 1).unwrap()[0].source.start,
            Scalar(0)
        );
    }

    // Blank logical lines consume real projected text storage too. Enforce the
    // byte ceiling before the fragment-count ceiling and retain the final line.
    #[test]
    fn text_byte_bound() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&"\n".repeat(2200)).unwrap();
        let b = buffers.get(id).unwrap();
        let view = View::new(b);
        let rows = view.wrapped_rows(b, 512, 2201).unwrap();
        assert_eq!(rows.last().unwrap().source.end, Scalar(2200));
        let cache = view.wrap_cache.0.lock().unwrap();
        assert!(cache.fragments.len() < LIMIT);
        assert_eq!(cache.bytes, TEXT_BYTES);
        assert_eq!(cache.fragments.len(), TEXT_BYTES / 512);
    }
}
