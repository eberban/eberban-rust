use std::marker::PhantomData;

use crate::{ParseResult, Parser, Stream};

#[derive(Debug, Clone, Copy)]
/// See [`end`].
pub struct End<I, E>(PhantomData<(I, E)>);

impl<S, I, E> Parser<S, I, ()> for End<I, E> {
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream.transaction(|stream| match stream.next()? {
            Some(_) => Ok(None),
            None => Ok(Some(())),
        })
    }
}

/// Parse the end of the input, failing if the end is not reached.
pub fn end<I, E>() -> End<I, E> {
    End(PhantomData)
}
