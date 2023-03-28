fn main() {
    println!("Hello, world!");
}

pub fn ok<T>(start: usize, end: usize, value: T) -> Res<T> {
    Ok(Span::new(start, end).wrap(value))
}

pub fn err<T>(start: usize, end: usize, err: impl ToString) -> Res<T> {
    Err(Span::new(start, end).wrap(err.to_string()))
}

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

type Res<T> = Result<Spanned<T>, Spanned<String>>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Symbol {
    Letter(u8),
    Pause,
    Hyphen,
    Space,
}

fn is_letter(c: u8) -> bool {
    b"nrlmpbfvtdszcjgkieaouh".contains(&c)
}

fn peek_hyphen(r: &mut Reader<u8>) -> Option<usize> {
    r.peek_exact_among(&[
        "-".as_bytes(), // hyphen
        "–".as_bytes(), // en dash
        "—".as_bytes(), // em dash
        "−".as_bytes(), // minus sign
    ])
}

fn peek_pause(r: &mut Reader<u8>) -> Option<usize> {
    r.peek_exact_among(&["'".as_bytes(), "’".as_bytes(), "`".as_bytes()])
}

pub fn parse_symbol(r: &mut Reader<u8>) -> Res<Symbol> {
    let start = r.cursor();

    // We start with multiple bytes characters like hyphens and pause.
    if let Some(len) = peek_hyphen(r) {
        let _ = r.consume_many(len);

        // 2 hyphens are not allowed in a row.
        if let Some(len) = peek_hyphen(r) {
            return err(start, r.cursor() + len, "2 hyphens cannot appear in a row");
        }

        return Ok(Span::new(start, r.cursor()).wrap(Symbol::Hyphen));
    }

    if let Some(len) = peek_pause(r) {
        let _ = r.consume_many(len);

        // 2 pauses are not allowed in a row.
        if let Some(len) = peek_pause(r) {
            return err(start, r.cursor() + len, "2 pauses cannot appear in a row");
        }

        return ok(start, r.cursor(), Symbol::Pause);
    }

    // Then we consume one char, which can be either a letter or something else
    // which will be considered a space.
    let c = r
        .consume()
        .ok_or(Span::new(start, start).wrap("End of input".into()))?
        .to_ascii_lowercase();

    if is_letter(c) {
        // We read until we don't find another instance of that same letter.
        while let Some(c2) = r.peek() {
            if c2.to_ascii_lowercase() != c {
                break;
            }
            let _ = r.consume();
        }

        return ok(start, r.cursor(), Symbol::Letter(c));
    }

    // If `c` is not a letter it is a space. We consume characters until we can
    // peak an hyphen, pause or letter.
    loop {
        if peek_hyphen(r).is_some() || peek_pause(r).is_some() {
            break;
        }

        let Some(&c) = r.peek() else {
            break;
        };

        if is_letter(c) {
            break;
        }

        r.consume();
    }

    ok(start, r.cursor(), Symbol::Space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_symbol_works() {
        let parse = |input: &str| {
            let mut reader = Reader::new(input.as_bytes());
            parse_symbol(&mut reader)
        };

        assert_eq!(parse("ab"), ok(0, 1, Symbol::Letter(b'a')));
        assert_eq!(parse("aab"), ok(0, 2, Symbol::Letter(b'a')));
        assert_eq!(parse("aaab"), ok(0, 3, Symbol::Letter(b'a')));

        assert_eq!(parse(" a"), ok(0, 1, Symbol::Space));
        assert_eq!(parse("  a"), ok(0, 2, Symbol::Space));
        assert_eq!(parse("   a"), ok(0, 3, Symbol::Space));
        assert_eq!(parse(".a"), ok(0, 1, Symbol::Space));
        assert_eq!(parse("./a"), ok(0, 2, Symbol::Space));
        assert_eq!(parse(" $#@a"), ok(0, 4, Symbol::Space));

        for hyphen in &["-", "–", "—", "−"] {
            assert_eq!(
                parse(&format!("{hyphen}a")),
                ok(0, hyphen.as_bytes().len(), Symbol::Hyphen)
            );

            for hyphen2 in &["-", "–", "—", "−"] {
                assert_eq!(
                    parse(&format!("{hyphen}{hyphen2}a")),
                    err(
                        0,
                        hyphen.as_bytes().len() + hyphen2.as_bytes().len(),
                        "2 hyphens cannot appear in a row"
                    )
                );
            }
        }

        for pause in &["'", "’", "`"] {
            assert_eq!(
                parse(&format!("{pause}a")),
                ok(0, pause.as_bytes().len(), Symbol::Pause)
            );

            for pause2 in &["'", "’", "`"] {
                assert_eq!(
                    parse(&format!("{pause}{pause2}a")),
                    err(
                        0,
                        pause.as_bytes().len() + pause2.as_bytes().len(),
                        "2 pauses cannot appear in a row"
                    )
                );
            }
        }
    }
}
