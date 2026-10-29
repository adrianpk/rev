use super::*;
#[derive(Debug, Clone, Copy)]
pub(super) struct Rectangle {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize,
}
impl Grid {
    pub fn new(rows: Vec<Vec<String>>, header_rows: usize) -> Result<Self, Error> {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        let count = rows
            .len()
            .checked_mul(columns)
            .ok_or_else(|| Error::new(ErrorKind::Limit, "grid dimension overflow"))?;
        if rows.is_empty() || rows.len() > 10000 || columns == 0 || columns > 256 || count > 1000000
        {
            return Err(Error::new(
                ErrorKind::Limit,
                "grid exceeds native row/column/cell bounds",
            ));
        }
        if header_rows > rows.len() {
            return Err(Error::new(ErrorKind::Reference, "header rows exceed grid"));
        }
        let bytes = rows
            .iter()
            .flatten()
            .try_fold(0usize, |sum, text| sum.checked_add(text.len()))
            .ok_or_else(|| Error::new(ErrorKind::Limit, "grid byte overflow"))?;
        if bytes > 33554432 {
            return Err(Error::new(
                ErrorKind::Limit,
                "grid exceeds source byte ceiling",
            ));
        }
        let length = rows.len();
        let cells = rows
            .into_iter()
            .flat_map(|mut r| {
                r.resize(columns, String::new());
                r
            })
            .collect();
        Ok(Self {
            cells,
            rows: length,
            columns,
            header_rows,
        })
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn columns(&self) -> usize {
        self.columns
    }
    pub fn header_rows(&self) -> usize {
        self.header_rows
    }
    pub fn cell(&self, cell: Cell) -> Option<&str> {
        let row = cell.row.checked_sub(1)?;
        let column = cell.column.checked_sub(1)?;
        if row >= self.rows || column >= self.columns {
            return None;
        }
        Some(&self.cells[row * self.columns + column])
    }
    pub(super) fn location(&self, index: usize) -> Cell {
        Cell {
            row: index / self.columns + 1,
            column: index % self.columns + 1,
        }
    }
    pub(super) fn resolve(&self, reference: &Reference, origin: usize) -> Result<usize, Error> {
        let row = coordinate(&reference.row, origin / self.columns, self.rows)?;
        let column = coordinate(&reference.column, origin % self.columns, self.columns)?;
        Ok(row * self.columns + column)
    }
    pub(super) fn rectangle(
        &self,
        start: &Reference,
        end: &Reference,
        origin: usize,
    ) -> Result<Rectangle, Error> {
        let a = self.location(self.resolve(start, origin)?);
        let b = self.location(self.resolve(end, origin)?);
        Ok(Rectangle {
            top: a.row.min(b.row) - 1,
            bottom: a.row.max(b.row) - 1,
            left: a.column.min(b.column) - 1,
            right: a.column.max(b.column) - 1,
        })
    }
}
impl Rectangle {
    pub(super) fn cells(self, columns: usize) -> impl Iterator<Item = usize> {
        (self.top..=self.bottom)
            .flat_map(move |r| (self.left..=self.right).map(move |c| r * columns + c))
    }
}
fn coordinate(
    component: &Option<Component>,
    current: usize,
    length: usize,
) -> Result<usize, Error> {
    let coordinate = component
        .as_ref()
        .map_or(Coordinate::Current, |c| c.coordinate);
    let target = match coordinate {
        Coordinate::Current => Some(current),
        Coordinate::Absolute(value) => value.checked_sub(1),
        Coordinate::Relative(offset) => isize::try_from(offset)
            .ok()
            .and_then(|o| current.checked_add_signed(o)),
    };
    target
        .filter(|value| *value < length)
        .ok_or_else(|| Error::new(ErrorKind::Reference, "reference is outside the data grid"))
}
