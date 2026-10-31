use crate::coordinates::Glyphs;
use crate::input::Palette;
use crate::panel::{Composition, Panel};

use super::{Band, Error, Frame, Row, Tone, clip};

impl Frame {
    pub(crate) fn names(
        &mut self,
        names: &crate::input::Names,
        composition: Option<&Composition>,
        notice: &str,
    ) -> Result<(), Error> {
        let Some(composition) = composition else {
            return self.prompt(names.prompt(), notice);
        };
        self.prompt_in(names.prompt(), notice, composition.prompt())?;
        if let Some(panel) = composition.auxiliary() {
            let candidates = names.candidates();
            let content = panel.content();
            for row in 0..content.height() {
                let index = names.top() + usize::from(row);
                let label = candidates
                    .get(index)
                    .copied()
                    .unwrap_or(if candidates.is_empty() {
                        "No matching bookmarks"
                    } else {
                        ""
                    });
                self.blit(
                    content,
                    candidate_row(panel, row, label, names.selected() == Some(index))?,
                )?;
            }
        }
        Ok(())
    }

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

    pub(crate) fn paths(
        &mut self,
        prompt: &crate::input::Prompt,
        path: &crate::file::completion::Navigation,
        composition: Option<&Composition>,
        notice: &str,
    ) -> Result<(), Error> {
        let Some(composition) = composition else {
            return self.prompt(prompt, notice);
        };
        self.prompt_in(prompt, notice, composition.prompt())?;
        if let Some(panel) = composition.auxiliary() {
            self.path_rows(path, panel)?;
        }
        Ok(())
    }

    fn path_rows(
        &mut self,
        path: &crate::file::completion::Navigation,
        panel: Panel,
    ) -> Result<(), Error> {
        let candidates = path.candidates();
        let content = panel.content();
        let diagnostic = path.diagnostic();
        let rows = content
            .height()
            .saturating_sub(u16::from(diagnostic.is_some()));
        for row in 0..rows {
            let index = path.top() + usize::from(row);
            let label = candidates
                .get(index)
                .map_or("No matching paths", |candidate| candidate.label());
            let source = candidate_row(panel, row, label, path.selected() == Some(index))?;
            self.blit(content, source)?;
        }
        if let Some(diagnostic) = diagnostic {
            let mut source = candidate_row(panel, rows, diagnostic, false)?;
            source.tone = Tone::Message(super::Message::Warning);
            self.blit(content, source)?;
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
            let source = candidate_row(panel, row, label, palette.selected() == Some(index))?;
            self.blit(content, source)?;
        }
        Ok(())
    }
}

fn candidate_row(panel: Panel, row: u16, label: &str, selected: bool) -> Result<Row, Error> {
    let content = panel.content();
    Ok(Row {
        row,
        text: clip::line(Glyphs::for_text(label)?, 0, content.width())?,
        band: Band::Completion,
        tone: Tone::Document,
        faces: Vec::new(),
        syntax: Vec::new(),
        search: None,
        selection: selected.then_some(0..content.width()),
    }
    .projected(panel))
}
