use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`or`](Self::or) method to all [`Parser`](super::Parser).
#[ext(OrExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self`, then if not successful parse from `B`.
    ///
    /// Rollbacks if both of them fail.
    ///
    /// Returns the output of either `Self` and `B` which must have the same
    /// output type.
    /// 
    /// If one wants to chain many ors, prefer using [`choice`](super::choice())
    /// instead.
    fn or<B>(self, other: B) -> Or<Self, B>
    where
        B: Parser<S, I, O, Error = Self::Error>,
    {
        Or(self, other)
    }
}

/// See [`or`](OrExt::or).
#[derive(Debug, Clone, Copy)]
pub struct Or<A, B>(A, B);

impl<S, A, B, I, O, E> Parser<S, I, O> for Or<A, B>
where
    A: Parser<S, I, O, Error = E>,
    B: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream)?;

                // If A parsed then we return the result.
                if let Some(out) = out_a {
                    return Ok(out);
                }

                // Otherwise we use B.
                self.1.parse(stream).option_in_err()
            })
            .option_in_ok()
    }
}
