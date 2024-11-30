//! # array_list
//!
//! `array_list` is a Rust crate that implements an **unrolled linked list** with features that
//! combine the simplicity of a `Vec` and the flexibility of a `LinkedList`.
//!
//! The underlying linked list is built using a **XOR linked list**.
//! This design requires only a single pointer for bidirectional traversal,
//! making it more memory-efficient compared to a traditional doubly linked list,
//! which requires two pointers per node.
//!
//! ## Features
//! - Efficient traversal in both directions with minimal memory overhead.
//! - Chunked storage for elements (unrolled list), which improves cache locality and reduces
//!   pointer overhead compared to traditional linked lists.
//! - Dynamic growth, striking a balance between `Vec` and `LinkedList` performance characteristics.
//!
//! ## Use Cases
//! `array_list` is ideal for scenarios where:
//! - You require frequent insertions and deletions in the middle of the list.
//! - Memory efficiency is crucial, and you want to minimize pointer overhead.
//! - Improved cache performance over a standard linked list is desired.
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
//! list.push_back(1);
//! list.push_back(2);
//! list.push_front(0);
//!
//! assert_eq!(list.pop_back(), Some(2));
//! assert_eq!(list.pop_front(), Some(0));
//! ```

mod node;
mod sailed;

use node::Node;
use sailed::Array;

use std::ptr::NonNull;

/// A dynamic container that combines the characteristics of a `Vec` and a `LinkedList`,
/// implemented as an **unrolled linked list** with chunked storage.
///
/// The `ArrayList` is backed by a **XOR linked list**, requiring only a single pointer
/// for bidirectional traversal between nodes. This design is both memory-efficient
/// and optimized for cache locality compared to traditional doubly linked lists.
///
/// # Type Parameters
/// - `T`: The type of elements stored in the list.
/// - `N`: The maximum number of elements that each node (or chunk) can hold.
///        This determines the granularity of the unrolled linked list.
///        Larger values reduce the number of nodes but increase the size of each node.
///
/// # Features
/// - **Chunked Storage**: Each node can hold up to `N` elements, reducing the overhead of
///   individual allocations compared to a traditional linked list.
/// - **Memory Efficiency**: The XOR linked list design minimizes pointer storage requirements.
/// - **Bidirectional Traversal**: Supports efficient iteration in both forward and backward directions.
/// - **Flexible Operations**: Allows efficient insertions, deletions, and access at arbitrary positions.
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
/// # Applications
/// `ArrayList` is suitable for use cases where:
/// - Frequent insertions or deletions occur in the middle of the list.
/// - Memory efficiency and improved cache performance are priorities.
/// - A hybrid structure that balances the strengths of `Vec` and `LinkedList` is needed.
///
/// # Note
/// The structure name `ArrayList` has no association with Java's `ArrayList`.
pub struct ArrayList<T, const N: usize>
where
    [T; N]: Array<T>,
{
    head: Option<NonNull<Node<T, N>>>,
    tail: Option<NonNull<Node<T, N>>>,
    len: usize,
}

impl<T, const N: usize> Default for ArrayList<T, N>
where
    [T; N]: Array<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> FromIterator<T> for ArrayList<T, N>
where
    [T; N]: Array<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        this.extend(iter);
        this
    }
}

impl<T, const N: usize> Extend<T> for ArrayList<T, N>
where
    [T; N]: Array<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|v| self.push_back(v));
    }
}

impl<T, const N: usize> ArrayList<T, N>
where
    [T; N]: Array<T>,
{
    /// Creates a new, empty `ArrayList`.
    ///
    /// This constructor initializes an `ArrayList` with no elements and no allocated nodes.
    /// It is a constant function, allowing the creation of `ArrayList` instances in
    /// compile-time contexts where applicable.
    ///
    /// # Returns
    /// A new, empty instance of `ArrayList`.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i64, 6> = ArrayList::new();
    ///
    /// assert!(list.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Adds an element to the front of the `ArrayList`.
    ///
    /// The element is inserted at the beginning of the list, shifting existing elements
    /// forward if necessary. If the first node is full, a new node will be allocated to
    /// accommodate the element.
    ///
    /// # Parameters
    /// - `value`: The element to add to the front of the `ArrayList`.
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
        if let Some(mut head_ptr) = self.head {
            let head = unsafe { head_ptr.as_mut() };

            if head.is_full() {
                let mut new_node = Box::new(Node::new_with_link(head_ptr.as_ptr() as usize));
                new_node.push_front(value);

                // Get the pointer to the new node
                let new_node_ptr: NonNull<Node<T, N>> = Box::leak(new_node).into();

                // Update the link in the current head with the XOR of the new node and its next node
                //
                // Before update:
                // link(head) = addr(prev(head)) ^ addr(next(head)) | prev(head) = null
                //            = null             ^ addr(next(head))
                //            = addr(next(head))
                //
                // After update:
                // link(head) = addr(new_node) ^ addr(next(head))
                *head.link_mut() ^= new_node_ptr.as_ptr() as usize;

                // Update the head pointer
                self.head = Some(new_node_ptr);
            } else {
                // There's room in the head node; insert at the front
                head.push_front(value);
            }
        } else {
            // The list is empty; create the first node
            let mut new_node = Box::new(Node::new());
            new_node.push_front(value);
            // Make both head and tail point to the new node
            let new_node_ptr = Box::leak(new_node).into();
            self.head = Some(new_node_ptr);
            self.tail = Some(new_node_ptr);
        }

        // Increment the len of the list
        self.len += 1;
    }

    /// Adds an element to the back of the `ArrayList`.
    ///
    /// The element is inserted at the end of the list.
    /// If the last node is full, a new node will be allocated to
    /// accommodate the element.
    ///
    /// # Parameters
    /// - `value`: The element to add to the back of the `ArrayList`.
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
        if let Some(mut tail_ptr) = self.tail {
            let tail = unsafe { tail_ptr.as_mut() };

            if tail.is_full() {
                let mut new_node = Box::new(Node::new_with_link(tail_ptr.as_ptr() as usize));
                new_node.push_back(value);

                // Get the pointer to the new node
                let new_node_ptr: NonNull<Node<T, N>> = Box::leak(new_node).into();

                // Update the link in the current tail with the XOR of the new node and its previous node
                //
                // Before update:
                // link(tail) = addr(prev(tail)) ^ addr(next(tail)) | next(tail) = null
                //            = addr(prev(tail)) ^ null
                //            = addr(prev(tail))
                //
                // After update:
                // link(tail) = addr(prev(tail)) ^ addr(new_node)
                *tail.link_mut() ^= new_node_ptr.as_ptr() as usize;

                // Update the head pointer
                self.tail = Some(new_node_ptr);
            } else {
                // There's room in the tail node; insert at the end
                tail.push_back(value);
            }
        } else {
            // The list is empty; create the first node
            let mut new_node = Box::new(Node::new());
            new_node.push_back(value);
            // Make both head and tail point to the new node
            let new_node_ptr = Box::leak(new_node).into();
            self.head = Some(new_node_ptr);
            self.tail = Some(new_node_ptr);
        }

        // Increment the len of the list
        self.len += 1;
    }

    /// Inserts an element at the specified index, shifting subsequent elements to the right.
    ///
    /// # Arguments
    /// - `index`: The position at which to insert the new element.
    /// - `value`: The value to insert.
    ///
    /// # Panics
    /// - Panics if the `index` is out of bounds (greater than the list's current length).
    ///
    /// # Behavior
    /// - Traverses the list to locate the appropriate node and index.
    /// - If the target node is full, the method splits the node, redistributes elements,
    ///   and inserts the value.
    ///
    /// # Complexity
    /// - O(n) for traversal to find the target node.
    /// - O(N) within the node for shifting elements, where `N` is the node's capacity.
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
        if index > self.len {
            panic!("Index out of bounds");
        }

        if index == 0 {
            self.push_front(value);
            return;
        }

        if index == self.len {
            self.push_back(value);
            return;
        }

        let (left, middle, right, index) = self.get_closest_mut(index).unwrap();

        if !middle.is_full() {
            middle.insert(index, value);
            self.len += 1;
            return;
        }

        if let Some(mut left) = left {
            let left = unsafe { left.as_mut() };
            if !left.is_full() {
                if index == 0 {
                    left.push_back(value);
                } else {
                    let tmp = middle.pop_front().unwrap();
                    left.push_back(tmp);
                    middle.insert(index - 1, value);
                }
                self.len += 1;
                return;
            }
        }

        if let Some(mut right) = right {
            let right = unsafe { right.as_mut() };
            if !right.is_full() {
                let tmp = middle.pop_back().unwrap();
                right.push_front(tmp);
                middle.insert(index, value);
                self.len += 1;
                return;
            }
        }

        // Create a new node
        let mut new_node = Box::new(Node::new());
        let split_index = N / 2;
        let count_items = N - split_index;

        // Move elements from the original node to the new node
        let src_ptr = unsafe { middle.data().as_ptr().add(split_index) };
        let dest_ptr = new_node.data_mut().as_mut_ptr();
        unsafe { std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, count_items) };

        // Update sizes for both nodes
        new_node.set_len(count_items);
        middle.set_len(split_index);

        // Insert the value into the appropriate node
        if index <= split_index {
            middle.insert(index, value);
        } else {
            new_node.insert(index - split_index, value);
        }

        let mut middle_ptr: NonNull<Node<T, N>> = unsafe { NonNull::new_unchecked(middle as _) };

        // remove middle mut ref from scope
        #[allow(unused_variables)]
        let middle = ();

        // Handle the case where the list contains only one node
        if self.head == self.tail {
            // Update the links for the old and new nodes
            *new_node.link_mut() = middle_ptr.as_ptr() as usize;

            let new_node_ptr: NonNull<Node<T, N>> = Box::leak(new_node).into();
            unsafe { *middle_ptr.as_mut().link_mut() = new_node_ptr.as_ptr() as usize };

            // Update the head and tail of the list
            self.head = Some(middle_ptr);
            self.tail = Some(new_node_ptr);
        } else {
            // Update the link for the new node
            *new_node.link_mut() =
                middle_ptr.as_ptr() as usize ^ right.map_or(0, |p| p.as_ptr() as usize);

            let new_node_ptr: NonNull<Node<T, N>> = Box::leak(new_node).into();

            // Update the link for the node at the right of the new node
            if let Some(mut right) = right {
                let right = unsafe { right.as_mut() };
                *right.link_mut() ^= middle_ptr.as_ptr() as usize ^ new_node_ptr.as_ptr() as usize;
            }

            // Update the current node's link
            unsafe {
                *middle_ptr.as_mut().link_mut() ^=
                    right.map_or(0, |p| p.as_ptr() as usize) ^ new_node_ptr.as_ptr() as usize
            };

            // Update tail if necessary
            if Some(middle_ptr) == self.tail {
                self.tail = Some(new_node_ptr);
            }
        }

        self.len += 1;
    }

    /// Moves all elements from other to the end of the list.
    ///
    /// This reuses all the nodes from other and moves them into self.
    /// After this operation, other becomes empty.
    ///
    /// # Complexity
    /// - Time complexity: O(1)
    /// - Memory complexity: O(1)
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
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            *self = std::mem::replace(other, Self::new());
            return;
        }

        let mut self_tail = self.tail.unwrap();

        let mut other_head = other.head.take().unwrap();
        let other_tail = other.tail.take().unwrap();

        unsafe { *self_tail.as_mut().link_mut() ^= other_head.as_ptr() as usize };
        unsafe { *other_head.as_mut().link_mut() ^= self_tail.as_ptr() as usize };

        self.tail = Some(other_tail);
        self.len += other.len;
        other.len = 0;
    }

    /// Removes and returns the first element of the `ArrayList`, if any.
    ///
    /// This method removes the first element of the list and adjusts the head pointer
    /// and internal structure accordingly. If the list is empty, it returns `None`.
    ///
    /// # Returns
    /// - `Some(T)` if the list contains elements, where `T` is the removed element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1) when the first element is removed and there are no structural changes
    ///   (e.g., the head node has more elements).
    /// - O(1) amortized, including the occasional node removal and re-linking operations.
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
        if let Some(mut head_ptr) = self.head {
            let head = unsafe { head_ptr.as_mut() };
            let value = head.pop_front()?;
            self.len -= 1;

            if head.is_empty() {
                if let Some(mut new_head_ptr) = NonNull::new(head.link() as *mut Node<T, N>) {
                    // Update the link in the next node to exclude the current head
                    let new_head = unsafe { new_head_ptr.as_mut() };
                    *new_head.link_mut() ^= head_ptr.as_ptr() as usize;
                    self.head = Some(new_head_ptr);
                } else {
                    // Empty the list
                    assert!(self.is_empty());
                    self.head = None;
                    self.tail = None;
                }

                // Deallocate the old head node
                drop(unsafe { Box::from_raw(head_ptr.as_ptr()) });
            }

            return Some(value);
        }

        None
    }

    /// Removes and returns the last element of the `ArrayList`, if any.
    ///
    /// This method removes the last element of the list and adjusts the tail pointer
    /// and internal structure accordingly. If the list is empty, it returns `None`.
    ///
    /// # Returns
    /// - `Some(T)` if the list contains elements, where `T` is the removed element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1) when the last element is removed and there are no structural changes
    ///   (e.g., the tail node has more elements).
    /// - O(1) amortized, including the occasional node removal and re-linking operations.
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
        if let Some(mut tail_ptr) = self.tail {
            let tail = unsafe { tail_ptr.as_mut() };
            let value = tail.pop_back()?;
            self.len -= 1;

            if tail.is_empty() {
                if let Some(mut new_tail_ptr) = NonNull::new(tail.link() as *mut Node<T, N>) {
                    // Update the link in the previous node to exclude the current tail
                    let new_tail = unsafe { new_tail_ptr.as_mut() };
                    *new_tail.link_mut() ^= tail_ptr.as_ptr() as usize;
                    self.tail = Some(new_tail_ptr);
                } else {
                    // Empty the list
                    assert!(self.is_empty());
                    self.head = None;
                    self.tail = None;
                }

                // Deallocate the old tail node
                drop(unsafe { Box::from_raw(tail_ptr.as_ptr()) });
            }

            return Some(value);
        }

        None
    }

    /// Removes and returns the element at the specified index, shifting subsequent elements left.
    ///
    /// # Arguments
    /// - `index`: The position of the element to remove.
    ///
    /// # Returns
    /// - The element removed from the list.
    ///
    /// # Panics
    /// - Panics if the `index` is out of bounds.
    ///
    /// # Behavior
    /// - The method traverses the list to locate the node containing the element at the given index.
    /// - The element is removed from the node, and subsequent elements within the node are shifted to the left.
    /// - If the node becomes empty after removal, it is removed from the list, and the links between nodes are updated.
    ///
    /// # Complexity
    /// - O(n) to locate the target node.
    /// - O(N) within the node for shifting elements, where `N` is the node's capacity.
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
    /// assert_eq!(list.remove(1), 20);
    /// assert_eq!(list.get(1), Some(&30));
    /// assert_eq!(list.len(), 4);
    ///
    /// // Attempting to remove out-of-bounds index will panic
    /// // list.remove(10);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len() {
            panic!("Index out of bounds: cannot remove at index {}", index);
        }

        let (left, middle, right, index) = self.get_closest_mut(index).unwrap();
        let value = middle.remove(index);

        if middle.is_empty() {
            let middle_ptr = unsafe { NonNull::new_unchecked(middle as _) };
            // remove middle from scope
            #[allow(unused_variables)]
            let middle = ();

            // Update links to skip the now-empty node
            if let Some(mut left_ptr) = left {
                let left = unsafe { left_ptr.as_mut() };
                *left.link_mut() ^=
                    middle_ptr.as_ptr() as usize ^ right.map_or(0, |n| n.as_ptr() as usize);
            }

            if let Some(mut right_ptr) = right {
                let right = unsafe { right_ptr.as_mut() };
                *right.link_mut() ^=
                    middle_ptr.as_ptr() as usize ^ left.map_or(0, |p| p.as_ptr() as usize);
            }

            // Update head/tail pointers if necessary
            if Some(middle_ptr) == self.head {
                self.head = right;
            }

            if Some(middle_ptr) == self.tail {
                self.tail = left;
            }

            // Deallocate the old node
            drop(unsafe { Box::from_raw(middle_ptr.as_ptr()) });
        }

        self.len -= 1;
        value
    }

    /// Removes all elements from the `ArrayList`, effectively making it empty.
    ///
    /// This method deallocates all nodes in the list and resets the head, tail, and length
    /// to their initial states. The method does not shrink the capacity of the list's nodes.
    ///
    /// # Complexity
    /// - Time complexity: O(n), where n is the total number of elements in the list.
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
        let mut right: Option<NonNull<Node<T, N>>> = None;
        let mut cursor = self.tail;

        while let Some(mut node) = cursor {
            let tmp = cursor;

            cursor = NonNull::new(
                (unsafe { node.as_ref().link() } ^ right.map_or(0, |p| p.as_ptr() as usize))
                    as *mut Node<T, N>,
            );

            right = tmp;

            drop(unsafe { Box::from_raw(node.as_mut()) })
        }

        self.tail = None;
        self.head = None;
        self.len = 0;
    }

    /// Returns a reference to the first element of the `ArrayList`, if any.
    ///
    /// This method provides read-only access to the first element of the list without removing it.
    ///
    /// # Returns
    /// - `Some(&T)` if the list contains elements, where `&T` is a reference to the first element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1), as it directly accesses the first element in the list.
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
        if let Some(head_ptr) = self.head {
            let head = unsafe { head_ptr.as_ref() };
            return head.front();
        }

        None
    }

    /// Returns a mutable reference to the first element of the `ArrayList`, if any.
    ///
    /// This method provides mutable access to the first element of the list without removing it.
    ///
    /// # Returns
    /// - `Some(&mut T)` if the list contains elements, where `&mut T` is a mutable reference to the first element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1), as it directly accesses the first element in the list.
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
        if let Some(mut head_ptr) = self.head {
            let head = unsafe { head_ptr.as_mut() };
            return head.front_mut();
        }

        None
    }

    /// Returns a reference to the last element of the `ArrayList`, if any.
    ///
    /// This method provides read-only access to the last element of the list without removing it.
    ///
    /// # Returns
    /// - `Some(&T)` if the list contains elements, where `&T` is a reference to the last element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1), as it directly accesses the last element in the list.
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
        if let Some(tail_ptr) = self.tail {
            let tail = unsafe { tail_ptr.as_ref() };
            return tail.back();
        }

        None
    }

    /// Returns a mutable reference to the last element of the `ArrayList`, if any.
    ///
    /// This method provides mutable access to the last element of the list without removing it.
    ///
    /// # Returns
    /// - `Some(&mut T)` if the list contains elements, where `&mut T` is a mutable reference to the last element.
    /// - `None` if the list is empty.
    ///
    /// # Complexity
    /// - O(1), as it directly accesses the last element in the list.
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
        if let Some(mut tail_ptr) = self.tail {
            let tail = unsafe { tail_ptr.as_mut() };
            return tail.back_mut();
        }

        None
    }

    /// Returns a reference to the element at the specified index, if present.
    ///
    /// # Arguments
    /// - `index`: The position of the element in the list.
    ///
    /// # Returns
    /// - `Some(&T)` if the index is valid and the element exists.
    /// - `None` if the index is out of bounds.
    ///
    /// # Complexity
    /// - O(n), where `n` is the number of nodes in the list, as nodes are traversed sequentially.
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
        self.get_closest(index)
            .and_then(|(_, node, _, index)| node.get(index))
    }

    /// Returns a mutable reference to the element at the specified index, if present.
    ///
    /// # Arguments
    /// - `index`: The position of the element in the list.
    ///
    /// # Returns
    /// - `Some(&mut T)` if the index is valid and the element exists.
    /// - `None` if the index is out of bounds.
    ///
    /// # Complexity
    /// - O(n), where `n` is the number of nodes in the list, as nodes are traversed sequentially.
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
        self.get_closest_mut(index)
            .and_then(|(_, node, _, index)| node.get_mut(index))
    }

    /// Returns the number of elements currently stored in the `ArrayList`.
    ///
    /// # Returns
    /// The total number of elements across all nodes in the `ArrayList`.
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
    /// # Returns
    /// `true` if the `ArrayList` contains no elements, otherwise `false`.
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
        self.len() == 0
    }

    #[inline]
    fn get_closest(
        &self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index <= (self.len() >> 1) {
            self.get_forward(index)
        } else {
            self.get_backward(index)
        }
    }

    #[inline]
    fn get_closest_mut(
        &mut self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &mut Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index <= (self.len() >> 1) {
            self.get_forward_mut(index)
        } else {
            self.get_backward_mut(index)
        }
    }

    fn get_forward(
        &self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index >= self.len {
            return None;
        }

        let mut left: Option<NonNull<Node<T, N>>> = None;
        let mut cursor = self.head?;
        let mut current_index = 0;

        loop {
            let middle = unsafe { cursor.as_ref() };
            let right = NonNull::new(
                (middle.link() ^ left.map_or(0, |p| p.as_ptr() as usize)) as *mut Node<T, N>,
            );

            if current_index + middle.len() > index {
                let index_inside_node = index - current_index;
                return Some((left, middle, right, index_inside_node));
            }

            left = Some(cursor);
            cursor = right?;
            current_index += middle.len();
        }
    }

    fn get_forward_mut(
        &mut self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &mut Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index >= self.len {
            return None;
        }

        let mut left: Option<NonNull<Node<T, N>>> = None;
        let mut cursor = self.head?;
        let mut current_index = 0;

        loop {
            let middle = unsafe { cursor.as_mut() };
            let right = NonNull::new(
                (middle.link() ^ left.map_or(0, |p| p.as_ptr() as usize)) as *mut Node<T, N>,
            );

            if current_index + middle.len() > index {
                let index_inside_node = index - current_index;
                return Some((left, middle, right, index_inside_node));
            }

            left = Some(cursor);
            cursor = right?;
            current_index += middle.len();
        }
    }

    fn get_backward(
        &self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index >= self.len {
            return None;
        }

        let mut right: Option<NonNull<Node<T, N>>> = None;
        let mut cursor = self.tail?;
        let mut current_index = self.len;

        loop {
            let middle = unsafe { cursor.as_ref() };
            let left = NonNull::new(
                (middle.link() ^ right.map_or(0, |p| p.as_ptr() as usize)) as *mut Node<T, N>,
            );

            current_index -= middle.len();

            if index >= current_index {
                let index_inside_node = index - current_index;
                return Some((left, middle, right, index_inside_node));
            }

            right = Some(cursor);
            cursor = left?;
        }
    }

    fn get_backward_mut(
        &mut self,
        index: usize,
    ) -> Option<(
        Option<NonNull<Node<T, N>>>,
        &mut Node<T, N>,
        Option<NonNull<Node<T, N>>>,
        usize,
    )> {
        if index >= self.len {
            return None;
        }

        let mut right: Option<NonNull<Node<T, N>>> = None;
        let mut cursor = self.tail?;
        let mut current_index = self.len;

        loop {
            let middle = unsafe { cursor.as_mut() };
            current_index -= middle.len();

            let left = NonNull::new(
                (middle.link() ^ right.map_or(0, |p| p.as_ptr() as usize)) as *mut Node<T, N>,
            );

            if index >= current_index {
                let index_inside_node = index - current_index;
                return Some((left, middle, right, index_inside_node));
            }

            right = Some(cursor);
            cursor = left?;
        }
    }
}

impl<T, const N: usize> Drop for ArrayList<T, N>
where
    [T; N]: Array<T>,
{
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::ArrayList;

    const _: () = assert!(size_of::<ArrayList<usize, 14>>() == size_of::<usize>() * 3);

    #[test]
    fn new_creates_empty_array_list() {
        let sut: ArrayList<i64, 2> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn default_creates_empty_array_list() {
        let sut: ArrayList<i64, 2> = ArrayList::default();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);
    }

    #[test]
    fn push_front_adds_element_to_front() {
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
    fn push_back_adds_element_to_back() {
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
    fn insert_uses_spare_space_in_left_node() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        sut.push_back(0);
        sut.push_back(1);
        sut.push_back(2);
        sut.push_back(3);
        sut.push_back(4);
        sut.push_back(5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&5));
        assert_eq!(6, sut.len());

        assert_eq!(sut.remove(2), 2);
        assert_eq!(5, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));
        assert_eq!(sut.get(3), Some(&4));
        assert_eq!(sut.get(4), Some(&5));

        sut.insert(3, 42);
        assert_eq!(6, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&5));

        assert_eq!(sut.remove(2), 3);
        assert_eq!(5, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&42));
        assert_eq!(sut.get(3), Some(&4));
        assert_eq!(sut.get(4), Some(&5));

        sut.insert(2, 2);
        assert_eq!(6, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&5));
    }

    #[test]
    fn insert_create_new_node_if_necessary() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        sut.push_back(0);
        sut.push_back(1);
        sut.push_back(2);
        sut.push_back(3);
        sut.push_back(4);
        sut.push_back(5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&4));
        assert_eq!(sut.get(5), Some(&5));
        assert_eq!(6, sut.len());

        sut.insert(3, 42);
        assert_eq!(7, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&3));
        assert_eq!(sut.get(5), Some(&4));
        assert_eq!(sut.get(6), Some(&5));

        sut.insert(4, 21);
        assert_eq!(8, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&21));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&4));
        assert_eq!(sut.get(7), Some(&5));

        sut.insert(4, 18);
        assert_eq!(9, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&18));
        assert_eq!(sut.get(5), Some(&21));
        assert_eq!(sut.get(6), Some(&3));
        assert_eq!(sut.get(7), Some(&4));
        assert_eq!(sut.get(8), Some(&5));

        sut.insert(4, 12);
        assert_eq!(10, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&42));
        assert_eq!(sut.get(4), Some(&12));
        assert_eq!(sut.get(5), Some(&18));
        assert_eq!(sut.get(6), Some(&21));
        assert_eq!(sut.get(7), Some(&3));
        assert_eq!(sut.get(8), Some(&4));
        assert_eq!(sut.get(9), Some(&5));

        sut.insert(2, 33);
        assert_eq!(11, sut.len());
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&33));
        assert_eq!(sut.get(3), Some(&2));
        assert_eq!(sut.get(4), Some(&42));
        assert_eq!(sut.get(5), Some(&12));
        assert_eq!(sut.get(6), Some(&18));
        assert_eq!(sut.get(7), Some(&21));
        assert_eq!(sut.get(8), Some(&3));
        assert_eq!(sut.get(9), Some(&4));
        assert_eq!(sut.get(10), Some(&5));
    }

    #[test]
    fn insert_work_with_node_of_capacity_1() {
        let mut sut: ArrayList<i64, 1> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        sut.insert(0, 1);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.get(0), Some(&1));

        sut.insert(0, 0);
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));

        sut.insert(2, 3);
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));

        sut.insert(2, 2);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));

        sut.insert(0, -2);
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&0));
        assert_eq!(sut.get(2), Some(&1));
        assert_eq!(sut.get(3), Some(&2));
        assert_eq!(sut.get(4), Some(&3));

        sut.insert(1, -1);
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));

        sut.insert(6, 5);
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&5));

        sut.insert(6, 4);
        assert_eq!(sut.len(), 8);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&4));
        assert_eq!(sut.get(7), Some(&5));
    }

    #[test]
    fn insert_work_with_node_of_capacity_2() {
        let mut sut: ArrayList<i64, 2> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        sut.insert(0, 1);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.get(0), Some(&1));

        sut.insert(0, 0);
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));

        sut.insert(2, 3);
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));

        sut.insert(2, 2);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));

        sut.insert(0, -2);
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&0));
        assert_eq!(sut.get(2), Some(&1));
        assert_eq!(sut.get(3), Some(&2));
        assert_eq!(sut.get(4), Some(&3));

        sut.insert(1, -1);
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));

        sut.insert(6, 5);
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&5));

        sut.insert(6, 4);
        assert_eq!(sut.len(), 8);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&4));
        assert_eq!(sut.get(7), Some(&5));
    }

    #[test]
    fn insert_work_with_node_of_capacity_3() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());
        assert_eq!(sut.len(), 0);

        sut.insert(0, 1);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.get(0), Some(&1));

        sut.insert(0, 0);
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));

        sut.insert(2, 3);
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));

        sut.insert(2, 2);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));

        sut.insert(0, -2);
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&0));
        assert_eq!(sut.get(2), Some(&1));
        assert_eq!(sut.get(3), Some(&2));
        assert_eq!(sut.get(4), Some(&3));

        sut.insert(1, -1);
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));

        sut.insert(6, 5);
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&5));

        sut.insert(6, 4);
        assert_eq!(sut.len(), 8);
        assert_eq!(sut.get(0), Some(&-2));
        assert_eq!(sut.get(1), Some(&-1));
        assert_eq!(sut.get(2), Some(&0));
        assert_eq!(sut.get(3), Some(&1));
        assert_eq!(sut.get(4), Some(&2));
        assert_eq!(sut.get(5), Some(&3));
        assert_eq!(sut.get(6), Some(&4));
        assert_eq!(sut.get(7), Some(&5));
    }

    #[test]
    fn insert_inserts_element_at_correct_index() {
        let mut sut: ArrayList<i64, 4> = ArrayList::new();

        // Insert into an empty list
        sut.insert(0, 10); // List: [10]
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.get(0), Some(&10));

        // Insert at the beginning
        sut.insert(0, 5); // List: [5, 10]
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));

        // Insert at the end
        sut.insert(2, 20); // List: [5, 10, 20]
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(2), Some(&20));

        // Insert in the middle
        sut.insert(1, 7); // List: [5, 7, 10, 20]
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(1), Some(&7));
        assert_eq!(sut.get(2), Some(&10));

        // Fill the node
        sut.insert(4, 25); // List: [5, 7, 10, 20, 25]
        sut.insert(5, 30); // List: [5, 7, 10, 20, 25, 30]
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(4), Some(&25));
        assert_eq!(sut.get(5), Some(&30));

        // Force a node split
        sut.insert(3, 15); // List: [5, 7, 10, 15, 20, 25, 30]
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(3), Some(&15));
        assert_eq!(sut.get(4), Some(&20));

        // Insert at the new node boundary
        sut.insert(6, 27); // List: [5, 7, 10, 15, 20, 25, 27, 30]
        assert_eq!(sut.len(), 8);
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
    fn pop_front_removes_and_returns_the_first_element() {
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
    fn pop_back_removes_and_returns_the_last_element() {
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
    fn remove_removes_element_at_index() {
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
        assert_eq!(sut.remove(0), 10); // Removes 10, shifts 20 to index 0
        assert_eq!(sut.get(0), Some(&20));
        assert_eq!(sut.len(), 5);

        assert_eq!(sut.remove(2), 40); // Removes 40, shifts 50 to index 2
        assert_eq!(sut.get(2), Some(&50));
        assert_eq!(sut.len(), 4);

        assert_eq!(sut.remove(3), 60); // Removes 60, second node becomes empty
        assert_eq!(sut.get(3), None); // No more elements at index 3
        assert_eq!(sut.len(), 3);

        // Test removal of remaining elements
        assert_eq!(sut.remove(1), 30); // Removes 30
        assert_eq!(sut.get(1), Some(&50));
        assert_eq!(sut.len(), 2);

        assert_eq!(sut.remove(1), 50); // Removes 50
        assert_eq!(sut.get(1), None);
        assert_eq!(sut.len(), 1);

        assert_eq!(sut.remove(0), 20); // Removes 20, list becomes empty
        assert_eq!(sut.get(0), None);
        assert_eq!(sut.len(), 0);

        // Test out-of-bounds index (should panic)
        let result = std::panic::catch_unwind(move || sut.remove(0));
        assert!(result.is_err());
    }

    #[test]
    fn remove_element_in_middle_node() {
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

        assert_eq!(sut.remove(5), 5);
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

        assert_eq!(sut.remove(4), 4);
        assert!(!sut.is_empty());
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&2));
        assert_eq!(sut.get(3), Some(&3));
        assert_eq!(sut.get(4), Some(&6));
        assert_eq!(sut.get(5), Some(&7));
        assert_eq!(sut.get(6), Some(&8));

        assert_eq!(sut.remove(3), 3);
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
    fn clear_resets_the_list() {
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
    fn front_returns_the_first_element() {
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
    fn front_mut_returns_the_first_element() {
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
    fn back_returns_the_last_element() {
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
    fn back_mut_returns_the_last_element() {
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
    fn get_retrieves_correct_element() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());

        assert_eq!(sut.get(0), None);
        assert_eq!(sut.get(5), None);

        // Ensure to allocate at leat 2 nodes
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
    fn get_mut_retrieves_correct_element() {
        let mut sut: ArrayList<i64, 3> = ArrayList::new();
        assert!(sut.is_empty());

        assert_eq!(sut.get_mut(0), None);
        assert_eq!(sut.get_mut(5), None);

        // Ensure to allocate at leat 2 nodes
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
    fn len_returns_correct_length() {
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
    fn list_remains_functional_after_multiple_operations() {
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
        assert_eq!(sut.remove(2), 5);
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
    fn append_combines_two_lists() {
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

        other.push_back(100);
        other.push_back(200);
        assert_eq!(other.len(), 2);
        assert_eq!(other.front(), Some(&100));
        assert_eq!(other.back(), Some(&200));

        // Verify the combined list
        assert_eq!(sut.len(), 6);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&20));
        assert_eq!(sut.get(2), Some(&30));
        assert_eq!(sut.get(3), Some(&40));
        assert_eq!(sut.get(4), Some(&50));
        assert_eq!(sut.get(5), Some(&60));

        // Verify the combined list is still functional
        sut.push_back(70);
        assert_eq!(sut.len(), 7);
        assert_eq!(sut.get(6), Some(&70));
    }

    #[test]
    fn append_an_empty_list_do_nothing() {
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
    fn append_on_an_empty_list_adds_all_elements() {
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
    fn from_iter_works_correctly() {
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
    fn extend_works_correctly() {
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
}
