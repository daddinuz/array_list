use std::collections::VecDeque;

use crate::{ArrayList, ChunkCapacity, Cursor, Usize};

/// A cursor over a ArrayList.
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.  
/// Cursors always rest between two elements in the list, and index in a logically circular way.  
/// To accommodate this, there is a “ghost” non-element that yields None between the head and tail of the list.
/// When created, cursors start at the front of the list, or the “ghost” non-element if the list is empty.
pub struct CursorMut<'a, T, const N: usize>
where
    T: 'a,
    Usize<N>: ChunkCapacity,
{
    list: &'a mut ArrayList<T, N>,
    index: usize,
    chunk_index: usize,
    inner_index: usize,
}

const _: [(); core::mem::size_of::<usize>() * 4] =
    [(); core::mem::size_of::<CursorMut<usize, 2>>()];

impl<'a, T, const N: usize> CursorMut<'a, T, N>
where
    Usize<N>: ChunkCapacity,
{
    pub(crate) fn from_front(list: &'a mut ArrayList<T, N>) -> Self {
        Self {
            index: 0,
            chunk_index: 0,
            inner_index: 0,
            list,
        }
    }

    pub(crate) fn from_back(list: &'a mut ArrayList<T, N>) -> Self {
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

    pub fn as_cursor(&self) -> Cursor<'_, T, N> {
        Cursor {
            list: self.list,
            index: self.index,
            chunk_index: self.chunk_index,
            inner_index: self.inner_index,
        }
    }

    pub fn as_list(&self) -> &ArrayList<T, N> {
        self.list
    }

    pub fn back(&self) -> Option<&T> {
        self.list.back()
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.list.back_mut()
    }

    pub fn current(&mut self) -> Option<&mut T> {
        self.list
            .chunks
            .get_mut(self.chunk_index)
            .and_then(|chunk| chunk.get_mut(self.inner_index))
    }

    pub fn front(&self) -> Option<&T> {
        self.list.front()
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.list.front_mut()
    }

    pub fn index(&self) -> Option<usize> {
        if self.is_ghost() {
            return None;
        }

        Some(self.index)
    }

    pub fn insert_after(&mut self, value: T) {
        if self.is_ghost() {
            self.push_front(value);
            return;
        }

        let chunk_index = self.chunk_index;
        let inner_index = self.inner_index;
        self.list.raw_insert(chunk_index, inner_index + 1, value);
    }

    pub fn insert_before(&mut self, value: T) {
        if self.is_ghost() {
            self.push_back(value);
            return;
        }

        let chunk_index = self.chunk_index;
        let inner_index = self.inner_index;

        let chunk = &self.list.chunks[self.chunk_index];
        if (chunk.len() + 1 > N) && (self.index == 0 || self.inner_index + 1 >= N) {
            self.chunk_index += 1;
            self.inner_index = 0;
        } else {
            self.inner_index += 1;
        }

        self.index += 1;
        self.list.raw_insert(chunk_index, inner_index, value);
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

    pub fn peek_next(&mut self) -> Option<&mut T> {
        if self.is_ghost() {
            return self.front_mut();
        }

        if self.inner_index + 1 < self.list.chunks[self.chunk_index].len() {
            return self.list.chunks[self.chunk_index].get_mut(self.inner_index + 1);
        }

        self.list
            .chunks
            .get_mut(self.chunk_index + 1)
            .and_then(VecDeque::front_mut)
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        if self.index == 0 {
            return None;
        }

        if self.inner_index > 0 {
            return Some(&mut self.list.chunks[self.chunk_index][self.inner_index - 1]);
        }

        self.list
            .chunks
            .get_mut(self.chunk_index.saturating_sub(1))
            .and_then(VecDeque::back_mut)
    }

    pub fn push_front(&mut self, value: T) {
        let is_ghost = self.is_ghost();
        let chunks_len_backup = self.list.chunks.len();

        self.list.push_front(value);

        if is_ghost {
            self.index = self.list.len();
            self.chunk_index = self.list.chunks.len();
            self.inner_index = 0;
            return;
        }

        self.index += 1;

        if self.list.chunks.len() > chunks_len_backup {
            self.chunk_index += 1;
        }

        if self.chunk_index == 0 {
            self.inner_index += 1;
        }
    }

    pub fn push_back(&mut self, value: T) {
        let is_ghost = self.is_ghost();

        self.list.push_back(value);

        if is_ghost {
            self.index = self.list.len();
            self.chunk_index = self.list.chunks.len();
            self.inner_index = 0;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let chunks_len_backup = self.list.chunks.len();

        let out = self.list.pop_front();

        self.index = self.index.saturating_sub(1);

        if self.chunk_index == 0 {
            self.inner_index = self.inner_index.saturating_sub(1);
        }

        if self.list.chunks.len() < chunks_len_backup {
            self.chunk_index = self.chunk_index.saturating_sub(1);
        }

        out
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let out = self.list.pop_back();

        if self.is_ghost() {
            self.chunk_index = self.list.chunks.len().saturating_sub(1);
            self.inner_index = self
                .list
                .chunks
                .get(self.chunk_index)
                .map_or(0, VecDeque::len);
        }

        out
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let index = self.index()?;

        let chunk = &self.list.chunks[self.chunk_index];
        if self.inner_index > 0 && self.inner_index + 1 >= chunk.len() {
            self.chunk_index += 1;
            self.inner_index = 0;
        }

        self.list.remove(index)
    }

    #[inline]
    fn is_ghost(&self) -> bool {
        self.index >= self.list.len()
    }
}

impl<T, const N: usize> core::fmt::Debug for CursorMut<'_, T, N>
where
    T: core::fmt::Debug,
    Usize<N>: ChunkCapacity,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CursorMut")
            .field("list", self.list)
            .field("current", &self.as_cursor().current())
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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let ptr = &list as *const _;

            let sut = list.cursor_front_mut();
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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front_mut();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first().copied().as_mut());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());

            for _ in 0..2 {
                for i in 0..seed.len() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i).copied().as_mut());
                    assert_eq!(
                        sut.peek_prev(),
                        seed.get(i.wrapping_sub(1)).copied().as_mut()
                    );
                    assert_eq!(sut.peek_next(), seed.get(i + 1).copied().as_mut());

                    sut.move_next();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last().copied().as_mut());
                assert_eq!(sut.peek_next(), seed.first().copied().as_mut());

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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front_mut();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first().copied().as_mut());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());

            for _ in 0..2 {
                sut.move_prev();
                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last().copied().as_mut());
                assert_eq!(sut.peek_next(), seed.first().copied().as_mut());

                for i in (0..seed.len()).rev() {
                    sut.move_prev();
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i).copied().as_mut());
                    assert_eq!(
                        sut.peek_prev(),
                        seed.get(i.wrapping_sub(1)).copied().as_mut()
                    );
                    assert_eq!(sut.peek_next(), seed.get(i + 1).copied().as_mut());
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

    #[quickcheck]
    fn test_cursor_front_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut list = ArrayList::<i32, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front_mut();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first().copied().as_mut());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());

            let begin = -8;
            for (i, value) in (begin..0).enumerate() {
                sut.insert_before(value);

                assert_eq!(sut.index(), seed.is_empty().not().then_some(i + 1));
                assert_eq!(sut.current(), seed.first().copied().as_mut());
                assert_eq!(sut.peek_prev(), Some(&mut value.clone()));
                if seed.is_empty() {
                    assert_eq!(sut.peek_next(), Some(&mut begin.clone()));
                } else {
                    assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());
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

    #[quickcheck]
    fn test_cursor_front_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut list = ArrayList::<i32, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_front_mut();

            assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
            assert_eq!(sut.current(), seed.first().copied().as_mut());
            assert_eq!(sut.peek_prev(), None);
            assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());

            let begin = -8;
            for value in begin..0 {
                sut.insert_after(value);
                assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
                assert_eq!(sut.current(), seed.first().copied().as_mut());
                if seed.is_empty() {
                    assert_eq!(sut.peek_prev(), Some(&mut begin.clone()));
                } else {
                    assert_eq!(sut.peek_prev(), None);
                }
                assert_eq!(sut.peek_next(), Some(&mut value.clone()));
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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let ptr = &list as *const _;

            let sut = list.cursor_back_mut();
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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back_mut();

            assert_eq!(sut.index(), seed.len().checked_sub(1));
            assert_eq!(sut.current(), seed.last().copied().as_mut());
            assert_eq!(
                sut.peek_prev(),
                seed.get(seed.len().wrapping_sub(2)).copied().as_mut()
            );
            assert_eq!(sut.peek_next(), None);

            sut.move_next();
            assert_eq!(sut.index(), None);
            assert_eq!(sut.current(), None);
            assert_eq!(sut.peek_prev(), seed.last().copied().as_mut());
            assert_eq!(sut.peek_next(), seed.first().copied().as_mut());

            for _ in 0..2 {
                sut.move_next();
                assert_eq!(sut.index(), seed.is_empty().not().then_some(0));
                assert_eq!(sut.current(), seed.first().copied().as_mut());
                assert_eq!(sut.peek_prev(), None);
                assert_eq!(sut.peek_next(), seed.get(1).copied().as_mut());

                for i in 0..seed.len() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), Some(&mut seed[i].clone()));
                    assert_eq!(
                        sut.peek_prev(),
                        seed.get(i.wrapping_sub(1)).copied().as_mut()
                    );
                    assert_eq!(sut.peek_next(), seed.get(i + 1).copied().as_mut());
                    sut.move_next();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last().copied().as_mut());
                assert_eq!(sut.peek_next(), seed.first().copied().as_mut());
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
            let mut list = ArrayList::<_, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back_mut();

            for _ in 0..2 {
                assert_eq!(sut.index(), seed.len().checked_sub(1));
                assert_eq!(sut.current(), seed.last().copied().as_mut());
                assert_eq!(
                    sut.peek_prev(),
                    seed.get(seed.len().wrapping_sub(2)).copied().as_mut()
                );
                assert_eq!(sut.peek_next(), None);

                for i in (0..seed.len()).rev() {
                    assert_eq!(sut.index(), Some(i));
                    assert_eq!(sut.current(), seed.get(i).copied().as_mut());
                    assert_eq!(
                        sut.peek_prev(),
                        seed.get(i.wrapping_sub(1)).copied().as_mut()
                    );
                    assert_eq!(sut.peek_next(), seed.get(i + 1).copied().as_mut());
                    sut.move_prev();
                }

                assert_eq!(sut.index(), None);
                assert_eq!(sut.current(), None);
                assert_eq!(sut.peek_prev(), seed.last().copied().as_mut());
                assert_eq!(sut.peek_next(), seed.first().copied().as_mut());

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

    #[quickcheck]
    fn test_cursor_back_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut list = ArrayList::<i32, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back_mut();

            assert_eq!(sut.index(), seed.len().checked_sub(1));
            assert_eq!(sut.current(), seed.last().copied().as_mut());
            assert_eq!(
                sut.peek_prev(),
                seed.get(seed.len().wrapping_sub(2)).copied().as_mut()
            );
            assert_eq!(sut.peek_next(), None);

            let begin = -8;
            for (i, value) in (begin..0).enumerate() {
                sut.insert_before(value);

                assert_eq!(sut.index(), seed.is_empty().not().then_some(seed.len() + i));
                assert_eq!(sut.current(), seed.last().copied().as_mut());
                assert_eq!(sut.peek_prev(), Some(&mut value.clone()));
                if seed.is_empty() {
                    assert_eq!(sut.peek_next(), Some(&mut begin.clone()));
                } else {
                    assert_eq!(sut.peek_next(), None);
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

    #[quickcheck]
    fn test_cursor_back_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut list = ArrayList::<i32, N>::from_iter(seed.iter().copied());
            let mut sut = list.cursor_back_mut();

            assert_eq!(sut.index(), seed.len().checked_sub(1));
            assert_eq!(sut.current(), seed.last().copied().as_mut());
            assert_eq!(
                sut.peek_prev(),
                seed.get(seed.len().wrapping_sub(2)).copied().as_mut()
            );
            assert_eq!(sut.peek_next(), None);

            let begin = -8;
            for value in begin..0 {
                sut.insert_after(value);
                assert_eq!(sut.index(), seed.len().checked_sub(1));
                assert_eq!(sut.current(), seed.last().copied().as_mut());
                if seed.is_empty() {
                    assert_eq!(sut.peek_prev(), Some(&mut begin.clone()));
                } else {
                    assert_eq!(
                        sut.peek_prev(),
                        seed.get(seed.len().wrapping_sub(2)).copied().as_mut()
                    );
                }
                assert_eq!(sut.peek_next(), Some(&mut value.clone()));
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

    use super::CursorMut;

    #[quickcheck]
    fn nightly_test_cursor_front_move_next(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.pop_front();
            actual.pop_front();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.pop_front();
            actual.pop_front();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.pop_back();
            actual.pop_back();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.pop_back();
            actual.pop_back();

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_eq!(expected.pop_front(), actual.pop_front());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_eq!(expected.pop_back(), actual.pop_back());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_eq!(expected.pop_front(), actual.pop_front());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_eq!(expected.pop_back(), actual.pop_back());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_eq!(expected.pop_front(), actual.pop_front());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            assert_eq!(expected.pop_back(), actual.pop_back());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_pop_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_eq!(expected.pop_front(), actual.pop_front());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_pop_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            assert_eq!(expected.pop_back(), actual.pop_back());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_push_front(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            let value = rand::random();
            expected.push_front(value);
            actual.push_front(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_push_back(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();

            let value = rand::random();
            expected.push_back(value);
            actual.push_back(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_insert_after(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_after(value);
            actual.insert_after(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_insert_before(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            let value = rand::random();
            expected.insert_before(value);
            actual.insert_before(value);
            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_prev_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_front_move_next_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_front_mut();
            let mut actual = array_list.cursor_front_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_prev_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_prev();
            actual.move_prev();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
    fn nightly_test_cursor_back_move_next_remove_current(seed: Vec<i32>) {
        fn _test<const N: usize>(seed: &[i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut linked_list = LinkedList::from_iter(seed.iter().copied());
            let mut array_list = ArrayList::<_, N>::from_iter(seed.iter().copied());

            let mut expected = linked_list.cursor_back_mut();
            let mut actual = array_list.cursor_back_mut();

            assert_cursors_give_same_results(&mut expected, &mut actual);

            expected.move_next();
            actual.move_next();
            assert_cursors_give_same_results(&mut expected, &mut actual);

            assert_eq!(expected.remove_current(), actual.remove_current());

            assert_cursors_give_same_results(&mut expected, &mut actual);
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
        expected: &mut linked_list::CursorMut<'_, i32>,
        actual: &mut CursorMut<'_, i32, N>,
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
        assert_eq!(expected.current(), actual.current(),);

        assert_eq!(expected.front(), actual.front());
        assert_eq!(expected.front_mut(), actual.front_mut());

        assert_eq!(expected.back(), actual.back());
        assert_eq!(expected.back_mut(), actual.back_mut());

        assert_eq!(expected.peek_next(), actual.peek_next());
        assert_eq!(expected.peek_prev(), actual.peek_prev());
    }
}
