use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use crate::node::Node;
use crate::sailed::{Array, ConstCast, NonZero, Usize};
use crate::ArrayList;

/// An iterator over the elements of a ArrayList.
///
/// This struct is created by ArrayList::iter_mut().
pub struct IterMut<'a, T, const N: usize>
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

    marker: PhantomData<&'a mut Node<T, N>>,
}

const _: [(); core::mem::size_of::<usize>() * 6] = [(); core::mem::size_of::<IterMut<usize, 2>>()];

impl<T, const N: usize> Default for IterMut<'_, T, N>
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

impl<T, const N: usize> IterMut<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    pub(crate) fn from_list(list: &mut ArrayList<T, N>) -> Self {
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

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    type Item = &'a mut T;

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

        let mut front = self.front.unwrap();
        let node = unsafe { front.as_mut() };
        let out = node.get_mut(self.front_index as usize);
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

impl<T, const N: usize> DoubleEndedIterator for IterMut<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let current_index = self.back_index as usize;
        let current = unsafe { self.back.unwrap().as_mut() };

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
        current.get_mut(current_index)
    }
}

impl<T, const N: usize> ExactSizeIterator for IterMut<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

impl<T, const N: usize> FusedIterator for IterMut<'_, T, N>
where
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

impl<T, const N: usize> core::fmt::Debug for IterMut<'_, T, N>
where
    T: core::fmt::Debug,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("IterMut")
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

unsafe impl<T, const N: usize> Send for IterMut<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

unsafe impl<T, const N: usize> Sync for IterMut<'_, T, N>
where
    T: Sync,
    [T; N]: Array,
    Usize<N>: NonZero + ConstCast<u16>,
{
}

#[cfg(test)]
mod tests {
    use crate::ArrayList;

    use super::IterMut;

    #[test]
    fn default_iterator_yelds_nothing() {
        let mut sut: IterMut<i32, 2> = Default::default();
        assert_eq!(sut.len(), 0);
        assert_eq!(sut.next(), None);
        assert_eq!(sut.next_back(), None);
    }

    #[test]
    fn iter_forward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter_mut();
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.next(), Some(&mut 0));
        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.next(), Some(&mut 3));
        assert_eq!(sut.next(), Some(&mut 4));

        list.clear();
        let mut sut = list.iter_mut();
        assert_eq!(sut.len(), 0);
        assert_eq!(sut.next(), None);
    }

    #[test]
    fn iter_backward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter_mut().rev();
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.next(), Some(&mut 4));
        assert_eq!(sut.next(), Some(&mut 3));
        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.next(), Some(&mut 0));

        list.clear();
        let mut sut = list.iter_mut().rev();
        assert_eq!(sut.len(), 0);
        assert_eq!(sut.next(), None);
    }

    #[test]
    fn double_ended_iterator_works_correctly() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter_mut();
        assert_eq!(sut.len(), 5);

        assert_eq!(sut.next(), Some(&mut 0));
        assert_eq!(sut.len(), 4);

        assert_eq!(sut.next_back(), Some(&mut 4));
        assert_eq!(sut.len(), 3);

        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.len(), 2);

        assert_eq!(sut.next_back(), Some(&mut 3));
        assert_eq!(sut.len(), 1);

        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.next_back(), None);
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.next(), None);
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn last_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let mut list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter_mut();
        assert_eq!(sut.last(), Some(&mut 4));
    }

    #[test]
    fn debug_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let mut list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter_mut();
        assert_eq!(
            format!("{sut:?}"),
            format!("IterMut({:?}, {})", array, array.len())
        );
    }

    #[test]
    fn miri_complains() {
        let array = [0, 1, 2, 3, 4];
        let mut list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter_mut();
        let mut vec: Vec<_> = sut.collect();
        *vec[0] += 1;
    }
}
