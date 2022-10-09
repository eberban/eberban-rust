use std::marker::PhantomData;

use crate::{ParseResult, Parser, Stream};

/// See [`end`].
#[derive(Debug, Clone, Copy)]
pub struct End<S, E>(PhantomData<(S, E)>);

impl<S, I, E> Parser<S, I, ()> for End<S, E> {
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
pub fn end<S, E>() -> End<S, E> {
    End(PhantomData)
}
