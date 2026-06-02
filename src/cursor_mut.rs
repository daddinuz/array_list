use crate::position::Position;
use crate::{ArrayList, Cursor};

/// A mutable cursor over an [`ArrayList`].
///
/// A cursor is like an iterator, except that it can move back and forth without
/// consuming elements. It points either at an element or at the ghost position
/// after the back of the list.
///
/// The ghost position is the cursor equivalent of "past the end": moving next
/// from the ghost wraps to the front, and moving previous from the front moves
/// back to the ghost. Mutation methods keep the cursor on the same logical
/// element when possible.
///
/// # Example
/// ```rust
/// use array_list::ArrayList;
///
/// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 3]);
/// let mut cursor = list.cursor_front_mut();
///
/// cursor.insert_after(2);
/// cursor.move_next();
///
/// assert_eq!(cursor.current(), Some(&mut 2));
/// assert_eq!(cursor.as_list(), &[1, 2, 3]);
/// ```
pub struct CursorMut<'a, T, const N: usize> {
    position: Position,
    list: &'a mut ArrayList<T, N>,
}

const _: [(); core::mem::size_of::<usize>() * 4] =
    [(); core::mem::size_of::<CursorMut<usize, 4>>()];

impl<'a, T, const N: usize> CursorMut<'a, T, N> {
    pub(crate) fn from_front(list: &'a mut ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            position: Position::front(list),
            list,
        }
    }

    pub(crate) fn from_back(list: &'a mut ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            position: Position::back(list),
            list,
        }
    }

    /// Returns an immutable cursor at the same position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.as_cursor().current(), Some(&1));
    /// ```
    pub fn as_cursor(&self) -> Cursor<'_, T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Cursor::from_position(self.list, self.position)
    }

    /// Returns an immutable reference to the list this cursor was created from.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.as_list().len(), 2);
    /// ```
    pub fn as_list(&self) -> &ArrayList<T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list
    }

    /// Returns the last element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.back(), Some(&2));
    /// ```
    pub fn back(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.back()
    }

    /// Returns a mutable reference to the last element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// *cursor.back_mut().unwrap() = 3;
    ///
    /// assert_eq!(cursor.back(), Some(&3));
    /// ```
    pub fn back_mut(&mut self) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.back_mut()
    }

    /// Returns a mutable reference to the current element.
    ///
    /// Returns `None` when the cursor is at the ghost position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// *cursor.current().unwrap() = 10;
    ///
    /// assert_eq!(cursor.front(), Some(&10));
    /// ```
    pub fn current(&mut self) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.current_mut(self.list)
    }

    /// Returns the first element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_back_mut();
    ///
    /// assert_eq!(cursor.front(), Some(&1));
    /// ```
    pub fn front(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.front()
    }

    /// Returns a mutable reference to the first element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_back_mut();
    ///
    /// *cursor.front_mut().unwrap() = 0;
    ///
    /// assert_eq!(cursor.front(), Some(&0));
    /// ```
    pub fn front_mut(&mut self) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.front_mut()
    }

    /// Returns the current element index.
    ///
    /// Returns `None` when the cursor is at the ghost position.
    ///
    /// The index is the logical element index in the list, not the internal
    /// chunk or slot position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.index(), Some(0));
    /// cursor.move_next();
    /// assert_eq!(cursor.index(), Some(1));
    /// ```
    pub fn index(&self) -> Option<usize> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.index(self.list)
    }

    /// Inserts `value` after the current cursor position.
    ///
    /// If the cursor is at the ghost position, the value is inserted at the
    /// front of the list and the cursor remains at the ghost position.
    /// Otherwise, the cursor remains on the same logical element.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 3]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.insert_after(2);
    ///
    /// assert_eq!(cursor.current(), Some(&mut 1));
    /// assert_eq!(cursor.peek_next(), Some(&mut 2));
    /// ```
    pub fn insert_after(&mut self, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.position.is_ghost(self.list) {
            self.push_front(value);
            return;
        }

        self.list.insert_after_position(&mut self.position, value);
    }

    /// Inserts `value` before the current cursor position.
    ///
    /// If the cursor is at the ghost position, the value is inserted at the
    /// back of the list and the cursor remains at the ghost position.
    /// Otherwise, the cursor remains on the same logical element, now shifted
    /// one logical index to the right.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([2, 3]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.insert_before(1);
    ///
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// assert_eq!(cursor.peek_prev(), Some(&mut 1));
    /// ```
    pub fn insert_before(&mut self, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.position.is_ghost(self.list) {
            self.push_back(value);
            return;
        }

        self.list.insert_before_position(&mut self.position, value);
    }

    /// Moves the cursor to the next position.
    ///
    /// Moving next from the last element moves to the ghost position. Moving
    /// next from the ghost position wraps to the front of the list.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.move_next();
    ///
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// ```
    pub fn move_next(&mut self) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.move_next(self.list);
    }

    /// Moves the cursor to the previous position.
    ///
    /// Moving previous from the front element moves to the ghost position.
    /// Moving previous from the ghost position moves to the back of the list.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.move_prev();
    /// assert_eq!(cursor.index(), None);
    ///
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// ```
    pub fn move_prev(&mut self) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.move_prev(self.list);
    }

    /// Returns the next element mutably without moving the cursor.
    ///
    /// When the cursor is at the ghost position, this returns the front
    /// element. When the cursor is at the back element, this returns `None`.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// assert_eq!(cursor.peek_next(), Some(&mut 2));
    /// assert_eq!(cursor.current(), Some(&mut 1));
    /// ```
    pub fn peek_next(&mut self) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.peek_next_mut(self.list)
    }

    /// Returns the previous element mutably without moving the cursor.
    ///
    /// When the cursor is at the front element, this returns `None`. When the
    /// cursor is at the ghost position, this returns the back element.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_back_mut();
    ///
    /// assert_eq!(cursor.peek_prev(), Some(&mut 1));
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// ```
    pub fn peek_prev(&mut self) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.peek_prev_mut(self.list)
    }

    /// Pushes `value` to the front of the list.
    ///
    /// If the cursor is pointing at an existing element, it continues to point
    /// at that same element after the insertion. If the cursor is at the ghost
    /// position, it remains at the ghost position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([2]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.push_front(1);
    ///
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// assert_eq!(cursor.front(), Some(&1));
    /// ```
    pub fn push_front(&mut self, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let was_ghost = self.position.is_ghost(self.list);
        let chunks_len_backup = self.list.chunks.len();

        self.list.push_front(value);

        if was_ghost {
            self.position = Position::ghost(self.list);
            return;
        }

        self.position.element += 1;

        if self.list.chunks.len() > chunks_len_backup {
            self.position.chunk += 1;
        }

        if self.position.chunk == 0 {
            self.position.slot += 1;
        }
    }

    /// Pushes `value` to the back of the list.
    ///
    /// If the cursor is at the ghost position, it remains at the ghost position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1]);
    /// let mut cursor = list.cursor_front_mut();
    ///
    /// cursor.push_back(2);
    ///
    /// assert_eq!(cursor.current(), Some(&mut 1));
    /// assert_eq!(cursor.back(), Some(&2));
    /// ```
    pub fn push_back(&mut self, value: T) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let was_ghost = self.position.is_ghost(self.list);

        self.list.push_back(value);

        if was_ghost {
            self.position = Position::ghost(self.list);
        }
    }

    /// Removes and returns the front element of the list.
    ///
    /// The cursor is adjusted to keep pointing at the same logical element when
    /// possible. If the removed element is before the cursor, the cursor index
    /// decreases by one.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_back_mut();
    ///
    /// assert_eq!(cursor.pop_front(), Some(1));
    /// assert_eq!(cursor.current(), Some(&mut 2));
    /// assert_eq!(cursor.index(), Some(0));
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let chunks_len_backup = self.list.chunks.len();

        let out = self.list.pop_front();

        self.position.element = self.position.element.saturating_sub(1);

        if self.position.chunk == 0 {
            self.position.slot = self.position.slot.saturating_sub(1);
        }

        if self.list.chunks.len() < chunks_len_backup {
            self.position.chunk = self.position.chunk.saturating_sub(1);
        }

        out
    }

    /// Removes and returns the back element of the list.
    ///
    /// If the cursor becomes past-the-end, it is adjusted to the ghost position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_back_mut();
    ///
    /// assert_eq!(cursor.pop_back(), Some(2));
    /// assert_eq!(cursor.index(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        let out = self.list.pop_back();

        if self.position.is_ghost(self.list) {
            self.position = Position::ghost(self.list);
        }

        out
    }

    /// Removes and returns the current element.
    ///
    /// Returns `None` when the cursor is at the ghost position. After removal,
    /// the cursor points to the next element, or to the ghost position if the
    /// removed element was the last one.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let mut list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3]);
    /// let mut cursor = list.cursor_front_mut();
    /// cursor.move_next();
    ///
    /// assert_eq!(cursor.remove_current(), Some(2));
    /// assert_eq!(cursor.current(), Some(&mut 3));
    /// ```
    pub fn remove_current(&mut self) -> Option<T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.position.is_ghost(self.list) {
            return None;
        }

        self.list.remove_at_position(&mut self.position)
    }
}

impl<T, const N: usize> core::fmt::Debug for CursorMut<'_, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        f.debug_struct("CursorMut")
            .field("position", &self.position)
            .finish()
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
    fn current_front_back_and_peeks_can_mutate_neighbors() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut cursor = list.cursor_front_mut();

            *cursor.current().unwrap() = 10;
            *cursor.peek_next().unwrap() = 20;
            *cursor.back_mut().unwrap() = 30;
            *cursor.front_mut().unwrap() = 1;

            assert_eq!(cursor.front(), Some(&1));
            assert_eq!(cursor.back(), Some(&30));
            assert_eq!(cursor.as_cursor().current(), Some(&1));
            assert_eq!(values(cursor.as_list()), [1, 20, 30]);
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_before_and_after_keep_cursor_on_current_element() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([2, 4]);
            let mut cursor = list.cursor_front_mut();

            cursor.insert_before(1);
            cursor.insert_after(3);

            assert_eq!(cursor.current(), Some(&mut 2));
            assert_eq!(cursor.peek_prev(), Some(&mut 1));
            assert_eq!(cursor.peek_next(), Some(&mut 3));
            assert_eq!(values(cursor.as_list()), [1, 2, 3, 4]);
        }

        check_capacities!(check);
    }

    #[test]
    fn insert_before_and_after_from_ghost_target_list_edges() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2]);
            let mut cursor = list.cursor_back_mut();
            cursor.move_next();

            assert_eq!(cursor.index(), None);

            cursor.insert_after(0);
            assert_eq!(cursor.index(), None);
            cursor.insert_before(3);
            assert_eq!(cursor.index(), None);

            assert_eq!(values(cursor.as_list()), [0, 1, 2, 3]);
        }

        check_capacities!(check);
    }

    #[test]
    fn push_front_and_back_adjust_cursor_position() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([2, 3]);
            let mut cursor = list.cursor_front_mut();

            cursor.push_front(1);
            cursor.push_back(4);

            assert_eq!(cursor.current(), Some(&mut 2));
            assert_eq!(cursor.index(), Some(1));
            assert_eq!(values(cursor.as_list()), [1, 2, 3, 4]);
        }

        check_capacities!(check);
    }

    #[test]
    fn pop_front_and_back_adjust_or_ghost_cursor() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut cursor = list.cursor_back_mut();

            assert_eq!(cursor.pop_front(), Some(1));
            assert_eq!(cursor.current(), Some(&mut 3));
            assert_eq!(cursor.index(), Some(1));
            assert_eq!(cursor.pop_back(), Some(3));
            assert_eq!(cursor.index(), None);
            assert_eq!(values(cursor.as_list()), [2]);
        }

        check_capacities!(check);
    }

    #[test]
    fn remove_current_moves_to_next_or_ghost() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut cursor = list.cursor_front_mut();

            cursor.move_next();
            assert_eq!(cursor.remove_current(), Some(2));
            assert_eq!(cursor.current(), Some(&mut 3));
            assert_eq!(cursor.remove_current(), Some(3));
            assert_eq!(cursor.current(), None);
            assert_eq!(cursor.remove_current(), None);
            assert_eq!(values(cursor.as_list()), [1]);
        }

        check_capacities!(check);
    }

    #[test]
    fn empty_cursor_mutation_edges() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::new();
            let mut cursor = list.cursor_front_mut();

            assert_eq!(cursor.current(), None);
            assert_eq!(cursor.pop_front(), None);
            assert_eq!(cursor.pop_back(), None);
            cursor.push_back(1);
            assert_eq!(cursor.index(), None);
            assert_eq!(cursor.peek_prev(), Some(&mut 1));
            cursor.move_prev();
            assert_eq!(cursor.current(), Some(&mut 1));
        }

        check_capacities!(check);
    }

    #[test]
    fn debug_includes_cursor_mut_position() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2]);
            let cursor = list.cursor_back_mut();

            let debug = alloc::format!("{cursor:?}");
            assert!(debug.starts_with("CursorMut { position: Position"));
            assert!(debug.contains("element: 1"));
        }

        check_capacities!(check);
    }

    #[test]
    fn mutable_cursor_edits_work_for_required_capacities() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 3, 5, 7]);
            let mut cursor = list.cursor_front_mut();

            cursor.insert_before(0);
            cursor.insert_after(2);
            cursor.move_next();
            cursor.move_next();
            cursor.insert_after(4);
            cursor.move_next();
            cursor.move_next();
            cursor.insert_after(6);
            cursor.push_back(8);
            cursor.push_front(-1);

            assert_eq!(cursor.remove_current(), Some(5));
            assert_eq!(cursor.current(), Some(&mut 6));
            assert_eq!(cursor.pop_front(), Some(-1));
            assert_eq!(cursor.pop_back(), Some(8));

            assert_eq!(values(cursor.as_list()), [0, 1, 2, 3, 4, 6, 7]);
        }

        check_capacities!(check);
    }
}
