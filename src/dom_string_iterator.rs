use web_sys::DomStringList;

/// An [Iterator] for a [`DomStringList`]
pub(crate) struct DomStringIterator {
    inner: DomStringList,
    idx: u32,
}

impl From<DomStringList> for DomStringIterator {
    #[inline]
    fn from(inner: DomStringList) -> Self {
        Self { inner, idx: 0 }
    }
}

impl DomStringIterator {
    #[inline]
    fn len(&self) -> u32 {
        self.inner.length()
    }

    #[inline]
    fn size_hint_base(&self) -> usize {
        (self.len() - self.idx) as usize
    }
}

impl Iterator for DomStringIterator {
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
        let out = self.size_hint_base();
        (out, Some(out))
    }
}

impl ExactSizeIterator for DomStringIterator {
    #[inline]
    fn len(&self) -> usize {
        self.size_hint_base()
    }
}
