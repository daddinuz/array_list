use alloc::collections::vec_deque;

use core::iter::{Flatten, FusedIterator};

use cap_vec::CapVec;

use crate::ArrayList;

type Delegate<'a, T, const N: usize> = Flatten<vec_deque::Iter<'a, CapVec<T, N>>>;

/// A read-only iterator over an [`ArrayList`].
///
/// `Iter` yields shared references in logical list order. It can also consume
/// elements from the back.
///
/// This struct is created by [`ArrayList::iter`].
///
/// # Example
/// ```rust
/// use array_list::ArrayList;
///
/// let list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3]);
/// let mut iter = list.iter();
///
/// assert_eq!(iter.next(), Some(&1));
/// assert_eq!(iter.next_back(), Some(&3));
/// assert_eq!(iter.next(), Some(&2));
/// assert_eq!(iter.next(), None);
/// assert_eq!(iter.next_back(), None);
/// ```
///
/// ```compile_fail
/// use array_list::Iter;
///
/// let _ = Iter::<i32, 3>::default();
/// ```
pub struct Iter<'a, T, const N: usize> {
    delegate: Delegate<'a, T, N>,
}

const _: [(); core::mem::size_of::<usize>() * 8] = [(); core::mem::size_of::<Iter<usize, 4>>()];

impl<T, const N: usize> Default for Iter<'_, T, N> {
    fn default() -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            delegate: Default::default(),
        }
    }
}

impl<'a, T, const N: usize> Iter<'a, T, N> {
    pub(crate) fn from_list(list: &'a ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            delegate: list.chunks.iter().flatten(),
        }
    }
}

impl<T, const N: usize> Clone for Iter<'_, T, N> {
    fn clone(&self) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            delegate: self.delegate.clone(),
        }
    }
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.nth(n)
    }

    fn last(self) -> Option<Self::Item> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.last()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.fold(init, f)
    }

    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.for_each(f);
    }
}

impl<T, const N: usize> DoubleEndedIterator for Iter<'_, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.nth_back(n)
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.rfold(init, f)
    }
}

impl<T, const N: usize> FusedIterator for Iter<'_, T, N> {}

impl<T, const N: usize> core::fmt::Debug for Iter<'_, T, N>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.delegate.fmt(f)
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::ArrayList;
    use alloc::vec::Vec;

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    #[test]
    fn iterates_forward_and_backward_across_chunks() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3, 4]);
            let mut iter = list.iter();

            assert_eq!(iter.size_hint(), (0, None));
            assert_eq!(iter.next(), Some(&1));
            assert_eq!(iter.next_back(), Some(&4));
            assert_eq!(iter.nth(1), Some(&3));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        check_capacities!(check);
    }

    #[test]
    fn clone_preserves_remaining_iteration_state() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3, 4, 5]);
            let mut iter = list.iter();

            assert_eq!(iter.next(), Some(&1));

            let cloned = iter.clone();
            assert_eq!(iter.copied().collect::<Vec<_>>(), [2, 3, 4, 5]);
            assert_eq!(cloned.copied().collect::<Vec<_>>(), [2, 3, 4, 5]);
        }

        check_capacities!(check);
    }

    #[test]
    fn default_iterator_is_empty_and_fused() {
        fn check<const N: usize>() {
            let mut iter = super::Iter::<i32, N>::default();

            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        check_capacities!(check);
    }

    #[test]
    fn debug_formats_remaining_items() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut iter = list.iter();

            assert_eq!(iter.next(), Some(&1));

            let debug = alloc::format!("{iter:?}");
            assert!(debug.contains("Flatten"));
            assert!(debug.contains('2'));
            assert!(debug.contains('3'));
        }

        check_capacities!(check);
    }

    #[test]
    fn iteration_order_is_stable_for_required_capacities() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

            assert_eq!(
                list.iter().copied().collect::<Vec<_>>(),
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            );

            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(&0));
            assert_eq!(iter.next_back(), Some(&9));
            assert_eq!(iter.nth(2), Some(&3));
            assert_eq!(iter.nth_back(1), Some(&7));
            assert_eq!(iter.copied().collect::<Vec<_>>(), [4, 5, 6]);
        }

        check_capacities!(check);
    }
}
