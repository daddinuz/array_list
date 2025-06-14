use std::collections::{VecDeque, vec_deque};
use std::iter::Flatten;
use std::iter::FusedIterator;

use crate::{ArrayList, ChunkCapacity, Usize};

/// An iterator over the elements of a ArrayList.
///
/// This struct is created by ArrayList::iter_mut().
#[derive(Default)]
pub struct IterMut<'a, T, const N: usize>
where
    Usize<N>: ChunkCapacity,
{
    delegate: Flatten<vec_deque::IterMut<'a, VecDeque<T>>>,
}

const _: [(); core::mem::size_of::<usize>() * 12] = [(); core::mem::size_of::<IterMut<usize, 2>>()];

impl<'a, T, const N: usize> IterMut<'a, T, N>
where
    Usize<N>: ChunkCapacity,
{
    pub(crate) fn from_list(list: &'a mut ArrayList<T, N>) -> Self {
        Self {
            delegate: list.chunks.iter_mut().flatten(),
        }
    }
}

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N>
where
    Usize<N>: ChunkCapacity,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.delegate.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.delegate.size_hint()
    }

    fn last(self) -> Option<Self::Item> {
        self.delegate.last()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.delegate.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.delegate.nth(n)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        self.delegate.for_each(f);
    }

    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        self.delegate.collect()
    }

    fn partition<B, F>(self, f: F) -> (B, B)
    where
        Self: Sized,
        B: Default + Extend<Self::Item>,
        F: FnMut(&Self::Item) -> bool,
    {
        self.delegate.partition(f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.delegate.fold(init, f)
    }

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        self.delegate.reduce(f)
    }

    fn all<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.delegate.all(f)
    }

    fn any<F>(&mut self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.delegate.any(f)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.delegate.find(predicate)
    }

    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.delegate.find_map(f)
    }

    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        self.delegate.position(predicate)
    }

    fn max(self) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        self.delegate.max()
    }

    fn min(self) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        self.delegate.min()
    }

    fn max_by_key<B: Ord, F>(self, f: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> B,
    {
        self.delegate.max_by_key(f)
    }

    fn max_by<F>(self, compare: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> std::cmp::Ordering,
    {
        self.delegate.max_by(compare)
    }

    fn min_by_key<B: Ord, F>(self, f: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> B,
    {
        self.delegate.min_by_key(f)
    }

    fn min_by<F>(self, compare: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> std::cmp::Ordering,
    {
        self.delegate.min_by(compare)
    }

    fn sum<S>(self) -> S
    where
        Self: Sized,
        S: std::iter::Sum<Self::Item>,
    {
        self.delegate.sum()
    }

    fn product<P>(self) -> P
    where
        Self: Sized,
        P: std::iter::Product<Self::Item>,
    {
        self.delegate.product()
    }

    fn cmp<I>(self, other: I) -> std::cmp::Ordering
    where
        I: IntoIterator<Item = Self::Item>,
        Self::Item: Ord,
        Self: Sized,
    {
        self.delegate.cmp(other)
    }

    fn partial_cmp<I>(self, other: I) -> Option<std::cmp::Ordering>
    where
        I: IntoIterator,
        Self::Item: PartialOrd<I::Item>,
        Self: Sized,
    {
        self.delegate.partial_cmp(other)
    }

    fn eq<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialEq<I::Item>,
        Self: Sized,
    {
        self.delegate.eq(other)
    }

    fn ne<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialEq<I::Item>,
        Self: Sized,
    {
        self.delegate.ne(other)
    }

    fn lt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialOrd<I::Item>,
        Self: Sized,
    {
        self.delegate.lt(other)
    }

    fn le<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialOrd<I::Item>,
        Self: Sized,
    {
        self.delegate.le(other)
    }

    fn gt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialOrd<I::Item>,
        Self: Sized,
    {
        self.delegate.gt(other)
    }

    fn ge<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Self::Item: PartialOrd<I::Item>,
        Self: Sized,
    {
        self.delegate.ge(other)
    }

    fn is_sorted(self) -> bool
    where
        Self: Sized,
        Self::Item: PartialOrd,
    {
        self.delegate.is_sorted()
    }

    fn is_sorted_by<F>(self, compare: F) -> bool
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> bool,
    {
        self.delegate.is_sorted_by(compare)
    }

    fn is_sorted_by_key<F, K>(self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> K,
        K: PartialOrd,
    {
        self.delegate.is_sorted_by_key(f)
    }
}

impl<T, const N: usize> DoubleEndedIterator for IterMut<'_, T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.delegate.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.delegate.nth_back(n)
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.delegate.rfold(init, f)
    }

    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.delegate.rfind(predicate)
    }
}

impl<T, const N: usize> FusedIterator for IterMut<'_, T, N> where Usize<N>: ChunkCapacity {}

impl<T, const N: usize> core::fmt::Debug for IterMut<'_, T, N>
where
    T: core::fmt::Debug,
    Usize<N>: ChunkCapacity,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.delegate)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use quickcheck_macros::quickcheck;

    use crate::{ArrayList, ChunkCapacity, Usize};

    use super::IterMut;

    #[test]
    fn test_default_iterator_yields_nothing() {
        let mut sut: IterMut<i32, 2> = Default::default();
        assert_eq!(sut.next(), None);
        assert_eq!(sut.next_back(), None);
    }

    #[test]
    fn test_iter_forward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter_mut();
        assert_eq!(sut.next(), Some(&mut 0));
        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.next(), Some(&mut 3));
        assert_eq!(sut.next(), Some(&mut 4));

        list.clear();
        let mut sut = list.iter_mut();
        assert_eq!(sut.next(), None);
    }

    #[test]
    fn test_iter_backward() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);

        let mut sut = list.iter_mut().rev();
        assert_eq!(sut.next(), Some(&mut 4));
        assert_eq!(sut.next(), Some(&mut 3));
        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.next(), Some(&mut 0));

        list.clear();
        let mut sut = list.iter_mut().rev();
        assert_eq!(sut.next(), None);
    }

    #[test]
    fn test_double_ended_iterator_works_correctly() {
        let mut list = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut sut = list.iter_mut();

        assert_eq!(sut.next(), Some(&mut 0));
        assert_eq!(sut.next_back(), Some(&mut 4));
        assert_eq!(sut.next(), Some(&mut 1));
        assert_eq!(sut.next_back(), Some(&mut 3));
        assert_eq!(sut.next(), Some(&mut 2));
        assert_eq!(sut.next_back(), None);
        assert_eq!(sut.next(), None);
    }

    #[test]
    fn test_last_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let mut list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter_mut();
        assert_eq!(sut.last(), Some(&mut 4));
    }

    #[test]
    fn test_miri_complains() {
        let array = [0, 1, 2, 3, 4];
        let mut list = ArrayList::<usize, 2>::from(array);
        let sut = list.iter_mut();
        let mut vec: Vec<_> = sut.collect();
        *vec[0] += 1;
        assert_eq!(list.get(0), Some(&1));
        assert_eq!(list.get(1), Some(&1));
    }

    #[quickcheck]
    fn nightly_test_iter_mut_behavioural(mut seed: Vec<i32>) {
        fn _test<const N: usize>(expected: &mut [i32])
        where
            Usize<N>: ChunkCapacity,
        {
            let mut actual = ArrayList::<_, N>::new();
            actual.extend(expected.iter().copied());

            assert!(actual.iter_mut().eq(expected.iter_mut()));
            assert!(actual.iter_mut().rev().eq(expected.iter_mut().rev()));
            assert_eq!(
                actual.iter_mut().partial_cmp(expected.iter_mut()),
                Some(Ordering::Equal)
            );
            assert_eq!(actual.iter_mut().count(), expected.iter_mut().count());
            assert_eq!(actual.iter_mut().max(), expected.iter_mut().max());
            assert_eq!(actual.iter_mut().min(), expected.iter_mut().min());
            assert_eq!(
                actual.iter_mut().is_sorted(),
                expected.iter_mut().is_sorted()
            );
            assert_eq!(
                actual.iter_mut().map(|n| *n).collect::<ArrayList<_, N>>(),
                actual
            );
        }

        _test::<1>(&mut seed);
        _test::<2>(&mut seed);
        _test::<3>(&mut seed);
        _test::<4>(&mut seed);
        _test::<5>(&mut seed);
        _test::<8>(&mut seed);
        _test::<16>(&mut seed);
        _test::<32>(&mut seed);
        _test::<64>(&mut seed);
        _test::<128>(&mut seed);
        _test::<256>(&mut seed);
        _test::<512>(&mut seed);
    }
}
