use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use easy_ext::ext;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

type RepeatedIndex = u32;

/// Provide [`repeated`](Self::repeated) method to all [`Parser`].
#[ext(RepeatedExt)]
pub impl<Self_, S, I, O, E> Self_
where
    Self: Sized + Parser<S, I, O, Error = E>,
{
    /// Parse from `Self` a number of times described by the provided
    /// range.
    ///
    /// Parsing will fail if it can't parse up to the start of the
    /// range, and will stop parsing when reaching the end of the range, **even
    /// if it could parse more**.
    fn repeated<R>(self, range: R) -> Repeated<S, Self, O, R>
    where
        R: RangeBounds<RepeatedIndex> + Clone,
    {
        Repeated(self, range, PhantomData)
    }
}

/// See [`repeated`](RepeatedExt::repeated).
#[derive(Debug, Clone, Copy)]
pub struct Repeated<S, A, O, R>(A, R, PhantomData<(S, O)>);

impl<S, A, I, O, E, R> Parser<S, I, Vec<O>> for Repeated<S, A, O, R>
where
    A: Parser<S, I, O, Error = E>,
    R: RangeBounds<RepeatedIndex> + Clone,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<Vec<O>, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let mut out_vec = Vec::new();

                for i in 0.. {
                    // We check that we're not going out of range.
                    match self.1.end_bound() {
                        Bound::Included(end) if &i > end => return Ok(out_vec),
                        Bound::Excluded(end) if &i >= end => return Ok(out_vec),
                        _ => (),
                    }

                    // We parse using `A`, and rollback if it fails or error.
                    let out = stream.transaction(|stream| self.0.parse(stream).option_in_err());

                    // We get rid of the error only.
                    let out = out.option_in_ok()?;

                    // If it parses then we add the output to the Vec.
                    if let Some(v) = out {
                        out_vec.push(v);
                        continue;
                    }

                    // If it didn't parse, that's fine `i` is in the range.
                    if self.1.contains(&i) {
                        break;
                    }

                    // Otherwise the whole parser reverts.
                    return Err(None);
                }

                Ok(out_vec)
            })
            .option_in_ok()
    }
}
