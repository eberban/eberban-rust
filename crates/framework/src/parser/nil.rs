use std::marker::PhantomData;

use crate::{ParseResult, Parser, Stream};

#[derive(Debug, Clone, Copy)]
/// See [`nil`].
pub struct Nil<I, E>(PhantomData<(I, E)>);

impl<S, I, E> Parser<S, I, ()> for Nil<I, E> {
    type Error = E;

    fn parse(&self, _stream: &mut S) -> ParseResult<(), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        Ok(Some(()))
    }
}

/// Parse nothing, which is always successful.
///
/// Useful inside a `then_*` combinator to get a [`Parser`] on which
/// chained combinators can be called.
pub fn nil<I, E>() -> Nil<I, E> {
    Nil(PhantomData)
}
