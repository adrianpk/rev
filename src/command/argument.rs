/// Distinct input forms retained for command-specific interpretation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArgumentForm {
    #[default]
    Absent,
    PlainPrefix {
        depth: u8,
    },
    Integer(i32),
    NegativeOnly,
}

/// Checked, transient command input. Construction stays with the input owner;
/// consumers can inspect both the original form and its numeric interpretation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// The flag retains the sign of explicit zero without changing its numeric value.
pub struct PrefixArgument(ArgumentForm, bool);

impl PrefixArgument {
    pub const LIMIT: i32 = 1_000_000;

    pub fn form(self) -> ArgumentForm {
        self.0
    }

    pub fn supplied(self) -> bool {
        self.0 != ArgumentForm::Absent
    }

    pub fn value(self) -> i32 {
        match self.0 {
            ArgumentForm::Absent => 1,
            ArgumentForm::PlainPrefix { depth } => 4_i32.pow(u32::from(depth)),
            ArgumentForm::Integer(value) => value,
            ArgumentForm::NegativeOnly => -1,
        }
    }

    pub(crate) fn extend(self) -> Option<Self> {
        let depth = match self.0 {
            ArgumentForm::Absent => 1,
            ArgumentForm::PlainPrefix { depth } => depth.checked_add(1)?,
            _ => return Some(self),
        };
        4_i32
            .checked_pow(u32::from(depth))
            .filter(|value| *value <= Self::LIMIT)
            .map(|_| Self(ArgumentForm::PlainPrefix { depth }, false))
    }

    pub(crate) fn digit(self, digit: u32) -> Option<Self> {
        if digit > 9 {
            return None;
        }
        let (value, negative) = match self.0 {
            ArgumentForm::Integer(value) => (value.unsigned_abs(), value < 0 || self.1),
            ArgumentForm::NegativeOnly => (0, true),
            _ => (0, false),
        };
        let value = value.checked_mul(10)?.checked_add(digit)?;
        if value > Self::LIMIT as u32 {
            return None;
        }
        let value = i32::try_from(value).expect("prefix magnitude is bounded by LIMIT");
        Some(Self(
            ArgumentForm::Integer(if negative { -value } else { value }),
            negative && value == 0,
        ))
    }

    pub(crate) fn negate(self) -> Self {
        Self(
            match self.0 {
                ArgumentForm::Integer(value) => ArgumentForm::Integer(-value),
                ArgumentForm::NegativeOnly => ArgumentForm::Integer(1),
                _ => ArgumentForm::NegativeOnly,
            },
            matches!(self.0, ArgumentForm::Integer(0)) && !self.1,
        )
    }
}
