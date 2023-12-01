// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A [point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree) with support for overlapping
//! regions.
//!
//! # Quick Start
//! ```
//! use quadtree_rs::{Area, Quadtree};
//!
//! // Instantiate a new quadtree which associates String values with u64 coordinates.
//! let mut qt = Quadtree::<u64, String>::new(/*depth=*/4);
//!
//! // A depth of four means a square with width (and height) 2^4.
//! assert_eq!(qt.width(), 16);
//!
//! // Associate the value "foo" with a rectangle of size 2x1, anchored at (0, 0).
//! let region_a: Area<u64> = ((0,0),(2,1)).into();
//! qt.insert(region_a, "foo".to_string());
//!
//! // Query over a region of size 2x2, anchored at (1, 0).
//! let region_b: Area<u64> = ((1,0),(2,2)).into();
//! let mut query = qt.query(region_b);
//!
//! // The query region (region_b) intersects the region "foo" is associated with (region_a), so the query iterator returns "foo" by reference.
//! assert_eq!(query.next().unwrap().value_ref(), "foo");
//! ```
//!
//! # Implementation
//! ```
//! use quadtree_rs::{Area, Point, Quadtree};
//!
//! let mut qt = Quadtree::<u8, char>::new(2);
//!
//! // In a quadtree, every region is (lazily) subdivided into subqudrants.
//!
//! // Associating a value with a point, which is represented by a region with dimensions 1x1, means traversing the full height of the quadtree.
//! qt.insert_pt(Point {x: 0, y: 0}, 'a');
//!
//! // (0,0)->4x4                +---+---+---+---+
//! //   (0,0)->2x2              | a |   |       |
//! //     (0,0)->1x1 ['a']      +---+   +       +
//! //                           |       |       |
//! //                           +---+---+---+---+
//! //                           |       |       |
//! //                           +       +       +
//! //                           |       |       |
//! //                           +---+---+---+---+
//!
//! // Often inserting a large region requires traversing only as far down as necessary to fully cover that region.
//! let region_b: Area<u8> = ((0,0),(2,2)).into();
//! qt.insert(region_b, 'b');
//!
//! // (0,0)->4x4                +---+---+---+---+
//! //   (0,0)->2x2 ['b']        | a |   |       |
//! //     (0,0)->1x1 ['a']      +---+   +       +
//! //                           |     b |       |
//! //                           +---+---+---+---+
//! //                           |       |       |
//! //                           +       +       +
//! //                           |       |       |
//! //                           +---+---+---+---+
//!
//! // If a region cannot be represented by one node in the tree, a handle type is inserted in multiple places.
//! let region_c: Area<u8> = ((0,0),(3,3)).into();
//! qt.insert(region_c, 'c');
//!
//! // (0,0)->4x4                +---+---+---+---+
//! //   (0,0)->2x2 ['b', 'c']   | a |   | c |   |
//! //     (0,0)->1x1 ['a']      +---+   +---+---+
//! //   (0,2)->2x2              |   b,c | c |   |
//! //     (0,2)->1x1 ['c']      +---+---+---+---+
//! //     (1,2)->1x1 ['c']      | c | c | c |   |
//! //   (2,0)->2x2              +---+---+---+---+
//! //     (2,0)->1x1 ['c']      |   |   |   |   |
//! //     (2,1)->1x1 ['c']      +---+---+---+---+
//! //   (2,2)->2x2
//! //     (2,2)->1x1 ['c']
//! ```
//!
//! Duplicating the storage handle allows for fast lookup and insertion at the cost of slow
//! deletion. `quadtree_rs` is well-suited for scenarios with low churn but frequent read access.
//!
//! # Usage
//!
//! For further usage details, see the documentations for the [`Quadtree`] struct.
//!
//! [`Quadtree`]: struct.Quadtree.html

// For extra-pedantic documentation tests.
#![doc(test(attr(deny(warnings))))]

pub mod entry;
pub mod geometry;
pub mod iter;

mod handle_iter;
mod qtinner;
mod quadtree;
mod traversal;
mod types;

pub use geometry::{Area, Point};
pub use quadtree::Quadtree;
