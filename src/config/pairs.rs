use serde::Deserialize;

/// Serde's struct visitor otherwise admits an empty sequence as all-absent.
/// Both schema entry points require a TOML table, including an empty table.
pub(super) fn table<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<PartialPairs, D::Error> {
    deserializer.deserialize_map(Table)
}

struct Table;

impl<'de> serde::de::Visitor<'de> for Table {
    type Value = PartialPairs;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a table of automatic pair preferences")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        PartialPairs::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

/// Automatic editing policy. Language overrides inherit each omitted global leaf.
/// These preferences do not control syntax, navigation or pair presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pairs {
    pub enabled: bool,
    pub wrap_selection: bool,
    pub delete_empty: bool,
    pub skip_closing: bool,
    pub quotes: bool,
}

impl Default for Pairs {
    fn default() -> Self {
        Self {
            enabled: true,
            wrap_selection: true,
            delete_empty: true,
            skip_closing: true,
            quotes: true,
        }
    }
}

/// Source presence remains separate from compiled defaults at both scopes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPairs {
    pub enabled: Option<bool>,
    pub wrap_selection: Option<bool>,
    pub delete_empty: Option<bool>,
    pub skip_closing: Option<bool>,
    pub quotes: Option<bool>,
}

impl Pairs {
    pub(super) fn overlay(mut self, partial: &PartialPairs) -> Self {
        if let Some(value) = partial.enabled {
            self.enabled = value;
        }
        if let Some(value) = partial.wrap_selection {
            self.wrap_selection = value;
        }
        if let Some(value) = partial.delete_empty {
            self.delete_empty = value;
        }
        if let Some(value) = partial.skip_closing {
            self.skip_closing = value;
        }
        if let Some(value) = partial.quotes {
            self.quotes = value;
        }
        self
    }
}

impl super::merge::Merge {
    pub(super) fn pairs(&mut self, prefix: &[&str], base: &mut PartialPairs, later: &PartialPairs) {
        for (leaf, base, later) in [
            ("enabled", &mut base.enabled, &later.enabled),
            (
                "wrap_selection",
                &mut base.wrap_selection,
                &later.wrap_selection,
            ),
            ("delete_empty", &mut base.delete_empty, &later.delete_empty),
            ("skip_closing", &mut base.skip_closing, &later.skip_closing),
            ("quotes", &mut base.quotes, &later.quotes),
        ] {
            let mut key = prefix.to_vec();
            key.push(leaf);
            self.leaf(&key, base, later);
        }
    }
}
