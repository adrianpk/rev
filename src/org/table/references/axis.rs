/// Each entry is the new coordinate of one original logical row/column. New
/// coordinates have no old identity; removed targets remain explicitly absent.
pub(super) struct Axis {
    values: Vec<Option<usize>>,
}
impl Axis {
    pub(super) fn new(
        length: usize,
        order: impl Iterator<Item = Option<usize>>,
    ) -> Result<Self, String> {
        let mut values = vec![None; length];
        for (index, old) in order.enumerate() {
            if let Some(old) = old {
                let value = values
                    .get_mut(
                        old.checked_sub(1)
                            .ok_or("Invalid original table coordinate")?,
                    )
                    .ok_or("Invalid original table coordinate")?;
                if value.replace(index + 1).is_some() {
                    return Err("Duplicate original table coordinate".into());
                }
            }
        }
        Ok(Self { values })
    }
    pub(super) fn target(&self, old: usize) -> Result<usize, String> {
        old.checked_sub(1)
            .and_then(|i| self.values.get(i))
            .copied()
            .flatten()
            .ok_or_else(|| "Reference target is removed or outside the original table".into())
    }
    pub(super) fn range(&self, start: usize, end: usize) -> Result<(), String> {
        if start > end {
            return Err("Reversed reference ranges are unsupported".into());
        }
        let first = self.target(start)?;
        self.target(end)?;
        for old in start..=end {
            if self.target(old)? != first + old - start {
                return Err("Range no longer preserves all original targets in order".into());
            }
        }
        Ok(())
    }
}
