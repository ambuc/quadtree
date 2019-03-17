# quadtree

[Point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree) with support for 
overlapping regions.

Under active development.

# Example Usage

```rust
extern crate quadtree_rs;

use quadtree_rs::Quadtree;
use quadtree_rs::Entry;

// Create a new Quadtree with (u64, u64) x/y coordinates, String values, and a
// depth of four layers. Since 2^4 = 16, this grid will be of width and
// height 16.
let mut qt = Quadtree::<u64, String>::new(4);

// Insert "foo" in the coordinate system such that it occupies a rectangle with
// top-left "anchor" (0, 0), and width/height 2x1.
//
//   0  1  2  3
// 0 ░░░░░░░--+
//   ░░░░░░░ <--foo @ (0,0)->2x1
// 1 ░░░░░░░--+
//   |  |  |  |
// 2 +--+--+--+
let handle = qt.insert((0, 0), (2, 1), "foo".to_string());

// We've received an handle for our insertion which can be used to get (and 
// mutate) the value in-place.
assert_eq!(qt.get(handle), Some("foo"));

// A Quadtree can be queried by region. 
//
//   0  1  2  3
// 0 ░░░▓▓▓▓▒▒▒
//   ░░░▓▓▓▓▒▒▒ <--query region @ (1,0)->2x2
// 1 ░░░▓▓▓▓▒▒▒
//   |  ▒▒▒▒▒▒▒
// 2 +--▒▒▒▒▒▒▒
let mut query = qt.query((1, 0), (2, 2));

// There is an overlap between our query region and the region holding "foo",
// so we expect that iterator to return the `Entry` corresponding to the
// inserted "foo".
let result: Entry<u64, String> = query.next().unwrap();
assert_eq!(result.value_ref(), "foo");
```

# Questions?

Please file an issue on GitHub.

# Authors

See [`Cargo.toml`](Cargo.toml).

# Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`NOTES.md`](NOTES.md)

# License

This project is licensed under the Apache 2.0 license.

# Disclaimer

This is not an official Google product. 

# TODOS
 - Run `cargo clippy` over the whole repo.
 - Do a final pass over all public-facing documentation and language with a
   preference for 
    - association rather than insertion,
    - handle over index/pointer/reference,
 - Do away with `((x,y),(w,h))` pattern.
