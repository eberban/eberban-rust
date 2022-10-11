use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// Parse using a `Vec` or tuple of parsers having the same input and output types.
///
/// Will try each one in order, will return an error as soon as one errors, and
/// return the output of the first one that parses. If none parses, this parser
/// don't parse too.
///
/// Equivalent of chaining many parsers using [`or`](super::OrExt::or).
///
/// It is implemented for tuples containing up to 40 members, which should be enough
/// for must usages. If more are needed, use another second choice inside the
/// first one or use [`or`](super::OrExt::or).
pub fn choice<T>(tuple: T) -> Choice<T> {
    Choice(tuple)
}

/// See [`choice`].
#[derive(Debug, Clone, Copy)]
pub struct Choice<Tuple>(Tuple);

impl<S, I, O, E, A> Parser<S, I, O> for Choice<Vec<A>>
where
    A: Parser<S, I, O, Error = E>,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                // We try each parser in order.
                for member in &self.0 {
                    let out = stream
                        .transaction(|stream| member.parse(stream).option_in_err())
                        .option_in_ok()?;

                    // If it parsed then we return the result.
                    if let Some(out) = out {
                        return Ok(out);
                    }
                }

                // None parsed.
                Err(None)
            })
            .option_in_ok()
    }
}

macro_rules! impl_for_tuple {
    () => {};
    ($head:ident $($more:ident)*) => {
        // Implement for this tuple.
        impl_for_tuple!(# $head $($more)*);
        // Call recursively which will match with a $more smaller by 1
        impl_for_tuple!($($more)*);
    };
    (# $($member:ident)*) => {
        #[allow(non_snake_case)]
        impl<S, I, O, E, $($member),*> Parser<S, I, O> for Choice<($($member,)*)>
        where
            $( $member: Parser<S, I, O, Error = E>, )*
        {
            type Error = E;

            fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
            where
                S: Stream<I, Self::Error>,
            {
                stream
                    .transaction(|stream| {
                        // We destructure using the names to access each
                        // element of the tuple.
                        let Choice(($($member,)*)) = self;

                        // We try each parser in order.
                        $(
                            let out = stream.transaction(|stream| {
                                $member.parse(stream).option_in_err()
                            }).option_in_ok()?;

                            // If it parsed then we return the result.
                            if let Some(out) = out {
                                return Ok(out);
                            }
                        )*

                        // None parsed.
                        Err(None)
                    })
                    .option_in_ok()
            }
        }
    };
}

impl_for_tuple!(
    T39 T38 T37 T36 T35 T34 T33 T32 T31 T30 T29 T28 T27 T26 T25 T24 T23 T22 T21
    T20 T19 T18 T17 T16 T15 T14 T13 T12 T11 T10 T9 T8 T7 T6 T5 T4 T3 T2 T1 T0
);
