use crate::{support::Result, surface};
use rev::config::{Rgb, Snapshot};
use rev::display::{Frame, Size};
use rev::session::Session;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

pub(super) const SIZE: Size = Size {
    width: 120,
    height: 30,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Org,
    Markdown,
}

impl Format {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Markdown => "md",
        }
    }
    pub(super) fn marker(self) -> char {
        match self {
            Self::Org => '*',
            Self::Markdown => '#',
        }
    }
    pub(super) fn document(self, bytes: usize, headings: usize) -> String {
        let marker = self.marker();
        let mut source = format!(
            "{marker} Root\nRoot body.\n{marker}{marker} Child\nChild body.\n{marker}{marker}{marker} Deep\nDeep body.\n"
        );
        if self == Self::Markdown {
            source.push_str("\n- Neighbor\n  - Nested\n\n");
        }
        for index in 3..headings {
            source.push_str(&format!("{marker} Extra {index}\n"));
        }
        while source.len() < bytes {
            source.push_str("ordinary body words remain source text\n");
        }
        source
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Colors {
    Enabled,
    Disabled,
    Monochrome,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub(super) format: Format,
    pub(super) profile: &'static str,
    pub(super) indent: bool,
    pub(super) pretty: bool,
    pub(super) colors: Colors,
}

impl Case {
    pub(super) fn config(self) -> String {
        let format = if self.format == Format::Org {
            "org.headings"
        } else {
            "markdown"
        };
        let palette = if self.format == Format::Org {
            "org"
        } else {
            "markdown"
        };
        format!(
            "editor.interaction_profile='{}'\neditor.soft_wrap=true\ntree.visible=false\ngutter.visible=false\nbookmarks.markers=false\ncompletion.automatic=false\norg.indent={}\n{format}.pretty={}\nsyntax.enabled={}\npalette.{palette}.heading='#123456'\npalette.{palette}.heading_2='#234567'\npalette.{palette}.heading_3='#345678'\n",
            self.profile,
            self.indent,
            self.pretty,
            !matches!(self.colors, Colors::Disabled)
        )
    }
    pub(super) fn settings(self, root: &Path) -> Result<Snapshot> {
        let path = root.join("continuity.toml");
        std::fs::write(&path, self.config())?;
        let startup = rev::config::prepare(&path);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        Ok(startup.snapshot)
    }
    pub(super) fn open(self, root: &Path, source: &str) -> Result<(Session, Snapshot)> {
        let settings = self.settings(root)?;
        let path = root.join(format!("workspace/continuity.{}", self.format.name()));
        std::fs::write(&path, source)?;
        let mut buffers = rev::buffer::Buffers::with_settings(settings.clone())?;
        let id = buffers.visit(&path)?;
        seed(&mut buffers, id, self.format)?;
        let view = rev::view::View::with_settings(buffers.get(id)?, &settings);
        let mut editor = Session::new(buffers, view, SIZE)?;
        crate::support::settle(&mut editor)?;
        Ok((editor, settings))
    }
    pub(super) fn owns_work(self) -> bool {
        self.format == Format::Markdown || self.pretty || !matches!(self.colors, Colors::Disabled)
    }

    pub(super) fn foreground(self, settings: &Snapshot, level: usize) -> Option<Rgb> {
        let palette = &settings.settings().palette;
        match self.colors {
            Colors::Monochrome => None,
            Colors::Disabled => Some(palette.foreground),
            Colors::Enabled => {
                let colors = if self.format == Format::Org {
                    [
                        palette.org.heading,
                        palette.org.heading_2,
                        palette.org.heading_3,
                    ]
                } else {
                    [
                        palette.markdown.heading,
                        palette.markdown.heading_2,
                        palette.markdown.heading_3,
                    ]
                };
                Some(colors[level - 1])
            }
        }
    }
}

pub(super) fn cases(format: Format) -> Vec<Case> {
    let policies: &[(bool, bool)] = if format == Format::Org {
        &[(false, false), (false, true), (true, false), (true, true)]
    } else {
        &[(false, false), (false, true)]
    };
    let mut cases = Vec::new();
    for profile in ["non-modal", "modal"] {
        for &(indent, pretty) in policies {
            for colors in [Colors::Enabled, Colors::Disabled, Colors::Monochrome] {
                cases.push(Case {
                    format,
                    profile,
                    indent,
                    pretty,
                    colors,
                });
            }
        }
    }
    cases
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Sample {
    row: usize,
    marker: usize,
    title: usize,
    glyph: char,
    foreground: Option<Rgb>,
}

pub(super) fn sample(frame: &str, case: Case, settings: &Snapshot) -> Vec<Sample> {
    if matches!(case.colors, Colors::Monochrome) {
        monochrome(frame);
    }
    let cells = surface::decode(frame.as_bytes().to_vec());
    let rows: Vec<_> = (1..=9)
        .map(|row| crate::terminal::row(frame, row))
        .collect();
    samples(&rows, &cells, case, settings)
}

fn samples(
    rows: &[String],
    cells: &std::collections::BTreeMap<(usize, usize), surface::Cell>,
    case: Case,
    settings: &Snapshot,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    for (row, word, level) in [(0, "Root", 1), (2, "Child", 2), (4, "Deep", 3)] {
        let text = &rows[row];
        let byte = text.find(word).expect("visible heading source identity");
        let title = text[..byte].width();
        let marker = title - 2;
        let cell = cells[&(row, marker)];
        let expected = if case.pretty {
            if case.format == Format::Org {
                settings.settings().org.headings.bullets[level - 1]
                    .chars()
                    .next()
                    .unwrap()
            } else {
                crate::support::MARKERS[level - 1]
            }
        } else {
            case.format.marker()
        };
        assert_eq!(cell.character, expected, "{case:?}/{word}");
        let foreground = case.foreground(settings, level);
        assert_eq!(cell.foreground, foreground, "marker {case:?}/{word}");
        assert_eq!(
            cells[&(row, title)].foreground,
            foreground,
            "title {case:?}/{word}"
        );
        samples.push(Sample {
            row,
            marker,
            title,
            glyph: cell.character,
            foreground,
        });
    }
    if case.format == Format::Markdown {
        samples.extend(lists(rows, cells, case, settings));
    }
    samples
}

pub(super) fn emitted(editor: &Session, case: Case, settings: &Snapshot) -> Result<Vec<Sample>> {
    let frame = Frame::project(editor.buffer()?, editor.view(), SIZE, "")?
        .with_palette(&settings.settings().palette);
    let cells = if matches!(case.colors, Colors::Monochrome) {
        crossterm::style::force_color_output(false);
        let mut output = Vec::new();
        frame.write(&mut output)?;
        monochrome(std::str::from_utf8(&output)?);
        surface::decode(output)
    } else {
        surface::draw(&frame)
    };
    let rows: Vec<_> = frame
        .rows()
        .iter()
        .take(9)
        .map(|row| row.text().to_owned())
        .collect();
    Ok(samples(&rows, &cells, case, settings))
}

// Item text is lexical and can retire; carry promises the neighboring marker.
fn lists(
    rows: &[String],
    cells: &std::collections::BTreeMap<(usize, usize), surface::Cell>,
    case: Case,
    settings: &Snapshot,
) -> Vec<Sample> {
    let palette = &settings.settings().palette;
    let plain = if matches!(case.colors, Colors::Monochrome) {
        None
    } else {
        Some(palette.foreground)
    };
    let foreground = if matches!(case.colors, Colors::Enabled) {
        Some(palette.markdown.list)
    } else {
        plain
    };
    let mut samples = Vec::new();
    for (row, word, glyph) in [(7, "Neighbor", '•'), (8, "Nested", '◦')] {
        let byte = rows[row]
            .find(word)
            .expect("visible neighboring list source");
        let title = rows[row][..byte].width();
        let marker = title - 2;
        let cell = cells[&(row, marker)];
        assert_eq!(cell.character, if case.pretty { glyph } else { '-' });
        assert_eq!(cell.foreground, foreground);
        samples.push(Sample {
            row,
            marker,
            title,
            glyph: cell.character,
            foreground,
        });
    }
    samples
}

// Org may legitimately request no worker when no color or pretty-heading observation is
// requested. Give every case the required complete base before testing carry.
fn seed(
    buffers: &mut rev::buffer::Buffers,
    id: rev::buffer::BufferId,
    format: Format,
) -> Result<()> {
    if format == Format::Org {
        let buffer = buffers.get_mut(id)?;
        let coloring = rev::org::ColorScan::new(buffer, rev::org::Limits::default())?
            .run(&rev::org::Cancellation::default());
        assert_eq!(
            coloring.highlights().coverage(),
            rev::org::Coverage::Complete
        );
        buffer.install_org_coloring(coloring)?;
    }
    Ok(())
}

// A separate default pretty projection proves actual admission even when the
// selected case intentionally shows raw markers with no syntax colors.
pub(super) fn admitted(editor: &Session) -> Result<()> {
    let buffer = editor.buffer()?;
    let view = rev::view::View::new(buffer);
    let frame = Frame::project(buffer, &view, SIZE, "")?;
    assert!(
        frame.rows()[0].text().contains('◉'),
        "mandatory positive carry was not admitted"
    );
    Ok(())
}

fn monochrome(output: &str) {
    assert!(
        !output.contains("[38;2;") && !output.contains("[48;2;"),
        "NO_COLOR must suppress RGB over the whole emitted update"
    );
}
