use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Scalar, Work};

use super::{Continuation, Fragment};

const LIMIT: usize = 8192;
const TEXT_BYTES: usize = 1024 * 1024;

type Key = (usize, Scalar);

#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    stamp: Stamp,
    width: u16,
    tab: usize,
    wrap: bool,
    indent: bool,
    outline: Option<u64>,
}

#[derive(Default)]
struct Entries {
    work: Option<Work>,
    identity: Option<Identity>,
    fragments: BTreeMap<Key, Fragment>,
    order: VecDeque<Key>,
    bytes: usize,
    building: Option<Continuation>,
    scrolling: Option<crate::view::visual::scroll::Work>,
}

/// Candidate snapshots share disposable observations; a real split forks them.
/// Derived work is excluded from semantic View equality and retained requests.
#[derive(Clone, Default)]
pub(in crate::view) struct Cache(Arc<Mutex<Entries>>, bool);
pub(crate) struct Scope(Arc<Mutex<Entries>>, bool);
impl Drop for Scope {
    fn drop(&mut self) {
        if self.1 {
            self.0.lock().expect("fragment cache poisoned").work = None;
        }
    }
}
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
    #[cfg(test)]
    pub(in crate::view) fn retained(&self, buffer: &Buffer, width: u16) -> (usize, usize) {
        let entries = self.observe(buffer, width);
        (entries.fragments.len(), entries.bytes)
    }
    pub(in crate::view) fn headings(compact: bool) -> Self {
        Self(Arc::default(), compact)
    }

    pub(in crate::view) fn scope(&self, work: Work) -> Scope {
        let mut entries = self.0.lock().expect("fragment cache poisoned");
        let owner = entries.work.is_none();
        if owner {
            entries.work = Some(work);
        }
        drop(entries);
        Scope(std::sync::Arc::clone(&self.0), owner)
    }

    pub(in crate::view) fn work(&self) -> Option<Work> {
        self.0.lock().expect("fragment cache poisoned").work.clone()
    }

    pub(in crate::view) fn remember_point_columns(
        &self,
        buffer: &Buffer,
        row: usize,
        point: Scalar,
    ) -> Result<(), crate::view::Error> {
        let width = self
            .0
            .lock()
            .expect("fragment cache poisoned")
            .identity
            .map(|id| id.width);
        if let Some(width) = width
            && let Some(fragment) = self.before(buffer, width, row, point)
        {
            fragment.remember_columns(buffer)?;
        }
        Ok(())
    }

    pub(in crate::view) fn cancel_work(&self) {
        let mut entries = self.0.lock().expect("fragment cache poisoned");
        entries.building = None;
        entries.scrolling = None;
    }

    fn observe(&self, buffer: &Buffer, width: u16) -> MutexGuard<'_, Entries> {
        let identity = Identity {
            stamp: buffer.stamp(),
            width,
            tab: buffer.policy().tab_width,
            wrap: buffer.soft_wrap(),
            indent: buffer.org_indentation(),
            outline: (buffer.org_indentation() || self.1)
                .then(|| buffer.org_geometry())
                .flatten(),
        };
        let mut entries = self.0.lock().expect("fragment cache poisoned");
        if entries.identity != Some(identity) {
            let prefix = entries.identity.and_then(|old| {
                (Identity {
                    stamp: identity.stamp,
                    ..old
                } == identity)
                    .then(|| buffer.unchanged_prefix(old.stamp))
                    .flatten()
            });
            if let Some(prefix) = prefix {
                // A segmentation continuation retains the old complete line's
                // byte length/context. Resume it only at the exact revision.
                entries.building = None;
                entries.scrolling = None;
                entries.fragments.retain(|_, f| f.observed_end < prefix);
                let mut order = std::mem::take(&mut entries.order);
                order.retain(|key| entries.fragments.contains_key(key));
                entries.order = order;
                entries.bytes = 0;
                for f in entries.fragments.values_mut() {
                    f.stamp = identity.stamp;
                }
                entries.bytes = entries.fragments.values().map(|f| f.text.len()).sum();
                entries.identity = Some(identity);
            } else {
                let work = entries.work.take();
                *entries = Entries {
                    work,
                    identity: Some(identity),
                    ..Entries::default()
                };
            }
        }
        entries
    }

    pub(in crate::view) fn take_scroll(
        &self,
        buffer: &Buffer,
        width: u16,
    ) -> Option<crate::view::visual::scroll::Work> {
        self.observe(buffer, width).scrolling.take()
    }

    pub(in crate::view) fn suspend_scroll(
        &self,
        buffer: &Buffer,
        width: u16,
        work: crate::view::visual::scroll::Work,
    ) {
        self.observe(buffer, width).scrolling = Some(work);
    }

    pub(in crate::view) fn take_work(
        &self,
        buffer: &Buffer,
        width: u16,
        row: usize,
        point: Scalar,
    ) -> Option<Continuation> {
        self.observe(buffer, width)
            .building
            .take()
            .filter(|at| at.line == row && at.end <= point)
    }

    pub(in crate::view) fn suspend(&self, buffer: &Buffer, width: u16, at: Continuation) {
        self.observe(buffer, width).building = Some(at);
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

    pub(in crate::view) fn remember(
        &self,
        buffer: &Buffer,
        width: u16,
        fragment: Fragment,
    ) -> Result<(), crate::view::Error> {
        fragment.remember_columns(buffer)?;
        let mut entries = self.observe(buffer, width);
        let key = (fragment.line, fragment.source.start);
        if entries.fragments.contains_key(&key) {
            return Ok(());
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
        drop(entries);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffers;
    use crate::view::View;

    // Both format owners preserve unaffected prefix entries when a carry is
    // replaced by equal exact evidence. Structural edits still retire old hits.
    #[test]
    fn continuity_replacement() {
        use crate::buffer::{Mode, edit::Replacement};
        for mode in [Mode::Org, Mode::Markdown] {
            let mut buffers = Buffers::new().unwrap();
            let marker = if mode == Mode::Org { "**" } else { "##" };
            let source = format!("{marker} Root\nprefix body\n\n{marker} Child\nchild body\n");
            let id = buffers.create(&source).unwrap();
            let buffer = buffers.get_mut(id).unwrap();
            buffer.select_mode(mode).unwrap();
            complete(buffer);
            let mut view = View::new(buffer);
            let old = view.wrapped_rows(buffer, 30, 20).unwrap()[0].clone();
            let at = source.find("child body").unwrap() + 5;
            let replacement = Replacement {
                source: buffer.span(at..at).unwrap(),
                text: "界".into(),
            };
            let mut points = [view.point];
            let prepared = buffer.prepare(&[replacement], &points, None).unwrap();
            buffer.publish(prepared, &mut points).unwrap();
            view.point = points[0];
            assert!(old.hit(buffer, 30, 0).is_err());
            let prefix = view.wrap_cache.retained(buffer, 30);
            assert!(prefix.0 > 0, "{mode:?}");
            let carried = view.wrapped_rows(buffer, 30, 20).unwrap();
            let populated = view.wrap_cache.retained(buffer, 30);
            complete(buffer);
            assert_eq!(view.wrap_cache.retained(buffer, 30), populated, "{mode:?}");
            assert_eq!(view.wrapped_rows(buffer, 30, 20).unwrap(), carried);
            let replacement = Replacement {
                source: buffer.span(0..1).unwrap(),
                text: String::new(),
            };
            let prepared = buffer.prepare(&[replacement], &points, None).unwrap();
            buffer.publish(prepared, &mut points).unwrap();
            view.point = points[0];
            assert!(carried[0].hit(buffer, 30, 0).is_err());
            complete(buffer);
            assert_eq!(view.wrap_cache.retained(buffer, 30).0, 0);
            let changed = view.wrapped_rows(buffer, 30, 20).unwrap();
            assert_ne!(changed[0].text, carried[0].text);
            println!(
                "Cache continuity {mode:?}: unchanged_prefix={}, equal_exact_retained={}",
                prefix.0, populated.0
            );
        }
    }

    fn complete(buffer: &mut crate::buffer::Buffer) {
        if buffer.is_org() {
            let coloring = crate::org::ColorScan::new(buffer, crate::org::Limits::default())
                .unwrap()
                .run(&crate::org::Cancellation::default());
            buffer.install_org_coloring(coloring).unwrap();
        } else {
            buffer.request_markdown();
            let work = buffer.markdown_work().unwrap();
            buffer.install_markdown(work.run(&crate::process::Cancellation::default()).unwrap());
        }
    }

    // A live carried outline exercises both actual eviction paths. Repeated
    // edits/reflows keep fixed cache caps and account for virtual padding bytes.
    #[test]
    fn continuity_retention() {
        use crate::buffer::{Mode, edit::Replacement};
        let mut buffers = Buffers::new().unwrap();
        let source = format!("** H\n{}", "ordinary body\n".repeat(9000));
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.select_mode(Mode::Org).unwrap();
        complete(buffer);
        let mut view = View::new(buffer);
        let mut peaks = (0, 0);
        for step in 0..4 {
            let at = source.find("ordinary").unwrap() + 4 + step;
            let replacement = Replacement {
                source: buffer.span(at..at).unwrap(),
                text: "x".into(),
            };
            let mut points = [view.point];
            let prepared = buffer.prepare(&[replacement], &points, None).unwrap();
            buffer.publish(prepared, &mut points).unwrap();
            view.point = points[0];
            assert!(buffer.org_outline().is_none());
            assert_eq!(buffer.org_heading_level(0), Some(2));
            for width in [1, 512] {
                view.wrapped_rows(buffer, width, 14000).unwrap();
                let retained = view.wrap_cache.retained(buffer, width);
                assert!(retained.0 <= LIMIT && retained.1 <= TEXT_BYTES);
                peaks.0 = peaks.0.max(retained.0);
                peaks.1 = peaks.1.max(retained.1);
            }
        }
        assert_eq!(peaks, (LIMIT, TEXT_BYTES));
        println!(
            "Carried cache: edits=4, widths=1/512, peak_fragments={}, peak_text_bytes={}",
            peaks.0, peaks.1
        );
    }

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
        let fragments = view
            .wrapped_rows(b, 1, u16::try_from(LIMIT + 40).unwrap())
            .unwrap();
        assert_eq!(fragments.last().unwrap().source.end, Scalar(b.len_chars()));
        assert_eq!(view.wrap_cache.0.lock().unwrap().fragments.len(), LIMIT);
        assert!(view.wrap_cache.0.lock().unwrap().bytes <= TEXT_BYTES);
        assert_eq!(view, retained);
        let peer = view.clone_for_split();
        assert!(!Arc::ptr_eq(&view.wrap_cache.0, &peer.wrap_cache.0));
        peer.wrapped_rows(b, 13, 3).unwrap();
        assert_eq!(
            peer.wrap_cache.0.lock().unwrap().identity.unwrap().width,
            13
        );
        assert_eq!(view.wrap_cache.0.lock().unwrap().identity.unwrap().width, 1);
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

    // Virtual padding is charged as cached text. Repeated reflow and internal
    // runtime policy changes keep both existing ceilings and rebuild the source end.
    #[test]
    fn indented_retention() {
        let mut buffers = Buffers::new().unwrap();
        let source = format!("** H\n{}{}", "x".repeat(LIMIT * 2 + 100), "\n".repeat(2200));
        let id = buffers.create(&source).unwrap();
        let b = buffers.get_mut(id).unwrap();
        b.select_mode(crate::buffer::Mode::Org).unwrap();
        let coloring = crate::org::ColorScan::new(b, crate::org::Limits::default())
            .unwrap()
            .run(&crate::org::Cancellation::default());
        b.install_org_coloring(coloring).unwrap();
        let view = View::new(b);
        let mut peaks = (0, 0);
        for enabled in [false, true].into_iter().cycle().take(8) {
            b.set_org_indentation(enabled);
            for width in [6, 512] {
                let rows = view.wrapped_rows(b, width, 14000).unwrap();
                assert_eq!(rows.last().unwrap().source.end, Scalar(b.len_chars()));
                let cache = view.wrap_cache.0.lock().unwrap();
                assert!(cache.fragments.len() <= LIMIT);
                assert!(cache.bytes <= TEXT_BYTES);
                peaks.0 = peaks.0.max(cache.fragments.len());
                peaks.1 = peaks.1.max(cache.bytes);
                assert_eq!(
                    cache.bytes,
                    cache
                        .fragments
                        .values()
                        .map(|f| f.text.len())
                        .sum::<usize>()
                );
                if enabled && width == 6 {
                    assert_eq!(cache.fragments.len(), LIMIT);
                }
                if width == 512 {
                    assert_eq!(cache.bytes, TEXT_BYTES);
                }
            }
        }
        println!(
            "MEASURE runtime indentation retention: 8 policy changes, widths 6/512, peak_fragments={} limit={LIMIT}, peak_text_bytes={} limit={TEXT_BYTES}",
            peaks.0, peaks.1
        );
    }
}
