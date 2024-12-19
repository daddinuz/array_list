use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use crate::node::Node;
use crate::sailed::{Array, ConstCast, NonZero, Usize};
use crate::ArrayList;

/// An iterator over the elements of a ArrayList.
///
/// This struct is created by ArrayList::iter().
pub struct Iter<'a, T, const N: usize>
where
    T: 'a,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    front_prev: Option<NonNull<Node<T, N>>>,
    front: Option<NonNull<Node<T, N>>>,

    back: Option<NonNull<Node<T, N>>>,
    back_prev: Option<NonNull<Node<T, N>>>,

    len: usize,
    front_index: u16,
    back_index: u16,

    marker: PhantomData<&'a Node<T, N>>,
}

const _: [(); core::mem::size_of::<usize>() * 6] = [(); core::mem::size_of::<Iter<usize, 2>>()];

impl<T, const N: usize> Default for Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn default() -> Self {
        Self {
            front_prev: None,
            front: None,
            back: None,
            back_prev: None,
            len: 0,
            front_index: 0,
            back_index: 0,
            marker: PhantomData,
        }
    }
}

impl<T, const N: usize> Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    pub(crate) fn from_list(list: &ArrayList<T, N>) -> Self {
        Self {
            front_prev: None,
            front: list.head,
            back: list.tail,
            back_prev: None,
            len: list.len,
            front_index: 0,
            back_index: list.tail.map_or(0, |node| unsafe {
                node.as_ref().len().saturating_sub(1) as u16
            }),
            marker: PhantomData,
        }
    }
}

impl<T, const N: usize> Clone for Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let front = self.front.unwrap();
        let node = unsafe { front.as_ref() };
        if (self.front_index as usize) >= node.len() {
            let new_front = NonNull::new(
                (node.link() ^ self.front_prev.map_or(0, |node| node.as_ptr() as usize))
                    as *mut Node<T, N>,
            );

            self.front_index = 0;
            self.front_prev = self.front;
            self.front = new_front;
        }

        let front = self.front.unwrap();
        let node = unsafe { front.as_ref() };
        let out = node.get(self.front_index as usize);
        self.front_index += 1;
        self.len -= 1;
        out
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T, const N: usize> DoubleEndedIterator for Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let current_index = self.back_index as usize;
        let current = unsafe { self.back.unwrap().as_ref() };

        if current_index == 0 {
            let new_back = NonNull::new(
                (current.link() ^ self.back_prev.map_or(0, |node| node.as_ptr() as usize))
                    as *mut Node<T, N>,
            );

            self.back_index = new_back.map_or(0, |node| unsafe { node.as_ref().len() as u16 });
            self.back_prev = self.back;
            self.back = new_back;
        }

        self.back_index = self.back_index.saturating_sub(1);
        self.len -= 1;
        current.get(current_index)
    }
}

impl<T, const N: usize> ExactSizeIterator for Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<T, const N: usize> FusedIterator for Iter<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

impl<T, const N: usize> core::fmt::Debug for Iter<'_, T, N>
where
    T: core::fmt::Debug,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Iter")
            .field(&*ManuallyDrop::new(ArrayList {
                head: self.front,
                tail: self.back,
                len: self.len,
                marker: PhantomData,
            }))
            .field(&self.len)
            .finish()
    }
}

unsafe impl<T, const N: usize> Send for Iter<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

unsafe impl<T, const N: usize> Sync for Iter<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

#[cfg(test)]
mod tests {
    use crate::ArrayList;

    use super::Iter;

    #[test]
    fn default_iterator_yelds_nothing() {
        let mut sut: Iter<i32, 2> = Default::default();
        assert_eq!(sut.len(), 0);
        assert_eq!(sut.next(), None);
        assert_eq!(sut.next_back(), None);
    }

    #[test]
    fn iter_forward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let sut = list.iter();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[0, 1, 2, 3, 4]);

        list.clear();
        let sut = list.iter();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn iter_backward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let sut = list.iter().rev();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[4, 3, 2, 1, 0]);

        list.clear();
        let sut = list.iter().rev();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn double_ended_iterator_works_correctly() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter();
        assert_eq!(sut.len(), 5);

        assert_eq!(sut.next(), Some(&0));
        assert_eq!(sut.len(), 4);

        assert_eq!(sut.next_back(), Some(&4));
        assert_eq!(sut.len(), 3);

        assert_eq!(sut.next(), Some(&1));
        assert_eq!(sut.len(), 2);

        assert_eq!(sut.next_back(), Some(&3));
        assert_eq!(sut.len(), 1);

        assert_eq!(sut.next(), Some(&2));
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.next_back(), None);
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.next(), None);
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn last_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter();
        assert_eq!(sut.last(), Some(&4));
    }

    #[test]
    fn clone_works_correctly() {
        let list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut base = list.iter();

        let sut = base.clone();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[0, 1, 2, 3, 4]);

        base.next();

        let sut = base.clone();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[1, 2, 3, 4]);

        base.next_back();

        let sut = base.clone();
        assert_eq!(&sut.copied().collect::<Vec<_>>(), &[1, 2, 3]);
    }

    #[test]
    fn debug_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter();
        assert_eq!(
            format!("{sut:?}"),
            format!("Iter({:?}, {})", array, array.len())
        );
    }
}
