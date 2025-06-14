use std::collections::VecDeque;

use crate::{ArrayList, ChunkCapacity, Usize};

/// A cursor over a ArrayList.
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.  
/// Cursors always rest between two elements in the list, and index in a logically circular way.  
/// To accommodate this, there is a “ghost” non-element that yields None between the head and tail of the list.
/// When created, cursors start at the front of the list, or the “ghost” non-element if the list is empty.
#[derive(Clone)]
pub struct Cursor<'a, T, const N: usize>
where
    T: 'a,
    Usize<N>: ChunkCapacity,
{
    pub(crate) list: &'a ArrayList<T, N>,
    pub(crate) index: usize,
    pub(crate) chunk_index: usize,
    pub(crate) inner_index: usize,
}

const _: [(); core::mem::size_of::<usize>() * 4] = [(); core::mem::size_of::<Cursor<usize, 2>>()];

impl<'a, T, const N: usize> Cursor<'a, T, N>
where
    T: 'a,
    Usize<N>: ChunkCapacity,
{
    pub(crate) fn from_front(list: &'a ArrayList<T, N>) -> Self {
        Self {
            index: 0,
            chunk_index: 0,
            inner_index: 0,
            list,
        }
    }

    pub(crate) fn from_back(list: &'a ArrayList<T, N>) -> Self {
        Self {
            index: list.len().saturating_sub(1),
            chunk_index: list.chunks.len().saturating_sub(1),
            inner_index: list
                .chunks
                .back()
                .map_or(0, VecDeque::len)
                .saturating_sub(1),
            list,
        }
    }

    pub fn as_list(&self) -> &'a ArrayList<T, N> {
        self.list
    }

    pub fn back(&self) -> Option<&'a T> {
        self.list.back()
    }

    pub fn current(&self) -> Option<&'a T> {
        self.list
            .chunks
            .get(self.chunk_index)
            .and_then(|chunk| chunk.get(self.inner_index))
    }

    pub fn front(&self) -> Option<&'a T> {
        self.list.front()
    }

    pub fn index(&self) -> Option<usize> {
        if self.is_ghost() {
            return None;
        }

        Some(self.index)
    }

    pub fn move_next(&mut self) {
        if self.is_ghost() {
            self.index = 0;
            self.chunk_index = 0;
            self.inner_index = 0;
            return;
        }

        self.index += 1;

        self.inner_index += 1;
        if self.inner_index >= self.list.chunks[self.chunk_index].len() {
            self.chunk_index += 1;
            self.inner_index = 0;
        }
    }

    pub fn move_prev(&mut self) {
        if self.index == 0 {
            self.index = self.list.len();
            self.chunk_index = self.list.chunks.len();
            self.inner_index = 0;
            return;
        }

        self.index -= 1;

        if self.inner_index > 0 {
            self.inner_index -= 1;
            return;
        }

        self.chunk_index -= 1;
        self.inner_index = self.list.chunks[self.chunk_index].len().saturating_sub(1);
    }

    pub fn peek_next(&self) -> Option<&'a T> {
        if self.is_ghost() {
            return self.front();
        }

        if self.inner_index + 1 < self.list.chunks[self.chunk_index].len() {
            return self.list.chunks[self.chunk_index].get(self.inner_index + 1);
        }

        self.list
            .chunks
            .get(self.chunk_index + 1)
            .and_then(VecDeque::front)
    }

    pub fn peek_prev(&self) -> Option<&'a T> {
        if self.index == 0 {
            return None;
        }

        if self.inner_index > 0 {
            return Some(&self.list.chunks[self.chunk_index][self.inner_index - 1]);
        }

        self.list
            .chunks
            .get(self.chunk_index - 1)
            .and_then(VecDeque::back)
    }

    #[inline]
    fn is_ghost(&self) -> bool {
        self.index >= self.list.len()
    }
}

impl<T, const N: usize> core::fmt::Debug for Cursor<'_, T, N>
where
    T: core::fmt::Debug,
    Usize<N>: ChunkCapacity,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cursor")
            .field("list", self.list)
            .field("current", &self.current())
            .field("index", &self.index())
            .field("chunk_index", &self.chunk_index)
            .field("inner_index", &self.inner_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use quickcheck_macros::quickcheck;

    use crate::{ArrayList, ChunkCapacity, Usize};

    #[quickcheck]
    fn test_cursor_front_as_list(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let ptr = &list as *const _;

            let sut = list.cursor_front();
            assert_eq!(sut.as_list() as *const _, ptr);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }

    #[quickcheck]
    fn test_cursor_front_move_next(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1));

            for _ in 0..2 {
                for i in 0..seed.len() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i));
                    assert_eq!(sut.peek_prev(), seed.get(i.wrapping_sub(1)));
                    assert_eq!(sut.peek_next(), seed.get(i + 1));

                    sut.move_next();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last());
                assert_eq!(sut.peek_next(), seed.first());

                sut.move_next();
            }
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }

    #[quickcheck]
    fn test_cursor_front_move_prev(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1));

            for _ in 0..2 {
                sut.move_prev();
                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last());
                assert_eq!(sut.peek_next(), seed.first());

                for i in (0..seed.len()).rev() {
                    sut.move_prev();
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i));
                    assert_eq!(sut.peek_prev(), seed.get(i.wrapping_sub(1)));
                    assert_eq!(sut.peek_next(), seed.get(i + 1));
                }
            }
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }

    // --------------------------------------------------------

    #[quickcheck]
    fn test_cursor_back_as_list(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let ptr = &list as *const _;

            let sut = list.cursor_back();
            assert_eq!(sut.as_list() as *const _, ptr);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }

    #[quickcheck]
    fn test_cursor_back_move_next(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back();

            assert_eq!(sut.index(), seed.len().checked_sub(1));
            assert_eq!(sut.current(), seed.last());
            assert_eq!(sut.peek_prev(), seed.get(seed.len().wrapping_sub(2)));
            assert_eq!(sut.peek_next(), None);

            sut.move_next();
            assert_eq!(sut.index(), None);
            assert_eq!(sut.current(), None);
            assert_eq!(sut.peek_prev(), seed.last());
            assert_eq!(sut.peek_next(), seed.first());

            for _ in 0..2 {
                sut.move_next();
                assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
                assert_eq!(sut.current(), seed.first());
                assert_eq!(sut.peek_prev(), None);
                assert_eq!(sut.peek_next(), seed.get(1));

                for i in 0..seed.len() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), Some(&seed[i]));
                    assert_eq!(sut.peek_prev(), seed.get(i.wrapping_sub(1)));
                    assert_eq!(sut.peek_next(), seed.get(i + 1));
                    sut.move_next();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last());
                assert_eq!(sut.peek_next(), seed.first());
            }
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }

    #[quickcheck]
    fn test_cursor_back_move_prev(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back();

            for _ in 0..2 {
                assert_eq!(sut.index(), seed.len().checked_sub(1));
                assert_eq!(sut.current(), seed.last());
                assert_eq!(sut.peek_prev(), seed.get(seed.len().wrapping_sub(2)));
                assert_eq!(sut.peek_next(), None);

                for i in (0..seed.len()).rev() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i));
                    assert_eq!(sut.peek_prev(), seed.get(i.wrapping_sub(1)));
                    assert_eq!(sut.peek_next(), seed.get(i + 1));
                    sut.move_prev();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last());
                assert_eq!(sut.peek_next(), seed.first());

                sut.move_prev();
            }
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<6>(&seed);
        _test::<7>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
    }
}

#[cfg(feature = "nightly_tests")]
#[cfg(test)]
mod nightly_tests {
    use std::collections::{LinkedList, linked_list};

    use quickcheck_macros::quickcheck;

    use crate::{ArrayList, ChunkCapacity, Usize};

    use super::Cursor;

    #[quickcheck]
    fn nightly_test_cursor_front_move_next(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let linked_list = LinkedList::from_iter(seed.iter().copied());
            let array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front();
            let mut actual = array_list.cursor_front();

            assert_cursors_give_same_results(&expected, &actual);

            expected.move_next();
            actual.move_next();

            assert_cursors_give_same_results(&expected, &actual);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
        _test::<64>(&seed);
        _test::<128>(&seed);
        _test::<256>(&seed);
        _test::<512>(&seed);
    }

    #[quickcheck]
    fn nightly_test_cursor_back_move_next(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let linked_list = LinkedList::from_iter(seed.iter().copied());
            let array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back();
            let mut actual = array_list.cursor_back();

            assert_cursors_give_same_results(&expected, &actual);

            expected.move_next();
            actual.move_next();

            assert_cursors_give_same_results(&expected, &actual);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
        _test::<64>(&seed);
        _test::<128>(&seed);
        _test::<256>(&seed);
        _test::<512>(&seed);
    }

    #[quickcheck]
    fn nightly_test_cursor_front_move_prev(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let linked_list = LinkedList::from_iter(seed.iter().copied());
            let array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front();
            let mut actual = array_list.cursor_front();

            assert_cursors_give_same_results(&expected, &actual);

            expected.move_prev();
            actual.move_prev();

            assert_cursors_give_same_results(&expected, &actual);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
        _test::<64>(&seed);
        _test::<128>(&seed);
        _test::<256>(&seed);
        _test::<512>(&seed);
    }

    #[quickcheck]
    fn nightly_test_cursor_back_move_prev(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let linked_list = LinkedList::from_iter(seed.iter().copied());
            let array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back();
            let mut actual = array_list.cursor_back();

            assert_cursors_give_same_results(&expected, &actual);

            expected.move_prev();
            actual.move_prev();

            assert_cursors_give_same_results(&expected, &actual);
        }

        _test::<1>(&seed);
        _test::<2>(&seed);
        _test::<3>(&seed);
        _test::<4>(&seed);
        _test::<5>(&seed);
        _test::<8>(&seed);
        _test::<16>(&seed);
        _test::<32>(&seed);
        _test::<64>(&seed);
        _test::<128>(&seed);
        _test::<256>(&seed);
        _test::<512>(&seed);
    }

    fn assert_cursors_give_same_results<const N: usize>(
        expected: &linked_list::Cursor<'_, i32>,
        actual: &Cursor<'_, i32, N>,
    ) where
        Usize<N>: ChunkCapacity,
    {
        // FIXME
        assert_eq!(
            // see: https://github.com/rust-lang/rust/issues/147616
            expected
                .index()
                .map(|n| expected.peek_prev().map_or(0, |_| n)),
            actual.index()
        );
        assert_eq!(expected.current(), actual.current());

        assert_eq!(expected.front(), actual.front());
        assert_eq!(expected.back(), actual.back());

        assert_eq!(expected.peek_next(), actual.peek_next());
        assert_eq!(expected.peek_prev(), actual.peek_prev());
    }
}
