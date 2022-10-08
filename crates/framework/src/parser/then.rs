use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`then`](Self::then) method to all [`Parser`](super::Parser).
#[ext(ThenExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, then if successful parse from `B`.
    ///
    /// Rollbacks if either of them fail.
    ///
    /// Returns the output of both `Self` and `B`.
    fn then<B, BO>(self, other: B) -> Then<Self, B>
    where
        B: Parser<S, I, BO, Error = Self::Error>,
    {
        Then(self, other)
    }
}

/// See [`then`](ThenExt::then).
#[derive(Debug, Clone, Copy)]
pub struct Then<A, B>(A, B);

impl<S, A, B, I, AO, BO, E> Parser<S, I, (AO, BO)> for Then<A, B>
where
    A: Parser<S, I, AO, Error = E>,
    B: Parser<S, I, BO, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(AO, BO), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream).option_in_err()?;
                let out_b = self.1.parse(stream).option_in_err()?;
                Ok((out_a, out_b))
            })
            .option_in_ok()
    }
}
