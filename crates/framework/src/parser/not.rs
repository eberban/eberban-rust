use std::marker::PhantomData;

use crate::{ParseResult, Parser, Stream};

/// Don't parse if `parser` parses, otherwise returns `()`.
///
/// However if `parser` returns an error it propagates it.
pub fn not<S, I, O, P: Parser<S, I, O>>(parser: P) -> Not<P, O> {
    Not(parser, PhantomData)
}

/// See [`not`](not).
#[derive(Debug, Clone, Copy)]
pub struct Not<A, O>(A, PhantomData<O>);

impl<S, A, I, O, E> Parser<S, I, ()> for Not<A, O>
where
    A: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream.transaction(|stream| match self.0.parse(stream)? {
            Some(_) => Ok(None),
            None => Ok(Some(())),
        })
    }
}
