# array_list

[![Crates.io](https://img.shields.io/crates/v/array_list.svg)](https://crates.io/crates/array_list)
[![Docs.rs](https://img.shields.io/docsrs/array_list)](https://docs.rs/array_list)
[![License](https://img.shields.io/crates/l/array_list.svg)](https://github.com/daddinuz/array_list/blob/main/LICENSE)

`array_list` is an ordered collection backed by fixed-capacity chunks. It gives
you familiar sequence operations, iterators, and cursors while avoiding one
allocation per element.

## Installation

Add `array_list` to your `Cargo.toml`:

```bash
cargo add array_list
```

or edit your Cargo.toml manually by adding:

```toml
[dependencies]
array_list = "0.5"
```

This crate requires Rust 1.87 or newer.

## Quick Start

```rust
use array_list::ArrayList;

let mut list: ArrayList<i32, 8> = ArrayList::from([1, 2, 4]);

list.insert(2, 3);
list.push_front(0);
list.push_back(5);

assert_eq!(list, [0, 1, 2, 3, 4, 5]);

for value in &mut list {
    *value *= 2;
}

assert_eq!(list.iter().copied().collect::<Vec<_>>(), [0, 2, 4, 6, 8, 10]);
```

## Features
- Ordered sequence with index-based access.
- Chunked storage with less per-element pointer overhead than a traditional linked list.
- Shared, mutable, and owning iterators.
- Cursor and mutable cursor APIs for moving around the list and editing near the cursor.
- `#![no_std]` support with `alloc`.

## When To Use It

`array_list` is useful when you want an ordered sequence that supports indexed
access, stable iteration order, and cursor-local edits without allocating one
node per element.

It is not a drop-in replacement for `Vec`. Index lookup may need to scan chunks
when the list is not densely packed, and insertions/removals inside a chunk can
move elements within that chunk. It is also not a drop-in replacement for
`LinkedList`: elements inside each chunk are stored contiguously, but operations
can still move values inside a chunk or split chunks when capacity is reached.

Use `Vec` when you primarily push/pop at the back and need dense contiguous
storage. Use `ArrayList` when cursor-local edits and chunked growth are a better
fit than a single contiguous allocation. Use `LinkedList` only when its exact
node-based semantics are required.

## API Overview

- Build lists with `ArrayList::new`, `ArrayList::from`, `collect`, `extend`,
  or `append`.
- Edit at the ends with `push_front`, `push_back`, `pop_front`, and `pop_back`.
- Edit by index with `insert`, `remove`, `get`, and `get_mut`.
- Traverse with `iter`, `iter_mut`, `into_iter`, or the `IntoIterator`
  implementations for `ArrayList`, `&ArrayList`, and `&mut ArrayList`.
- Navigate and edit near a position with `Cursor` and `CursorMut`.
- Inspect storage with `capacity`, `spare_capacity`, and `shrink`.

## Storage Model

`ArrayList<T, N>` stores elements in chunks that can each hold up to `N`
elements. The logical order is independent from the internal chunk boundaries:
iteration and indexing always see one continuous sequence.

Middle insertions and removals may leave spare capacity inside chunks. After a
middle removal that leaves the edited chunk non-empty, or after a middle
insertion that splits a full chunk, the edited chunk is locally refilled up to
half capacity by taking elements from immediate sibling chunks when those
siblings contain more than half capacity. Siblings at or below half capacity are
left untouched, so edit-heavy workloads can still become fragmented. Use
`spare_capacity` to inspect unused chunk slots and `shrink` to compact elements
toward the front while preserving logical order.

This is a local minimum-occupancy heuristic, not a global invariant. The minimum
is `N / 2` using integer division, so odd capacities round down. Refill only
touches the edited chunk and its immediate siblings, and it prefers bounded
local movement over globally dense storage. Inserting into a full middle chunk
splits that chunk locally. Front/back pushes and pops keep their direct boundary
behavior, so edge chunks may be less than half full.

## Chunk Capacity

The second type parameter is the maximum number of elements per chunk:

```rust
use array_list::ArrayList;

let list: ArrayList<i32, 32> = ArrayList::new();
```

Capacity must be at least 4. Smaller chunk sizes are rejected at compile time by
constructors and operations because this data structure is designed around
multi-element chunks and a local half-full occupancy heuristic.

Choose `N` based on your workload. Chunk size has a large effect on performance:
small capacities reduce the maximum amount moved inside one chunk, but they also
create many more chunks, more boundary crossings, more allocations, and more
metadata to scan. Larger capacities improve locality and reduce chunk count,
which is especially important for cursor-local edits, but middle insertions and
removals can move more elements within the edited chunk and its immediate
siblings.

Very small capacities are mostly useful for stress-testing chunk boundaries. For
general use, prefer capacities large enough to amortize chunk overhead. Values in
the 32-64 range are usually more representative than the minimum supported size;
mutation-heavy workloads often benefit from the larger end of that range.

Keeping edited chunks at least half full where possible reduces the chance of
many tiny chunks after repeated mutations. The tradeoff is that the list may
contain more chunks than a fully compacted layout, so locating an indexed element
can require scanning more chunk metadata until `shrink` is called.

## Performance Notes

The exact cost depends on chunk occupancy and the chosen chunk capacity:

- `len` and `is_empty` are constant time.
- End pushes and pops usually touch only one edge chunk.
- `get`, `get_mut`, `insert`, and `remove` locate elements by scanning chunk
  metadata unless the list is densely packed.
- Middle insertion/removal may move elements within the edited chunk and its
  immediate siblings.
- Iteration is in logical order and does not expose chunk boundaries.

Call `shrink` after edit-heavy phases if a compact front-filled layout is more
important than preserving spare chunk slots for future edits.

## `no_std`

`array_list` is `#![no_std]` and uses `alloc`, so it can be used in environments
that provide an allocator but not the full standard library:

```rust
extern crate alloc;

use array_list::ArrayList;

let list: ArrayList<i32, 8> = ArrayList::from([1, 2, 3]);
assert_eq!(list.len(), 3);
```

## Example

```rust
use array_list::ArrayList;

fn main() {
    let mut list: ArrayList<i32, 4> = ArrayList::new();

    // Insert elements
    list.push_back(1);
    list.push_back(3);
    list.push_front(0);
    list.insert(1, 2);

    // Access elements
    println!("front: {:?}", list.front()); // Some(0)
    println!("back: {:?}", list.back());   // Some(3)

    // Remove elements
    assert_eq!(list.pop_front(), Some(0));
    assert_eq!(list.pop_back(), Some(3));

    // Cursors can move without consuming elements.
    let mut cursor = list.cursor_front();
    assert_eq!(cursor.current(), Some(&2));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&1));
}
```

## Cursors

`Cursor` and `CursorMut` point either at an element or at the ghost position
after the back of the list. Moving forward from the ghost wraps to the front;
moving backward from the front moves to the ghost.

Mutable cursor edits keep the cursor on the same logical element when possible.
For example, `insert_before` inserts before the current element and leaves the
cursor on that original element; `remove_current` removes the current element and
moves the cursor to the next element, or to the ghost position if there is no
next element.

```rust
use array_list::ArrayList;

let mut list: ArrayList<i32, 4> = ArrayList::from([1, 3]);
let mut cursor = list.cursor_front_mut();

cursor.insert_after(2);
cursor.move_next();
cursor.insert_after(4);

assert_eq!(cursor.current(), Some(&mut 2));
assert_eq!(cursor.as_list(), &[1, 2, 4, 3]);
```

## Testing

Run the default test suite:

```bash
cargo test
```

Run the focused Miri suite:

```bash
cargo +nightly miri test --lib
```

When `cfg(miri)` is active, the regular unit and property tests are compiled
out so Miri only runs the targeted ownership, cursor, and iterator checks.

Build the benchmark suite:

```bash
cargo bench --no-run
```

Run the Criterion benchmarks:

```bash
cargo bench
```

Check formatting, lints, and public API documentation:

```bash
cargo fmt --check
cargo clippy --tests --benches
cargo rustdoc --lib -- -D missing_docs
```

## Contributing

Contributions are welcome. Whether it is improving documentation, fixing bugs,
or suggesting new features, feel free to open an issue or submit a pull request.

When contributing, please ensure:
- Code is formatted with `cargo fmt`.
- Tests are added or updated as necessary.
- Safety is maintained for any `unsafe` code introduced.

By contributing, you agree that your contributions will be licensed under the terms of the MIT license.

## License

This crate is licensed under the [MIT License](LICENSE).
You are free to use, modify, and distribute it under the terms of the license.
