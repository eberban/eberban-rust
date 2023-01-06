use std::{cell::RefCell, rc::Rc};

use crate::{ParseResult, Parser, Stream};

type DynParser<'a, S, I, O, E> = Box<dyn Parser<S, I, O, Error = E> + 'a>;
type InnerParser<'a, S, I, O, E> = RefCell<Option<DynParser<'a, S, I, O, E>>>;

/// Creates a recursive parser, a parser that can call itself.
///
/// The provided function is given the recursive parser so it can be
/// composed with other parsers to match recursive structures.
///
/// However one must not call [`Parser::parse`] on this parser until it is
/// returned by the function.
pub fn recursive<'a, S, I, F, P, O, E>(f: F) -> Recursive<'a, S, I, O, E>
where
    P: Parser<S, I, O, Error = E> + 'a,
    F: Fn(Recursive<'a, S, I, O, E>) -> P + 'a,
{
    let ref_cell = Rc::new(RefCell::new(None));

    let parser = f(Recursive(Rc::clone(&ref_cell)));
    *ref_cell.borrow_mut() = Some(Box::new(parser));

    Recursive(ref_cell)
}

/// See [`recursive`].
#[derive(Clone)]
pub struct Recursive<'a, S, I, O, E>(Rc<InnerParser<'a, S, I, O, E>>);

impl<'a, S, I, O, E> Parser<S, I, O> for Recursive<'a, S, I, O, E> {
    type Error = E;

    fn parse(&self, stream: &mut S) -> ParseResult<O, Self::Error>
    where
        S: Stream<I, Self::Error>,
    {
        stream.transaction(|stream| {
            self.0
                .borrow()
                .as_ref()
                .expect(
                    "you must not call `parse` directly inside the definition \
                    of a recursive parser.",
                )
                .parse(stream)
        })
    }
}
