# array_list

`array_list` implements an **unrolled linked list** datastructure with features
that combine the simplicity of a `Vec` and the flexibility of a `LinkedList`.

## Features
- Ordered sequence with index based elements access and efficient random access lookups.
- Chunked storage, which improves cache locality and reduces pointer overhead compared to traditional linked lists.
- Stable `Cursor` API similar to the `LinkedList` one on nightly, allowing efficient operations around a point in the list.
- Dynamic growth, balancing between `Vec` and `LinkedList` characteristics.

## Use Cases
`array_list` is ideal for scenarios where:
- You need a ordered collection with random access capabilities.
- You require frequent insertions and deletions anywhere in the list.
- Memory efficiency and improved cache performance over plain LinkedList are priorities.

## Note
This crate is not related to Java's `ArrayList` despite its name.  
The design and functionality are entirely tailored to Rust's ecosystem.

## Installation

Add `array_list` to your `Cargo.toml`:

```bash
cargo add array_list
```

or edit your Cargo.toml manually by adding:

```toml
[dependencies]
array_list = "0.4"
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

- The code coverage is approximately **75%**, providing strong confidence in correctness.
  You can run the complete test suite as:
  ```bash
  cargo +nightly test --features nightly_tests
  ```
- You can generate the code coverage report using **tarpaulin**.
  You can run the code coverage report like this:
  ```bash
  cargo +nightly tarpaulin --features nightly_tests
  ```
- All code is tested under **Miri** to ensure memory safety.
  You can run the complete test suite under miri as:
  ```bash
  # NOTE: it may take a while to complete.
  cargo +nightly miri test --features nightly_tests
  ```

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
