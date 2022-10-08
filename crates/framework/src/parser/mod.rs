mod end;
mod map;
mod nil;
mod not;
mod one_of;
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
    end::*, map::*, nil::*, not::*, one_of::*, or::*, or_error::*, recursive::*, repeated::*,
    spanned::*, then::*, then_error::*, then_peek::*, then_peek_with::*, then_with::*,
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
