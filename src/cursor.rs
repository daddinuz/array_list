use core::ptr::NonNull;

use crate::node::Node;
use crate::sailed::{Array, ConstCast, NonZero, Usize};
use crate::ArrayList;

/// A cursor over a ArrayList.
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.  
/// Cursors always rest between two elements in the list, and index in a logically circular way.  
/// To accommodate this, there is a “ghost” non-element that yields None between the head and tail of the list.
/// When created, cursors start at the front of the list, or the “ghost” non-element if the list is empty.
pub struct Cursor<'a, T, const N: usize>
where
    T: 'a,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    global_index: usize,
    local_index: usize,
    left: Option<NonNull<Node<T, N>>>,
    current: Option<NonNull<Node<T, N>>>,
    list: &'a ArrayList<T, N>,
}

const _: [(); core::mem::size_of::<usize>() * 5] = [(); core::mem::size_of::<Cursor<usize, 2>>()];

impl<'a, T, const N: usize> Cursor<'a, T, N>
where
    T: 'a,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    pub(crate) fn from_front(list: &'a ArrayList<T, N>) -> Self {
        Self {
            global_index: 0,
            local_index: 0,
            left: None,
            current: list.head,
            list,
        }
    }

    pub(crate) fn from_back(list: &'a ArrayList<T, N>) -> Self {
        Self {
            global_index: list.len().saturating_sub(1),
            local_index: list
                .tail
                .map_or(0, |t| unsafe { t.as_ref().len().saturating_sub(1) }),
            left: NonNull::new(
                list.tail.map_or(0, |t| unsafe { t.as_ref().link() }) as *mut Node<T, N>
            ),
            current: list.tail,
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
        self.current
            .and_then(|p| unsafe { p.as_ref().get(self.local_index) })
    }

    pub fn front(&self) -> Option<&'a T> {
        self.list.front()
    }

    pub fn index(&self) -> Option<usize> {
        if self.global_index < self.list.len() {
            return Some(self.global_index);
        }

        None
    }

    pub fn move_next(&mut self) {
        if self.global_index >= self.list.len() {
            *self = Self::from_front(self.list);
            return;
        }

        self.global_index += 1;
        self.local_index += 1;
        if self.local_index >= self.current.map_or(0, |n| unsafe { n.as_ref().len() }) {
            let next = NonNull::new(
                (self.current.map_or(0, |n| unsafe { n.as_ref().link() })
                    ^ self.left.map_or(0, |n| n.as_ptr() as usize))
                    as *mut Node<T, N>,
            );
            self.left = self.current;
            self.current = next;
            self.local_index = 0;
        }
    }

    pub fn move_prev(&mut self) {
        if self.global_index >= self.list.len() {
            *self = Self::from_back(self.list);
            return;
        }

        self.global_index = self.global_index.overflowing_sub(1).0;

        if self.local_index == 0 {
            let prev = NonNull::new(
                (self.left.map_or(0, |n| unsafe { n.as_ref().link() })
                    ^ self.current.map_or(0, |n| n.as_ptr() as usize))
                    as *mut Node<T, N>,
            );
            self.current = self.left;
            self.left = prev;
            self.local_index = self.current.map_or(0, |n| unsafe { n.as_ref().len() } - 1);
        } else {
            self.local_index -= 1;
        }
    }

    pub fn peek_next(&self) -> Option<&'a T> {
        if self.global_index >= self.list.len() {
            return self.list.front();
        }

        let mut node = self.current;
        let mut local_index = self.local_index + 1;

        if local_index >= self.current.map_or(0, |n| unsafe { n.as_ref().len() }) {
            node = NonNull::new(
                (self.current.map_or(0, |n| unsafe { n.as_ref().link() })
                    ^ self.left.map_or(0, |n| n.as_ptr() as usize))
                    as *mut Node<T, N>,
            );
            local_index = 0;
        }

        node.and_then(|p| unsafe { p.as_ref().get(local_index) })
    }

    pub fn peek_prev(&self) -> Option<&'a T> {
        if self.global_index == 0 {
            return None;
        }

        if self.global_index >= self.list.len() {
            return self.list.back();
        }

        let mut node = self.current;
        let mut local_index = self.local_index;

        if local_index == 0 {
            node = self.left;
            local_index = node.map_or(0, |n| unsafe { n.as_ref().len() - 1 });
        } else {
            local_index -= 1;
        }

        node.and_then(|p| unsafe { p.as_ref().get(local_index) })
    }
}

impl<'a, T, const N: usize> Clone for Cursor<'a, T, N>
where
    T: 'a,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T, const N: usize> core::fmt::Debug for Cursor<'_, T, N>
where
    T: core::fmt::Debug,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Cursor")
            .field(self.list)
            .field(&self.global_index)
            .finish()
    }
}

unsafe impl<T, const N: usize> Send for Cursor<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

unsafe impl<T, const N: usize> Sync for Cursor<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

#[cfg(test)]
mod tests {
    use crate::ArrayList;

    #[test]
    fn cursor_from_front_move_next() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_front();
        for i in 0..list.len() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            sut.move_next();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);

        sut.move_next();
        for i in 0..list.len() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            sut.move_next();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);
    }

    #[test]
    fn cursor_from_back_move_next() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_back();

        assert_eq!(sut.index(), Some(4));
        assert_eq!(sut.current(), Some(&4));

        sut.move_next();
        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);

        sut.move_next();
        for i in 0..list.len() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            sut.move_next();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);

        sut.move_next();
        for i in 0..list.len() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            sut.move_next();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);
    }

    #[test]
    fn cursor_from_front_move_prev() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_front();

        assert_eq!(sut.index(), Some(0));
        assert_eq!(sut.current(), Some(&0));

        sut.move_prev();
        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);

        for i in (0..list.len()).rev() {
            sut.move_prev();
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
        }

        sut.move_prev();
        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);

        for i in (0..list.len()).rev() {
            sut.move_prev();
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
        }
    }

    #[test]
    fn cursor_from_front_peek_next() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_front();

        for i in 0..list.len() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            if i + 1 < list.len() {
                assert_eq!(sut.peek_next(), Some(&(i + 1)));
            }
            sut.move_next();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);
        assert_eq!(sut.peek_next(), Some(&0));

        for i in 0..list.len() {
            sut.move_next();
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            if i + 1 < list.len() {
                assert_eq!(sut.peek_next(), Some(&(i + 1)));
            }
        }
    }

    #[test]
    fn cursor_from_back_peek_next() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_back();

        for i in (0..list.len()).rev() {
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            if i + 1 < list.len() {
                assert_eq!(sut.peek_next(), Some(&(i + 1)));
            }
            sut.move_prev();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);
        assert_eq!(sut.peek_next(), Some(&0));

        for i in (0..list.len()).rev() {
            sut.move_prev();
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            if i + 1 < list.len() {
                assert_eq!(sut.peek_next(), Some(&(i + 1)));
            }
        }
    }

    #[test]
    fn cursor_from_front_peek_prev() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_back();

        for i in (0..list.len()).rev() {
            dbg!(i);
            assert_eq!(sut.index(), Some(i));
            assert_eq!(sut.current(), Some(&i));
            if i > 0 {
                assert_eq!(sut.peek_prev(), Some(&(i - 1)));
            }
            sut.move_prev();
        }

        assert_eq!(sut.index(), None);
        assert_eq!(sut.current(), None);
        assert_eq!(sut.peek_prev(), Some(&4));
    }

    #[test]
    fn cursor_as_list() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.cursor_back();

        assert_eq!(&list, sut.as_list());

        sut.move_next();
        assert_eq!(&list, sut.as_list());

        sut.back();
        assert_eq!(&list, sut.as_list());

        sut.move_next();
        assert_eq!(&list, sut.as_list());

        sut.front();
        assert_eq!(&list, sut.as_list());
    }

    #[test]
    fn clone_works_correctly() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let base = list.cursor_front();
        assert_eq!(base.peek_prev(), None);
        assert_eq!(base.current(), Some(&0));
        assert_eq!(base.peek_next(), Some(&1));

        let mut sut = base.clone();
        assert_eq!(sut.peek_prev(), None);
        assert_eq!(sut.current(), Some(&0));
        assert_eq!(sut.peek_next(), Some(&1));

        sut.move_next();

        assert_eq!(sut.peek_prev(), Some(&0));
        assert_eq!(sut.current(), Some(&1));
        assert_eq!(sut.peek_next(), Some(&2));

        assert_eq!(base.peek_prev(), None);
        assert_eq!(base.current(), Some(&0));
        assert_eq!(base.peek_next(), Some(&1));
    }

    #[test]
    fn debug_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let list = ArrayList::<usize, 2>::from(array);

        let sut = list.cursor_front();
        assert_eq!(format!("{sut:?}"), format!("Cursor({:?}, {})", array, 0));

        let sut = list.cursor_back();
        assert_eq!(
            format!("{sut:?}"),
            format!("Cursor({:?}, {})", array, array.len() - 1)
        );
    }

    /* TODO: uncomment this test when LinkedList's cursor becomes stable
    use std::collections::LinkedList;

    #[test]
    fn array_list_cursor_is_aligned_with_linked_list_cursor() {
        let array = [0, 1, 2, 3, 4];
        let array_list = ArrayList::<i32, 2>::from(array);
        let linked_list = LinkedList::from(array);

        let mut array_list_cursor = array_list.cursor_front();
        let mut linked_list_cursor = linked_list.cursor_front();

        for _ in 0..(array.len() * 2) {
            assert_eq!(
                array_list_cursor.peek_prev(),
                linked_list_cursor.peek_prev()
            );
            assert_eq!(array_list_cursor.current(), linked_list_cursor.current());
            assert_eq!(
                array_list_cursor.peek_next(),
                linked_list_cursor.peek_next()
            );
            array_list_cursor.move_next();
            linked_list_cursor.move_next();
        }

        for _ in 0..(array.len() * 2) {
            assert_eq!(
                array_list_cursor.peek_prev(),
                linked_list_cursor.peek_prev()
            );
            assert_eq!(array_list_cursor.current(), linked_list_cursor.current());
            assert_eq!(
                array_list_cursor.peek_next(),
                linked_list_cursor.peek_next()
            );
            array_list_cursor.move_prev();
            linked_list_cursor.move_prev();
        }
    }
    */
}
