use std::marker::PhantomData;

use easy_ext::ext;

use crate::{ParseResult, Parser, Stream};

/// Provide [`opt`](Self::opt) method to all [`Parser`].
#[ext(OptExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self` if present, and return an Option of the output
    /// of `Self`.
    fn opt(self) -> Opt<S, Self, Self::Error>
where {
        Opt(self, PhantomData)
    }
}

/// See [`opt`](OptExt::opt).
pub struct Opt<S, A, E>(A, PhantomData<(S, E)>);

impl<S, A, I, O, E> Parser<S, I, Option<O>> for Opt<S, A, E>
where
    A: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<Option<O>, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        Ok(Some(self.0.parse(stream)?))
    }
}
