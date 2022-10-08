use std::marker::PhantomData;

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Provide [`map`](Self::map) method to all [`Parser`].
#[ext(MapExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, then transform its output using the provided function.
    ///
    /// Rollbacks if `Self` fails to parse, and stream will only progress with
    /// what `Self` parsed.
    fn map<F, FO>(self, f: F) -> Map<S, Self, AO, F, FO>
    where
        F: Fn(AO) -> FO,
    {
        Map(self, f, PhantomData)
    }
}

/// See [`map`](MapExt::map).
#[derive(Debug, Clone, Copy)]
pub struct Map<S, A, O, F, FO>(A, F, PhantomData<(S, O, FO)>);

impl<S, A, I, AO, E, F, FO> Parser<S, I, FO> for Map<S, A, AO, F, FO>
where
    A: Parser<S, I, AO, Error = E>,
    F: Fn(AO) -> FO,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<FO, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out_a = self.0.parse(stream).option_in_err()?;

                Ok(self.1(out_a))
            })
            .option_in_ok()
    }
}
