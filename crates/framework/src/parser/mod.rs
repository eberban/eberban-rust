mod any;
mod choice;
mod dbg;
mod discard;
mod discard_error;
mod end;
mod exact_utf8;
mod map;
mod nil;
mod not;
mod one_of;
mod opt;
mod or;
mod or_error;
mod recursive;
mod repeated;
mod spanned;
mod then;
mod then_error;
mod then_peek;
mod then_peek_with;
mod then_with;

pub use {
    any::*, choice::*, dbg::*, discard::*, discard_error::*, end::*, exact_utf8::*, map::*, nil::*,
    not::*, one_of::*, opt::*, or::*, or_error::*, recursive::*, repeated::*, spanned::*, then::*,
    then_error::*, then_peek::*, then_peek_with::*, then_with::*,
};

use crate::{ParseResult, Stream};

/// Main trait allowing to parse from a [`Stream`].
///
/// Extension traits are used to add chaining combinators to all types that
/// implement [`Parser`], such as [`ThenExt`].
pub trait Parser<S, I, O> {
    type Error;

    /// Parse 0 to many `I` from `stream`, then can return an `O`.
    ///
    /// Makes the distinction between not finding what it looks for (`Ok(None)`)
    /// and encountering an error (`Err(_)`) that should be reported to the
    /// user.
    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>;
}

/// Shortand to write a deny rule.
/// If the pattern is encountered, provided closure is used to produce an error.
/// Otherwise, parse nothing (`()`) succesfully without making progress.
pub fn deny<S, I, O, P, F>(pattern: P, error: F) -> impl Parser<S, I, (), Error = P::Error>
where
    P: Parser<S, I, O>,
    F: Fn(crate::Span, O) -> P::Error,
{
    pattern.then_error(error).opt().discard()
}
