use super::geometry::Allocation;
use super::{Area, Layout, Split, ViewId, Windows};
use std::collections::{BTreeMap, BTreeSet};

type Fork = (ViewId, ViewId);

/// Explicit per-division geometry overrides; startup policy remains immutable.
#[derive(Clone)]
pub(super) struct Ratios {
    pub default: f64,
    balanced: BTreeMap<Fork, Weights>,
}

impl Ratios {
    pub(super) fn new(default: f64) -> Self {
        Self {
            default,
            balanced: BTreeMap::new(),
        }
    }
    pub(super) fn get(&self, layout: &Layout, area: Area) -> Allocation {
        let weights = layout.fork().and_then(|key| self.balanced.get(&key));
        if let (Some(weights), Layout::Split { direction, .. }) = (weights, layout) {
            let extent = match direction {
                Split::Below => area.height(),
                Split::Right => area.width(),
            };
            return Allocation::Cells(weights.first_cells(extent));
        }
        Allocation::Ratio(self.default)
    }
    pub(super) fn retain(&mut self, layout: &Layout) {
        let mut live = BTreeSet::new();
        layout.forks(&mut live);
        self.balanced.retain(|key, _| live.contains(key));
    }
}

impl Windows {
    /// Equalize existing divisions by their pane units on the split axis.
    /// New splits still use the admitted startup ratio; no settings are written.
    pub fn balance(&mut self) -> bool {
        let mut balanced = BTreeMap::new();
        self.layout.balanced_ratios(&mut balanced);
        let changed = self.ratios.balanced != balanced;
        self.ratios.balanced = balanced;
        changed
    }
}

impl Layout {
    fn fork(&self) -> Option<Fork> {
        let Self::Split { first, second, .. } = self else {
            return None;
        };
        Some((first.first_leaf(), second.first_leaf()))
    }
    fn forks(&self, output: &mut BTreeSet<Fork>) {
        if let Self::Split { first, second, .. } = self {
            output.insert(self.fork().expect("split key"));
            first.forks(output);
            second.forks(output);
        }
    }
    fn balanced_ratios(&self, output: &mut BTreeMap<Fork, Weights>) {
        if let Self::Split {
            direction,
            first,
            second,
        } = self
        {
            let a = first.units(*direction);
            let b = second.units(*direction);
            output.insert(
                self.fork().expect("split key"),
                Weights {
                    first: a,
                    second: b,
                },
            );
            first.balanced_ratios(output);
            second.balanced_ratios(output);
        }
    }
    fn units(&self, axis: Split) -> f64 {
        match self {
            Self::Leaf(_) => 1.0,
            Self::Split {
                direction,
                first,
                second,
            } => {
                let a = first.units(axis);
                let b = second.units(axis);
                if *direction == axis { a + b } else { a.max(b) }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Weights {
    first: f64,
    second: f64,
}

impl Weights {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Finite positive pane weights allocate whole terminal cells by flooring; separator counts are bounded by the live layout."
    )]
    fn first_cells(self, extent: u16) -> u16 {
        let available = f64::from(extent.saturating_sub(1));
        // Reserve all internal separators before sharing actual leaf cells.
        let gaps = self.first + self.second - 2.0;
        let leaves = (available - gaps).max(0.0);
        (leaves * self.first / (self.first + self.second)).floor() as u16
            + (self.first - 1.0) as u16
    }
}
