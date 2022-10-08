use std::marker::PhantomData;

use easy_ext::ext;

use crate::{ParseResult, Parser, Stream};

/// Provide [`not`](Self::not) method to all [`Parser`].
#[ext(NotExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Don't parse if `Self` parses, otherwise returns `()`.
    ///
    /// However if `Self` returns an error it propagates it.
    fn not(self) -> Not<Self, O> {
        Not(self, PhantomData)
    }
}

/// See [`not`](NotExt::not).
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
