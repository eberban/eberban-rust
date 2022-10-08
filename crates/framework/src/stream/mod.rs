mod iter;

pub use iter::*;

use crate::{ParseResult, Span};

/// A stream of things that can be parsed.
///
/// Supports transactionnal operations (rewind on failure) and keep tracks of
/// spans.
pub trait Stream<O, E> {
    /// Get the next value in the stream if any.
    ///
    /// Might produce errors if it is made from a [`Parser`](super::Parser) that
    /// produces errors.
    fn next(&mut self) -> ParseResult<O, E>;

    /// Execute given function in a transaction. Progress will be reverted if
    /// the function returns `Err(_)`.
    fn transaction<F, T, E2>(&mut self, f: F) -> Result<T, E2>
    where
        F: FnOnce(&mut Self) -> Result<T, E2>;

    /// Get the span of all input that was parsed since the start of the
    /// latest transaction.
    fn span(&self) -> Span;
}
