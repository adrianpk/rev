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
        let columns = command_columns(
            candidates
                .iter()
                .skip(palette.top())
                .take(usize::from(content.height()))
                .map(|command| (command.name(), palette.hint(*command))),
        )?;
        for row in 0..content.height() {
            let index = palette.top() + usize::from(row);
            let command = candidates.get(index);
            let label = command
                .map(|command| command.name())
                .unwrap_or("No matching commands");
            let hint = command.and_then(|command| palette.hint(*command));
            let source = if let Some(command) = command {
                command_row(
                    panel,
                    row,
                    Annotation {
                        label,
                        hint,
                        description: palette.description(*command),
                    },
                    columns,
                    palette.selected() == Some(index),
                )?
            } else {
                candidate_row(panel, row, label, palette.selected() == Some(index))?
            };
            self.blit(content, source)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct CommandColumns {
    shortcut: usize,
    description: usize,
}

fn command_columns<'a>(
    rows: impl Iterator<Item = (&'a str, Option<&'a str>)>,
) -> Result<CommandColumns, Error> {
    let mut name_width = 0;
    let mut shortcut_width = 0;
    for (name, hint) in rows {
        name_width = name_width.max(cells(name)?);
        if let Some(hint) = hint {
            shortcut_width = shortcut_width.max(cells(hint)? + 2);
        }
    }
    let shortcut = name_width + 2;
    Ok(CommandColumns {
        shortcut,
        description: shortcut + shortcut_width + 2,
    })
}

#[derive(Clone, Copy)]
struct Annotation<'a> {
    label: &'a str,
    hint: Option<&'a str>,
    description: Option<&'a str>,
}

fn command_row(
    panel: Panel,
    row: u16,
    annotation: Annotation<'_>,
    columns: CommandColumns,
    selected: bool,
) -> Result<Row, Error> {
    let Annotation {
        label,
        hint,
        description,
    } = annotation;
    let mut source = candidate_row(panel, row, label, selected)?;
    let width = usize::from(panel.content().width());
    let name_cells = cells(label)?;
    source.faces.push(super::style::Face {
        columns: panel.content().column()
            ..panel.content().column()
                + u16::try_from(name_cells.min(width))
                    .expect("name cells are clipped to panel width"),
        tone: Tone::CommandName,
    });
    let mut text = label.to_owned();
    let mut end = name_cells;
    if let Some(hint) = hint {
        end = columns.shortcut(&mut source, panel, &mut text, hint)?;
    }
    if let Some(description) = description.filter(|value| !value.trim().is_empty())
        && columns.description < width
        && columns.description >= end + 2
    {
        text.extend(std::iter::repeat_n(' ', columns.description - end));
        text.push_str(description);
        source.faces.push(super::style::Face {
            columns: panel.content().column()
                + u16::try_from(columns.description).expect("description starts inside panel width")
                ..panel.content().column() + panel.content().width(),
            tone: Tone::Secondary,
        });
    }
    source.text = clip::line(Glyphs::for_text(&text)?, 0, panel.content().width())?;
    Ok(source)
}

impl CommandColumns {
    fn shortcut(
        self,
        source: &mut Row,
        panel: Panel,
        text: &mut String,
        hint: &str,
    ) -> Result<usize, Error> {
        let width = usize::from(panel.content().width());
        let name_cells = cells(text)?;
        let mut end = name_cells;
        let hint_cells = cells(hint)? + 2;
        // Keep a complete adjacent hint in narrow geometry even if another
        // visible candidate makes the shared name column too wide.
        let column = if self.shortcut + hint_cells <= width {
            self.shortcut
        } else {
            name_cells + 2
        };
        if column + hint_cells <= width {
            text.extend(std::iter::repeat_n(' ', column - name_cells));
            text.push('[');
            text.push_str(hint);
            text.push(']');
            end = column + hint_cells;
            source.faces.push(super::style::Face {
                columns: panel.content().column()
                    + u16::try_from(column).expect("shortcut starts inside panel width")
                    ..panel.content().column()
                        + u16::try_from(end).expect("shortcut ends inside panel width"),
                tone: Tone::Secondary,
            });
        }
        Ok(end)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::Area;

    // Real metadata and absent metadata share one layout. Narrow output loses
    // description cells before a complete shortcut; no missing-text marker is added.
    #[test]
    fn command_annotations() {
        let columns = command_columns([("save-buffer", Some("C-x C-s"))].into_iter()).unwrap();
        for (case, width, description, expected) in [
            (
                "summary",
                80,
                Some("Write the buffer."),
                "save-buffer  [C-x C-s]  Write the buffer.",
            ),
            ("missing", 80, None, "save-buffer  [C-x C-s]"),
            ("empty", 80, Some(""), "save-buffer  [C-x C-s]"),
            (
                "clipped summary",
                27,
                Some("abcd"),
                "save-buffer  [C-x C-s]  abc",
            ),
            ("exact shortcut", 22, Some("abcd"), "save-buffer  [C-x C-s]"),
            ("no shortcut room", 21, Some("abcd"), "save-buffer"),
        ] {
            let panel = Panel::new(Area::new(7, 3, width, 1).unwrap(), false, false);
            let row = command_row(
                panel,
                0,
                Annotation {
                    label: "save-buffer",
                    hint: Some("C-x C-s"),
                    description,
                },
                columns,
                false,
            )
            .unwrap();
            assert_eq!(row.text.trim_end(), expected, "{case}");
        }
    }

    // Description clipping respects terminal cells and selection remains one
    // full row even when an annotation has a secondary foreground role.
    #[test]
    fn description_cells() {
        let columns = command_columns([("x", None)].into_iter()).unwrap();
        let panel = Panel::new(Area::new(7, 3, 8, 1).unwrap(), false, true);
        let palette = crate::config::Palette {
            syntax: crate::config::SyntaxPalette {
                function: crate::config::Rgb(1, 2, 3),
                ..Default::default()
            },
            ..Default::default()
        };
        for selected in [false, true] {
            let row = command_row(
                panel,
                0,
                Annotation {
                    label: "x",
                    hint: None,
                    description: Some("界界"),
                },
                columns,
                selected,
            )
            .unwrap();
            assert_eq!(row.text.trim_end(), "x    界");
            for column in 7..15 {
                let foreground = description_foreground(&palette, selected, column);
                assert_eq!(
                    row.colors(column, &palette),
                    super::super::style::Colors::new(
                        foreground,
                        if selected {
                            palette.selection
                        } else {
                            palette.background
                        }
                    )
                );
            }
        }
    }
    fn description_foreground(
        palette: &crate::config::Palette,
        selected: bool,
        column: usize,
    ) -> crate::config::Rgb {
        if selected {
            palette.selection_text
        } else if column == 7 {
            palette.syntax.function
        } else if column >= 12 {
            palette.status_text
        } else {
            palette.foreground
        }
    }
}
