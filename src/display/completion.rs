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
        let widths = palette.annotation_widths();
        let columns = CommandColumns::new(widths.name, widths.shortcut, content.width());
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
    name: u16,
    shortcut: u16,
    description: u16,
}

impl CommandColumns {
    fn new(name: usize, shortcut: usize, width: u16) -> Self {
        let width = usize::from(width);
        let fields = width.saturating_sub(4);
        let (name, shortcut) = if name + shortcut <= fields {
            (name, shortcut)
        } else {
            // Compressed panels retain room for all three fields. The catalog
            // ratio determines annotation widths, never the visible candidates.
            let annotations = fields - fields / 4;
            let name = annotations * name / (name + shortcut).max(1);
            (name, annotations - name)
        };
        Self {
            name: u16::try_from(if width <= 4 { width } else { name })
                .expect("name field is bounded by panel width"),
            shortcut: u16::try_from(shortcut).expect("shortcut field is bounded by panel width"),
            description: u16::try_from(fields - name - shortcut)
                .expect("description field is bounded by panel width"),
        }
    }
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
    let mut source = candidate_row(panel, row, "", selected)?;
    source.text.clear();
    let hint = hint.map(|hint| format!("[{hint}]"));
    let fields = [
        (label, columns.name, Tone::CommandName),
        (
            hint.as_deref().unwrap_or(""),
            columns.shortcut,
            Tone::Secondary,
        ),
        (
            description.unwrap_or(""),
            columns.description,
            Tone::Secondary,
        ),
    ];
    for (index, (value, width, tone)) in fields.into_iter().enumerate() {
        if index > 0 {
            source.text.push_str("  ");
        }
        command_field(&mut source, panel, value, width, tone)?;
    }
    source.text = clip::line(Glyphs::for_text(&source.text)?, 0, panel.content().width())?;
    Ok(source)
}

fn command_field(
    source: &mut Row,
    panel: Panel,
    value: &str,
    width: u16,
    tone: Tone,
) -> Result<(), Error> {
    let start = cells(&source.text)?;
    source
        .text
        .push_str(&clip::line(Glyphs::for_text(value)?, 0, width)?);
    if !value.is_empty() && width > 0 {
        let column = usize::from(panel.content().column()) + start;
        source.faces.push(super::style::Face {
            columns: u16::try_from(column).expect("field starts inside panel width")
                ..u16::try_from(column + usize::from(width))
                    .expect("field ends inside panel width"),
            tone,
        });
    }
    Ok(())
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

    // Optional fields keep their columns, and compressed geometry clips each
    // field independently instead of moving a hint next to a shorter name.
    #[test]
    fn command_annotations() {
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
            ("compressed", 22, Some("abcd"), "save-bu  [C-x C-  abcd"),
            ("narrower", 21, Some("abcd"), "save-bu  [C-x C  abcd"),
        ] {
            let columns = CommandColumns::new(11, 9, width);
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
        let columns = CommandColumns::new(1, 0, 8);
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

    // Field boundaries never split a wide or combining cluster. Clipping one
    // field cannot push the following field to a different column.
    #[test]
    fn clipped_clusters() {
        for (width, expected) in [(12, "界  [M-   界"), (16, "界e\u{301}  [M-界]  界e\u{301}")]
        {
            let panel = Panel::new(Area::new(0, 0, width, 1).unwrap(), false, false);
            let row = command_row(
                panel,
                0,
                Annotation {
                    label: "界e\u{301}",
                    hint: Some("M-界"),
                    description: Some("界e\u{301}"),
                },
                CommandColumns::new(3, 6, width),
                false,
            )
            .unwrap();
            assert_eq!(row.text.trim_end(), expected, "width {width}");
        }
    }
}
