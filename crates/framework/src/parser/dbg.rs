use std::fmt::Debug;

use easy_ext::ext;

use crate::{ParseResult, Parser, Stream};

/// Provide [`dbg`](Self::dbg) method to all [`Parser`].
#[ext(DbgExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self`, and prints its output or error.
    ///
    /// Returns what Self returns.
    fn dbg(self, title: impl ToString) -> Dbg<Self> {
        Dbg(self, title.to_string())
    }
}

/// See [`dbg`](DbgExt::dbg).
#[derive(Debug, Clone)]
pub struct Dbg<A>(A, String);

impl<S, A, I, O, E> Parser<S, I, O> for Dbg<A>
where
    A: Parser<S, I, O, Error = E>,
    O: Debug,
    E: Debug,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        let out = self.0.parse(stream);

        eprintln!("{}: {} {:?}", self.1, stream.span(), out);

        out
    }
}
