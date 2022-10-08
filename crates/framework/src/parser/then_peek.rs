use std::marker::PhantomData;

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`then_peek`](Self::then_peek) method to all [`Parser`](super::Parser).
#[ext(ThenPeekExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, then if successful parse from `B`.
    ///
    /// Rollbacks if either of them fail, and stream will only progress with
    /// what `Self` parsed.
    ///
    /// Returns only the output of `Self`.
    fn then_peek<B, BO>(self, other: B) -> ThenPeek<Self, B, BO>
    where
        B: Parser<S, I, BO, Error = Self::Error>,
    {
        ThenPeek(self, other, PhantomData)
    }
}

/// See [`then_peek`](ThenPeekExt::then_peek).
#[derive(Debug, Clone, Copy)]
pub struct ThenPeek<A, B, BO>(A, B, PhantomData<BO>);

impl<S, A, B, I, AO, BO, E> Parser<S, I, AO> for ThenPeek<A, B, BO>
where
    A: Parser<S, I, AO, Error = E>,
    B: Parser<S, I, BO, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<AO, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream).option_in_err()?;

                // We parse the output but discard it.
                let _out_b: BO = match stream
                    .transaction(|stream| Err::<(), _>(self.1.parse(stream).option_in_err()))
                {
                    Ok(_) => unreachable!("Ok(_) is never returned to always rollback"),
                    Err(e) => e?,
                };

                Ok(out_a)
            })
            .option_in_ok()
    }
}
