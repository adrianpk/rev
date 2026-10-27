use crate::coordinates::Glyphs;
use crate::input::Palette;
use crate::panel::{Composition, Panel};

use super::{Band, Error, Frame, Row, Tone, clip};

impl Frame {
    pub(crate) fn commands(
        &mut self,
        palette: &Palette,
        composition: Option<&Composition>,
        notice: &str,
    ) -> Result<(), Error> {
        let Some(composition) = composition else {
            return self.prompt(palette.prompt(), notice);
        };
        self.prompt_in(palette.prompt(), notice, composition.prompt())?;
        if let Some(panel) = composition.auxiliary() {
            self.command_rows(palette, panel)?;
        }
        Ok(())
    }

    fn command_rows(&mut self, palette: &Palette, panel: Panel) -> Result<(), Error> {
        let candidates = palette.candidates();
        let content = panel.content();
        for row in 0..content.height() {
            let index = palette.top() + usize::from(row);
            let label = candidates
                .get(index)
                .map(|command| command.name())
                .unwrap_or("No matching commands");
            let source = Row {
                row,
                text: clip::line(Glyphs::for_text(label)?, 0, content.width())?,
                band: Band::Completion,
                tone: Tone::Document,
                faces: Vec::new(),
                syntax: Vec::new(),
                search: None,
                selection: (palette.selected() == Some(index)).then_some(0..content.width()),
            }
            .projected(panel);
            self.blit(content, source)?;
        }
        Ok(())
    }
}
