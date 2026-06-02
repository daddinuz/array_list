use alloc::collections::vec_deque;

use core::iter::{Flatten, FusedIterator};

use cap_vec::CapVec;

use crate::ArrayList;

type Delegate<'a, T, const N: usize> = Flatten<vec_deque::IterMut<'a, CapVec<T, N>>>;

/// A mutable iterator over an [`ArrayList`].
///
/// `IterMut` yields mutable references in logical list order. It can also
/// consume elements from the back.
///
/// This struct is created by [`ArrayList::iter_mut`].
///
/// # Example
/// ```rust
/// use array_list::ArrayList;
///
/// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3]);
/// let mut iter = list.iter_mut();
///
/// if let Some(value) = iter.next() {
///     *value *= 10;
/// }
///
/// if let Some(value) = iter.next_back() {
///     *value *= 10;
/// }
///
/// if let Some(value) = iter.next() {
///     *value *= 10;
/// }
///
/// assert_eq!(iter.next(), None);
/// assert_eq!(list, [10, 20, 30]);
/// ```
///
/// ```compile_fail
/// use array_list::IterMut;
///
/// let _ = IterMut::<i32, 3>::default();
/// ```
pub struct IterMut<'a, T, const N: usize> {
    delegate: Delegate<'a, T, N>,
}

const _: [(); core::mem::size_of::<usize>() * 8] = [(); core::mem::size_of::<IterMut<usize, 4>>()];

impl<T, const N: usize> Default for IterMut<'_, T, N> {
    fn default() -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            delegate: Default::default(),
        }
    }
}

impl<'a, T, const N: usize> IterMut<'a, T, N> {
    pub(crate) fn from_list(list: &'a mut ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            delegate: list.chunks.iter_mut().flatten(),
        }
    }
}

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N> {
    type Item = &'a mut T;

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

impl<T, const N: usize> DoubleEndedIterator for IterMut<'_, T, N> {
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

impl<T, const N: usize> FusedIterator for IterMut<'_, T, N> {}

impl<T, const N: usize> core::fmt::Debug for IterMut<'_, T, N>
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

    fn values<const N: usize>(list: &ArrayList<i32, N>) -> Vec<i32> {
        list.iter().copied().collect()
    }

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    #[test]
    fn mutates_items_from_both_ends_across_chunks() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3, 4, 5]);

            {
                let mut iter = list.iter_mut();
                *iter.next().unwrap() *= 10;
                *iter.next_back().unwrap() *= 10;
                *iter.nth(1).unwrap() *= 10;
            }

            assert_eq!(values(&list), [10, 2, 30, 4, 50]);
        }

        check_capacities!(check);
    }

    #[test]
    fn default_iterator_is_empty_and_fused() {
        fn check<const N: usize>() {
            let mut iter = super::IterMut::<i32, N>::default();

            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        check_capacities!(check);
    }

    #[test]
    fn into_iterator_for_mutable_list_yields_mutable_references() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);

            for value in &mut list {
                *value += 1;
            }

            assert_eq!(values(&list), [2, 3, 4]);
        }

        check_capacities!(check);
    }

    #[test]
    fn debug_formats_remaining_mutable_items() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut iter = list.iter_mut();

            assert_eq!(iter.next(), Some(&mut 1));

            let debug = alloc::format!("{iter:?}");
            assert!(debug.contains("Flatten"));
            assert!(debug.contains('2'));
            assert!(debug.contains('3'));
        }

        check_capacities!(check);
    }

    #[test]
    fn mutable_iteration_order_is_stable_for_required_capacities() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

            for value in list.iter_mut() {
                *value += 10;
            }

            let mut iter = list.iter_mut();
            *iter.next().unwrap() += 100;
            *iter.next_back().unwrap() += 100;
            *iter.nth(2).unwrap() += 100;
            *iter.nth_back(1).unwrap() += 100;

            assert_eq!(values(&list), [110, 11, 12, 113, 14, 15, 16, 117, 18, 119]);
        }

        check_capacities!(check);
    }
}
