use std::marker::PhantomData;

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`then_with`](Self::then_with) method to all [`Parser`].
#[ext(ThenWithExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, then if successful calls the function that takes the
    /// output of `Self` and returns another parser `B` which is finally used.
    ///
    /// Rollbacks if either of them fail.
    ///
    /// Returns the output of both `Self` and `B`.
    /// Since the output of `Self` is used both in the function and as an
    /// output, it must implement `Clone`.
    fn then_with<F, B, BO>(self, f: F) -> ThenWith<Self, B, AO, Self::Error, F>
    where
        B: Parser<S, I, BO, Error = Self::Error>,
        AO: Clone,
        F: Fn(AO) -> B,
    {
        ThenWith(self, f, PhantomData)
    }
}

/// See [`then_with`](ThenWithExt::then_with).
#[derive(Debug, Clone, Copy)]
pub struct ThenWith<A, B, AO, E, F>(A, F, PhantomData<(B, AO, E)>);

impl<S, A, B, I, AO, BO, E, F> Parser<S, I, (AO, BO)> for ThenWith<A, B, AO, E, F>
where
    A: Parser<S, I, AO, Error = E>,
    B: Parser<S, I, BO, Error = E>,
    AO: Clone,
    F: Fn(AO) -> B,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(AO, BO), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream).option_in_err()?;
                let b = self.1(out_a.clone());
                let out_b = b.parse(stream).option_in_err()?;
                Ok((out_a, out_b))
            })
            .option_in_ok()
    }
}
