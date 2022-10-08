use std::marker::PhantomData;

use easy_ext::ext;

use crate::{ParseResult, Parser, Span, Stream};

/// Provide [`or_error`](Self::or_error) method to all [`Parser`](super::Parser).
#[ext(OrErrorExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, and if it fails to parse it is transformed into
    /// an error with the provided function.
    ///
    /// Rollbacks if `Self` fails to parse, and stream will only progress with
    /// what `Self` parsed.
    ///
    /// Returns the output of `Self`.
    fn or_error<F>(self, error: F) -> OrError<S, Self, F>
    where
        F: Fn(Span) -> Self::Error,
    {
        OrError(self, error, PhantomData)
    }
}

/// See [`or_error`](OrErrorExt::or_error).
#[derive(Debug, Clone, Copy)]
pub struct OrError<S, A, F>(A, F, PhantomData<S>);

impl<S, A, I, O, E, F> Parser<S, I, O> for OrError<S, A, F>
where
    A: Parser<S, I, O, Error = E>,
    F: Fn(Span) -> E,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream.transaction(|stream| {
            let out_a = self.0.parse(stream)?.ok_or_else(|| self.1(stream.span()))?;
            Ok(Some(out_a))
        })
    }
}
