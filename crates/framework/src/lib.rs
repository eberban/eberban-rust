//! A small parser combinator framework inspired by [`chumsky`] but with a
//! different take on error handling.
//!
//! This parser framework makes the distinction between not finding what it
//! looks for (`Ok(None)`) and encountering an error (`Err(_)`) that should be
//! reported to the user. Most combinators will directly forward upward any
//! error it encounters.
//!
//! # How to convert a grammar from PEG
//!
//! This framework is written to rewrite the parser for conlang/loglang
//! [Eberban] which is current made on top of [`pegjs`], into a Rust parser with
//! better error reporting and multi-stage parsing.
//!
//! | PEG       | Equivalent
//! |:----------|:----
//! | `a b`     | [`a.then(b)`](ThenExt::then)
//! | `a / b`   | [`a.or(b)`](OrExt::or) or [`choice((a,b))`](choice)
//! | `a &b`    | [`a.then_peek(b)`](ThenPeekExt::then_peek)
//! | `a !b`    | [`a.then_peek(not(b))`](not)
//! | `a?`      | [`a.opt()`](OptExt::opt)
//! | `a*`      | [`a.repeated(..)`](RepeatedExt::repeated)
//! | `a+`      | [`a.repeated(1..)`](RepeatedExt::repeated)
//! | `.`       | [`any()`](any)
//! | `[chars]` | [`one_of(chars)`](one_of)
//!
//! # Spans
//!
//! Spans represents some range in the input that some error or token maps
//! to. By default parsers don't collect spans for outputs, which can be done
//! manually using [`spanned`]. When producing errors spans are provided too.
//!
//! # Producing errors
//!
//! By default parsers don't produce errors, and it is up to the parser writer to
//! produce errors using [`or_error`] or [`then_error`] when it is prefered
//! to report a problem to the user instead of ignoring this path and trying
//! other rules. [`or_error`] turns a failed parse into an error, while
//! [`then_error`] turns a successful parse into an
//! error. As the latter allows to access the output of the parse, it is better
//! to write `a.then_peek(b.then_error(|span, err| ...))` than
//! `a.then_peek(not(b).or_error(|span| ...))`.
//!
//! [`chumsky`]: https://github.com/zesterer/chumsky
//! [Eberban]: https://github.com/eberban/eberban
//! [`pegjs`]: https://github.com/pegjs/pegjs
//! [`or_error`]: OrErrorExt::or_error
//! [`then_error`]: ThenErrorExt::then_error
//! [`spanned`]: SpannedExt::spanned

mod as_iter;
mod macros;
mod parser;
mod span;
mod stream;

pub use {as_iter::*, parser::*, span::*, stream::*};

use easy_ext::ext;

/// Alias of `Result<Option<O>, E>` which is returned by [`Parser::parse`] and
/// [`Stream::next`].
///
/// `None` is used to represent an operation that didn't worked but that isn't
/// fatal, as opposed to `Err(_)` which should most of the time be propagated
/// upward.
///
/// [`OptionInOk`] and [`OptionInErr`] allows to move around the `Option` between
/// the `Ok` and `Err` variant. Functions can thus exit early using the `?`
/// operator on a `Result<T, Option<E>>`, which can be transformed into a
/// `Result<Option<T>, E>` such that only `?` only exit early on `E`.
pub type ParseResult<O, E> = Result<Option<O>, E>;

/// Convertion from `Result<T, Option<E>>` to `Result<Option<T>, E>`.
#[ext(OptionInOk)]
pub impl<T, E> Result<T, Option<E>> {
    fn option_in_ok(self) -> Result<Option<T>, E> {
        match self {
            Err(Some(e)) => Err(e),
            Err(None) => Ok(None),
            Ok(t) => Ok(Some(t)),
        }
    }
}

/// Convertion from `Result<Option<T>, E>` to `Result<T, Option<E>>`.
#[ext(OptionInErr)]
pub impl<T, E> Result<Option<T>, E> {
    fn option_in_err(self) -> Result<T, Option<E>> {
        match self {
            Ok(Some(e)) => Ok(e),
            Ok(None) => Err(None),
            Err(t) => Err(Some(t)),
        }
    }
}
