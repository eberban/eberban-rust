use std::{iter::Cloned, ops::RangeInclusive};

pub trait AsIter<T> {
    type Iter: Iterator<Item = T>;

    fn as_iter(&self) -> Self::Iter;
}

impl AsIter<char> for String {
    type Iter = std::vec::IntoIter<char>;

    fn as_iter(&self) -> Self::Iter {
        self.chars().collect::<Vec<_>>().into_iter()
    }
}

impl<'a> AsIter<char> for &'a str {
    type Iter = std::str::Chars<'a>;

    fn as_iter(&self) -> Self::Iter {
        self.chars()
    }
}

impl<'a, T: Clone> AsIter<T> for &'a [T] {
    type Iter = core::iter::Cloned<core::slice::Iter<'a, T>>;

    fn as_iter(&self) -> Self::Iter {
        self.iter().cloned()
    }
}

impl<T: Clone, const N: usize> AsIter<T> for [T; N] {
    type Iter = std::array::IntoIter<T, N>;

    fn as_iter(&self) -> Self::Iter {
        IntoIterator::into_iter(self.clone())
    }
}

impl<'a, T: Clone, const N: usize> AsIter<T> for &'a [T; N] {
    type Iter = Cloned<std::slice::Iter<'a, T>>;

    fn as_iter(&self) -> Self::Iter {
        IntoIterator::into_iter(*self).cloned()
    }
}

impl<T: Clone> AsIter<T> for Vec<T> {
    type Iter = std::vec::IntoIter<T>;

    fn as_iter(&self) -> Self::Iter {
        IntoIterator::into_iter(self.clone())
    }
}

impl<T> AsIter<T> for RangeInclusive<T>
where
    Self: Iterator<Item = T> + Clone,
{
    type Iter = Self;

    fn as_iter(&self) -> Self::Iter {
        self.clone()
    }
}
