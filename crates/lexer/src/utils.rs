use std::ops::Range;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Reader<'a, T> {
    input: &'a [T],
    cursor: usize,
}

impl<'a, T> Reader<'a, T> {
    pub fn new(input: &'a [T]) -> Self {
        Self { input, cursor: 0 }
    }

    pub fn peek(&self) -> Option<&T> {
        self.input.get(self.cursor)
    }

    pub fn consume(&mut self) -> Option<&T> {
        let item = self.input.get(self.cursor)?;
        self.cursor += 1;
        Some(item)
    }

    pub fn peek_many(&self, n: usize) -> Option<&[T]> {
        self.input.get(self.cursor..)?.get(..n)
    }

    pub fn consume_many(&mut self, n: usize) -> Option<&[T]> {
        let items = self.input.get(self.cursor..)?.get(..n)?;
        self.cursor += n;
        Some(items)
    }

    pub fn peek_exact(&self, exact: &[T]) -> bool
    where
        T: PartialEq,
    {
        let Some(data) = self.peek_many(exact.len()) else {
            return false;
        };

        data == exact
    }

    pub fn peek_exact_among(&self, among: &[&[T]]) -> Option<usize>
    where
        T: PartialEq,
    {
        for item in among {
            let data = match self.peek_many(item.len()) {
                None => continue,
                Some(data) => data,
            };

            if &data == item {
                return Some(item.len());
            }
        }

        None
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn wrap<T>(self, value: T) -> Spanned<T> {
        Spanned { span: self, value }
    }

    pub fn merge(self, other: Self) -> Span {
        use std::cmp::{max, min};
        Self {
            start: min(self.start, other.start),
            end: max(self.end, other.end),
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

pub type Error = Spanned<String>;

pub fn err<T>(span: impl Into<Span>, message: impl ToString) -> Result<T> {
    Err(span.into().wrap(message.to_string()))
}

pub fn ok<T>(span: impl Into<Span>, value: T) -> Result<Spanned<T>> {
    Ok(span.into().wrap(value))
}
