//! # array_list
//!
//! `array_list` implements an **unrolled linked list** -like datastructure with features
//! that combine the simplicity of a `Vec` and the flexibility of a `LinkedList`.
//!
//! ## Features
//! - Ordered sequence with index based elements access and
//!   efficient random access lookups.
//! - Chunked storage, which improves cache locality and reduces
//!   pointer overhead compared to traditional linked lists.
//! - Stable `Cursor` API similar to the `LinkedList` one on nightly, allowing
//!   efficient operations around a point in the list.
//! - Dynamic growth, balancing between `Vec` and `LinkedList` characteristics.
//!
//! ## Use Cases
//! `array_list` is ideal for scenarios where:
//! - You need a ordered collection with random access capabilities.
//! - You require frequent insertions and deletions anywhere in the list.
//! - Memory efficiency and improved cache performance over plain LinkedList are priorities.
//!
//! ## Note
//! This crate is not related to Java's `ArrayList` despite its name.  
//! The design and functionality are entirely tailored to Rust's ecosystem.
//!
//! ## Example
//! ```rust
//! use array_list::ArrayList;
//!
//! let mut list: ArrayList<i64, 6> = ArrayList::new();
//! list.push_back(2);
//! list.push_front(0);
//! list.insert(1, 1);
//!
//! assert_eq!(list.front(), Some(&0));
//! assert_eq!(list.get(1), Some(&1));
//! assert_eq!(list.back(), Some(&2));
//!
//! assert_eq!(list.remove(1), Some(1));
//! assert_eq!(list.pop_back(), Some(2));
//! assert_eq!(list.pop_front(), Some(0));
//! ```

#![cfg_attr(feature = "nightly_tests", feature(linked_list_cursors))]

mod cursor;
mod cursor_mut;
mod into_iter;
mod iter;
mod iter_mut;
mod sailed;

pub use cursor::Cursor;
pub use cursor_mut::CursorMut;
pub use iter::Iter;
pub use iter_mut::IterMut;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use crate::into_iter::IntoIter;

pub enum Usize<const N: usize> {}

pub trait ChunkCapacity: crate::sailed::Sailed {}

/// A dynamic container that combines the characteristics of a `Vec` and a `LinkedList`.
///
/// # Features
/// - **Chunked Storage**: Each chunk can hold up to `N` elements, reducing the overhead of
///   individual allocations compared to a traditional linked list.
/// - **Flexible Operations**: Index based lookups and efficient insertions, deletions, and access
///   at arbitrary positions.
///
/// # Type Parameters
/// - `T`: The type of elements stored in the list.
/// - `N`: The maximum number of elements that each chunk can hold.
///
/// # Example
/// ```rust
/// use array_list::ArrayList;
///
/// let mut list: ArrayList<i64, 6> = ArrayList::new();
/// list.push_back(3);
/// list.push_front(1);
/// list.insert(1, 2);
///
/// assert!(!list.is_empty());
/// assert_eq!(list.len(), 3);
///
/// assert_eq!(list.pop_front(), Some(1));
/// assert_eq!(list.pop_front(), Some(2));
/// assert_eq!(list.pop_front(), Some(3));
/// ```
pub struct ArrayList<T, const N: usize>
where
    Usize<N>: ChunkCapacity,
{
    chunks: VecDeque<VecDeque<T>>,
    len: usize,
}

impl<T, const N: usize, const M: usize> From<[T; M]> for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn from(values: [T; M]) -> Self {
        values.into_iter().collect()
    }
}

impl<T, const N: usize> FromIterator<T> for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        this.extend(iter);
        this
    }
}

impl<T, const N: usize> Extend<T> for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = &mut iter.into_iter().peekable();

        if let Some(chunk) = self.chunks.back_mut() {
            chunk.extend(iter.take(N - chunk.len()));
        }

        while iter.peek().is_some() {
            let mut chunk = VecDeque::with_capacity(N);
            chunk.extend(iter.take(N));

            self.len += chunk.len();
            self.chunks.push_back(chunk);
        }
    }
}

impl<'a, T, const N: usize> Extend<&'a T> for ArrayList<T, N>
where
    T: Clone,
    Usize<N>: ChunkCapacity,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T, const N: usize> Default for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    /// Creates a new, empty `ArrayList` with no elements and no allocated chunks.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i64, 6> = ArrayList::new();
    ///
    /// assert!(list.is_empty());
    /// ```
    pub const fn new() -> Self {
        Self {
            chunks: VecDeque::new(),
            len: 0,
        }
    }

    /// Adds an element to the front of the `ArrayList`.
    ///
    /// The element is inserted at the beginning of the list, shifting existing elements
    /// forward if necessary. If the first chunk is full, a new one will be allocated to
    /// accommodate the element.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 6> = ArrayList::new();
    /// list.push_front(10);
    /// list.push_front(20);
    ///
    /// assert_eq!(list.len(), 2);
    ///
    /// assert_eq!(list.pop_front(), Some(20));
    /// assert_eq!(list.pop_front(), Some(10));
    /// ```
    pub fn push_front(&mut self, value: T) {
        match self.chunks.front_mut() {
            Some(chunk) if chunk.len() < N => chunk.push_front(value),
            _ => {
                let mut chunk = VecDeque::with_capacity(N);
                chunk.push_front(value);
                self.chunks.push_front(chunk)
            }
        }

        self.len += 1;
    }

    /// Adds an element to the back of the `ArrayList`.
    ///
    /// The element is inserted at the end of the list.
    /// If the last chunk is full, a new one will be allocated to
    /// accommodate the element.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 6> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.len(), 2);
    ///
    /// assert_eq!(list.pop_back(), Some(20));
    /// assert_eq!(list.pop_back(), Some(10));
    /// ```
    pub fn push_back(&mut self, value: T) {
        match self.chunks.back_mut() {
            Some(chunk) if chunk.len() < N => chunk.push_back(value),
            _ => {
                let mut chunk = VecDeque::with_capacity(N);
                chunk.push_front(value);
                self.chunks.push_back(chunk)
            }
        }

        self.len += 1;
    }

    /// Inserts an element at the specified index, shifting subsequent elements to the right.
    /// If the target chunk is full, a new one will be allocated to accommodate the element.
    ///
    /// # Panics
    /// - Panics if the `index` is out of bounds (greater than the list's current length).
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 3> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(30);
    /// list.insert(1, 20);
    ///
    /// assert_eq!(list.get(0), Some(&10));
    /// assert_eq!(list.get(1), Some(&20));
    /// assert_eq!(list.get(2), Some(&30));
    /// ```
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());

        let SearchTarget {
            chunk_index,
            target_index,
        } = self.search_target(index).unwrap_or_else(|| SearchTarget {
            chunk_index: self.chunks.len().saturating_sub(1),
            target_index: self.chunks.back().map(VecDeque::len).unwrap_or(0),
        });

        self.raw_insert(chunk_index, target_index, value);
    }

    fn raw_insert(&mut self, chunk_index: usize, target_index: usize, value: T) {
        if chunk_index == 0 && target_index == 0 {
            self.push_front(value);
            return;
        }

        let chunks_len = self.chunks.len();
        assert!(chunk_index < chunks_len);

        let chunk = &mut self.chunks[chunk_index];
        assert!(target_index <= chunk.len());

        if chunk_index + 1 >= chunks_len && target_index >= chunk.len() {
            self.push_back(value);
            return;
        }

        if target_index >= N {
            let mut chunk = VecDeque::with_capacity(N);
            chunk.push_front(value);

            self.chunks.insert(chunk_index + 1, chunk);
            self.len += 1;
            return;
        }

        if chunk.len() >= N {
            let spilled_value = chunk.pop_back().unwrap();

            match self.chunks.get_mut(chunk_index + 1) {
                Some(chunk) if chunk.len() < N => chunk.push_front(spilled_value),
                next_chunk => {
                    let mut chunk = VecDeque::with_capacity(N);
                    chunk.push_front(spilled_value);

                    if next_chunk.is_none() {
                        self.chunks.push_back(chunk);
                    } else {
                        self.chunks.insert(chunk_index + 1, chunk);
                    }
                }
            }
        }

        let chunk = &mut self.chunks[chunk_index];
        chunk.insert(target_index, value);
        debug_assert!(chunk.capacity() <= N);
        self.len += 1;
    }

    /// Moves all elements from the `other` list to the end of this one.
    ///
    /// This reuses all the chunks from other list and moves them into self.
    /// After this operation, other becomes empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list1: ArrayList<i32, 4> = ArrayList::new();
    /// list1.push_back(1);
    /// list1.push_back(2);
    ///
    /// let mut list2: ArrayList<i32, 4> = ArrayList::new();
    /// list2.push_back(3);
    /// list2.push_back(4);
    ///
    /// list1.append(&mut list2);
    ///
    /// assert_eq!(list1.len(), 4);
    /// assert_eq!(list1.get(0), Some(&1));
    /// assert_eq!(list1.get(1), Some(&2));
    /// assert_eq!(list1.get(2), Some(&3));
    /// assert_eq!(list1.get(3), Some(&4));
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        self.chunks.append(&mut other.chunks);
        self.len += other.len;
        other.len = 0;
    }

    /// Removes and returns the first element of the `ArrayList`, if any.
    /// If the list is empty, it returns `None`.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_front(10);
    /// list.push_front(20);
    ///
    /// assert_eq!(list.pop_front(), Some(20));
    /// assert_eq!(list.pop_front(), Some(10));
    /// assert_eq!(list.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        let chunk = self.chunks.front_mut()?;

        let value = chunk.pop_front();
        if chunk.is_empty() {
            self.chunks.pop_front();
        }

        self.len -= 1;
        value
    }

    /// Removes and returns the last element of the `ArrayList`, if any.
    /// If the list is empty, it returns `None`.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.pop_back(), Some(20));
    /// assert_eq!(list.pop_back(), Some(10));
    /// assert_eq!(list.pop_back(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        let chunk = self.chunks.back_mut()?;

        let value = chunk.pop_back();
        if chunk.is_empty() {
            self.chunks.pop_back();
        }

        self.len -= 1;
        value
    }

    /// Removes and returns the element at the specified index, shifting subsequent elements left.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    /// list.push_back(30);
    /// list.push_back(40);
    /// list.push_back(50);
    ///
    /// assert_eq!(list.remove(1), Some(20));
    /// assert_eq!(list.get(1), Some(&30));
    /// assert_eq!(list.len(), 4);
    ///
    ///
    /// assert_eq!(list.remove(10), None);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        if index == 0 {
            return self.pop_front();
        }

        if index >= self.len.saturating_sub(1) {
            return self.pop_back();
        }

        let SearchTarget {
            chunk_index,
            target_index,
        } = self.search_target(index).unwrap();

        let chunk = &mut self.chunks[chunk_index];
        let value = chunk.remove(target_index);
        if chunk.is_empty() {
            self.chunks.remove(chunk_index);
        }

        self.len -= 1;
        value
    }

    /// Removes all elements from the `ArrayList`, effectively making it empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::new();
    /// list.push_back(1);
    /// list.push_back(2);
    /// list.push_back(3);
    ///
    /// assert_eq!(list.len(), 3);
    ///
    /// list.clear();
    ///
    /// assert_eq!(list.len(), 0);
    /// assert!(list.is_empty());
    /// assert_eq!(list.front(), None);
    /// assert_eq!(list.back(), None);
    /// ```
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.len = 0;
    }

    /// Returns a reference to the first element of the `ArrayList`, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.front(), Some(&10));
    ///
    /// list.pop_front();
    /// assert_eq!(list.front(), Some(&20));
    ///
    /// list.pop_front();
    /// assert_eq!(list.front(), None);
    /// ```
    pub fn front(&self) -> Option<&T> {
        self.chunks.front().and_then(|chunk| chunk.front())
    }

    /// Returns a mutable reference to the first element of the `ArrayList`, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.front_mut(), Some(&mut 10));
    ///
    /// list.pop_front();
    /// assert_eq!(list.front_mut(), Some(&mut 20));
    ///
    /// list.pop_front();
    /// assert_eq!(list.front_mut(), None);
    /// ```
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.chunks.front_mut().and_then(|chunk| chunk.front_mut())
    }

    /// Returns a reference to the last element of the `ArrayList`, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.back(), Some(&20));
    ///
    /// list.pop_back();
    /// assert_eq!(list.back(), Some(&10));
    ///
    /// list.pop_back();
    /// assert_eq!(list.back(), None);
    /// ```
    pub fn back(&self) -> Option<&T> {
        self.chunks.back().and_then(|chunk| chunk.back())
    }

    /// Returns a mutable reference to the last element of the `ArrayList`, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.back_mut(), Some(&mut 20));
    ///
    /// list.pop_back();
    /// assert_eq!(list.back_mut(), Some(&mut 10));
    ///
    /// list.pop_back();
    /// assert_eq!(list.back_mut(), None);
    /// ```
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.chunks.back_mut().and_then(|chunk| chunk.back_mut())
    }

    /// Returns a reference to the element at the specified index, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.get(0), Some(&10));
    /// assert_eq!(list.get(1), Some(&20));
    /// assert_eq!(list.get(2), None); // Out of bounds
    /// ```
    pub fn get(&self, mut index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        if index <= self.len() / 2 {
            return self
                .chunks
                .iter()
                .find(|chunk| {
                    if index < chunk.len() {
                        return true;
                    }

                    index -= chunk.len();
                    false
                })
                .map(|chunk| &chunk[index]);
        }

        let mut remaining_len = self.len();
        self.chunks
            .iter()
            .rfind(|chunk| {
                remaining_len -= chunk.len();

                if index + 1 > remaining_len {
                    index -= remaining_len;
                    return true;
                }

                false
            })
            .map(|chunk| &chunk[index])
    }

    /// Returns a mutable reference to the element at the specified index, if any.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(20);
    ///
    /// assert_eq!(list.get_mut(0), Some(&mut 10));
    /// assert_eq!(list.get_mut(1), Some(&mut 20));
    /// assert_eq!(list.get_mut(2), None); // Out of bounds
    /// ```
    pub fn get_mut(&mut self, mut index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        if index <= self.len() / 2 {
            return self
                .chunks
                .iter_mut()
                .find(|chunk| {
                    if index < chunk.len() {
                        return true;
                    }

                    index -= chunk.len();
                    false
                })
                .map(|chunk| &mut chunk[index]);
        }

        let mut remaining_len = self.len();
        self.chunks
            .iter_mut()
            .rfind(|chunk| {
                remaining_len -= chunk.len();

                if index + 1 > remaining_len {
                    index -= remaining_len;
                    return true;
                }

                false
            })
            .map(|chunk| &mut chunk[index])
    }

    /// Returns the number of elements currently stored in the `ArrayList`.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 6> = ArrayList::new();
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// assert_eq!(list.len(), 2);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Checks if the `ArrayList` is empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 6> = ArrayList::new();
    /// assert!(list.is_empty());
    ///
    /// list.push_back(1);
    /// assert!(!list.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Provides an iterator over list's elements.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<_, 2> = ArrayList::new();
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter::from_list(self)
    }

    /// Provides a mutable iterator over list's elements.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<_, 2> = ArrayList::new();
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, N> {
        IterMut::from_list(self)
    }

    /// Provides a cursor at the front element.
    ///
    /// The cursor is pointing to the “ghost” non-element if the list is empty.
    #[inline]
    pub fn cursor_front(&self) -> Cursor<'_, T, N> {
        Cursor::from_front(self)
    }

    /// Provides a cursor at the back element.
    ///
    /// The cursor is pointing to the “ghost” non-element if the list is empty.
    #[inline]
    pub fn cursor_back(&self) -> Cursor<'_, T, N> {
        Cursor::from_back(self)
    }

    /// Provides a cursor at the front element.
    ///
    /// The cursor is pointing to the “ghost” non-element if the list is empty.
    #[inline]
    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T, N> {
        CursorMut::from_front(self)
    }

    /// Provides a cursor at the back element.
    ///
    /// The cursor is pointing to the “ghost” non-element if the list is empty.
    #[inline]
    pub fn cursor_back_mut(&mut self) -> CursorMut<'_, T, N> {
        CursorMut::from_back(self)
    }

    fn search_target(&mut self, mut index: usize) -> Option<SearchTarget> {
        if index >= self.len() {
            return None;
        }

        if index <= self.len() / 2 {
            return self
                .chunks
                .iter()
                .position(|chunk| {
                    if index < chunk.len() {
                        return true;
                    }

                    index -= chunk.len();
                    false
                })
                .map(|chunk_index| SearchTarget {
                    chunk_index,
                    target_index: index,
                });
        }

        let mut remaining_len = self.len();
        self.chunks
            .iter_mut()
            .rposition(|chunk| {
                remaining_len -= chunk.len();

                if index + 1 > remaining_len {
                    index -= remaining_len;
                    return true;
                }

                false
            })
            .map(|chunk_index| SearchTarget {
                chunk_index,
                target_index: index,
            })
    }
}

#[derive(Debug, Default)]
struct SearchTarget {
    chunk_index: usize,
    target_index: usize,
}

impl<T: Clone, const N: usize> Clone for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T, const N: usize, const M: usize> PartialEq<[T; M]> for ArrayList<T, N>
where
    T: PartialEq,
    Usize<N>: ChunkCapacity,
{
    fn eq(&self, other: &[T; M]) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> PartialEq<&[T]> for ArrayList<T, N>
where
    T: PartialEq,
    Usize<N>: ChunkCapacity,
{
    fn eq(&self, other: &&[T]) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T, const N: usize> PartialEq<[T]> for ArrayList<T, N>
where
    T: PartialEq,
    Usize<N>: ChunkCapacity,
{
    fn eq(&self, other: &[T]) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> PartialEq for ArrayList<T, N>
where
    T: PartialEq,
    Usize<N>: ChunkCapacity,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> Eq for ArrayList<T, N>
where
    T: Eq,
    Usize<N>: ChunkCapacity,
{
}

impl<T, const N: usize> PartialOrd for ArrayList<T, N>
where
    T: PartialOrd,
    Usize<N>: ChunkCapacity,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> Ord for ArrayList<T, N>
where
    T: Ord,
    Usize<N>: ChunkCapacity,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T, const N: usize> Hash for ArrayList<T, N>
where
    T: Hash,
    Usize<N>: ChunkCapacity,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.len());
        self.iter().for_each(|v| v.hash(state));
    }
}

impl<T, const N: usize> std::fmt::Debug for ArrayList<T, N>
where
    T: std::fmt::Debug,
    Usize<N>: ChunkCapacity,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.chunks.iter()).finish()
    }
}

impl<T, const N: usize> IntoIterator for ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from_list(self)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::from_list(self)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayList<T, N>
where
    Usize<N>: ChunkCapacity,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut::from_list(self)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::VecDeque;
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    use std::mem::size_of;

    use quickcheck_macros::quickcheck;

    use crate::{ArrayList, ChunkCapacity, Usize};

    const _: () = assert!(
        size_of::<ArrayList<usize, 32>>() == size_of::<usize>() * 5,
        "unexpected memory layout"
    );

    #[test]
    fn test_new_creates_empty_array_list() {
        let sut: ArrayList<i64, 2> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn test_default_creates_empty_array_list() {
        let sut: ArrayList<i64, 2> = ArrayList::default();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn test_push_front_adds_element_to_front() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.push_front(10);
        assert_eq!(sut.len(), 1);
        assert!(!sut.is_empty());

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.push_front(40);
        sut.push_front(30);
        sut.push_front(20);
        sut.push_front(10);
        assert_eq!(sut.len(), 4);
        assert!(!sut.is_empty());

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.pop_front(), Some(30));
        assert_eq!(sut.pop_front(), Some(40));
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
    }

    #[test]
    fn test_push_back_adds_element_to_back() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.push_back(10);
        assert_eq!(sut.len(), 1);
        assert!(!sut.is_empty());

        assert_eq!(sut.pop_back(), Some(10));
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        sut.push_back(40);
        assert_eq!(sut.len(), 4);
        assert!(!sut.is_empty());

        assert_eq!(sut.pop_back(), Some(40));
        assert_eq!(sut.pop_back(), Some(30));
        assert_eq!(sut.pop_back(), Some(20));
        assert_eq!(sut.pop_back(), Some(10));
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
    }

    #[test]
    fn test_insert_inserts_element_at_correct_index() {
        let mut sut: ArrayList<i64, 4> = ArrayList::new();

        // Insert into an empty list
        sut.insert(0, 10); // List: [10]
        assert_eq!(sut.len(), 1);
        assert_eq!(sut, [10]);
        assert_eq!(sut.get(0), Some(&10));

        // Insert at the beginning
        sut.insert(0, 5); // List: [5, 10]
        assert_eq!(sut.len(), 2);
        assert_eq!(sut, [5, 10]);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));

        // Insert at the end
        sut.insert(2, 20); // List: [5, 10, 20]
        assert_eq!(sut.len(), 3);
        assert_eq!(sut, [5, 10, 20]);
        assert_eq!(sut.get(2), Some(&20));

        // Insert in the middle
        sut.insert(1, 7); // List: [5, 7, 10, 20]
        assert_eq!(sut.len(), 4);
        assert_eq!(sut, [5, 7, 10, 20]);
        assert_eq!(sut.get(1), Some(&7));
        assert_eq!(sut.get(2), Some(&10));

        // Fill the chunk
        sut.insert(4, 25); // List: [5, 7, 10, 20, 25]
        assert_eq!(sut.len(), 5);
        assert_eq!(sut, [5, 7, 10, 20, 25]);
        assert_eq!(sut.get(4), Some(&25));

        sut.insert(5, 30); // List: [5, 7, 10, 20, 25, 30]
        assert_eq!(sut.len(), 6);
        assert_eq!(sut, [5, 7, 10, 20, 25, 30]);
        assert_eq!(sut.get(5), Some(&30));

        // Force a chunk split
        sut.insert(3, 15); // List: [5, 7, 10, 15, 20, 25, 30]
        assert_eq!(sut.len(), 7);
        assert_eq!(sut, [5, 7, 10, 15, 20, 25, 30]);
        assert_eq!(sut.get(3), Some(&15));
        assert_eq!(sut.get(4), Some(&20));

        // Insert at the new chunk boundary
        sut.insert(6, 27); // List: [5, 7, 10, 15, 20, 25, 27, 30]
        assert_eq!(sut.len(), 8);
        assert_eq!(sut, [5, 7, 10, 15, 20, 25, 27, 30]);
        assert_eq!(sut.get(6), Some(&27));
        assert_eq!(sut.get(7), Some(&30));

        // Check all elements
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&7));
        assert_eq!(sut.get(2), Some(&10));
        assert_eq!(sut.get(3), Some(&15));
        assert_eq!(sut.get(4), Some(&20));
        assert_eq!(sut.get(5), Some(&25));
        assert_eq!(sut.get(6), Some(&27));
        assert_eq!(sut.get(7), Some(&30));

        // Insert out-of-bounds (should panic)
        let result = std::panic::catch_unwind(move || sut.insert(10, 100));
        assert!(result.is_err());
    }

    #[test]
    fn test_pop_front_removes_and_returns_the_first_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        sut.push_front(30);
        sut.push_front(20);
        sut.push_front(10);
        assert_eq!(sut.len(), 3);

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.pop_front(), Some(30));
        assert_eq!(sut.len(), 0);

        assert!(sut.is_empty());
        assert_eq!(sut.pop_front(), None);
    }

    #[test]
    fn test_pop_back_removes_and_returns_the_last_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        assert_eq!(sut.len(), 3);

        assert_eq!(sut.pop_back(), Some(30));
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.pop_back(), Some(20));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.pop_back(), Some(10));
        assert_eq!(sut.len(), 0);

        assert!(sut.is_empty());
        assert_eq!(sut.pop_back(), None);
    }

    #[test]
    fn test_remove_removes_element_at_index() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();

        // Fill the list with elements
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        sut.push_back(40);
        sut.push_back(50);
        sut.push_back(60);
        assert_eq!(sut.len(), 6);

        // Test removal of elements at various indices
        assert_eq!(sut.remove(0).unwrap(), 10); // Removes 10, shifts 20 to index 0
        assert_eq!(sut.get(0), Some(&20));
        assert_eq!(sut.len(), 5);

        assert_eq!(sut.remove(2).unwrap(), 40); // Removes 40, shifts 50 to index 2
        assert_eq!(sut.get(2), Some(&50));
        assert_eq!(sut.len(), 4);

        assert_eq!(sut.remove(3).unwrap(), 60); // Removes 60, second chunk becomes empty
        assert_eq!(sut.get(3), None); // No more elements at index 3
        assert_eq!(sut.len(), 3);

        // Test removal of remaining elements
        assert_eq!(sut.remove(1).unwrap(), 30); // Removes 30
        assert_eq!(sut.get(1), Some(&50));
        assert_eq!(sut.len(), 2);

        assert_eq!(sut.remove(1).unwrap(), 50); // Removes 50
        assert_eq!(sut.get(1), None);
        assert_eq!(sut.len(), 1);

        assert_eq!(sut.remove(0).unwrap(), 20); // Removes 20, list becomes empty
        assert_eq!(sut.get(0), None);
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.remove(0), None);
    }

    #[test]
    fn test_remove_element_in_middle_chunk() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        sut.push_back(0);
        sut.push_back(1);
        sut.push_back(2);
        sut.push_back(3);
        sut.push_back(4);
        sut.push_back(5);
        sut.push_back(6);
        sut.push_back(7);
        sut.push_back(8);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 9);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&5));
        assert_eq!(sut.get(6), Some(&6));
        assert_eq!(sut.get(7), Some(&7));
        assert_eq!(sut.get(8), Some(&8));

        assert_eq!(sut.remove(5).unwrap(), 5);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 8);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&6));
        assert_eq!(sut.get(6), Some(&7));
        assert_eq!(sut.get(7), Some(&8));

        assert_eq!(sut.remove(4).unwrap(), 4);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&6));
        assert_eq!(sut.get(5), Some(&7));
        assert_eq!(sut.get(6), Some(&8));

        assert_eq!(sut.remove(3).unwrap(), 3);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&6));
        assert_eq!(sut.get(4), Some(&7));
        assert_eq!(sut.get(5), Some(&8));
    }

    #[test]
    fn test_clear_resets_the_list() {
        let mut sut: ArrayList<i32, 2> = ArrayList::new();

        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 3);

        sut.clear();

        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        assert_eq!(sut.front(), None);
        assert_eq!(sut.back(), None);

        // Verify the list is still functional after clearing
        sut.push_back(40);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.front(), Some(&40));
        assert_eq!(sut.back(), Some(&40));
    }

    #[test]
    fn test_front_returns_the_first_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.front(), None);

        sut.push_back(10);
        assert_eq!(sut.front(), Some(&10));

        sut.push_back(20);
        assert_eq!(sut.front(), Some(&10));

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.front(), Some(&20));

        sut.push_front(10);
        assert_eq!(sut.front(), Some(&10));

        assert_eq!(sut.pop_back(), Some(20));
        assert_eq!(sut.front(), Some(&10));

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.front(), None);
    }

    #[test]
    fn test_front_mut_returns_the_first_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.front_mut(), None);

        sut.push_back(10);
        assert_eq!(sut.front_mut(), Some(&mut 10));

        sut.push_back(20);
        assert_eq!(sut.front_mut(), Some(&mut 10));

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.front_mut(), Some(&mut 20));

        sut.push_front(10);
        assert_eq!(sut.front_mut(), Some(&mut 10));

        assert_eq!(sut.pop_back(), Some(20));
        assert_eq!(sut.front_mut(), Some(&mut 10));

        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.front_mut(), None);
    }

    #[test]
    fn test_back_returns_the_last_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.back(), None);

        sut.push_front(10);
        assert_eq!(sut.back(), Some(&10));

        sut.push_front(20);
        assert_eq!(sut.back(), Some(&10));

        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.back(), Some(&10));

        sut.push_front(20);
        assert_eq!(sut.back(), Some(&10));

        assert_eq!(sut.pop_back(), Some(10));
        assert_eq!(sut.back(), Some(&20));

        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.back(), None);
    }

    #[test]
    fn test_back_mut_returns_the_last_element() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.back_mut(), None);

        sut.push_front(10);
        assert_eq!(sut.back_mut(), Some(&mut 10));

        sut.push_front(20);
        assert_eq!(sut.back_mut(), Some(&mut 10));

        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.back_mut(), Some(&mut 10));

        sut.push_front(20);
        assert_eq!(sut.back_mut(), Some(&mut 10));

        assert_eq!(sut.pop_back(), Some(10));
        assert_eq!(sut.back_mut(), Some(&mut 20));

        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.back_mut(), None);
    }

    #[test]
    fn test_get_retrieves_correct_element() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());

        assert_eq!(sut.get(0), None);
        assert_eq!(sut.get(5), None);

        // Ensure to allocate at leat 2 chunks
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        sut.push_back(40);
        sut.push_back(50);
        sut.push_back(60);

        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));
        assert_eq!(sut.get(3), Some(&40));
        assert_eq!(sut.get(4), Some(&50));
        assert_eq!(sut.get(5), Some(&60));

        // Out-of-bounds indices, should return None
        assert_eq!(sut.get(6), None);
        assert_eq!(sut.get(10), None);
    }

    #[test]
    fn test_get_mut_retrieves_correct_element() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());

        assert_eq!(sut.get_mut(0), None);
        assert_eq!(sut.get_mut(5), None);

        // Ensure to allocate at leat 2 chunks
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        sut.push_back(40);
        sut.push_back(50);
        sut.push_back(60);

        assert_eq!(sut.get_mut(0), Some(&mut 10));
        assert_eq!(sut.get_mut(1), Some(&mut 20));
        assert_eq!(sut.get_mut(2), Some(&mut 30));
        assert_eq!(sut.get_mut(3), Some(&mut 40));
        assert_eq!(sut.get_mut(4), Some(&mut 50));
        assert_eq!(sut.get_mut(5), Some(&mut 60));

        // Out-of-bounds indices, should return None
        assert_eq!(sut.get_mut(6), None);
        assert_eq!(sut.get_mut(10), None);
    }

    #[test]
    fn test_len_returns_correct_length() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.push_back(10);
        assert_eq!(sut.len(), 1);
        assert!(!sut.is_empty());

        sut.push_back(20);
        assert_eq!(sut.len(), 2);
        assert!(!sut.is_empty());

        sut.push_back(30);
        assert_eq!(sut.len(), 3);
        assert!(!sut.is_empty());
    }

    #[test]
    fn test_list_remains_functional_after_multiple_operations() {
        let mut sut: ArrayList<i32, 4> = ArrayList::new();

        // Initial insertions
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);
        sut.push_back(40);
        sut.push_back(50);

        assert_eq!(sut.len(), 5);
        assert_eq!(sut.front(), Some(&10));
        assert_eq!(sut.back(), Some(&50));

        // Remove elements from the front
        assert_eq!(sut.pop_front(), Some(10));
        assert_eq!(sut.pop_front(), Some(20));
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.front(), Some(&30));
        assert_eq!(sut.back(), Some(&50));

        // Insert elements at the front
        sut.push_front(5);
        sut.push_front(0);
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&50));

        // Remove elements from the back
        assert_eq!(sut.pop_back(), Some(50));
        assert_eq!(sut.pop_back(), Some(40));
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&30));

        // Insert in the middle
        sut.insert(1, 15);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&15));
        assert_eq!(sut.get(2), Some(&5));
        assert_eq!(sut.get(3), Some(&30));

        // Remove an element from the middle
        assert_eq!(sut.remove(2).unwrap(), 5);
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&15));
        assert_eq!(sut.get(2), Some(&30));

        // Clear and verify reusability
        sut.clear();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        sut.push_back(100);
        sut.push_back(200);
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.front(), Some(&100));
        assert_eq!(sut.back(), Some(&200));
    }

    #[test]
    fn test_append_combines_two_lists() {
        // Create and populate the first list
        let mut sut: ArrayList<i32, 2> = ArrayList::new();
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);

        // Create and populate the second list
        let mut other: ArrayList<i32, 2> = ArrayList::new();
        other.push_back(40);
        other.push_back(50);
        other.push_back(60);

        // Append the second list into the first
        sut.append(&mut other);

        assert!(other.is_empty());
        assert_eq!(other.len(), 0);

        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));
        assert_eq!(sut.get(3), Some(&40));
        assert_eq!(sut.get(4), Some(&50));
        assert_eq!(sut.get(5), Some(&60));

        // ensure appending an empty list does nothing
        sut.append(&mut other);
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));
        assert_eq!(sut.get(3), Some(&40));
        assert_eq!(sut.get(4), Some(&50));
        assert_eq!(sut.get(5), Some(&60));

        // ensure other remains functional
        other.push_back(100);
        other.push_back(200);
        assert_eq!(other.len(), 2);
        assert_eq!(other.front(), Some(&100));
        assert_eq!(other.back(), Some(&200));

        // ensure sut remains functional
        sut.push_back(70);
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));
        assert_eq!(sut.get(3), Some(&40));
        assert_eq!(sut.get(4), Some(&50));
        assert_eq!(sut.get(5), Some(&60));
        assert_eq!(sut.get(6), Some(&70));
    }

    #[test]
    fn test_append_an_empty_list_do_nothing() {
        // Create and populate the first list
        let mut sut: ArrayList<i32, 2> = ArrayList::new();
        sut.push_back(10);
        sut.push_back(20);
        sut.push_back(30);

        // Create and populate the second list
        let mut other: ArrayList<i32, 2> = ArrayList::new();

        // Append the second list into the first
        sut.append(&mut other);
        assert!(other.is_empty());
        assert_eq!(other.len(), 0);

        other.push_back(100);
        other.push_back(200);
        assert_eq!(other.len(), 2);
        assert_eq!(other.front(), Some(&100));
        assert_eq!(other.back(), Some(&200));

        // Verify the combined list
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));

        // Verify the combined list is still functional
        sut.push_back(40);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(3), Some(&40));
    }

    #[test]
    fn test_append_on_an_empty_list_adds_all_elements() {
        // Create and populate the first list
        let mut sut: ArrayList<i32, 2> = ArrayList::new();

        // Create and populate the second list
        let mut other: ArrayList<i32, 2> = ArrayList::new();
        other.push_back(10);
        other.push_back(20);
        other.push_back(30);

        // Append the second list into the first
        sut.append(&mut other);
        assert!(other.is_empty());
        assert_eq!(other.len(), 0);

        other.push_back(100);
        other.push_back(200);
        assert_eq!(other.len(), 2);
        assert_eq!(other.front(), Some(&100));
        assert_eq!(other.back(), Some(&200));

        // Verify the combined list
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));

        // Verify the combined list is still functional
        sut.push_back(40);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(3), Some(&40));
    }

    #[test]
    fn test_from_iter_works_correctly() {
        let sut: ArrayList<i32, 2> = ArrayList::from_iter(0..5);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));

        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&4));
    }

    #[test]
    fn test_extend_works_correctly() {
        let mut sut: ArrayList<i32, 2> = ArrayList::new();
        sut.extend(0..5);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));

        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&4));
    }

    #[test]
    fn test_extend_with_refs_works_correctly() {
        let mut sut: ArrayList<i32, 2> = ArrayList::new();
        sut.extend([0, 1, 2, 3, 4].iter());
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));

        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&4));
    }

    #[test]
    fn test_from_array_works_correctly() {
        let sut: ArrayList<i32, 2> = ArrayList::from([0, 1, 2, 3, 4]);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));

        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&4));
    }

    #[test]
    fn test_clone_works_correctly() {
        let mut other: ArrayList<i32, 2> = ArrayList::from([0, 1, 2, 3, 4]);

        let sut = other.clone();
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.front(), Some(&0));
        assert_eq!(sut.back(), Some(&4));
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));

        other.clear();
        let sut = other.clone();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);
        assert_eq!(sut.front(), None);
        assert_eq!(sut.back(), None);
        assert_eq!(sut.get(0), None);
    }

    #[test]
    fn test_eq_works_correctly() {
        let l = ArrayList::<usize, 2>::from([0, 1, 2, 3, 4]);
        let mut r = ArrayList::<usize, 2>::from([0, 2, 3, 4, 1]);

        assert_eq!(l, l);
        assert_eq!(r, r);
        assert_ne!(l, r);

        r.pop_back();
        assert_ne!(l, r);

        r.insert(1, 1);
        assert_eq!(l, r);
    }

    #[test]
    fn test_debug_works_correctly() {
        let array = [0, 1, 2, 3, 4];
        let list = ArrayList::<usize, 2>::from(array);
        assert_eq!(format!("{list:?}"), "[[0, 1], [2, 3], [4]]");
    }

    #[test]
    fn test_cmp_works_correctly() {
        let a = ArrayList::<usize, 2>::from([0, 1, 2]);
        let b = ArrayList::<usize, 2>::from([4, 5, 6]);
        assert_eq!(a.cmp(&a), Ordering::Equal);
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
    }

    #[test]
    fn test_partial_cmp_works_correctly() {
        let a = ArrayList::<f64, 2>::from([0.0, 1.0, 2.0]);
        let b = ArrayList::<f64, 2>::from([4.0, 5.0, 6.0]);
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        assert_eq!(b.partial_cmp(&a), Some(Ordering::Greater));
    }

    #[test]
    fn test_hash_works_correctly() {
        let bh = BuildHasherDefault::<DefaultHasher>::default();
        let a = ArrayList::<usize, 2>::from([0, 1, 2]);
        let b = ArrayList::<usize, 2>::from([4, 5, 6]);
        assert_ne!(bh.hash_one(&a), bh.hash_one(&b));
        assert_eq!(bh.hash_one(&a), bh.hash_one(&a));
        assert_eq!(bh.hash_one(&a), bh.hash_one(&(a.clone())));
    }

    #[test]
    fn test_push_front() {
        let mut sut: ArrayList<_, 3> = ArrayList::new();

        sut.push_front(6);
        assert_eq!(sut, [6]);

        sut.push_front(5);
        assert_eq!(sut, [5, 6]);

        sut.push_front(4);
        assert_eq!(sut, [4, 5, 6]);

        sut.push_front(3);
        assert_eq!(sut, [3, 4, 5, 6]);

        sut.push_front(2);
        assert_eq!(sut, [2, 3, 4, 5, 6]);

        sut.push_front(1);
        assert_eq!(sut, [1, 2, 3, 4, 5, 6]);

        sut.push_front(0);
        assert_eq!(sut, [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_push_back() {
        let mut sut: ArrayList<_, 3> = ArrayList::new();

        sut.push_back(6);
        assert_eq!(sut, [6]);

        sut.push_back(5);
        assert_eq!(sut, [6, 5]);

        sut.push_back(4);
        assert_eq!(sut, [6, 5, 4]);

        sut.push_back(3);
        assert_eq!(sut, [6, 5, 4, 3]);

        sut.push_back(2);
        assert_eq!(sut, [6, 5, 4, 3, 2]);

        sut.push_back(1);
        assert_eq!(sut, [6, 5, 4, 3, 2, 1]);

        sut.push_back(0);
        assert_eq!(sut, [6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_insert() {
        let mut sut: ArrayList<_, 3> = ArrayList::new();
        sut.insert(0, 4);
        assert_eq!(sut, [4]);

        sut.insert(0, 3);
        assert_eq!(sut, [3, 4]);

        sut.insert(2, 5);
        assert_eq!(sut, [3, 4, 5]);

        sut.insert(3, 6);
        assert_eq!(sut, [3, 4, 5, 6]);

        sut.insert(4, 7);
        assert_eq!(sut, [3, 4, 5, 6, 7]);

        sut.insert(5, 8);
        assert_eq!(sut, [3, 4, 5, 6, 7, 8]);

        sut.insert(0, 1);
        assert_eq!(sut, [1, 3, 4, 5, 6, 7, 8]);

        sut.insert(0, 0);
        assert_eq!(sut, [0, 1, 3, 4, 5, 6, 7, 8]);

        sut.insert(2, 2);
        assert_eq!(sut, [0, 1, 2, 3, 4, 5, 6, 7, 8],);

        sut.insert(2, 42);
        assert_eq!(sut, [0, 1, 42, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_insert_into_full_array_at_every_index() {
        for i in 0..4 {
            let mut sut: ArrayList<_, 4> = ArrayList::new();
            sut.extend(0..4);
            assert_eq!(sut.len(), 4);
            assert_eq!(sut, [0, 1, 2, 3]);

            sut.insert(i, 42);
            assert_eq!(sut.len(), 5);

            let mut expected = vec![0, 1, 2, 3];
            expected.insert(i, 42);

            assert_eq!(
                sut,
                expected.as_slice(),
                "{:?} {:?}",
                sut.chunks,
                expected.as_slice()
            );
        }
    }

    #[test]
    fn test_remove_from_full_array_at_every_index() {
        for i in 0..4 {
            let mut sut: ArrayList<_, 4> = ArrayList::new();
            sut.extend(0..4);
            assert_eq!(sut.len(), 4);
            assert_eq!(sut, [0, 1, 2, 3]);

            sut.remove(i);
            assert_eq!(sut.len(), 3);

            let mut expected = vec![0, 1, 2, 3];
            expected.remove(i);

            assert_eq!(
                sut,
                expected.as_slice(),
                "{:?} {:?}",
                sut.chunks,
                expected.as_slice()
            );
        }
    }

    #[test]
    fn test_raw_insert() {
        fn _test<const N: usize>()
        where
            Usize<N>: ChunkCapacity,
        {
            let mut sut = ArrayList::<i32, N>::new();
            assert!(sut.is_empty());
            assert_eq!(sut.len(), 0);
            assert_eq!(sut.front(), None);
            assert_eq!(sut.back(), None);
            assert_eq!(sut.get(0), None);
            assert_eq!(sut, []);

            sut.raw_insert(0, 0, 42);
            assert!(!sut.is_empty());
            assert_eq!(sut.len(), 1);
            assert_eq!(sut.front(), Some(&42));
            assert_eq!(sut.back(), Some(&42));
            assert_eq!(sut.get(0), Some(&42));
            assert_eq!(sut, [42]);

            sut.raw_insert(0, 1, 96);
            assert!(!sut.is_empty());
            assert_eq!(sut.len(), 2);
            assert_eq!(sut.front(), Some(&42));
            assert_eq!(sut.back(), Some(&96));
            assert_eq!(sut.get(0), Some(&42));
            assert_eq!(sut.get(1), Some(&96));
            assert_eq!(sut, [42, 96]);

            sut.raw_insert(0, 1, 64);
            assert!(!sut.is_empty());
            assert_eq!(sut.len(), 3);
            assert_eq!(sut.front(), Some(&42));
            assert_eq!(sut.back(), Some(&96));
            assert_eq!(sut.get(0), Some(&42));
            assert_eq!(sut.get(1), Some(&64));
            assert_eq!(sut.get(2), Some(&96));
            assert_eq!(sut, [42, 64, 96]);

            sut.raw_insert(0, 0, 32);
            assert!(!sut.is_empty());
            assert_eq!(sut.len(), 4);
            assert_eq!(sut.front(), Some(&32));
            assert_eq!(sut.back(), Some(&96));
            assert_eq!(sut.get(0), Some(&32));
            assert_eq!(sut.get(1), Some(&42));
            assert_eq!(sut.get(2), Some(&64));
            assert_eq!(sut.get(3), Some(&96));
            assert_eq!(sut, [32, 42, 64, 96]);

            let back_chunk_index = sut.chunks.len().saturating_sub(1);
            let back_target_index = sut.chunks[back_chunk_index].len();
            sut.raw_insert(back_chunk_index, back_target_index, 128);
            assert!(!sut.is_empty());
            assert_eq!(sut.len(), 5);
            assert_eq!(sut.front(), Some(&32));
            assert_eq!(sut.back(), Some(&128));
            assert_eq!(sut.get(0), Some(&32));
            assert_eq!(sut.get(1), Some(&42));
            assert_eq!(sut.get(2), Some(&64));
            assert_eq!(sut.get(3), Some(&96));
            assert_eq!(sut.get(4), Some(&128));
            assert_eq!(sut, [32, 42, 64, 96, 128]);
        }

        _test::<1>();
        _test::<2>();
        _test::<3>();
        _test::<4>();
        _test::<5>();
        _test::<8>();
    }

    #[test]
    fn test_raw_insert_front() {
        fn _test<const N: usize>()
        where
            Usize<N>: ChunkCapacity,
        {
            let mut sut = ArrayList::<i32, N>::new();
            assert!(sut.is_empty());
            assert_eq!(sut.len(), 0);
            assert_eq!(sut.front(), None);
            assert_eq!(sut.back(), None);
            assert_eq!(sut.get(0), None);
            assert_eq!(sut, []);

            let mut acc = Vec::with_capacity(128);
            for i in (0..128).rev() {
                acc.insert(0, i);

                sut.raw_insert(0, 0, i);
                assert!(!sut.is_empty());
                assert_eq!(sut.len(), acc.len());
                assert_eq!(sut.front(), acc.first());
                assert_eq!(sut.back(), acc.last());
                assert_eq!(sut, acc.as_slice());
            }
        }

        _test::<1>();
        _test::<2>();
        _test::<3>();
        _test::<4>();
        _test::<5>();
        _test::<8>();
        _test::<16>();
        _test::<32>();
        _test::<64>();
    }

    #[test]
    fn test_raw_insert_back() {
        fn _test<const N: usize>()
        where
            Usize<N>: ChunkCapacity,
        {
            let mut sut = ArrayList::<i32, N>::new();
            assert!(sut.is_empty());
            assert_eq!(sut.len(), 0);
            assert_eq!(sut.front(), None);
            assert_eq!(sut.back(), None);
            assert_eq!(sut.get(0), None);
            assert_eq!(sut, []);

            let mut acc = Vec::with_capacity(128);
            for i in 0..128 {
                acc.push(i);

                let back_chunk_index = sut.chunks.len().saturating_sub(1);
                let back_target_index = sut.chunks.back().map(VecDeque::len).unwrap_or(0);
                sut.raw_insert(back_chunk_index, back_target_index, i);
                assert!(!sut.is_empty());
                assert_eq!(sut.len(), acc.len());
                assert_eq!(sut.front(), acc.first());
                assert_eq!(sut.back(), acc.last());
                assert_eq!(sut, acc.as_slice());
            }
        }

        _test::<1>();
        _test::<2>();
        _test::<3>();
        _test::<4>();
        _test::<5>();
        _test::<8>();
        _test::<16>();
        _test::<32>();
        _test::<64>();
    }

    #[quickcheck]
    fn nightly_test_array_list_behavioural(seed: VecDeque<i32>) {
        fn _test<const N: usize>(mut expected: VecDeque<i32>)
        where
            Usize<N>: ChunkCapacity,
        {
            let mut actual = ArrayList::<_, N>::from_iter(expected.iter().copied());

            for _ in 0..32 {
                let len = expected.len();

                assert_eq!(expected.is_empty(), actual.is_empty());
                assert_eq!(expected.len(), actual.len());

                assert_eq!(expected.front(), actual.front());
                assert_eq!(expected.front_mut(), actual.front_mut());

                assert_eq!(expected.back(), actual.back());
                assert_eq!(expected.back_mut(), actual.back_mut());

                assert_eq!(expected.get(0), actual.get(0));
                assert_eq!(expected.get_mut(0), actual.get_mut(0));

                assert_eq!(expected.get(1), actual.get(1));
                assert_eq!(expected.get_mut(1), actual.get_mut(1));

                assert_eq!(
                    expected.get(len.checked_div(2).unwrap_or(8)),
                    actual.get(len.checked_div(2).unwrap_or(8))
                );
                assert_eq!(
                    expected.get_mut(len.checked_div(2).unwrap_or(8)),
                    actual.get_mut(len.checked_div(2).unwrap_or(8))
                );

                assert_eq!(
                    expected.get(len.saturating_sub(2)),
                    actual.get(len.saturating_sub(2))
                );
                assert_eq!(
                    expected.get_mut(len.saturating_sub(2)),
                    actual.get_mut(len.saturating_sub(2))
                );

                assert_eq!(
                    expected.get(len.saturating_sub(1)),
                    actual.get(len.saturating_sub(1))
                );
                assert_eq!(
                    expected.get_mut(len.saturating_sub(1)),
                    actual.get_mut(len.saturating_sub(1))
                );

                assert_eq!(expected.get(len), actual.get(len));
                assert_eq!(expected.get_mut(len), actual.get_mut(len));

                assert_eq!(actual, expected.make_contiguous() as &[_]);

                let choice = rand::random_range(0..=5);
                match choice {
                    0 => {
                        let value = rand::random();
                        expected.push_front(value);
                        actual.push_front(value);
                    }
                    1 => {
                        let index = rand::random_range(0..=len);
                        let value = rand::random();
                        expected.insert(index, value);
                        actual.insert(index, value);
                    }
                    2 => {
                        let value = rand::random();
                        expected.push_back(value);
                        actual.push_back(value);
                    }
                    3 => assert_eq!(expected.pop_front(), actual.pop_front()),
                    4 => {
                        let index = rand::random_range(0..=len);
                        assert_eq!(expected.remove(index), actual.remove(index))
                    }
                    5 => assert_eq!(expected.pop_back(), actual.pop_back()),
                    _ => unreachable!(),
                }
            }

            expected.clear();
            actual.clear();

            assert_eq!(expected.is_empty(), actual.is_empty());
            assert_eq!(expected.len(), actual.len());

            assert_eq!(expected.front(), actual.front());
            assert_eq!(expected.front_mut(), actual.front_mut());

            assert_eq!(expected.back(), actual.back());
            assert_eq!(expected.back_mut(), actual.back_mut());
        }

        _test::<1>(seed.clone());
        _test::<2>(seed.clone());
        _test::<3>(seed.clone());
        _test::<4>(seed.clone());
        _test::<5>(seed.clone());
        _test::<8>(seed.clone());
        _test::<16>(seed.clone());
        _test::<32>(seed.clone());
        _test::<64>(seed.clone());
        _test::<128>(seed.clone());
        _test::<256>(seed.clone());
        _test::<512>(seed.clone());
    }
}
