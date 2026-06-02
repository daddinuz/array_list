use crate::ArrayList;
use crate::position::Position;

/// A read-only cursor over an [`ArrayList`].
///
/// A cursor is like an iterator, except that it can move back and forth without
/// consuming elements. It points either at an element or at the ghost position
/// after the back of the list.
///
/// The ghost position is the cursor equivalent of "past the end": moving next
/// from the ghost wraps to the front, and moving previous from the front moves
/// back to the ghost.
///
/// # Example
/// ```rust
/// use array_list::ArrayList;
///
/// let list: ArrayList<i32, 4> = ArrayList::from([1, 2, 3]);
/// let mut cursor = list.cursor_front();
///
/// assert_eq!(cursor.current(), Some(&1));
/// cursor.move_next();
/// assert_eq!(cursor.current(), Some(&2));
/// ```
#[derive(Clone)]
pub struct Cursor<'a, T, const N: usize> {
    position: Position,
    list: &'a ArrayList<T, N>,
}

const _: [(); core::mem::size_of::<usize>() * 4] = [(); core::mem::size_of::<Cursor<usize, 4>>()];

impl<'a, T, const N: usize> Cursor<'a, T, N> {
    pub(crate) fn from_front(list: &'a ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            position: Position::front(list),
            list,
        }
    }

    pub(crate) fn from_back(list: &'a ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            position: Position::back(list),
            list,
        }
    }

    pub(crate) fn from_position(list: &'a ArrayList<T, N>, position: Position) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self { position, list }
    }

    /// Returns the list this cursor was created from.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.as_list().len(), 2);
    /// ```
    pub fn as_list(&self) -> &'a ArrayList<T, N> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list
    }

    /// Returns the last element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.back(), Some(&2));
    /// ```
    pub fn back(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.back()
    }

    /// Returns the element currently pointed to by the cursor.
    ///
    /// Returns `None` when the cursor is at the ghost position.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.current(), Some(&1));
    /// ```
    pub fn current(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.current(self.list)
    }

    /// Returns the first element of the list, if any.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_back();
    ///
    /// assert_eq!(cursor.front(), Some(&1));
    /// ```
    pub fn front(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.list.front()
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
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.index(), Some(0));
    /// cursor.move_next();
    /// assert_eq!(cursor.index(), Some(1));
    /// ```
    pub fn index(&self) -> Option<usize> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.index(self.list)
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
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front();
    ///
    /// cursor.move_next();
    ///
    /// assert_eq!(cursor.current(), Some(&2));
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
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let mut cursor = list.cursor_front();
    ///
    /// cursor.move_prev();
    /// assert_eq!(cursor.index(), None);
    ///
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), Some(&2));
    /// ```
    pub fn move_prev(&mut self) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.move_prev(self.list);
    }

    /// Returns the next element without moving the cursor.
    ///
    /// When the cursor is at the ghost position, this returns the front
    /// element. When the cursor is at the back element, this returns `None`.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_front();
    ///
    /// assert_eq!(cursor.peek_next(), Some(&2));
    /// assert_eq!(cursor.current(), Some(&1));
    /// ```
    pub fn peek_next(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.peek_next(self.list)
    }

    /// Returns the previous element without moving the cursor.
    ///
    /// When the cursor is at the front element, this returns `None`. When the
    /// cursor is at the ghost position, this returns the back element.
    ///
    /// # Example
    /// ```rust
    /// use array_list::ArrayList;
    ///
    /// let list: ArrayList<i32, 4> = ArrayList::from([1, 2]);
    /// let cursor = list.cursor_back();
    ///
    /// assert_eq!(cursor.peek_prev(), Some(&1));
    /// assert_eq!(cursor.current(), Some(&2));
    /// ```
    pub fn peek_prev(&self) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.position.peek_prev(self.list)
    }
}

impl<T, const N: usize> core::fmt::Debug for Cursor<'_, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        f.debug_struct("Cursor")
            .field("position", &self.position)
            .finish()
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::ArrayList;

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    #[test]
    fn empty_cursor_stays_on_ghost_position() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::new();
            let mut cursor = list.cursor_front();

            assert_eq!(cursor.current(), None);
            assert_eq!(cursor.index(), None);
            assert_eq!(cursor.peek_next(), None);
            assert_eq!(cursor.peek_prev(), None);

            cursor.move_next();
            assert_eq!(cursor.current(), None);
            cursor.move_prev();
            assert_eq!(cursor.current(), None);
        }

        check_capacities!(check);
    }

    #[test]
    fn moves_forward_backward_and_wraps_through_ghost() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut cursor = list.cursor_front();

            assert_eq!(cursor.current(), Some(&1));
            assert_eq!(cursor.index(), Some(0));
            assert_eq!(cursor.peek_prev(), None);
            assert_eq!(cursor.peek_next(), Some(&2));

            cursor.move_next();
            assert_eq!(cursor.current(), Some(&2));
            cursor.move_next();
            assert_eq!(cursor.current(), Some(&3));
            assert_eq!(cursor.peek_next(), None);
            cursor.move_next();
            assert_eq!(cursor.current(), None);
            assert_eq!(cursor.peek_next(), Some(&1));
            assert_eq!(cursor.peek_prev(), Some(&3));
            cursor.move_next();
            assert_eq!(cursor.current(), Some(&1));

            cursor.move_prev();
            assert_eq!(cursor.current(), None);
            cursor.move_prev();
            assert_eq!(cursor.current(), Some(&3));
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_back_handles_chunk_boundaries() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut cursor = list.cursor_back();

            assert_eq!(cursor.current(), Some(&3));
            assert_eq!(cursor.index(), Some(2));
            assert_eq!(cursor.peek_prev(), Some(&2));

            cursor.move_prev();
            assert_eq!(cursor.current(), Some(&2));
            cursor.move_prev();
            assert_eq!(cursor.current(), Some(&1));
            cursor.move_prev();
            assert_eq!(cursor.index(), None);
        }

        check_capacities!(check);
    }

    #[test]
    fn clone_and_as_list_preserve_cursor_state() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3, 4]);
            let mut cursor = list.cursor_front();
            cursor.move_next();

            let clone = cursor.clone();

            assert_eq!(clone.current(), Some(&2));
            assert_eq!(clone.as_list().len(), 4);
            assert_eq!(clone.front(), Some(&1));
            assert_eq!(clone.back(), Some(&4));
        }

        check_capacities!(check);
    }

    #[test]
    fn debug_includes_cursor_position() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2]);
            let cursor = list.cursor_back();

            let debug = alloc::format!("{cursor:?}");
            assert!(debug.starts_with("Cursor { position: Position"));
            assert!(debug.contains("element: 1"));
        }

        check_capacities!(check);
    }

    #[test]
    fn cursor_navigation_works_for_required_capacities() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
            let mut cursor = list.cursor_front();

            for expected in 0..10 {
                assert_eq!(cursor.index(), Some(expected as usize));
                assert_eq!(cursor.current(), Some(&expected));
                cursor.move_next();
            }

            assert_eq!(cursor.index(), None);
            assert_eq!(cursor.peek_next(), Some(&0));
            assert_eq!(cursor.peek_prev(), Some(&9));

            for expected in (0..10).rev() {
                cursor.move_prev();
                assert_eq!(cursor.current(), Some(&expected));
            }

            cursor.move_prev();
            assert_eq!(cursor.index(), None);
        }

        check_capacities!(check);
    }
}
