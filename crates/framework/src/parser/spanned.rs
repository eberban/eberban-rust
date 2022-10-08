use std::marker::PhantomData;

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Span, Stream};

/// Provide [`spanned`](Self::spanned) method to all [`Parser`].
#[ext(SpannedExt)]
pub impl<Self_, S, I, O> Self_
where
    Self: Sized + Parser<S, I, O>,
{
    /// Parse from `Self`, and returns the output with its span.
    ///
    /// Rollbacks if `Self` fails to parse, and stream will only progress with
    /// what `Self` parsed.
    fn spanned(self) -> Spanned<Self, O> {
        Spanned(self, PhantomData)
    }
}

/// See [`spanned`](SpannedExt::spanned).
#[derive(Debug, Clone, Copy)]
pub struct Spanned<A, O>(A, PhantomData<O>);

impl<S, A, I, O, E> Parser<S, I, (Span, O)> for Spanned<A, O>
where
    A: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<(Span, O), Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let out = self.0.parse(stream).option_in_err()?;
                Ok((stream.span(), out))
            })
            .option_in_ok()
    }
}
