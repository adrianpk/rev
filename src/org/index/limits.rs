use std::fmt;

/// A finite source observation bound, never a reason to change canonical text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    AnalysisBytes,
    SourceNodes,
    SourceDepth,
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AnalysisBytes => "Org source analysis byte limit",
            Self::SourceNodes => "Org source node limit",
            Self::SourceDepth => "Org source depth limit",
        })
    }
}

impl std::error::Error for Limit {}

/// The approved analysis profile; all values remain within finite maxima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub(in crate::org) bytes: usize,
    pub(in crate::org) nodes: usize,
    pub(in crate::org) depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 8_388_608,
            nodes: 200_000,
            depth: 256,
        }
    }
}

impl Limits {
    pub fn new(bytes: usize, nodes: usize, depth: usize) -> Result<Self, Limit> {
        admit(bytes, 33_554_432, Limit::AnalysisBytes)?;
        admit(nodes, 1_000_000, Limit::SourceNodes)?;
        admit(depth, 1024, Limit::SourceDepth)?;
        Ok(Self {
            bytes,
            nodes,
            depth,
        })
    }
}

fn admit(value: usize, maximum: usize, kind: Limit) -> Result<(), Limit> {
    if value == 0 || value > maximum {
        return Err(kind);
    }
    Ok(())
}
