# ArrayList

`ArrayList` is a Rust crate that implements an **unrolled linked list** powered by a **XOR linked list** as its underlying structure.
It combines features of both a `Vec` and a `LinkedList`, offering efficient sequential access with reduced pointer overhead compared
to a traditional doubly linked list.  

This crate is designed to handle dynamic collections efficiently, particularly in scenarios where frequent insertions, deletions,
or iterations are required.

## Features

- **Dynamic:** Combines the benefits of a `Vec` (compact, cache-friendly storage) and a `LinkedList` (efficient insertions and deletions).
- **Reduced pointer overhead:** Implements a **XOR linked list**, requiring only a single pointer per node for bidirectional traversal.
- **Customizable chunk size:** The size of each chunk is determined at compile time via a const generic parameter up to 64 elements.
- **Efficient memory operations:** Splits and merges nodes dynamically, redistributing elements when necessary.
- **Rich API:** Provides functionality for:
  - Insertions, deletions and access at arbitrary positions.
  - Index-based access with `get` methods.
  - Access to front and back elements.

## Strengths

- **Sequential access:** By grouping multiple elements in each node, it reduces the pointer-following overhead inherent in linked lists.
- **Low allocation cost:** Nodes store multiple elements in contiguous memory, minimizing allocation frequency.
- **Customizable performance:** The chunk size can be tuned to balance memory usage and cache performance.

## Pitfalls

- **Random access is slower than `Vec`:** While `ArrayList` provides index-based access, it requires traversing the list, making it less efficient than `Vec` for frequent random access.
- **Unsafe code:** The implementation relies on `unsafe` code for managing memory and pointers. However, it is extensively tested.
- **More complex than `Vec` or `LinkedList`:** The additional logic for node splitting, merging, and XOR-linked traversal increases complexity compared to simpler collections.

## Installation

Add `array_list` to your `Cargo.toml`:

```bash
cargo add array_list
```

or edit your Cargo.toml manually by adding:

```toml
[dependencies]
array_list = "0.2"
```

## Example Usage

```rust
use array_list::ArrayList;

fn main() {
    let mut list: ArrayList<i32, 2> = ArrayList::new();

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
}
```

## Safety

This crate contains `unsafe` code to achieve optimal performance and memory management.

However:
- All code is tested under **Miri** to ensure memory safety.
- The code coverage is approximately **90%**, providing strong confidence in correctness.
- You can generate the code coverage report using **tarpaulin**.

## Contributing

Contributions are welcome!
Whether it’s improving documentation, fixing bugs, or suggesting new features, feel free to open an issue or submit a pull request (PR).  

When contributing, please ensure:
- Code is formatted with `cargo fmt`.
- Tests are added or updated as necessary.
- Safety is maintained for any `unsafe` code introduced.

By contributing, you agree that your contributions will be licensed under the terms of the MIT license.

## License

This crate is licensed under the [MIT License](LICENSE).
You are free to use, modify, and distribute it under the terms of the license.
