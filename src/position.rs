use crate::ArrayList;
use cap_vec::CapVec;

#[derive(Clone, Copy)]
pub(crate) struct Position {
    pub(crate) element: usize,
    pub(crate) chunk: usize,
    pub(crate) slot: usize,
}

impl Position {
    pub(crate) fn front<T, const N: usize>(_: &ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            element: 0,
            chunk: 0,
            slot: 0,
        }
    }

    pub(crate) fn back<T, const N: usize>(list: &ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            element: list.len().saturating_sub(1),
            chunk: list.chunks.len().saturating_sub(1),
            slot: list.chunks.back().map_or(0, CapVec::len).saturating_sub(1),
        }
    }

    pub(crate) fn ghost<T, const N: usize>(list: &ArrayList<T, N>) -> Self {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        Self {
            element: list.len(),
            chunk: list.chunks.len(),
            slot: 0,
        }
    }

    pub(crate) fn current<T, const N: usize>(self, list: &ArrayList<T, N>) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        list.chunks
            .get(self.chunk)
            .and_then(|chunk| chunk.get(self.slot))
    }

    pub(crate) fn current_mut<T, const N: usize>(
        self,
        list: &mut ArrayList<T, N>,
    ) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        list.chunks
            .get_mut(self.chunk)
            .and_then(|chunk| chunk.get_mut(self.slot))
    }

    pub(crate) fn index<T, const N: usize>(self, list: &ArrayList<T, N>) -> Option<usize> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        (!self.is_ghost(list)).then_some(self.element)
    }

    pub(crate) fn is_ghost<T, const N: usize>(self, list: &ArrayList<T, N>) -> bool {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        self.element >= list.len()
    }

    pub(crate) fn move_next<T, const N: usize>(&mut self, list: &ArrayList<T, N>) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.is_ghost(list) {
            *self = Self::front(list);
            return;
        }

        self.element += 1;
        self.slot += 1;

        if self.slot >= list.chunks[self.chunk].len() {
            self.chunk += 1;
            self.slot = 0;
        }
    }

    pub(crate) fn move_prev<T, const N: usize>(&mut self, list: &ArrayList<T, N>) {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.element == 0 {
            *self = Self::ghost(list);
            return;
        }

        self.element -= 1;

        if self.slot > 0 {
            self.slot -= 1;
            return;
        }

        self.chunk -= 1;
        self.slot = list.chunks[self.chunk].len().saturating_sub(1);
    }

    pub(crate) fn peek_next<T, const N: usize>(self, list: &ArrayList<T, N>) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.is_ghost(list) {
            return list.front();
        }

        if self.slot + 1 < list.chunks[self.chunk].len() {
            return list.chunks[self.chunk].get(self.slot + 1);
        }

        list.chunks
            .get(self.chunk + 1)
            .and_then(|chunk| chunk.first())
    }

    pub(crate) fn peek_next_mut<T, const N: usize>(
        self,
        list: &mut ArrayList<T, N>,
    ) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.is_ghost(list) {
            return list.front_mut();
        }

        if self.slot + 1 < list.chunks[self.chunk].len() {
            return list.chunks[self.chunk].get_mut(self.slot + 1);
        }

        list.chunks
            .get_mut(self.chunk + 1)
            .and_then(|chunk| chunk.first_mut())
    }

    pub(crate) fn peek_prev<T, const N: usize>(self, list: &ArrayList<T, N>) -> Option<&T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.element == 0 {
            return None;
        }

        if self.slot > 0 {
            return Some(&list.chunks[self.chunk][self.slot - 1]);
        }

        list.chunks
            .get(self.chunk.saturating_sub(1))
            .and_then(|chunk| chunk.last())
    }

    pub(crate) fn peek_prev_mut<T, const N: usize>(
        self,
        list: &mut ArrayList<T, N>,
    ) -> Option<&mut T> {
        const { assert!(N >= crate::MIN_CHUNK_CAPACITY) };

        if self.element == 0 {
            return None;
        }

        if self.slot > 0 {
            return Some(&mut list.chunks[self.chunk][self.slot - 1]);
        }

        list.chunks
            .get_mut(self.chunk.saturating_sub(1))
            .and_then(|chunk| chunk.last_mut())
    }
}

impl core::fmt::Debug for Position {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Position")
            .field("element", &self.element)
            .field("chunk", &self.chunk)
            .field("slot", &self.slot)
            .finish()
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::Position;
    use crate::ArrayList;

    macro_rules! check_capacities {
        ($check:ident) => {{
            $check::<4>();
            $check::<8>();
            $check::<16>();
        }};
    }

    #[test]
    fn front_back_and_ghost_positions_for_empty_and_populated_lists() {
        fn check<const N: usize>() {
            let empty = ArrayList::<i32, N>::new();
            let list = ArrayList::<i32, N>::from([1, 2, 3]);

            assert_eq!(Position::front(&empty).index(&empty), None);
            assert_eq!(Position::back(&empty).index(&empty), None);
            assert!(Position::ghost(&empty).is_ghost(&empty));

            let back = Position::back(&list);
            assert_eq!(back.index(&list), Some(2));
            assert_eq!(back.current(&list), Some(&3));
        }

        check_capacities!(check);
    }

    #[test]
    fn move_next_crosses_chunks_and_wraps_from_ghost_to_front() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut position = Position::front(&list);

            position.move_next(&list);
            assert_eq!(position.current(&list), Some(&2));

            position.move_next(&list);
            assert_eq!(position.current(&list), Some(&3));

            position.move_next(&list);
            assert!(position.is_ghost(&list));

            position.move_next(&list);
            assert_eq!(position.current(&list), Some(&1));
        }

        check_capacities!(check);
    }

    #[test]
    fn move_prev_crosses_chunks_and_wraps_from_front_to_ghost() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3]);
            let mut position = Position::back(&list);

            position.move_prev(&list);
            assert_eq!(position.current(&list), Some(&2));

            position.move_prev(&list);
            assert_eq!(position.current(&list), Some(&1));

            position.move_prev(&list);
            assert!(position.is_ghost(&list));

            position.move_prev(&list);
            assert_eq!(position.current(&list), Some(&3));
        }

        check_capacities!(check);
    }

    #[test]
    fn peek_next_and_prev_handle_boundaries_and_ghost() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3, 4]);
            let front = Position::front(&list);
            let back = Position::back(&list);
            let ghost = Position::ghost(&list);

            assert_eq!(front.peek_prev(&list), None);
            assert_eq!(front.peek_next(&list), Some(&2));
            assert_eq!(back.peek_next(&list), None);
            assert_eq!(back.peek_prev(&list), Some(&3));
            assert_eq!(ghost.peek_next(&list), Some(&1));
            assert_eq!(ghost.peek_prev(&list), Some(&4));
        }

        check_capacities!(check);
    }

    #[test]
    fn mutable_accessors_update_current_and_neighbors() {
        fn check<const N: usize>() {
            let mut list = ArrayList::<i32, N>::from([1, 2, 3]);
            let position = Position::front(&list);

            *position.current_mut(&mut list).unwrap() = 10;
            *position.peek_next_mut(&mut list).unwrap() = 20;
            *Position::ghost(&list).peek_prev_mut(&mut list).unwrap() = 30;

            assert_eq!(list, [10, 20, 30]);
        }

        check_capacities!(check);
    }

    #[test]
    fn debug_reports_all_coordinates() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([1, 2, 3, 4]);
            let position = Position::back(&list);

            let debug = alloc::format!("{position:?}");
            assert!(debug.contains("Position"));
            assert!(debug.contains("element: 3"));
        }

        check_capacities!(check);
    }

    #[test]
    fn position_navigation_works_for_required_capacities() {
        fn check<const N: usize>() {
            let list = ArrayList::<i32, N>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
            let mut position = Position::front(&list);

            for expected in 0..10 {
                assert_eq!(position.index(&list), Some(expected as usize));
                assert_eq!(position.current(&list), Some(&expected));
                position.move_next(&list);
            }

            assert!(position.is_ghost(&list));
            assert_eq!(position.peek_next(&list), Some(&0));
            assert_eq!(position.peek_prev(&list), Some(&9));

            for expected in (0..10).rev() {
                position.move_prev(&list);
                assert_eq!(position.current(&list), Some(&expected));
            }

            position.move_prev(&list);
            assert!(position.is_ghost(&list));
        }

        check_capacities!(check);
    }
}
