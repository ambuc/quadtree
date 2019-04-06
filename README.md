# [quadtree_rs](https://crates.io/crates/quadtree_rs)

[![crates.io
badge](https://img.shields.io/crates/v/quadtree_rs.svg)](https://crates.io/crates/quadtree_rs)
[![docs.rs
badge](https://docs.rs/quadtree_rs/badge.svg)](https://docs.rs/quadtree_rs)
[![license](https://img.shields.io/crates/l/quadtree_rs.svg)](https://github.com/ambuc/quadtree/blob/master/LICENSE)

[Point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree) with support for 
overlapping regions.

For documentation, see [docs.rs/quadtree_rs](https://docs.rs/quadtree_rs/).

# Quick Start

```rust
use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};

// Instantiate a new quadtree which associates String values with u64 
// coordinates.
let mut qt = Quadtree::<u64, String>::new(/*depth=*/4);

// A depth of four means a square with width (and height) 2^4.
assert_eq!(qt.width(), 16);

// Associate the value "foo" with a rectangle of size 2x1, anchored at (0, 0).
let region_a = AreaBuilder::default()
    .anchor(Point {x: 0, y: 0})
    .dimensions((2, 1))
    .build().unwrap();
qt.insert(region_a, "foo".to_string());

// Query over a region of size 2x2, anchored at (1, 0).
let region_b = AreaBuilder::default()
    .anchor(Point {x: 1, y: 0})
    .dimensions((2, 2))
    .build().unwrap();
let mut query = qt.query(region_b);

// The query region (region_b) intersects the region "foo" is associated with 
// (region_a), so the query iterator returns "foo" by reference.
assert_eq!(query.next().unwrap().value_ref(), "foo");
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

# TODO
 - [ ] Pretty-print quadtree function which plots a density map
 - [ ] Benchmark tests
