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