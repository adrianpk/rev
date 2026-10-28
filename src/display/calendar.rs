use super::{Band, Error, Frame, Row, Tone, clip};
use crate::coordinates::Glyphs;
use crate::org::time::{Date, Month};
use crate::panel::{Composition, Panel};
impl Frame {
    pub(crate) fn calendar(
        &mut self,
        prompt: &crate::input::Prompt,
        selected: Date,
        composition: Option<&Composition>,
        notice: &str,
    ) -> Result<(), Error> {
        let Some(composition) = composition else {
            return self.prompt(prompt, notice);
        };
        self.prompt_in(prompt, notice, composition.prompt())?;
        if let Some(panel) = composition.auxiliary() {
            self.calendar_rows(panel, selected)?;
        }
        Ok(())
    }
    fn calendar_rows(&mut self, panel: Panel, selected: Date) -> Result<(), Error> {
        let content = panel.content();
        let shifts: &[i64] = if content.width() >= 88 {
            &[-1, 0, 1]
        } else {
            &[0]
        };
        for row in 0..content.height().min(8) {
            let text = shifts
                .iter()
                .map(|shift| month_row(selected, *shift, row))
                .collect::<Vec<_>>()
                .join("  ");
            let source = Row {
                row,
                text: clip::line(Glyphs::for_text(&text)?, 0, content.width())?,
                band: Band::Completion,
                tone: Tone::Document,
                faces: Vec::new(),
                syntax: Vec::new(),
                search: None,
                selection: None,
            }
            .projected(panel);
            self.blit(content, source)?;
        }
        Ok(())
    }
}
fn month_row(selected: Date, shift: i64, row: u16) -> String {
    let Ok(month) = Month::new(selected, shift) else {
        return " ".repeat(28);
    };
    match row {
        0 => format!("{:^28}", month.label()),
        1 => " Su  Mo  Tu  We  Th  Fr  Sa ".to_string() + " ",
        _ => (0..7)
            .map(|column| day_cell(month.cell(row - 2, column), selected))
            .collect(),
    }
}
fn day_cell(date: Option<Date>, selected: Date) -> String {
    match date {
        Some(date) if date == selected => format!("[{:2}]", date.day()),
        Some(date) => format!(" {:2} ", date.day()),
        None => "    ".into(),
    }
}
