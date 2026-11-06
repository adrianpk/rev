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
                        "No matching names"
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
            let command = candidates.get(index);
            let label = command
                .map(|command| command.name())
                .unwrap_or("No matching commands");
            let hint = command.and_then(|command| palette.hint(*command));
            let source = command_row(panel, row, label, hint, palette.selected() == Some(index))?;
            self.blit(content, source)?;
        }
        Ok(())
    }
}

fn command_row(
    panel: Panel,
    row: u16,
    label: &str,
    hint: Option<&str>,
    selected: bool,
) -> Result<Row, Error> {
    let mut source = candidate_row(panel, row, label, selected)?;
    let Some(hint) = hint else {
        return Ok(source);
    };
    let width = usize::from(panel.content().width());
    let name_cells = cells(label)?;
    let hint_cells = cells(hint)?;
    if name_cells + 2 + hint_cells > width {
        return Ok(source);
    }
    // Right alignment keeps every complete hint in the secondary column while
    // allowing each row to retain its full name and at least two blank cells.
    let column = width - hint_cells;
    let text = format!("{label}{}{hint}", " ".repeat(column - name_cells));
    source.text = clip::line(Glyphs::for_text(&text)?, 0, panel.content().width())?;
    source.faces.push(super::style::Face {
        columns: panel.content().column() + column as u16
            ..panel.content().column() + panel.content().width(),
        tone: Tone::Secondary,
    });
    Ok(source)
}

fn cells(text: &str) -> Result<usize, Error> {
    Ok(Glyphs::for_text(text)?.try_fold(0, |_, glyph| glyph.map(|glyph| glyph.columns.end))?)
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
