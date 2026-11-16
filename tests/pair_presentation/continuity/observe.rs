use std::collections::BTreeMap;

use super::frames::completed;
use crate::surface::{self, Cell};
use rev::display::Frame;

pub type Cells = BTreeMap<(usize, usize), Cell>;

#[derive(Default)]
pub struct Observer {
    output: String,
    cursor: usize,
    pub frames: usize,
}

impl Observer {
    pub fn record(&mut self, frame: &Frame, check: &mut impl FnMut(&Frame, &Cells)) {
        let mut bytes = Vec::new();
        frame.write(&mut bytes).unwrap();
        self.output.push_str(std::str::from_utf8(&bytes).unwrap());
        assert!(
            self.output.len() <= 1024 * 1024,
            "bounded completed-frame stream"
        );
        let mut observed = 0;
        completed(&self.output, &mut self.cursor, &mut |update| {
            observed += 1;
            check(frame, &surface::decode(update.as_bytes().to_vec()));
        });
        assert_eq!(observed, usize::from(frame.cursor().is_some()));
        self.frames += observed;
    }
}
