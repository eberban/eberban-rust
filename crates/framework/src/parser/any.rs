use std::marker::PhantomData;

use crate::{ParseResult, Parser, Stream};

/// See [`any`].
pub struct Any<S, I, E>(PhantomData<(S, I, E)>);

impl<S, I, E> Parser<S, I, I> for Any<S, I, E> {
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<I, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream.next()
    }
}

/// Parse a single item from the input.
pub fn any<S, I, E>() -> Any<S, I, E> {
    Any(PhantomData)
}
