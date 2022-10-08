use std::marker::PhantomData;

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`then_peek_with`](Self::then_peek_with) method to all [`Parser`](super::Parser).
#[ext(ThenPeekWithExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Mix of [`then_peek`] and [`then_with`].
    ///
    /// [`then_peek`]: super::ThenPeekExt::then_peek
    /// [`then_with`]: super::ThenWithExt::then_with
    fn then_peek_with<B, BO, F>(self, f: F) -> ThenPeekWith<Self, B, AO, BO, Self::Error, F>
    where
        B: Parser<S, I, BO, Error = Self::Error>,
        AO: Clone,
        F: Fn(AO) -> B,
    {
        ThenPeekWith(self, f, PhantomData)
    }
}

/// See [`then_peek_with`](ThenPeekWithExt::then_peek_with).
#[derive(Debug, Clone, Copy)]
pub struct ThenPeekWith<A, B, AO, BO, E, F>(A, F, PhantomData<(B, AO, BO, E)>);

impl<S, A, B, I, AO, BO, E, F> Parser<S, I, AO> for ThenPeekWith<A, B, AO, BO, E, F>
where
    A: Parser<S, I, AO, Error = E>,
    B: Parser<S, I, BO, Error = E>,
    AO: Clone,
    F: Fn(AO) -> B,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<AO, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream).option_in_err()?;
                let b = self.1(out_a.clone());

                let _out_b: BO = match stream
                    .transaction(|stream| Err::<(), _>(b.parse(stream).option_in_err()))
                {
                    Ok(_) => unreachable!("Ok(_) is never returned to always rollback"),
                    Err(e) => e?,
                };

                Ok(out_a)
            })
            .option_in_ok()
    }
}
