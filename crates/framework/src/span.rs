use std::{
    fmt::{Debug, Display},
    ops::Range,
};

/// Offset since the start of the input. Mainly used in [`Span`].
pub type Location = usize;

/// A range in the source input.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}..{}", self.start, self.end)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Span {
    /// Expands the span to cover `more` before.
    pub fn expand_before(mut self, more: Location) -> Self {
        self.start -= more;
        self
    }

    /// Expands the span to cover `more` after.
    pub fn expand_after(mut self, more: Location) -> Self {
        self.end += more;
        self
    }

    /// Generates the span of length `len` that precedes this span.
    pub fn preceding(self, len: Location) -> Self {
        Self {
            start: self.start - len,
            end: self.start,
        }
    }

    /// Generates the span of length `len` that follows this span.
    pub fn following(self, len: Location) -> Self {
        Self {
            start: self.end,
            end: self.end + len,
        }
    }

    /// Is the span empty?
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl From<Range<Location>> for Span {
    fn from(value: Range<Location>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Span> for Range<Location> {
    fn from(value: Span) -> Self {
        Range {
            start: value.start,
            end: value.end,
        }
    }
}
