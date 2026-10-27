use serde::Deserialize;

/// Number calculation and current-line form are independent view preferences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Absolute,
    Relative,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CurrentLine {
    #[default]
    Absolute,
    Zero,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    #[default]
    Left,
    Right,
}

/// Admitted startup-only field preferences. Twenty digits fit every supported
/// usize line count; the minimum never truncates a larger actual line index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    minimum_digits: u8,
    left_padding: u8,
    right_padding: u8,
}

impl Format {
    pub const MAX_DIGITS: u8 = 20;

    pub fn new(minimum_digits: u8, left_padding: u8, right_padding: u8) -> Option<Self> {
        if !(1..=Self::MAX_DIGITS).contains(&minimum_digits)
            || left_padding > 4
            || right_padding > 4
        {
            return None;
        }
        Some(Self {
            minimum_digits,
            left_padding,
            right_padding,
        })
    }

    pub fn minimum_digits(self) -> u8 {
        self.minimum_digits
    }
    pub fn left_padding(self) -> u8 {
        self.left_padding
    }
    pub fn right_padding(self) -> u8 {
        self.right_padding
    }
}

impl Default for Format {
    fn default() -> Self {
        Self {
            minimum_digits: 4,
            left_padding: 0,
            right_padding: 1,
        }
    }
}

/// Complete admitted startup values, shared by configuration and native views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub visible: bool,
    pub mode: Mode,
    pub current_line: CurrentLine,
    pub side: Side,
    pub format: Format,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            visible: true,
            mode: Mode::Absolute,
            current_line: CurrentLine::Absolute,
            side: Side::Left,
            format: Format::default(),
        }
    }
}

/// Explicit per-leaf provenance, including an override equal to startup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    pub visible: Option<bool>,
    pub mode: Option<Mode>,
    pub current_line: Option<CurrentLine>,
    pub side: Option<Side>,
}

/// Native leaf updates. Mode updates here retain visibility; command-specific
/// show/cycle behavior composes these updates in the command owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Visible(bool),
    Mode(Mode),
    CurrentLine(CurrentLine),
    Side(Side),
    Reset,
}

/// Small value owned by one editing view. Cloning a view copies both its
/// startup values and override metadata without retaining a buffer reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    startup: Settings,
    overrides: Overrides,
}

impl Policy {
    pub fn new(startup: Settings) -> Self {
        Self {
            startup,
            overrides: Overrides::default(),
        }
    }

    pub fn startup(self) -> Settings {
        self.startup
    }
    pub fn overrides(self) -> Overrides {
        self.overrides
    }
    pub fn effective(self) -> Settings {
        Settings {
            visible: self.overrides.visible.unwrap_or(self.startup.visible),
            mode: self.overrides.mode.unwrap_or(self.startup.mode),
            current_line: self
                .overrides
                .current_line
                .unwrap_or(self.startup.current_line),
            side: self.overrides.side.unwrap_or(self.startup.side),
            format: self.startup.format,
        }
    }

    /// Return whether values or explicit provenance changed, for the existing
    /// redraw owner. Reset restores this view's immutable startup values.
    pub fn apply(&mut self, change: Change) -> bool {
        let before = self.overrides;
        match change {
            Change::Visible(value) => self.overrides.visible = Some(value),
            Change::Mode(value) => self.overrides.mode = Some(value),
            Change::CurrentLine(value) => self.overrides.current_line = Some(value),
            Change::Side(value) => self.overrides.side = Some(value),
            Change::Reset => self.overrides = Overrides::default(),
        }
        self.overrides != before
    }
}
