use crate::error::Error;
use fancy_constructor::new;
use std::iter::FusedIterator;
use wasm_bindgen::prelude::*;

/// An iterator that maps [js array](js_sys::Array) values.
#[derive(new)]
#[new(vis(pub(crate)))]
#[must_use]
pub struct ArrayMapIter<T, E = Error> {
    src: js_sys::ArrayIntoIter,
    mapper: fn(JsValue) -> Result<T, E>,
}

impl<T, E> Iterator for ArrayMapIter<T, E> {
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.next().map(self.mapper)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.src.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.src.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.src.last().map(self.mapper)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.src.nth(n).map(self.mapper)
    }
}

impl<T, E> DoubleEndedIterator for ArrayMapIter<T, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.src.next_back().map(self.mapper)
    }
}

impl<T, E> FusedIterator for ArrayMapIter<T, E> {}

impl<T, E> ExactSizeIterator for ArrayMapIter<T, E> {
    #[inline]
    fn len(&self) -> usize {
        self.src.len()
    }
}
