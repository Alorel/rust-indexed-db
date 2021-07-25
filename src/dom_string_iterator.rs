use web_sys::DomStringList;

/// An [Iterator] for a [DomStringList]
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
}

impl<'a> Iterator for DomStringIterator {
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
        let out = (self.len() - self.idx) as usize;
        (out, Some(out))
    }
}
