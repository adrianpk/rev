use serde::Deserialize;

/// Startup-only Org heading presentation. Admission preserves source cell width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headings {
    pub pretty: bool,
    pub bullets: Vec<String>,
    pub hide_leading_stars: bool,
    pub cycle_bullets: bool,
}

impl Default for Headings {
    fn default() -> Self {
        Self {
            pretty: true,
            bullets: ["◉", "○", "✸", "✿", "◆", "▶"].map(String::from).into(),
            hide_leading_stars: true,
            cycle_bullets: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialHeadings {
    pub pretty: Option<bool>,
    pub bullets: Option<Vec<String>>,
    pub hide_leading_stars: Option<bool>,
    pub cycle_bullets: Option<bool>,
}

pub(super) fn validate(value: &PartialHeadings) -> Result<(), super::validate::Failure> {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    let Some(bullets) = &value.bullets else {
        return Ok(());
    };
    if !(1..=32).contains(&bullets.len())
        || bullets.iter().any(|marker| {
            marker.len() > 32
                || marker.chars().any(|c| c.is_control() || c.is_whitespace())
                || marker.graphemes(true).count() != 1
                || marker.width() != 1
        })
    {
        return Err(super::validate::Failure::at(
            &["org", "headings", "bullets"],
            "expected 1..32 markers, each at most 32 UTF-8 bytes and one printable non-whitespace grapheme of terminal width one",
            super::validate::Target::Value,
        ));
    }
    Ok(())
}
