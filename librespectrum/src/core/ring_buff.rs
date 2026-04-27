
pub struct RingBuff<T, const N: usize> {
    items: [Option<T>; N],
    head: usize,
}

impl<T: Copy, const N: usize> RingBuff<T, N> {

    pub fn new() -> Self {
        assert!(N > 0);
        Self { items: [None; N], head: 0 }
    }

    pub fn from<const K: usize>(items: [T; K]) -> Self {
        assert!(K <= N);
        let mut buff = Self::new();
        for (i, &item) in items.iter().enumerate() {
            buff.items[i] = Some(item);
        }
        buff.head = Self::next(K - 1);
        buff
    }

    pub fn len(&self) -> usize {
        if self.items[self.head].is_some() { N } else { self.head }
    }

    pub fn push(&mut self, item: T) -> &mut Self {
        self.items[self.head] = Some(item);
        self.head = Self::next(self.head);
        self
    }

    pub fn iter_to_tail<'a>(&'a self) -> RingBuffIterator<'a, T, N> {
        RingBuffIterator {
            items: &self.items,
            index: Self::prev(self.head),
            count: self.len(),
        }
    }

    pub fn next(index: usize) -> usize {
        if index == N - 1 { 0 } else { index + 1 }
    }

    pub fn prev(index: usize) -> usize {
        if index == 0 { N - 1 } else { index - 1 }
    }

}

pub struct RingBuffIterator<'a, T, const N: usize> {
    items: &'a [Option<T>; N],
    index: usize,
    count: usize,
}

impl<T: Copy, const N: usize> Iterator for RingBuffIterator<'_, T, N> {

    type Item = T;

    fn next(&mut self) -> Option<T> {

        let next = if self.count > 0 { self.items[self.index] } else { None };
        if next.is_some() {
            self.index = RingBuff::<T, N>::prev(self.index);
            self.count -= 1;
        }
        next

    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_length() {
        let buff = RingBuff::<u8, 3>::from([1, 2]);
        assert_eq!(buff.len(), 2);
    }

    #[test]
    fn allows_to_iterate_through_items() {
        let buff = RingBuff::<u8, 3>::from([1, 2, 3]);
        assert_eq!(buff.iter_to_tail().collect::<Vec<u8>>(), vec![3, 2, 1]);
    }

    #[test]
    fn behaves_like_ring_buffer() {
        let mut buff = RingBuff::<u8, 3>::from([1, 2]);
        assert_eq!(buff.push(3).push(4).len(), 3);
        assert_eq!(buff.iter_to_tail().collect::<Vec<u8>>(), vec![4, 3, 2]);
    }

}
