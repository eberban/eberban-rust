use std::marker::PhantomData;

use easy_ext::ext;

use crate::{ParseResult, Parser, Stream};

/// Provide [`discard_error`](Self::discard_error) method to all [`Parser`].
#[ext(DiscardErrorExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self`, and ignore errors (they are transformed in failed
    /// parse).
    fn discard_error(self) -> DiscardError<S, Self, O> {
        DiscardError(self, PhantomData)
    }
}

/// See [`discard_error`](DiscardErrorExt::discard_error).
#[derive(Debug, Clone, Copy)]
pub struct DiscardError<S, A, O>(A, PhantomData<(S, O)>);

impl<S, A, I, O, E> Parser<S, I, O> for DiscardError<S, A, O>
where
    A: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        match self.0.parse(stream) {
            Err(_) => Ok(None),
            other => other,
        }
    }
}
