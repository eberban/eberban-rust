use std::marker::PhantomData;

use crate::{AsIter, ParseResult, Span, Stream};

/// Allows to use a collection, [`String`] or slice as a [`Stream`].
pub struct IterStream<Iter, O, E> {
    iter: Iter,
    span: Span,
    _phantom: PhantomData<(O, E)>,
}

impl<Iter, O, E> IterStream<Iter, O, E>
where
    Iter: Iterator<Item = O> + Clone,
{
    /// Create a [`Stream`] from provided source.
    pub fn new<T>(source: T) -> Self
    where
        T: AsIter<O, Iter = Iter>,
    {
        Self {
            iter: source.as_iter(),
            span: (0..0).into(),
            _phantom: PhantomData,
        }
    }

    fn branch(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            span: (self.span.end..self.span.end).into(),
            _phantom: PhantomData,
        }
    }

    fn merge(&mut self, other: Self) {
        self.iter = other.iter;
        self.span.end = other.span.end;
    }
}

impl<Iter, O, E> Stream<O, E> for IterStream<Iter, O, E>
where
    Iter: Iterator<Item = O> + Clone,
{
    fn next(&mut self) -> ParseResult<O, E> {
        Ok(match self.iter.next() {
            None => None,
            Some(v) => {
                self.span.end += 1;
                Some(v)
            }
        })
    }

    fn transaction<F, T, E2>(&mut self, f: F) -> Result<T, E2>
    where
        F: FnOnce(&mut Self) -> Result<T, E2>,
    {
        let mut branch = self.branch();
        let res = f(&mut branch);

        if res.is_ok() {
            self.merge(branch);
        }

        res
    }

    fn span(&self) -> Span {
        self.span
    }
}
