use std::marker::PhantomData;

use fancy_constructor::new;
use web_sys::DomStringList;

/// An iterator over a [`DOMStringList`](https://developer.mozilla.org/en-US/docs/Web/API/DOMStringList).
#[derive(new)]
#[new(vis(pub(crate)))]
#[must_use]
pub struct DomStringIter<'a> {
    inner: DomStringList,

    #[new(val(0))]
    idx: u32,

    #[new(val(PhantomData))]
    _marker: PhantomData<&'a ()>,
}

impl Iterator for DomStringIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.inner.item(self.idx) {
            self.idx += 1;
            Some(v)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let out = self.len();
        (out, Some(out))
    }
}

impl ExactSizeIterator for DomStringIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        (self.inner.length() - self.idx) as usize
    }
}
