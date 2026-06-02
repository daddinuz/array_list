#![no_std]

//! # array_list
//!
//! `array_list` provides an ordered collection backed by fixed-capacity chunks.
//! It offers sequence operations, iterators, and cursors while avoiding one
//! allocation per element.
//!
//! ## Features
//! - Ordered sequence with index-based element access.
//! - Chunked storage with less per-element pointer overhead than a traditional
//!   linked list.
//! - Shared, mutable, and owning iterators.
//! - Cursor and mutable cursor APIs for moving around the list and editing near
//!   the cursor.
//! - `#![no_std]` support with `alloc`.
//!
//! ## Quick Start
//! ```rust
//! use array_list::ArrayList;
//!
//! let mut list: ArrayList<i32, 8> = ArrayList::from([1, 2, 4]);
//!
//! list.insert(2, 3);
//! list.push_front(0);
//! list.push_back(5);
//!
//! assert_eq!(list, [0, 1, 2, 3, 4, 5]);
//!
//! for value in &mut list {
//!     *value *= 2;
//! }
//!
//! assert_eq!(list.iter().copied().collect::<Vec<_>>(), [0, 2, 4, 6, 8, 10]);
//! ```
//!
//! ## When To Use It
//! `array_list` is useful when you want an ordered sequence that supports indexed
//! access, stable iteration order, and cursor-local edits without allocating one
//! node per element.
//!
//! It is not a drop-in replacement for `Vec`. Index lookup may need to scan
//! chunks when the list is not densely packed, and insertions/removals inside a
//! chunk can move elements within that chunk.
//!
//! Use `Vec` when you primarily push/pop at the back and need dense contiguous
//! storage. Use `ArrayList` when cursor-local edits and chunked growth are a
//! better fit than a single contiguous allocation. Use `LinkedList` only when
//! its exact node-based semantics are required.
//!
//! ## API Overview
//! - Build lists with [`ArrayList::new`], [`ArrayList::from`], `collect`,
//!   `extend`, or [`ArrayList::append`].
//! - Edit at the ends with [`ArrayList::push_front`],
//!   [`ArrayList::push_back`], [`ArrayList::pop_front`], and
//!   [`ArrayList::pop_back`].
//! - Edit by index with [`ArrayList::insert`], [`ArrayList::remove`],
//!   [`ArrayList::get`], and [`ArrayList::get_mut`].
//! - Traverse with [`ArrayList::iter`], [`ArrayList::iter_mut`],
//!   [`IntoIter`], or the `IntoIterator` implementations for `ArrayList`,
//!   `&ArrayList`, and `&mut ArrayList`.
//! - Navigate and edit near a position with [`Cursor`] and [`CursorMut`].
//! - Inspect storage with [`ArrayList::capacity`],
//!   [`ArrayList::spare_capacity`], and [`ArrayList::shrink`].
//!
//! ## Storage Model
//! `ArrayList<T, N>` stores elements in chunks that can each hold up to `N`
//! elements. The logical order is independent from internal chunk boundaries:
//! iteration and indexing see one continuous sequence.
//!
//! Middle insertions and removals can leave spare capacity inside chunks. After
//! a middle removal that leaves the edited chunk non-empty, or after a middle
//! insertion that splits a full chunk, the edited chunk is locally refilled up
//! to half capacity from immediate sibling chunks when those siblings contain
//! more than half capacity. Siblings at or below half capacity are left
//! untouched, so edit-heavy workloads can still become fragmented. Use
//! [`ArrayList::spare_capacity`] to inspect unused chunk slots and
//! [`ArrayList::shrink`] to compact elements toward the front.
//!
//! This is a local minimum-occupancy heuristic, not a global invariant. The
//! minimum is `N / 2` using integer division, so odd capacities round down.
//! Refill only touches the edited chunk and its immediate siblings, preferring
//! bounded local movement over globally dense storage. Inserting into a full
//! middle chunk splits that chunk locally. Front/back pushes and pops keep their
//! direct boundary behavior, so edge chunks may be less than half full.
//!
//! ## Chunk Capacity
//! The second type parameter is the maximum number of elements per chunk.
//! Capacity must be at least 4. Smaller chunk sizes are rejected at compile time
//! by constructors and operations because this data structure is designed around
//! multi-element chunks and a local half-full occupancy heuristic.
//!
//! Chunk size has a large effect on performance. Small capacities reduce the
//! maximum amount moved inside one chunk, but they also create many more chunks,
//! more boundary crossings, more allocations, and more metadata to scan. Larger
//! capacities improve locality and reduce chunk count, which is especially
//! important for cursor-local edits, but middle insertions and removals can move
//! more elements within the edited chunk and its immediate siblings.
//!
//! Very small capacities are mostly useful for stress-testing chunk boundaries.
//! For general use, prefer capacities large enough to amortize chunk overhead.
//! Values in the 32-64 range are usually more representative than the minimum
//! supported size; mutation-heavy workloads often benefit from the larger end of
//! that range.
//!
//! Keeping edited chunks at least half full where possible reduces the chance of
//! many tiny chunks after repeated mutations. The tradeoff is that the list may
//! contain more chunks than a fully compacted layout, so locating an indexed
//! element can require scanning more chunk metadata until
//! [`ArrayList::shrink`] is called.
//!
//! ## Performance Notes
//! The exact cost depends on chunk occupancy and the chosen chunk capacity:
//! - [`ArrayList::len`] and [`ArrayList::is_empty`] are constant time.
//! - End pushes and pops usually touch only one edge chunk.
//! - [`ArrayList::get`], [`ArrayList::get_mut`], [`ArrayList::insert`], and
//!   [`ArrayList::remove`] locate elements by scanning chunk metadata unless
//!   the list is densely packed.
//! - Middle insertion/removal may move elements within the edited chunk and its
//!   immediate siblings.
//! - Iteration is in logical order and does not expose chunk boundaries.
//!
//! Call [`ArrayList::shrink`] after edit-heavy phases if a compact front-filled
//! layout is more important than preserving spare chunk slots for future edits.
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

extern crate alloc;

#[cfg(test)]
extern crate std;

mod cursor;
mod cursor_mut;
mod into_iter;
mod iter;
mod iter_mut;
mod position;

pub use cursor::Cursor;
pub use cursor_mut::CursorMut;
pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;

const MIN_CHUNK_CAPACITY: usize = 4;

use alloc::collections::VecDeque;

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use cap_vec::CapVec;

use crate::position::Position;

/// A dynamic ordered sequence stored as fixed-capacity chunks.
///
/// # Features
/// - **Chunked storage**: each chunk can hold up to `N` elements, reducing
///   allocation overhead compared to a traditional linked list.
/// - **Sequence operations**: supports front, back, indexed, iterator, and
///   cursor-based access.
/// - **Local editing**: mutable cursors can insert and remove near their current
///   position while keeping track of the logical element when possible.
///
/// # Type Parameters
/// - `T`: The type of elements stored in the list.
/// - `N`: The maximum number of elements that each chunk can hold.
///
/// `N` must be at least 4. Smaller chunk sizes are rejected at compile time by
/// constructors and operations because this data structure is designed around
/// multi-element chunks and a local half-full occupancy heuristic.
///
/// Chunk size is a performance tuning parameter. Small chunks reduce the maximum
/// movement within any one chunk, but they increase chunk count, allocation
/// count, boundary crossings, and indexed lookup metadata. Larger chunks reduce
/// that overhead and can improve cursor-local mutation performance, at the cost
/// of moving more elements within the edited chunk when a middle insertion or
/// removal occurs. Capacities around 32-64 are usually a better starting point
/// for real workloads than the minimum supported size.
///
/// # Fragmentation
/// Middle insertions and removals may leave spare capacity inside chunks. After
/// a middle removal that leaves the edited chunk non-empty, or after a middle
/// insertion that splits a full chunk, the edited chunk is locally refilled up
/// to half capacity from immediate sibling chunks when those siblings contain
/// more than half capacity. Siblings at or below half capacity are left
/// untouched.
///
/// This is a local minimum-occupancy heuristic, not a global invariant. The
/// minimum is `N / 2` using integer division, so odd capacities round down.
/// Front/back pushes and pops keep their direct boundary behavior, so edge
/// chunks may be less than half full. Use [`shrink`](Self::shrink) when
/// explicit front compaction is useful.
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
///
/// ```compile_fail
/// use array_list::ArrayList;
///
/// let mut list: ArrayList<i32, 3> = ArrayList::new();
/// list.push_back(1);
/// ```
pub struct ArrayList<T, const N: usize> {
    chunks: VecDeque<CapVec<T, N>>,
    len: usize,
}

impl<T, const N: usize, const M: usize> From<[T; M]> for ArrayList<T, N> {
    fn from(values: [T; M]) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        values.into_iter().collect()
    }
}

impl<T, const N: usize> FromIterator<T> for ArrayList<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let mut this = Self::new();
        this.extend(iter);
        this
    }
}

impl<T, const N: usize> Extend<T> for ArrayList<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let mut iter = iter.into_iter();

        let (lower_bound, _) = iter.size_hint();
        if lower_bound > 0 {
            let tail_spare = self
                .chunks
                .back()
                .map_or(0, |chunk| N.saturating_sub(chunk.len()));
            let new_chunks = lower_bound.saturating_sub(tail_spare).div_ceil(N);
            self.chunks.reserve(new_chunks);
        }

        if let Some(chunk) = self.chunks.back_mut() {
            let old_len = chunk.len();
            chunk.extend(&mut iter);
            self.len += chunk.len() - old_len;
        }

        while let Some(value) = iter.next() {
            let mut chunk = CapVec::new();
            chunk.push(value).unwrap_or_unreachable();
            chunk.extend(&mut iter);
            self.len += chunk.len();
            self.chunks.push_back(chunk);
        }
    }
}

impl<'a, T, const N: usize> Extend<&'a T> for ArrayList<T, N>
where
    T: Clone,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.extend(iter.into_iter().cloned());
    }
}

impl<T, const N: usize> Default for ArrayList<T, N> {
    fn default() -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self::new()
    }
}

impl<T, const N: usize> ArrayList<T, N> {
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        match self.chunks.front_mut() {
            Some(chunk) if chunk.len() < N => chunk.insert(0, value).unwrap_or_unreachable(),
            _ => {
                let mut chunk = CapVec::new();
                chunk.push(value).unwrap_or_unreachable();
                self.chunks.push_front(chunk);
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        match self.chunks.back_mut() {
            Some(chunk) if chunk.len() < N => chunk.push(value).unwrap_or_unreachable(),
            _ => {
                let mut chunk = CapVec::new();
                chunk.push(value).unwrap_or_unreachable();
                self.chunks.push_back(chunk);
            }
        }

        self.len += 1;
    }

    /// Inserts an element at `index`, shifting that element and all later
    /// elements to the right.
    ///
    /// If the target chunk is full, insertion splits that chunk locally. When a
    /// middle insertion splits a full chunk, the edited chunk may be refilled up
    /// to half capacity from immediate sibling chunks that contain more than
    /// half capacity.
    /// Inserting at [`len`](Self::len) is equivalent to
    /// [`push_back`](Self::push_back).
    ///
    /// # Panics
    /// - Panics if the `index` is out of bounds (greater than the list's current length).
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i64, 4> = ArrayList::new();
    /// list.push_back(10);
    /// list.push_back(30);
    /// list.insert(1, 20);
    ///
    /// assert_eq!(list.get(0), Some(&10));
    /// assert_eq!(list.get(1), Some(&20));
    /// assert_eq!(list.get(2), Some(&30));
    /// ```
    pub fn insert(&mut self, index: usize, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        assert!(index <= self.len());

        if index == 0 {
            self.push_front(value);
            return;
        }

        if index == self.len {
            self.push_back(value);
            return;
        }

        let mut position = self.position_at(index).unwrap();
        self.insert_before_position(&mut position, value);
    }

    /// Inserts `value` before `position` and keeps `position` pointing at the
    /// same logical element, now shifted right by the inserted value.
    ///
    /// `position` must point at an existing element. This helper is used for
    /// middle insertions only; front insertions are handled by the direct
    /// `push_front` path. When the target chunk is full, the chunk is split
    /// before insertion; if the edited chunk is under the local minimum after
    /// insertion, nearby siblings may refill it. `position` is adjusted across
    /// those storage moves so it still names the original element.
    fn insert_before_position(&mut self, position: &mut Position, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        assert!(position.element < self.len);
        assert!(position.chunk < self.chunks.len());
        assert!(position.slot < self.chunks[position.chunk].len());

        if position.element == 0 {
            self.push_front(value);
            *position = Position::front(self);
            position.move_next(self);
            return;
        }

        let target_chunk_was_full = self.chunks[position.chunk].len() >= N;

        if target_chunk_was_full {
            let split_at = Ord::max(position.slot, Self::local_min_occupancy());
            self.split_chunk_at(position.chunk, split_at);

            if position.slot >= split_at {
                position.chunk += 1;
                position.slot -= split_at;
            }
        }

        let inserted = *position;
        position.element += 1;
        position.slot += 1;

        let chunk = &mut self.chunks[inserted.chunk];
        chunk.insert(inserted.slot, value).unwrap_or_unreachable();
        self.len += 1;

        if target_chunk_was_full {
            self.refill_chunk_to_local_min_occupancy(inserted.chunk, position);
        }
    }

    /// Inserts `value` after `position` and keeps `position` pointing at the
    /// same logical element.
    ///
    /// `position` must point at an existing element. The back edge is handled
    /// directly with `push_back`. Otherwise, inserting after the current element
    /// is equivalent to inserting before the next element, so this temporarily
    /// moves `position` forward, delegates to `insert_before_position`, then
    /// walks back over the shifted target and the inserted value.
    fn insert_after_position(&mut self, position: &mut Position, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        assert!(position.element < self.len);
        assert!(position.chunk < self.chunks.len());
        assert!(position.slot < self.chunks[position.chunk].len());

        if (position.element + 1) == self.len {
            self.push_back(value);
            return;
        }

        position.move_next(self);
        self.insert_before_position(position, value);

        position.move_prev(self);
        position.move_prev(self);
    }

    /// Moves all elements from `other` to the back of this list.
    ///
    /// This preserves the logical order of both lists. When the tail chunk of
    /// this list has spare capacity, elements are first drained from the head of
    /// `other` to fill that boundary chunk; remaining chunks are then moved
    /// wholesale. After this operation, `other` is empty.
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if let Some(tail) = self.chunks.back_mut() {
            while tail.len() < N {
                let Some(head) = other.chunks.front_mut() else {
                    break;
                };

                let items_to_take = (N - tail.len()).min(head.len());
                if items_to_take > 0 {
                    tail.extend(head.drain(..items_to_take));
                }

                if head.is_empty() {
                    other.chunks.pop_front();
                }
            }
        }

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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let chunk = self.chunks.front_mut()?;

        let value = chunk.remove(0);
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let chunk = self.chunks.back_mut()?;

        let value = chunk.pop();
        if chunk.is_empty() {
            self.chunks.pop_back();
        }

        self.len -= 1;
        value
    }

    /// Removes and returns the element at `index`.
    ///
    /// Elements after `index` keep their relative order. After a middle removal
    /// that leaves the edited chunk non-empty, that chunk may be refilled up to
    /// half capacity from immediate sibling chunks that contain more than half
    /// capacity.
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
    /// assert_eq!(list.remove(10), None);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if index >= self.len {
            return None;
        }

        if index == 0 {
            return self.pop_front();
        }

        if index == self.len - 1 {
            return self.pop_back();
        }

        let mut position = self.position_at(index).unwrap();
        self.remove_at_position(&mut position)
    }

    /// Removes and returns the element at `position`.
    ///
    /// `position` must point at an existing element. After removal, it is
    /// updated to the next logical element, or to the ghost position when the
    /// removed element was the back element. If a middle removal leaves the
    /// edited chunk empty, that chunk is removed. Otherwise, the edited chunk
    /// may be locally refilled from siblings; `position` is adjusted across
    /// those storage moves and across empty chunk removal.
    fn remove_at_position(&mut self, position: &mut Position) -> Option<T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        assert!(position.element < self.len);
        assert!(position.chunk < self.chunks.len());
        assert!(position.slot < self.chunks[position.chunk].len());

        let current = *position;

        // Edge removals delegate to the direct pop paths, then normalize the
        // cursor position to the documented post-removal location.
        if current.element == 0 {
            let value = self.pop_front()?;
            *position = Position::front(self);
            return Some(value);
        }

        if current.element == self.len - 1 {
            let value = self.pop_back()?;
            *position = Position::ghost(self);
            return Some(value);
        }

        // Reuse `position` as the tracked successor before changing storage,
        // while the old chunk/slot coordinates are still valid.
        position.move_next(self);

        let value = self.chunks[current.chunk].remove(current.slot)?;
        self.len -= 1;

        // The successor takes the removed element's logical index. Its slot
        // only changes here when it was stored in the edited chunk.
        position.element = current.element;
        if position.chunk == current.chunk {
            position.slot = current.slot;
        }

        if self.chunks[current.chunk].is_empty() {
            self.chunks.remove(current.chunk);
            // If the tracked successor is after the removed empty chunk, its
            // chunk index shifts left with the remaining storage.
            if position.chunk > current.chunk {
                position.chunk -= 1;
            }
        } else {
            self.refill_chunk_to_local_min_occupancy(current.chunk, position);
        }

        Some(value)
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.chunks.front().and_then(|chunk| chunk.first())
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.chunks.front_mut().and_then(|chunk| chunk.first_mut())
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.chunks.back().and_then(|chunk| chunk.last())
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.chunks.back_mut().and_then(|chunk| chunk.last_mut())
    }

    /// Returns a reference to the element at `index`, if any.
    ///
    /// Index lookup scans chunk metadata to find the chunk containing `index`.
    /// Fragmented lists may require scanning more chunks than compact lists.
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
    pub fn get(&self, index: usize) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let position = self.position_at(index)?;
        self.chunks[position.chunk].get(position.slot)
    }

    /// Returns a mutable reference to the element at `index`, if any.
    ///
    /// Index lookup scans chunk metadata to find the chunk containing `index`.
    /// Fragmented lists may require scanning more chunks than compact lists.
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
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let position = self.position_at(index)?;
        self.chunks[position.chunk].get_mut(position.slot)
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len
    }

    /// Returns the total number of elements that can fit in the currently allocated chunks.
    ///
    /// This is the sum of the capacity of every internal chunk. It is always at
    /// least [`len`](Self::len), but it may be larger after edits until the list
    /// is compacted with [`shrink`](Self::shrink).
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::new();
    /// assert_eq!(list.capacity(), 0);
    ///
    /// list.push_back(1);
    /// assert!(list.capacity() >= list.len());
    /// ```
    pub fn capacity(&self) -> usize {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.chunks.iter().map(CapVec::capacity).sum()
    }

    /// Returns the number of unused element slots in allocated chunks.
    ///
    /// This is [`capacity`](Self::capacity) minus [`len`](Self::len). A larger
    /// value means the list has more spare chunk storage, usually after middle
    /// edits, front-heavy insertions, or uneven appends.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3, 4, 5]);
    /// list.remove(1);
    ///
    /// assert_eq!(list.spare_capacity(), list.capacity() - list.len());
    /// ```
    pub fn spare_capacity(&self) -> usize {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.capacity() - self.len()
    }

    /// Compacts elements toward the front of the list.
    ///
    /// This preserves element order and length. It can reduce spare capacity
    /// after removals or uneven insertions by filling earlier chunks from later
    /// chunks and removing chunks that become empty. It does not guarantee that
    /// all allocation capacity held by the internal `VecDeque` is released.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3]);
    /// list.remove(1);
    /// list.shrink();
    ///
    /// assert_eq!(list, [1, 3]);
    /// ```
    pub fn shrink(&mut self) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.compact_from(0);
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len == 0
    }

    /// Provides an iterator over the list's elements.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<_, 4> = ArrayList::new();
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
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Iter::from_list(self)
    }

    /// Provides a mutable iterator over the list's elements.
    ///
    /// # Examples
    /// ```
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<_, 4> = ArrayList::new();
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// let mut iter = list.iter_mut();
    /// assert_eq!(iter.next(), Some(&mut 0));
    /// assert_eq!(iter.next(), Some(&mut 1));
    /// assert_eq!(iter.next(), Some(&mut 2));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        IterMut::from_list(self)
    }

    /// Provides a read-only cursor at the front element.
    ///
    /// The cursor points to the ghost non-element if the list is empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.index(), Some(0));
    /// assert_eq!(cursor.current(), Some(&1));
    ///
    /// let empty: ArrayList<i32, 4> = ArrayList::new();
    /// assert_eq!(empty.cursor_front().index(), None);
    /// ```
    #[inline]
    pub fn cursor_front(&self) -> Cursor<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Cursor::from_front(self)
    }

    /// Provides a read-only cursor at the back element.
    ///
    /// The cursor points to the ghost non-element if the list is empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_back();
    ///
    /// assert_eq!(cursor.index(), Some(1));
    /// assert_eq!(cursor.current(), Some(&2));
    ///
    /// let empty: ArrayList<i32, 4> = ArrayList::new();
    /// assert_eq!(empty.cursor_back().index(), None);
    /// ```
    #[inline]
    pub fn cursor_back(&self) -> Cursor<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Cursor::from_back(self)
    }

    /// Provides a mutable cursor at the front element.
    ///
    /// The cursor points to the ghost non-element if the list is empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.index(), Some(0));
    /// assert_eq!(cursor.current(), Some(&mut 1));
    ///
    /// let mut empty: ArrayList<i32, 4> = ArrayList::new();
    /// assert_eq!(empty.cursor_front_mut().index(), None);
    /// ```
    #[inline]
    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        CursorMut::from_front(self)
    }

    /// Provides a mutable cursor at the back element.
    ///
    /// The cursor points to the ghost non-element if the list is empty.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_back_mut();
    ///
    /// assert_eq!(cursor.index(), Some(1));
    /// assert_eq!(cursor.current(), Some(&mut 2));
    ///
    /// let mut empty: ArrayList<i32, 4> = ArrayList::new();
    /// assert_eq!(empty.cursor_back_mut().index(), None);
    /// ```
    #[inline]
    pub fn cursor_back_mut(&mut self) -> CursorMut<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        CursorMut::from_back(self)
    }

    fn position_at(&self, mut index: usize) -> Option<Position> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let element = index;

        if index >= self.len() {
            return None;
        }

        if self.chunks.len() == 1 || self.chunks.len().checked_mul(N) == Some(self.len) {
            return Some(Position {
                element,
                chunk: index / N,
                slot: index % N,
            });
        }

        if index <= self.len() / 2 {
            for (chunk_index, chunk) in self.chunks.iter().enumerate() {
                let chunk_len = chunk.len();
                if index < chunk_len {
                    return Some(Position {
                        element,
                        chunk: chunk_index,
                        slot: index,
                    });
                }

                index -= chunk_len;
            }

            return None;
        }

        let mut remaining_len = self.len();
        for chunk_index in (0..self.chunks.len()).rev() {
            remaining_len -= self.chunks[chunk_index].len();

            if index >= remaining_len {
                return Some(Position {
                    element,
                    chunk: chunk_index,
                    slot: index - remaining_len,
                });
            }
        }

        None
    }

    fn split_chunk_at(&mut self, chunk_index: usize, split_at: usize) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let mut next = CapVec::new();
        let chunk = &mut self.chunks[chunk_index];
        next.extend(chunk.drain(split_at..));

        self.chunks.insert(chunk_index + 1, next);
    }

    const fn local_min_occupancy() -> usize {
        N / 2
    }

    /// Refills `chunk_index` up to the local minimum occupancy when possible.
    ///
    /// This is a local repair step used after middle insertions split a chunk
    /// and after middle removals shrink a non-empty chunk. It only pulls from
    /// immediate siblings while those siblings remain above the local minimum
    /// occupancy, preferring the previous sibling before the next sibling.
    /// `tracked` is a position that must continue to point at the same logical
    /// element while values move across chunk boundaries.
    fn refill_chunk_to_local_min_occupancy(&mut self, chunk_index: usize, tracked: &mut Position) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let min_occupancy = Self::local_min_occupancy();
        if chunk_index >= self.chunks.len() {
            return;
        }

        if chunk_index > 0 {
            let previous_chunk = chunk_index - 1;

            // Pull from the previous sibling first. Its last element becomes
            // the new first element of the edited chunk, preserving order.
            while self.chunks[chunk_index].len() < min_occupancy
                && self.chunks[previous_chunk].len() > min_occupancy
            {
                let moved_slot = self.chunks[previous_chunk].len() - 1;
                let value = self.chunks[previous_chunk].pop().unwrap();

                // Keep `tracked` on the same logical element when that element
                // is moved, or when prepending shifts slots in the edited chunk.
                if tracked.chunk == previous_chunk && tracked.slot == moved_slot {
                    tracked.chunk = chunk_index;
                    tracked.slot = 0;
                } else if tracked.chunk == chunk_index {
                    tracked.slot += 1;
                }

                self.chunks[chunk_index]
                    .insert(0, value)
                    .unwrap_or_unreachable();
            }
        }

        let next_chunk = chunk_index + 1;
        if next_chunk < self.chunks.len() {
            // Then pull from the next sibling. Its first element becomes the
            // new last element of the edited chunk.
            while self.chunks[chunk_index].len() < min_occupancy
                && self.chunks[next_chunk].len() > min_occupancy
            {
                let target_slot = self.chunks[chunk_index].len();
                let value = self.chunks[next_chunk].remove(0).unwrap();

                // The moved first element changes chunks; later elements in
                // the next sibling shift left by one slot.
                if tracked.chunk == next_chunk {
                    if tracked.slot == 0 {
                        tracked.chunk = chunk_index;
                        tracked.slot = target_slot;
                    } else {
                        tracked.slot -= 1;
                    }
                }

                self.chunks[chunk_index].push(value).unwrap_or_unreachable();
            }
        }
    }

    fn compact_from(&mut self, chunk_index: usize) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.chunks.len() <= 1 {
            return;
        }

        let mut chunk_index = chunk_index.min(self.chunks.len() - 1);

        while chunk_index + 1 < self.chunks.len() {
            if self.chunks[chunk_index].len() >= N {
                chunk_index += 1;
                continue;
            }

            while self.chunks[chunk_index].len() < N && chunk_index + 1 < self.chunks.len() {
                let mut source = self.chunks.remove(chunk_index + 1).unwrap();
                let missing = N - self.chunks[chunk_index].len();
                let take = missing.min(source.len());

                if take > 0 {
                    self.chunks[chunk_index].extend(source.drain(..take));
                }

                if !source.is_empty() {
                    self.chunks.insert(chunk_index + 1, source);
                }
            }

            chunk_index += 1;
        }
    }
}

impl<T: Clone, const N: usize> Clone for ArrayList<T, N> {
    fn clone(&self) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            chunks: self.chunks.clone(),
            len: self.len,
        }
    }
}

impl<T, const N: usize, const M: usize> PartialEq<[T; M]> for ArrayList<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T; M]) -> bool {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> PartialEq<&[T]> for ArrayList<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &&[T]) -> bool {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T, const N: usize> PartialEq<[T]> for ArrayList<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> PartialEq for ArrayList<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, const N: usize> Eq for ArrayList<T, N> where T: Eq {}

impl<T, const N: usize> PartialOrd for ArrayList<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> Ord for ArrayList<T, N>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.iter().cmp(other)
    }
}

impl<T, const N: usize> Hash for ArrayList<T, N>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        state.write_usize(self.len());
        self.iter().for_each(|v| v.hash(state));
    }
}

impl<T, const N: usize> core::fmt::Debug for ArrayList<T, N>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        f.debug_list().entries(self.chunks.iter()).finish()
    }
}

impl<T, const N: usize> IntoIterator for ArrayList<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        IntoIter::from_list(self)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayList<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Iter::from_list(self)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayList<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.iter_mut()
    }
}

trait UnwrapExt {
    type Output;

    fn unwrap_or_unreachable(self) -> Self::Output;
}

impl<T, E> UnwrapExt for Result<T, E> {
    type Output = T;

    fn unwrap_or_unreachable(self) -> Self::Output {
        match self {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::{ArrayList, Cursor, UnwrapExt};
    use alloc::collections::VecDeque;
    use alloc::rc::Rc;
    use alloc::vec::Vec;
    use cap_vec::CapVec;
    use core::cell::Cell;
    use core::cmp::Ordering;
    use core::hash::{Hash, Hasher};
    use proptest::prelude::*;

    fn values<const N: usize>(list: &ArrayList<i32, N>) -> Vec<i32> {
        list.iter().copied().collect()
    }

    fn chunk_lengths<const N: usize>(list: &ArrayList<i32, N>) -> Vec<usize> {
        list.chunks.iter().map(|chunk| chunk.len()).collect()
    }

    fn usize_chunk<const N: usize>(values: impl IntoIterator<Item = usize>) -> CapVec<usize, N> {
        let mut chunk = CapVec::new();
        for value in values {
            chunk.push(value).unwrap_or_unreachable();
        }
        chunk
    }

    fn assert_matches_model<const N: usize>(list: &ArrayList<i32, N>, model: &[i32]) {
        assert_eq!(list.len(), model.len());
        assert_eq!(list.is_empty(), model.is_empty());
        assert_eq!(values(list), model);
        assert_eq!(list.iter().count(), model.len());
        assert!(list.capacity() >= list.len());
        assert_eq!(list.spare_capacity(), list.capacity() - list.len());
        assert_eq!(list.front(), model.first());
        assert_eq!(list.back(), model.last());
        assert_eq!(list.get(model.len()), None);
        assert!(list.chunks.iter().all(|chunk| !chunk.is_empty()));
        assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        assert_eq!(
            list.chunks.iter().map(|chunk| chunk.len()).sum::<usize>(),
            list.len()
        );

        for (index, value) in model.iter().enumerate() {
            assert_eq!(list.get(index), Some(value));
        }
    }

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    #[derive(Clone, Debug)]
    enum ModelOp {
        PushFront(i32),
        PushBack(i32),
        PopFront,
        PopBack,
        Insert { index: usize, value: i32 },
        Remove { index: usize },
        Clear,
        Shrink,
    }

    fn model_op_strategy() -> impl Strategy<Value = ModelOp> {
        prop_oneof![
            any::<i8>().prop_map(|value| ModelOp::PushFront(value.into())),
            any::<i8>().prop_map(|value| ModelOp::PushBack(value.into())),
            Just(ModelOp::PopFront),
            Just(ModelOp::PopBack),
            (0usize..32, any::<i8>()).prop_map(|(index, value)| ModelOp::Insert {
                index,
                value: value.into()
            }),
            (0usize..32).prop_map(|index| ModelOp::Remove { index }),
            Just(ModelOp::Clear),
            Just(ModelOp::Shrink),
        ]
    }

    fn apply_model_ops<const N: usize>(ops: &[ModelOp]) {
        let mut list = ArrayList::<i32, N>::new();
        let mut model = Vec::new();

        assert_matches_model(&list, &model);

        for op in ops {
            match *op {
                ModelOp::PushFront(value) => {
                    list.push_front(value);
                    model.insert(0, value);
                }
                ModelOp::PushBack(value) => {
                    list.push_back(value);
                    model.push(value);
                }
                ModelOp::PopFront => {
                    let expected = (!model.is_empty()).then(|| model.remove(0));
                    assert_eq!(list.pop_front(), expected);
                }
                ModelOp::PopBack => {
                    assert_eq!(list.pop_back(), model.pop());
                }
                ModelOp::Insert { index, value } => {
                    let index = if model.is_empty() {
                        0
                    } else {
                        index % (model.len() + 1)
                    };
                    list.insert(index, value);
                    model.insert(index, value);
                }
                ModelOp::Remove { index } => {
                    let expected = (index < model.len()).then(|| model.remove(index));
                    assert_eq!(list.remove(index), expected);
                }
                ModelOp::Clear => {
                    list.clear();
                    model.clear();
                }
                ModelOp::Shrink => {
                    list.shrink();
                }
            }

            assert_matches_model(&list, &model);
        }
    }

    #[derive(Clone, Debug)]
    enum CursorModelOp {
        MoveNext,
        MovePrev,
        InsertBefore(i32),
        InsertAfter(i32),
        PushFront(i32),
        PushBack(i32),
        PopFront,
        PopBack,
        RemoveCurrent,
    }

    fn cursor_model_op_strategy() -> impl Strategy<Value = CursorModelOp> {
        prop_oneof![
            Just(CursorModelOp::MoveNext),
            Just(CursorModelOp::MovePrev),
            any::<i8>().prop_map(|value| CursorModelOp::InsertBefore(value.into())),
            any::<i8>().prop_map(|value| CursorModelOp::InsertAfter(value.into())),
            any::<i8>().prop_map(|value| CursorModelOp::PushFront(value.into())),
            any::<i8>().prop_map(|value| CursorModelOp::PushBack(value.into())),
            Just(CursorModelOp::PopFront),
            Just(CursorModelOp::PopBack),
            Just(CursorModelOp::RemoveCurrent),
        ]
    }

    fn model_move_next(cursor: &mut Option<usize>, len: usize) {
        *cursor = match *cursor {
            None => (len > 0).then_some(0),
            Some(index) if index + 1 < len => Some(index + 1),
            Some(_) => None,
        };
    }

    fn model_move_prev(cursor: &mut Option<usize>, len: usize) {
        *cursor = match *cursor {
            None => len.checked_sub(1),
            Some(0) => None,
            Some(index) => Some(index - 1),
        };
    }

    fn apply_cursor_model_ops<const N: usize>(ops: &[CursorModelOp]) {
        let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3]);
        let mut model = Vec::from([0, 1, 2, 3]);
        let mut model_cursor = Some(0usize);

        for op in ops {
            let actual_index = {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..model_cursor.unwrap_or(model.len()) {
                    cursor.move_next();
                }

                match *op {
                    CursorModelOp::MoveNext => {
                        cursor.move_next();
                        model_move_next(&mut model_cursor, model.len());
                    }
                    CursorModelOp::MovePrev => {
                        cursor.move_prev();
                        model_move_prev(&mut model_cursor, model.len());
                    }
                    CursorModelOp::InsertBefore(value) => {
                        cursor.insert_before(value);
                        let index = model_cursor.unwrap_or(model.len());
                        model.insert(index, value);
                        if let Some(cursor_index) = &mut model_cursor {
                            *cursor_index += 1;
                        }
                    }
                    CursorModelOp::InsertAfter(value) => {
                        cursor.insert_after(value);
                        let index = model_cursor.map_or(0, |index| index + 1);
                        model.insert(index, value);
                    }
                    CursorModelOp::PushFront(value) => {
                        cursor.push_front(value);
                        model.insert(0, value);
                        if let Some(cursor_index) = &mut model_cursor {
                            *cursor_index += 1;
                        }
                    }
                    CursorModelOp::PushBack(value) => {
                        cursor.push_back(value);
                        model.push(value);
                    }
                    CursorModelOp::PopFront => {
                        let expected = (!model.is_empty()).then(|| model.remove(0));
                        assert_eq!(cursor.pop_front(), expected);
                        if let Some(cursor_index) = &mut model_cursor {
                            *cursor_index = cursor_index.saturating_sub(1);
                        }
                        if model_cursor.is_some_and(|index| index >= model.len()) {
                            model_cursor = None;
                        }
                    }
                    CursorModelOp::PopBack => {
                        assert_eq!(cursor.pop_back(), model.pop());
                        if model_cursor.is_some_and(|index| index >= model.len()) {
                            model_cursor = None;
                        }
                    }
                    CursorModelOp::RemoveCurrent => {
                        let expected = model_cursor.map(|index| model.remove(index));
                        assert_eq!(cursor.remove_current(), expected);
                        if model_cursor.is_some_and(|index| index >= model.len()) {
                            model_cursor = None;
                        }
                    }
                };

                cursor.index()
            };

            assert_eq!(actual_index, model_cursor);
            assert_matches_model(&list, &model);
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum ReadOnlyCursorModelOp {
        MoveNext,
        MovePrev,
    }

    fn read_only_cursor_model_op_strategy() -> impl Strategy<Value = ReadOnlyCursorModelOp> {
        prop_oneof![
            Just(ReadOnlyCursorModelOp::MoveNext),
            Just(ReadOnlyCursorModelOp::MovePrev),
        ]
    }

    fn assert_cursor_matches_model<const N: usize>(
        cursor: &Cursor<'_, i32, N>,
        model: &[i32],
        model_cursor: Option<usize>,
    ) {
        assert_eq!(cursor.index(), model_cursor);
        assert_eq!(cursor.current(), model_cursor.map(|index| &model[index]));
        assert_eq!(cursor.front(), model.first());
        assert_eq!(cursor.back(), model.last());
        assert_eq!(cursor.as_list().len(), model.len());

        let expected_next =
            model_cursor.map_or_else(|| model.first(), |index| model.get(index.saturating_add(1)));
        let expected_prev = match model_cursor {
            None => model.last(),
            Some(0) => None,
            Some(index) => model.get(index - 1),
        };

        assert_eq!(cursor.peek_next(), expected_next);
        assert_eq!(cursor.peek_prev(), expected_prev);
    }

    fn apply_read_only_cursor_model_ops<const N: usize>(ops: &[ReadOnlyCursorModelOp]) {
        let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let model = Vec::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut cursor = list.cursor_front();
        let mut model_cursor = Some(0usize);

        assert_cursor_matches_model(&cursor, &model, model_cursor);

        for op in ops {
            match op {
                ReadOnlyCursorModelOp::MoveNext => {
                    cursor.move_next();
                    model_move_next(&mut model_cursor, model.len());
                }
                ReadOnlyCursorModelOp::MovePrev => {
                    cursor.move_prev();
                    model_move_prev(&mut model_cursor, model.len());
                }
            }

            assert_cursor_matches_model(&cursor, &model, model_cursor);
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum IteratorModelOp {
        Next,
        NextBack,
        Nth(usize),
        NthBack(usize),
    }

    fn iterator_model_op_strategy() -> impl Strategy<Value = IteratorModelOp> {
        prop_oneof![
            Just(IteratorModelOp::Next),
            Just(IteratorModelOp::NextBack),
            (0usize..8).prop_map(IteratorModelOp::Nth),
            (0usize..8).prop_map(IteratorModelOp::NthBack),
        ]
    }

    fn model_nth(model: &mut VecDeque<i32>, n: usize) -> Option<i32> {
        if n >= model.len() {
            model.clear();
            return None;
        }

        for _ in 0..n {
            model.pop_front();
        }

        model.pop_front()
    }

    fn model_nth_back(model: &mut VecDeque<i32>, n: usize) -> Option<i32> {
        if n >= model.len() {
            model.clear();
            return None;
        }

        for _ in 0..n {
            model.pop_back();
        }

        model.pop_back()
    }

    fn apply_iter_model_ops<const N: usize>(ops: &[IteratorModelOp]) {
        let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut iter = list.iter();
        let mut model = VecDeque::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        for op in ops {
            let expected = match *op {
                IteratorModelOp::Next => model.pop_front(),
                IteratorModelOp::NextBack => model.pop_back(),
                IteratorModelOp::Nth(n) => model_nth(&mut model, n),
                IteratorModelOp::NthBack(n) => model_nth_back(&mut model, n),
            };

            let actual = match *op {
                IteratorModelOp::Next => iter.next().copied(),
                IteratorModelOp::NextBack => iter.next_back().copied(),
                IteratorModelOp::Nth(n) => iter.nth(n).copied(),
                IteratorModelOp::NthBack(n) => iter.nth_back(n).copied(),
            };

            assert_eq!(actual, expected);
        }

        assert_eq!(
            iter.copied().collect::<Vec<_>>(),
            model.into_iter().collect::<Vec<_>>()
        );
    }

    fn apply_iter_mut_model_ops<const N: usize>(ops: &[IteratorModelOp]) {
        let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut iter = list.iter_mut();
        let mut model = VecDeque::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        for op in ops {
            let expected = match *op {
                IteratorModelOp::Next => model.pop_front(),
                IteratorModelOp::NextBack => model.pop_back(),
                IteratorModelOp::Nth(n) => model_nth(&mut model, n),
                IteratorModelOp::NthBack(n) => model_nth_back(&mut model, n),
            };

            let actual = match *op {
                IteratorModelOp::Next => iter.next().map(|value| {
                    *value += 100;
                    *value - 100
                }),
                IteratorModelOp::NextBack => iter.next_back().map(|value| {
                    *value += 100;
                    *value - 100
                }),
                IteratorModelOp::Nth(n) => iter.nth(n).map(|value| {
                    *value += 100;
                    *value - 100
                }),
                IteratorModelOp::NthBack(n) => iter.nth_back(n).map(|value| {
                    *value += 100;
                    *value - 100
                }),
            };

            assert_eq!(actual, expected);
        }

        assert_eq!(
            iter.map(|value| *value).collect::<Vec<_>>(),
            model.into_iter().collect::<Vec<_>>()
        );
    }

    fn apply_into_iter_model_ops<const N: usize>(ops: &[IteratorModelOp]) {
        let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut iter = list.into_iter();
        let mut model = VecDeque::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        for op in ops {
            let expected = match *op {
                IteratorModelOp::Next => model.pop_front(),
                IteratorModelOp::NextBack => model.pop_back(),
                IteratorModelOp::Nth(n) => model_nth(&mut model, n),
                IteratorModelOp::NthBack(n) => model_nth_back(&mut model, n),
            };

            let actual = match *op {
                IteratorModelOp::Next => iter.next(),
                IteratorModelOp::NextBack => iter.next_back(),
                IteratorModelOp::Nth(n) => iter.nth(n),
                IteratorModelOp::NthBack(n) => iter.nth_back(n),
            };

            assert_eq!(actual, expected);
        }

        assert_eq!(
            iter.collect::<Vec<_>>(),
            model.into_iter().collect::<Vec<_>>()
        );
    }

    #[derive(Default)]
    struct RecordingHasher {
        bytes: Vec<u8>,
    }

    impl Hasher for RecordingHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.bytes.extend_from_slice(bytes);
        }

        fn finish(&self) -> u64 {
            self.bytes.len() as u64
        }
    }

    #[derive(Debug)]
    struct DropProbe {
        value: i32,
        drops: Rc<Cell<usize>>,
    }

    impl DropProbe {
        fn new(value: i32, drops: &Rc<Cell<usize>>) -> Self {
            Self {
                value,
                drops: Rc::clone(drops),
            }
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    fn drop_probe_list<const N: usize>(
        drops: &Rc<Cell<usize>>,
        len: i32,
    ) -> ArrayList<DropProbe, N> {
        (0..len).map(|value| DropProbe::new(value, drops)).collect()
    }

    #[test]
    fn default_and_new_start_empty_without_allocated_chunks() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::new();
            let default = ArrayList::<i32, N>::default();

            assert!(list.is_empty());
            assert_eq!(list.len(), 0);
            assert_eq!(list.capacity(), 0);
            assert_eq!(list.front(), None);
            assert_eq!(list.back(), None);
            assert!(default.is_empty());
        }

        check_capacities!(check);
    }

    #[test]
    fn push_and_pop_work_across_chunks() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::new();

            list.push_back(2);
            list.push_front(1);
            list.push_back(3);

            assert_eq!(values(&list), [1, 2, 3]);
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
            assert_eq!(list.pop_front(), Some(1));
            assert_eq!(list.pop_back(), Some(3));
            assert_eq!(list.pop_back(), Some(2));
            assert_eq!(list.pop_back(), None);
            assert!(list.is_empty());
            assert_eq!(list.capacity(), 0);
        }

        check_capacities!(check);
    }

    #[test]
    fn push_front_reuses_spare_front_chunk_capacity() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::new();

            list.push_front(3);
            list.push_front(2);
            list.push_front(1);

            assert_eq!(values(&list), [1, 2, 3]);
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
            assert_eq!(list.front(), Some(&1));
            assert_eq!(list.back(), Some(&3));
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_covers_front_middle_back_and_chunk_spill() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([10, 20, 40, 50]);

            list.insert(0, 0);
            list.insert(3, 30);
            list.insert(list.len(), 60);

            assert_eq!(values(&list), [0, 10, 20, 30, 40, 50, 60]);
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_without_split_does_not_refill_from_siblings() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..N),
                usize_chunk::<N>(100..101),
                usize_chunk::<N>(200..(200 + min)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };

            list.insert(N, usize::MAX);

            assert_eq!(list.get(N), Some(&usize::MAX));
            assert_eq!(
                list.chunks.iter().map(CapVec::len).collect::<Vec<_>>(),
                [N, 2, min]
            );
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_split_refills_underfull_chunk_from_sibling_above_min() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..N),
                usize_chunk::<N>(100..(100 + N)),
                usize_chunk::<N>(200..(200 + min)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };
            let index = (N * 2) - 1;

            list.insert(index, usize::MAX);

            assert_eq!(list.get(index), Some(&usize::MAX));
            assert_eq!(
                list.chunks.iter().map(CapVec::len).collect::<Vec<_>>(),
                [N, min + 1, min, min]
            );
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_splits_full_chunk_locally() {
        fn check<const N: usize>() {
            let mut list = (0..(N * 2)).collect::<ArrayList<usize, N>>();

            list.insert(N - 1, usize::MAX);

            assert_eq!(list.get(N - 1), Some(&usize::MAX));
            assert_eq!(list.iter().copied().count(), (N * 2) + 1);
            assert!(list.chunks.iter().all(|chunk| !chunk.is_empty()));
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_leaves_siblings_at_min_untouched() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..min),
                usize_chunk::<N>(100..101),
                usize_chunk::<N>(200..(200 + min)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };
            let index = min;

            list.insert(index, usize::MAX);

            assert_eq!(
                list.chunks.iter().map(CapVec::len).collect::<Vec<_>>(),
                [min, 2, min]
            );
            assert_eq!(list.get(index), Some(&usize::MAX));
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_insert_before_keeps_cursor_on_element_spilled_to_next_chunk() {
        fn check<const N: usize>() {
            let mut list = (0..(N * 2)).collect::<ArrayList<usize, N>>();

            {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..(N - 1) {
                    cursor.move_next();
                }

                cursor.insert_before(usize::MAX);

                assert_eq!(cursor.current(), Some(&mut (N - 1)));
            }

            let expected = (0..(N - 1))
                .chain(core::iter::once(usize::MAX))
                .chain((N - 1)..(N * 2))
                .collect::<Vec<_>>();

            assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected);
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_insert_after_keeps_cursor_on_element_shifted_to_previous_chunk() {
        fn check<const N: usize>() {
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..(N - 1)),
                usize_chunk::<N>(100..(100 + N)),
                usize_chunk::<N>(200..(200 + N)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };

            {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..(N - 1) {
                    cursor.move_next();
                }

                cursor.insert_after(usize::MAX);

                assert_eq!(cursor.current(), Some(&mut 100));
            }

            let expected = (0..(N - 1))
                .chain(core::iter::once(100))
                .chain(core::iter::once(usize::MAX))
                .chain(101..(100 + N))
                .chain(200..(200 + N))
                .collect::<Vec<_>>();

            assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected);
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_insert_after_tracks_current_moved_by_refill_from_previous_sibling() {
        fn check<const N: usize>() {
            let mut list = (0..N).collect::<ArrayList<usize, N>>();

            {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..(N - 2) {
                    cursor.move_next();
                }

                cursor.insert_after(usize::MAX);

                assert_eq!(cursor.current().map(|value| *value), Some(N - 2));
                assert_eq!(cursor.peek_next().map(|value| *value), Some(usize::MAX));
            }

            let expected = (0..(N - 1))
                .chain(core::iter::once(usize::MAX))
                .chain((N - 1)..N)
                .collect::<Vec<_>>();

            assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected);
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_remove_current_tracks_next_moved_by_refill_from_next_sibling() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..min),
                usize_chunk::<N>(100..101),
                usize_chunk::<N>(200..(200 + N)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };

            {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..min {
                    cursor.move_next();
                }

                assert_eq!(cursor.remove_current(), Some(100));
                assert_eq!(cursor.current(), Some(&mut 200));
            }

            let expected = (0..min).chain(200..(200 + N)).collect::<Vec<_>>();
            assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected);
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_panics_when_index_is_greater_than_len() {
        fn check<const N: usize>() {
            let out = std::panic::catch_unwind(|| {
                let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
                list.insert(4, 4);
            });

            assert!(out.is_err());
        }

        check_capacities!(check);
    }

    #[test]
    fn remove_covers_edges_middle_and_out_of_bounds() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6]);

            assert_eq!(list.remove(0), Some(0));
            assert_eq!(list.remove(5), Some(6));
            assert_eq!(list.remove(2), Some(3));
            assert_eq!(list.remove(99), None);

            assert_eq!(values(&list), [1, 2, 4, 5]);
            assert_eq!(list.len(), 4);
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn repeated_middle_removals_preserve_order_and_chunk_bounds() {
        fn check<const N: usize>() {
            let mut list = (0..(N * 3)).collect::<ArrayList<usize, N>>();

            for _ in 0..N {
                assert!(list.remove(N).is_some());
            }

            assert_eq!(list.len(), N * 2);
            assert_eq!(
                list.iter().copied().collect::<Vec<_>>(),
                (0..N).chain((N * 2)..(N * 3)).collect::<Vec<_>>()
            );
            assert!(list.chunks.iter().all(|chunk| !chunk.is_empty()));
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn remove_refills_underfull_chunk_from_sibling_above_min() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..N),
                usize_chunk::<N>(100..(100 + min)),
                usize_chunk::<N>(200..(200 + min)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };

            assert_eq!(list.remove(N), Some(100));

            let expected_values = (0..N)
                .chain(101..(100 + min))
                .chain(200..(200 + min))
                .collect::<Vec<_>>();

            assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected_values);
            assert_eq!(
                list.chunks.iter().map(CapVec::len).collect::<Vec<_>>(),
                [N - 1, min, min]
            );
        }

        check_capacities!(check);
    }

    #[test]
    fn remove_leaves_siblings_at_min_untouched() {
        fn check<const N: usize>() {
            let min = N / 2;
            let chunks = VecDeque::from([
                usize_chunk::<N>(0..min),
                usize_chunk::<N>(100..(100 + min)),
                usize_chunk::<N>(200..(200 + min)),
            ]);
            let len = chunks.iter().map(CapVec::len).sum();
            let mut list = ArrayList { chunks, len };

            assert_eq!(list.remove(min), Some(100));

            assert_eq!(
                list.chunks.iter().map(CapVec::len).collect::<Vec<_>>(),
                [min, min - 1, min]
            );
            assert_eq!(
                list.iter().copied().collect::<Vec<_>>(),
                (0..min)
                    .chain(101..(100 + min))
                    .chain(200..(200 + min))
                    .collect::<Vec<_>>()
            );
        }

        check_capacities!(check);
    }

    #[test]
    fn get_and_get_mut_cross_chunk_boundaries() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3, 4, 5]);

            assert_eq!(list.get(0), Some(&1));
            assert_eq!(list.get(2), Some(&3));
            assert_eq!(list.get(4), Some(&5));
            assert_eq!(list.get(5), None);

            *list.get_mut(3).unwrap() = 40;
            assert_eq!(values(&list), [1, 2, 3, 40, 5]);
        }

        check_capacities!(check);
    }

    #[test]
    fn append_moves_all_chunks_and_empties_source() {
        fn check<const N: usize>() {
            let mut left = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut right = ArrayList::<i32, N>::from([4, 5]);

            left.append(&mut right);

            assert_eq!(values(&left), [1, 2, 3, 4, 5]);
            assert!(right.is_empty());
            assert_eq!(right.capacity(), 0);
            assert!(left.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn append_compacts_sparse_boundary_chunks() {
        fn check<const N: usize>() {
            let mut left = (0..(N + 1)).collect::<ArrayList<usize, N>>();
            let mut right = ((N + 1)..(N * 2)).collect::<ArrayList<usize, N>>();

            left.append(&mut right);

            assert_eq!(
                left.iter().copied().collect::<Vec<_>>(),
                (0..(N * 2)).collect::<Vec<_>>()
            );
            assert!(right.is_empty());
            assert_eq!(right.capacity(), 0);
            assert_eq!(left.spare_capacity(), 0);
        }

        check_capacities!(check);
    }

    #[test]
    fn clear_removes_elements_and_allocated_chunks() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3, 4]);

            list.clear();

            assert!(list.is_empty());
            assert_eq!(list.capacity(), 0);
            assert_eq!(list.pop_front(), None);
            assert_eq!(chunk_lengths(&list), Vec::<usize>::new());
        }

        check_capacities!(check);
    }

    #[test]
    fn shrink_compacts_sparse_chunks_without_reordering() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7]);

            assert_eq!(list.remove(1), Some(1));
            assert_eq!(list.remove(2), Some(3));

            list.shrink();

            assert_eq!(values(&list), [0, 2, 4, 5, 6, 7]);
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn from_array_collect_and_extend_fill_chunks_across_capacities() {
        fn check<const N: usize>() {
            let from_array = ArrayList::<i32, N>::from([1, 2, 3, 4, 5]);
            let collected = (1..=5).collect::<ArrayList<_, N>>();
            let mut extended = ArrayList::<i32, N>::from([1]);
            extended.extend([2, 3, 4, 5]);
            extended.extend([6, 7].iter());

            assert_eq!(values(&from_array), [1, 2, 3, 4, 5]);
            assert_eq!(values(&collected), [1, 2, 3, 4, 5]);
            assert_eq!(values(&extended), [1, 2, 3, 4, 5, 6, 7]);
            assert!(from_array.chunks.iter().all(|chunk| chunk.len() <= N));
            assert!(collected.chunks.iter().all(|chunk| chunk.len() <= N));
            assert!(extended.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn clone_debug_equality_ordering_and_hash_are_sequence_based() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let clone = list.clone();
            let greater = ArrayList::<i32, N>::from([1, 2, 4]);
            let different_chunking = {
                let mut list = ArrayList::<i32, N>::new();
                list.extend([1, 2, 3]);
                list
            };

            assert_eq!(list, clone);
            assert_eq!(list, [1, 2, 3]);
            assert_eq!(list, &[1, 2, 3][..]);
            assert_eq!(list.partial_cmp(&greater), Some(Ordering::Less));
            assert_eq!(list.cmp(&clone), Ordering::Equal);
            assert_eq!(different_chunking, [1, 2, 3]);
            assert!(alloc::format!("{list:?}").contains('1'));

            let mut left_hasher = RecordingHasher::default();
            let mut right_hasher = RecordingHasher::default();
            list.hash(&mut left_hasher);
            clone.hash(&mut right_hasher);
            assert_eq!(left_hasher.bytes, right_hasher.bytes);
        }

        check_capacities!(check);
    }

    #[test]
    fn mutable_front_and_back_allow_in_place_updates() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);

            *list.front_mut().unwrap() = 10;
            *list.back_mut().unwrap() = 30;

            assert_eq!(values(&list), [10, 2, 30]);
        }

        check_capacities!(check);
    }

    #[test]
    fn mixed_operations_work_for_required_capacities() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::new();

            for value in 0..10 {
                list.push_back(value);
            }

            list.insert(0, -1);
            list.insert(6, 99);
            list.insert(list.len(), 10);

            assert_eq!(list.remove(6), Some(99));
            assert_eq!(list.pop_front(), Some(-1));
            assert_eq!(list.pop_back(), Some(10));
            assert_eq!(list.get(0), Some(&0));
            assert_eq!(list.get(9), Some(&9));
            assert_eq!(list.get(10), None);

            list.shrink();

            assert_eq!(values(&list), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
            assert!(list.capacity() >= list.len());
            assert!(list.chunks.iter().all(|chunk| chunk.len() <= N));
        }

        check_capacities!(check);
    }

    #[test]
    fn clear_drops_each_element_once_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut list = drop_probe_list::<N>(&drops, 10);

            list.clear();

            assert_eq!(drops.get(), 10);
            assert!(list.is_empty());
            drop(list);
            assert_eq!(drops.get(), 10);
        }

        check_capacities!(check);
    }

    #[test]
    fn remove_and_pop_drop_removed_elements_once_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut list = drop_probe_list::<N>(&drops, 6);

            let removed = list.remove(2).unwrap();
            assert_eq!(removed.value, 2);
            assert_eq!(drops.get(), 0);
            drop(removed);
            assert_eq!(drops.get(), 1);

            let front = list.pop_front().unwrap();
            assert_eq!(front.value, 0);
            drop(front);
            assert_eq!(drops.get(), 2);

            let back = list.pop_back().unwrap();
            assert_eq!(back.value, 5);
            drop(back);
            assert_eq!(drops.get(), 3);

            drop(list);
            assert_eq!(drops.get(), 6);
        }

        check_capacities!(check);
    }

    #[test]
    fn append_moves_elements_without_dropping_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut left = drop_probe_list::<N>(&drops, 3);
            let mut right = (3..7)
                .map(|value| DropProbe::new(value, &drops))
                .collect::<ArrayList<_, N>>();

            left.append(&mut right);

            assert_eq!(drops.get(), 0);
            assert!(right.is_empty());
            drop(right);
            assert_eq!(drops.get(), 0);
            drop(left);
            assert_eq!(drops.get(), 7);
        }

        check_capacities!(check);
    }

    #[test]
    fn shrink_reorders_storage_without_dropping_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut list = drop_probe_list::<N>(&drops, 9);

            drop(list.remove(1));
            drop(list.remove(3));
            assert_eq!(drops.get(), 2);

            list.shrink();

            assert_eq!(drops.get(), 2);
            drop(list);
            assert_eq!(drops.get(), 9);
        }

        check_capacities!(check);
    }

    #[test]
    fn partially_consumed_into_iter_drops_remaining_elements_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let list = drop_probe_list::<N>(&drops, 6);
            let mut iter = list.into_iter();

            let first = iter.next().unwrap();
            assert_eq!(first.value, 0);
            drop(first);
            assert_eq!(drops.get(), 1);

            let last = iter.next_back().unwrap();
            assert_eq!(last.value, 5);
            drop(last);
            assert_eq!(drops.get(), 2);

            drop(iter);
            assert_eq!(drops.get(), 6);
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_remove_current_drops_returned_element_once_for_required_capacities() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut list = drop_probe_list::<N>(&drops, 4);
            let removed = {
                let mut cursor = list.cursor_front_mut();

                cursor.move_next();
                cursor.remove_current().unwrap()
            };

            assert_eq!(removed.value, 1);
            assert_eq!(drops.get(), 0);
            drop(removed);
            assert_eq!(drops.get(), 1);
            drop(list);
            assert_eq!(drops.get(), 4);
        }

        check_capacities!(check);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn model_based_collection_ops_match_vec(
            ops in prop::collection::vec(model_op_strategy(), 0..128)
        ) {
            apply_model_ops::<4>(&ops);
            apply_model_ops::<8>(&ops);
            apply_model_ops::<16>(&ops);
        }

        #[test]
        fn model_based_read_only_cursor_ops_match_indexed_vec_cursor(
            ops in prop::collection::vec(read_only_cursor_model_op_strategy(), 0..256)
        ) {
            apply_read_only_cursor_model_ops::<4>(&ops);
            apply_read_only_cursor_model_ops::<8>(&ops);
            apply_read_only_cursor_model_ops::<16>(&ops);
        }

        #[test]
        fn model_based_cursor_mut_ops_match_indexed_vec_cursor(
            ops in prop::collection::vec(cursor_model_op_strategy(), 0..128)
        ) {
            apply_cursor_model_ops::<4>(&ops);
            apply_cursor_model_ops::<8>(&ops);
            apply_cursor_model_ops::<16>(&ops);
        }

        #[test]
        fn model_based_shared_iterator_ops_match_vec_deque(
            ops in prop::collection::vec(iterator_model_op_strategy(), 0..128)
        ) {
            apply_iter_model_ops::<4>(&ops);
            apply_iter_model_ops::<8>(&ops);
            apply_iter_model_ops::<16>(&ops);
        }

        #[test]
        fn model_based_mutable_iterator_ops_match_vec_deque(
            ops in prop::collection::vec(iterator_model_op_strategy(), 0..128)
        ) {
            apply_iter_mut_model_ops::<4>(&ops);
            apply_iter_mut_model_ops::<8>(&ops);
            apply_iter_mut_model_ops::<16>(&ops);
        }

        #[test]
        fn model_based_owning_iterator_ops_match_vec_deque(
            ops in prop::collection::vec(iterator_model_op_strategy(), 0..128)
        ) {
            apply_into_iter_model_ops::<4>(&ops);
            apply_into_iter_model_ops::<8>(&ops);
            apply_into_iter_model_ops::<16>(&ops);
        }
    }
}

#[cfg(all(test, miri))]
mod miri_tests {
    use super::ArrayList;
    use alloc::rc::Rc;
    use alloc::vec::Vec;
    use core::cell::Cell;

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    fn values<const N: usize>(list: &ArrayList<i32, N>) -> Vec<i32> {
        list.iter().copied().collect()
    }

    #[derive(Debug)]
    struct DropProbe {
        value: i32,
        drops: Rc<Cell<usize>>,
    }

    impl DropProbe {
        fn new(value: i32, drops: &Rc<Cell<usize>>) -> Self {
            Self {
                value,
                drops: Rc::clone(drops),
            }
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    fn drop_probe_list<const N: usize>(
        drops: &Rc<Cell<usize>>,
        values: impl IntoIterator<Item = i32>,
    ) -> ArrayList<DropProbe, N> {
        values
            .into_iter()
            .map(|value| DropProbe::new(value, drops))
            .collect()
    }

    #[test]
    fn miri_drop_paths_cover_remove_append_shrink_and_into_iter() {
        fn check<const N: usize>() {
            let drops = Rc::new(Cell::new(0));
            let mut list = drop_probe_list::<N>(&drops, 0..6);
            let mut extra = drop_probe_list::<N>(&drops, 6..9);

            let removed = list.remove(2).unwrap();
            assert_eq!(removed.value, 2);
            assert_eq!(drops.get(), 0);
            drop(removed);
            assert_eq!(drops.get(), 1);

            list.append(&mut extra);
            drop(extra);
            assert_eq!(drops.get(), 1);

            list.shrink();
            assert_eq!(drops.get(), 1);

            let mut iter = list.into_iter();
            let first = iter.next().unwrap();
            assert_eq!(first.value, 0);
            drop(first);
            assert_eq!(drops.get(), 2);
            drop(iter);
            assert_eq!(drops.get(), 9);
        }

        check_capacities!(check);
    }

    #[test]
    fn miri_cursor_mut_paths_cover_mut_refs_inserts_and_removal() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7]);

            {
                let mut cursor = list.cursor_front_mut();
                *cursor.current().unwrap() = 10;
                *cursor.peek_next().unwrap() = 20;
                cursor.move_next();
                cursor.insert_before(15);
                cursor.insert_after(25);
                assert_eq!(cursor.index(), Some(2));
                assert_eq!(cursor.remove_current(), Some(20));
                *cursor.front_mut().unwrap() += 1;
                *cursor.back_mut().unwrap() += 1;
            }

            assert_eq!(values(&list), [11, 15, 25, 2, 3, 4, 5, 6, 8]);
        }

        check_capacities!(check);
    }

    #[test]
    fn miri_iterator_paths_cover_shared_mutable_and_owning_iterators() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7]);

            {
                let mut iter = list.iter_mut();
                *iter.next().unwrap() += 10;
                *iter.next_back().unwrap() += 10;
                *iter.nth(1).unwrap() += 10;
                *iter.nth_back(1).unwrap() += 10;
            }

            assert_eq!(values(&list), [10, 1, 12, 3, 4, 15, 6, 17]);

            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(&10));
            assert_eq!(iter.nth_back(2), Some(&15));
            assert_eq!(iter.next_back(), Some(&4));

            let mut into_iter = list.into_iter();
            assert_eq!(into_iter.next(), Some(10));
            assert_eq!(into_iter.next_back(), Some(17));
            assert_eq!(into_iter.nth(2), Some(3));
            assert_eq!(into_iter.nth_back(1), Some(15));
        }

        check_capacities!(check);
    }

    #[test]
    fn miri_read_only_cursor_paths_cover_ghost_and_cross_chunk_moves() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7]);
            assert_eq!(list.remove(1), Some(1));
            assert_eq!(list.remove(4), Some(5));
            list.shrink();

            let mut cursor = list.cursor_front();
            assert_eq!(cursor.current(), Some(&0));
            cursor.move_prev();
            assert_eq!(cursor.current(), None);
            assert_eq!(cursor.peek_prev(), Some(&7));
            assert_eq!(cursor.peek_next(), Some(&0));
            cursor.move_prev();
            assert_eq!(cursor.current(), Some(&7));
            cursor.move_next();
            assert_eq!(cursor.current(), None);
            cursor.move_next();
            assert_eq!(cursor.current(), Some(&0));
        }

        check_capacities!(check);
    }
}
