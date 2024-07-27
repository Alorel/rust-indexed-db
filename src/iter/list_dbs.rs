use std::iter::FusedIterator;

use crate::factory::DatabaseDetails;

type Inner = js_sys::ArrayIntoIter;

/// An iterator over the results of future returned by [`DBFactory::databases`](crate::DBFactory::databases).
#[must_use]
pub struct ListDatabasesIter(Inner);

impl ListDatabasesIter {
    pub(crate) fn new(array: js_sys::Array) -> Self {
        Self(array.into_iter())
    }
}

impl Iterator for ListDatabasesIter {
    type Item = crate::Result<DatabaseDetails>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(TryInto::try_into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.0.last().map(TryInto::try_into)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(TryInto::try_into)
    }
}

impl DoubleEndedIterator for ListDatabasesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(TryInto::try_into)
    }
}

impl FusedIterator for ListDatabasesIter {}

impl ExactSizeIterator for ListDatabasesIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
