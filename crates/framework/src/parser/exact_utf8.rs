use std::marker::PhantomData;

use crate::{OptionInErr, OptionInOk, ParseResult, Parser, Stream};

/// See [`exact_utf8`].
pub struct ExactUtf8<S, E>(String, PhantomData<(S, E)>);

impl<S, E> Parser<S, u8, String> for ExactUtf8<S, E> {
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<String, Self::Error>
    where
        S: Stream<u8, Self::Error>,
    {
        stream
            .transaction(|stream| {
                let target_bytes = self.0.as_bytes();

                // Read byte by byte and check it matches the target.
                for target in target_bytes {
                    let read = stream.next().option_in_err()?;

                    if target != &read {
                        return Err(None);
                    }
                }

                Ok(self.0.clone())
            })
            .option_in_ok()
    }
}

/// Parse the given `String` encoded as UTF8.
pub fn exact_utf8<S, E>(string: impl ToString) -> ExactUtf8<S, E> {
    ExactUtf8(string.to_string(), PhantomData)
}
