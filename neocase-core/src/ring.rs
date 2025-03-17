use anyhow::bail;

#[derive(Debug)]
pub struct NonemptyRing<T> {
    inner: Vec<T>,
    ind: usize,
}

impl<T> NonemptyRing<T>
where
    T: Copy + PartialEq,
{
    pub(crate) fn new(values: Vec<T>) -> anyhow::Result<Self> {
        if values.is_empty() {
            bail!("no values supplied when creating nonempty ring");
        }
        Ok(Self {
            inner: values,
            ind: 0,
        })
    }

    pub(crate) fn advance_to_after(&mut self, v: &T) -> anyhow::Result<()> {
        for (i, found) in self.inner.iter().enumerate() {
            if found == v {
                self.ind = i;
                self.get_and_progress();
                return Ok(());
            }
        }
        bail!("advance to failed, value not in ring");
    }

    #[inline]
    pub(crate) fn get(&self) -> &T {
        &self.inner[self.ind]
    }

    pub(crate) fn get_and_progress(&mut self) -> T {
        let cur = self.ind;
        self.ind += 1;
        if self.ind == self.inner.len() {
            self.ind = 0;
        }
        self.inner[cur]
    }

    pub(crate) fn iter_from<'a>(&'a mut self, v: &'a T) -> NonEmptyRingIter<'a, T> {
        while &self.get_and_progress() != v {}
        NonEmptyRingIter {
            parent: self,
            find: v,
            done: false,
        }
    }

    #[inline]
    pub(crate) fn into_inner(self) -> Vec<T> {
        self.inner
    }
}

pub(crate) struct NonEmptyRingIter<'a, T> {
    parent: &'a mut NonemptyRing<T>,
    find: &'a T,
    done: bool,
}

impl<T> Iterator for NonEmptyRingIter<'_, T>
where
    T: PartialEq + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let next = self.parent.get_and_progress();
        if &next == self.find {
            self.done = true;
        } else {
            return Some(next);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_ring_next_forever() {
        let values = vec![1, 2, 3];
        let mut ring = NonemptyRing::new(values).unwrap();
        assert_eq!(1, ring.get_and_progress());
        assert_eq!(2, *ring.get());
        assert_eq!(2, ring.get_and_progress());
        assert_eq!(3, *ring.get());
        assert_eq!(3, ring.get_and_progress());
        assert_eq!(1, *ring.get());
        assert_eq!(1, ring.get_and_progress());
        assert_eq!(2, *ring.get());
        assert_eq!(2, ring.get_and_progress());
        assert_eq!(3, *ring.get());
        assert_eq!(3, ring.get_and_progress());
        assert_eq!(1, *ring.get());
    }

    #[test]
    fn non_empty_ring_does_not_accept_empty() {
        let values: Vec<i32> = vec![];
        assert!(NonemptyRing::new(values).is_err());
    }

    #[test]
    fn can_start_from_provided() {
        let values = vec![2, 4, 6, 7];
        let mut ring = NonemptyRing::new(values).unwrap();
        ring.advance_to_after(&2).unwrap();
        assert_eq!(4, *ring.get());
        let mut ring = NonemptyRing::new(ring.into_inner()).unwrap();
        ring.advance_to_after(&6).unwrap();
        assert_eq!(7, *ring.get());
    }

    #[test]
    fn iter_finds_next() {
        let values = vec![1, 2, 3];
        let mut ring = NonemptyRing::new(values).unwrap();
        let val = ring.iter_from(&1);
        let found: Vec<i32> = val.collect();
        assert_eq!(vec![2, 3], found);
    }
}
