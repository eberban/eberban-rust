use std::marker::PhantomData;

use crate::{AsIter, OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// See [`one_of`].
#[derive(Debug, Clone, Copy)]
pub struct OneOf<T, O, E>(T, PhantomData<(O, E)>);

impl<S, T, I, E> Parser<S, I, I> for OneOf<T, I, E>
where
    T: AsIter<I>,
    I: Eq,
{
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<I, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let value = stream.next().option_in_err()?;
                for choice in self.0.as_iter() {
                    if value == choice {
                        return Ok(value);
                    }
                }

                Err(None)
            })
            .option_in_ok()
    }
}

/// Parse one item that is in `group`.
pub fn one_of<T, I, E>(group: T) -> OneOf<T, I, E>
where
    T: AsIter<I>,
    I: Eq,
{
    OneOf(group, PhantomData)
}
