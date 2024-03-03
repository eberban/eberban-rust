pub type Span = std::ops::Range<usize>;

#[derive(PartialEq, Eq, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "({:?})@{}..{}",
            self.inner, self.span.start, self.span.end
        )
    }
}

pub trait SpannedExt: Sized {
    fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { inner: self, span }
    }
}

impl<T> SpannedExt for T {}

#[derive(Copy, Clone, Debug)]
pub struct SpannedReader<'input, T> {
    input: &'input [T],
    /// Position of the next item to read (== to `input.len()` when all items are read).
    cursor: usize,
}

impl<'input, T> SpannedReader<'input, T> {
    pub fn new(input: &'input [T]) -> Self {
        Self { input, cursor: 0 }
    }

    pub fn next(&mut self) -> Option<&T> {
        let Some(item) = self.input.get(self.cursor) else {
            return None;
        };

        self.cursor += 1;
        Some(item)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.input.len() - self.cursor
    }

    pub fn forward(&mut self, delta: usize) {
        self.cursor += delta;
    }

    pub fn input(&self) -> &[T] {
        self.input
    }

    pub fn input_before(&self) -> &[T] {
        &self.input[..self.cursor]
    }

    pub fn input_after(&self) -> &[T] {
        &self.input[self.cursor..]
    }
}

pub trait Parse<T>: Sized {
    /// Custom error type for this parser.
    type Error;

    /// Custom state type provided to this parser. Aims to be used to report warnings.
    type State;

    /// Name for what it tries to parse.
    fn describe() -> &'static str;

    /// Tries to parse from the reader. If an error is returned the reader may have made progress.
    fn parse(reader: &mut SpannedReader<T>, state: &mut Self::State) -> Result<Self, Self::Error>;
}
