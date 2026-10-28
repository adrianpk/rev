use crate::config::{OrgPalette, Rgb};

/// Only semantic groups backed by currently recognized source forms belong here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Role {
    Heading,
    Keyword,
    Drawer,
    Property,
    Planning,
    Block,
    Code,
    Comment,
    List,
    Table,
    Formula,
    Emphasis,
    Link,
    Target,
    Footnote,
    Entity,
    Malformed,
}

impl Role {
    pub const ALL: [Self; 17] = [
        Self::Heading,
        Self::Keyword,
        Self::Drawer,
        Self::Property,
        Self::Planning,
        Self::Block,
        Self::Code,
        Self::Comment,
        Self::List,
        Self::Table,
        Self::Formula,
        Self::Emphasis,
        Self::Link,
        Self::Target,
        Self::Footnote,
        Self::Entity,
        Self::Malformed,
    ];

    /// Resolve against the frame's admitted palette; source roles never carry
    /// RGB channels or read configuration files during observation.
    pub fn color(self, palette: &OrgPalette) -> Rgb {
        [
            palette.heading,
            palette.keyword,
            palette.drawer,
            palette.property,
            palette.planning,
            palette.block,
            palette.code,
            palette.comment,
            palette.list,
            palette.table,
            palette.formula,
            palette.emphasis,
            palette.link,
            palette.target,
            palette.footnote,
            palette.entity,
            palette.malformed,
        ][self as usize]
    }
}
