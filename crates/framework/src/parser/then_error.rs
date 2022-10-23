use std::marker::PhantomData;

use easy_ext::ext;

use crate::{ParseResult, Parser, Span, Stream};

/// Provide [`then_error`](Self::then_error) method to all [`Parser`].
#[ext(ThenErrorExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self`, and if it fails to parse it is transformed into
    /// an error with the provided function.
    ///
    /// Rollbacks if `Self` fails to parse, and stream will only progress with
    /// what `Self` parsed.
    ///
    /// Returns the output of `Self`.
    fn then_error<F>(self, error: F) -> ThenError<S, Self, O, F>
    where
        F: Fn(Span, O) -> Self::Error,
    {
        ThenError(self, error, PhantomData)
    }
}

/// See [`then_error`](ThenErrorExt::then_error).
#[derive(Debug, Clone, Copy)]
pub struct ThenError<S, A, O, F>(A, F, PhantomData<(S, O)>);

impl<S, A, I, O, E, F> Parser<S, I, ()> for ThenError<S, A, O, F>
where
    A: Parser<S, I, O, Error = E>,
    F: Fn(Span, O) -> E,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        // we want to rollback in every case
        let (res, span) = match stream
            .transaction(|stream| Err::<(), _>((self.0.parse(stream), stream.span())))
        {
            Err(v) => v,
            _ => unreachable!(),
        };

        match res? {
            None => Ok(None),
            Some(out_a) => Err(self.1(span, out_a)),
        }
    }
}
