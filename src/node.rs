use crate::sailed::Array;

use std::mem::MaybeUninit;

pub struct Node<T, const N: usize>
where
    [T; N]: Array<T, N>,
{
    len: usize,
    link: usize,
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> Node<T, N>
where
    [T; N]: Array<T, N>,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            link: 0,
            data: <[T; N]>::uninit_array(),
        }
    }

    pub fn new_with_link(link: usize) -> Self {
        Self {
            len: 0,
            link,
            data: <[T; N]>::uninit_array(),
        }
    }

    #[inline]
    pub fn push_front(&mut self, value: T) {
        self.insert(0, value);
    }

    #[inline]
    pub fn push_back(&mut self, value: T) {
        self.insert(self.len(), value);
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.len() {
            panic!("Index out of bounds: cannot insert at index {}", index);
        }

        if self.len() >= N {
            panic!("Node is full: cannot insert more elements");
        }

        unsafe {
            let data_ptr = self.data.as_mut_ptr();

            // Shift elements starting from the index to the right
            std::ptr::copy(
                data_ptr.add(index),
                data_ptr.add(index + 1),
                self.len() - index,
            );

            // Write the new value at the specified index
            data_ptr.add(index).write(MaybeUninit::new(value));
        }

        // Increment the size of the node
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        Some(self.remove(0))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        Some(self.remove(self.len() - 1))
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len() {
            panic!("Index out of bounds: cannot remove at index {}", index);
        }

        // Read the value to be removed
        let value = unsafe { self.data[index].assume_init_read() };

        unsafe {
            // Shift elements from `index + 1` to fill the gap
            let data_ptr = self.data.as_mut_ptr();
            std::ptr::copy(
                data_ptr.add(index + 1),
                data_ptr.add(index),
                self.len() - index - 1,
            );
        }

        // Decrement the len of the node
        self.len -= 1;
        value
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            return Some(unsafe { self.data[index].assume_init_ref() });
        }

        None
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            return Some(unsafe { self.data[index].assume_init_mut() });
        }

        None
    }

    pub fn back(&self) -> Option<&T> {
        self.get(self.len().saturating_sub(1))
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len().saturating_sub(1))
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len() == N
    }

    #[inline]
    pub const fn link(&self) -> usize {
        self.link
    }

    #[inline]
    pub fn link_mut(&mut self) -> &mut usize {
        &mut self.link
    }

    pub fn data(&self) -> &[MaybeUninit<T>; N] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [MaybeUninit<T>; N] {
        &mut self.data
    }

    pub fn set_len(&mut self, new_len: usize) {
        assert!(new_len <= N);
        self.len = new_len
    }
}

impl<T, const N: usize> Drop for Node<T, N>
where
    [T; N]: Array<T, N>,
{
    fn drop(&mut self) {
        for i in (0..self.len).rev() {
            unsafe { self.data[i].assume_init_drop() };
        }

        self.link = 0;
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Node;

    #[test]
    fn node_insert_puts_elements_in_the_correct_positions() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.insert(0, 10);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.len(), 1);

        sut.insert(1, 15);
        assert_eq!(sut.get(0), Some(&10));
        assert_eq!(sut.get(1), Some(&15));
        assert_eq!(sut.len(), 2);

        sut.insert(0, 5);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));
        assert_eq!(sut.get(2), Some(&15));
        assert_eq!(sut.len(), 3);

        sut.insert(3, 20);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));
        assert_eq!(sut.get(2), Some(&15));
        assert_eq!(sut.get(3), Some(&20));
        assert_eq!(sut.len(), 4);

        sut.insert(2, 13);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));
        assert_eq!(sut.get(2), Some(&13));
        assert_eq!(sut.get(3), Some(&15));
        assert_eq!(sut.get(4), Some(&20));
        assert_eq!(sut.len(), 5);

        sut.insert(4, 17);
        assert_eq!(sut.get(0), Some(&5));
        assert_eq!(sut.get(1), Some(&10));
        assert_eq!(sut.get(2), Some(&13));
        assert_eq!(sut.get(3), Some(&15));
        assert_eq!(sut.get(4), Some(&17));
        assert_eq!(sut.get(5), Some(&20));
        assert_eq!(sut.len(), 6);

        let result = std::panic::catch_unwind(move || sut.insert(6, 100));
        assert!(result.is_err());
    }

    #[test]
    fn node_insert_panics_on_index_out_of_bounds() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        let result = std::panic::catch_unwind(move || sut.insert(usize::MAX, 100));
        assert!(result.is_err());
    }

    #[test]
    fn node_remove_removes_correct_elements() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());

        sut.insert(0, 0);
        sut.insert(1, 1);
        sut.insert(2, 2);
        sut.insert(3, 3);
        sut.insert(4, 4);
        sut.insert(5, 5);
        assert_eq!(sut.len(), 6);

        assert_eq!(sut.remove(2), 2);
        assert_eq!(sut.len(), 5);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));
        assert_eq!(sut.get(3), Some(&4));
        assert_eq!(sut.get(4), Some(&5));

        assert_eq!(sut.remove(3), 4);
        assert_eq!(sut.len(), 4);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&1));
        assert_eq!(sut.get(2), Some(&3));
        assert_eq!(sut.get(3), Some(&5));

        assert_eq!(sut.remove(1), 1);
        assert_eq!(sut.len(), 3);
        assert_eq!(sut.get(0), Some(&0));
        assert_eq!(sut.get(1), Some(&3));
        assert_eq!(sut.get(2), Some(&5));

        assert_eq!(sut.remove(0), 0);
        assert_eq!(sut.len(), 2);
        assert_eq!(sut.get(0), Some(&3));
        assert_eq!(sut.get(1), Some(&5));

        assert_eq!(sut.remove(1), 5);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.get(0), Some(&3));

        assert_eq!(sut.remove(0), 3);
        assert_eq!(sut.len(), 0);

        let result = std::panic::catch_unwind(move || sut.remove(0));
        assert!(result.is_err());
    }

    #[test]
    fn node_pop_front_with_empty_list_returns_none() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
        assert_eq!(sut.pop_front(), None);
    }

    #[test]
    fn node_pop_back_with_empty_list_returns_none() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
        assert_eq!(sut.pop_back(), None);
    }

    #[test]
    fn node_get_with_empty_list_returns_none() {
        let sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
        assert_eq!(sut.get(0), None);
    }

    #[test]
    fn node_get_mut_with_empty_list_returns_none() {
        let mut sut: Node<i64, 6> = Node::new();
        assert_eq!(sut.len(), 0);
        assert!(sut.is_empty());
        assert_eq!(sut.get_mut(0), None);
    }
}
