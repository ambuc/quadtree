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

//! [Point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree) with support for overlapping
//! regions.
//!
//! # Quick Start
//!
//! Add `quadtree_rs` to your `Cargo.toml`, and then add it to your main.
//! ```
//! extern crate quadtree_rs;
//! use quadtree_rs::Quadtree;
//!
//! // Create a new Quadtree with u64 coordinates and String values. Quadtree::new(4) initializes a
//! // tree with a depth of four layers, or a height and width of 2^4 = 16 .
//! let mut qt = Quadtree::<u64, String>::new(4);
//!
//! // Insert "foo" in the tree at the rectangle with
//! // top-left corner (0, 0), width 2, and height 1.
//! //
//! //   0  1  2  3
//! // 0 ░░░░░░░--+
//! //   ░░░░░░░ <--foo
//! // 1 ░░░░░░░--+
//! //   |  |  |  |
//! // 2 +--+--+--+
//! qt.insert((0, 0), (2, 1), "foo".to_string());
//!
//! // Perform a query over a region with top-left corner (1, 0), width 1, and height 1.
//! //
//! //   0  1  2  3
//! // 0 ░░░▓▓▓▓▒▒▒
//! //   ░░░▓▓▓▓▒▒▒ <--query region
//! // 1 ░░░▓▓▓▓▒▒▒
//! //   |  ▒▒▒▒▒▒▒
//! // 2 +--▒▒▒▒▒▒▒
//! let mut query = qt.query((1, 0), (2, 2));
//!
//! // @query implements `Iterator` over `Entry<u64, String>` entries, so we can call
//! // `Entry::value_ref()` and see that our query region contains the rectangle at which "foo" was
//! // inserted.
//! assert_eq!(query.next().unwrap().value_ref(), "foo");
//! ```
//!
//! # Implementation
//! ```
//! // The Quadtree is a tree where every node has four children, representing the four
//! // evenly-divided subquadrants beneath it in the grid.
//! let mut qt = quadtree_rs::Quadtree::<u8, f32>::new(2);
//!
//! // Inserting a point (a.k.a. a region of dimensions 1x1) means traversing that tree all the way
//! // to the bottom.
//! qt.insert((0, 0), (1, 1), 1.23456);
//! // (0,0)->4x4 ─┬─ (0,0)->2x2 ─┬─ (0,0)->1x1
//! //             │              │    └ [1.23456]
//! //             │              ├─ (0,2)->1x1
//! //             │              ├─ (2,0)->1x1
//! //             │              └─ (2,2)->1x1
//! //             ├─ (0,2)->2x2
//! //             ├─ (2,0)->2x2
//! //             └─ (2,2)->2x2
//!
//! // But inserting a region which coincides with a quadrant means inserting that value somewhere
//! // higher in the tree.
//! qt.insert((0, 0), (2, 2), 2.46810);
//! // (0,0)->4x4 ─┬─ (0,0)->2x2 ─────┬─ (0,0)->1x1
//! //             │    └ [2.46810]   │    └ [1.23456]
//! //             │                  ├─ (0,2)->1x1
//! //             │                  ├─ (2,0)->1x1
//! //             │                  └─ (2,2)->1x1
//! //             ├─ (0,2)->2x2
//! //             ├─ (2,0)->2x2
//! //             └─ (2,2)->2x2
//!
//! // Inserting a region which overlaps a few quadrants means inserting that value (actually, a
//! // key which points to that value in a store) in multiple places.
//! qt.insert((0, 0), (3, 3), 3.6912);
//! // (0,0)->4x4 ─┬─ (0,0)->2x2 ─────┬─ (0,0)->1x1
//! //             │    └ [ 2.46810,  │    └ [1.23456]
//! //             │        3.6912 ]  ├─ (0,1)->1x1
//! //             │                  ├─ (1,0)->1x1
//! //             │                  └─ (1,1)->1x1
//! //             │
//! //             ├─ (0,2)->2x2 ─────┬─ (0,2)->1x1
//! //             │                  │    └ [3.6912]
//! //             │                  ├─ (0,3)->1x1
//! //             │                  ├─ (1,2)->1x1
//! //             │                  │    └ [3.6912]
//! //             │                  └─ (1,3)->1x1
//! //             │
//! //             ├─ (2,0)->2x2 ─────┬─ (2,0)->1x1
//! //             │                  │    └ [3.6912]
//! //             │                  ├─ (2,1)->1x1
//! //             │                  │    └ [3.6912]
//! //             │                  ├─ (3,0)->1x1
//! //             │                  └─ (3,1)->1x1
//! //             │
//! //             └─ (2,2)->2x2 ─────┬─ (2,2)->1x1
//! //                                │    └ [3.6912]
//! //                                ├─ (2,3)->1x1
//! //                                ├─ (3,2)->1x1
//! //                                └─ (3,3)->1x1
//! ```
//! In practice this is relatively efficient. It allows for fast regional lookups at the cost of
//! more expensive insertions and deletions. This library optimizes for many small items which
//! don't overlap much as opposed to large items.
//!
//! # Usage
//!
//! For usage details, see [`Quadtree`].
//!
//! [`Quadtree`]: struct.Quadtree.html

// For extra-pedantic documentation tests.
#![doc(test(attr(deny(warnings))))]

extern crate num;

pub mod entry;

mod geometry;
mod handle_iter;
mod qtinner;
mod traversal;
mod types;

use {
    crate::{
        entry::Entry,
        geometry::{
            area::{Area, AreaType},
            point::PointType,
        },
        handle_iter::HandleIter,
        qtinner::QTInner,
        traversal::Traversal,
        types::StoreType,
    },
    num::PrimInt,
    std::{
        collections::{HashMap, HashSet},
        iter::FusedIterator,
    },
};

//   .d88b.  db    db  .d8b.  d8888b. d888888b d8888b. d88888b d88888b
//  .8P  Y8. 88    88 d8' `8b 88  `8D `~~88~~' 88  `8D 88'     88'
//  88    88 88    88 88ooo88 88   88    88    88oobY' 88ooooo 88ooooo
//  88    88 88    88 88~~~88 88   88    88    88`8b   88~~~~~ 88~~~~~
//  `8P  d8' 88b  d88 88   88 88  .8D    88    88 `88. 88.     88.
//   `Y88'Y8 ~Y8888P' YP   YP Y8888D'    YP    88   YD Y88888P Y88888P

/// A data structure for storing and accessing data by x/y coordinates.
/// (A [Quadtree](https://en.wikipedia.org/wiki/Quadtree).)
///
/// `Quadtree<U, V>` is parameterized over
///  - `U`, where `U` is the index type of the x/y coordinate, and
///  - `V`, where `V` is the value being stored in the data structure.
///
/// Regions are represented by the type
/// ```
/// type U = u64; // Or any primitive integer, signed or unsigned.
///
/// //   01234567
/// // 0 ░░░░░░░░
/// // 1 ░░▓▓▓░░░ (2,1)->3x1
/// // 2 ░░░░░░░░
/// let _region: ((U, U), (U, U)) = ((2, 1), (3, 1));
/// //             x  y    w  h
/// //            anchor  dimensions
/// ```
/// where
///  - `anchor` is the x/y coordinate of the top-left corner, and
///  - `dimensions` is a tuple containing the width and height of the region.
///
/// Points should be represented by regions with dimensions `(1, 1)`.
// TODO(ambuc): Implement `.delete_by(anchor, dimensions, fn)`: `.retain()` is the inverse.
// TODO(ambuc): Implement `FromIterator<(K, V)>` for `Quadtree`.
#[derive(Debug, PartialEq, Eq)]
pub struct Quadtree<U, V>
where
    U: PrimInt,
{
    depth: usize,
    inner: QTInner<U>,
    store: StoreType<U, V>,
}

impl<U, V> Quadtree<U, V>
where
    U: PrimInt,
{
    /// Creates a new, empty Quadtree with the requested depth.
    /// - The default anchor is `(0, 0)`, and the default width and height are both `2^depth`.
    /// - The Quadtree must be explicitly typed, since will contain items of a type.
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let qt = Quadtree::<u32, u8>::new(/*depth=*/ 2);
    ///
    /// assert_eq!(qt.depth(), 2);
    /// assert_eq!(qt.anchor(), (0, 0));
    /// assert_eq!(qt.width(), 4);
    /// assert_eq!(qt.height(), 4);
    /// ```
    pub fn new(depth: usize) -> Quadtree<U, V> {
        Quadtree::new_with_anchor((U::zero(), U::zero()), depth)
    }

    /// Creates a new Quadtree with the requested anchor and depth.
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let qt = Quadtree::<u32, u8>::new_with_anchor(/*anchor=*/ (2, 4), /*depth=*/ 3);
    ///
    /// assert_eq!(qt.depth(), 3);
    /// assert_eq!(qt.anchor(), (2, 4));
    /// assert_eq!(qt.width(), 8);
    /// assert_eq!(qt.height(), 8);
    /// ```
    pub fn new_with_anchor(anchor: PointType<U>, depth: usize) -> Quadtree<U, V> {
        Quadtree {
            depth,
            inner: QTInner::new(anchor, depth),
            store: HashMap::new(),
        }
    }

    /// The coordinate of the top-left corner of the represented region.
    pub fn anchor(&self) -> PointType<U> {
        self.inner.region.anchor().into()
    }

    /// The width of the represented region.
    pub fn width(&self) -> usize {
        self.inner.region.width().to_usize().unwrap()
    }

    /// The height of the represented region.
    pub fn height(&self) -> usize {
        self.inner.region.height().to_usize().unwrap()
    }

    /// The depth of the quadtree.
    /// - A quadtree created with depth 0 will have one node and no possibility for subdivision;
    /// - a quadtree created with depth 1 will have one node and four
    /// potential subquadrants.
    ///
    /// Thus both the width and height of a quadtree with depth `n` are `2^n`.
    pub fn depth(&self) -> usize {
        self.inner.depth
    }

    /// Returns the number of elements in the quadtree.
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    /// assert_eq!(qt.len(), 0);
    ///
    /// qt.insert((3, 1), (1, 1), 3.14159);
    /// assert_eq!(qt.len(), 1);
    ///
    /// qt.insert((2, 7), (1, 1), 2.71828);
    /// assert_eq!(qt.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Whether or not the quadtree is empty.
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let mut qt = Quadtree::<u32, f64>::new(3);
    /// assert!(qt.is_empty());
    ///
    /// qt.insert((1, 4), (1, 4), 1.4142135);
    /// assert!(!qt.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Whether or not the region represented by this quadtree could contain the given region.
    ///
    /// The region described may have an anchor anywhere on the plane, but it
    /// must have positive, nonzero values for its width and height.
    ///
    /// Perhaps before inserting a region, the callsite would like to check to see if that region
    /// could fit in the area represented by the quadtree.
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let qt = Quadtree::<u32, u32>::new_with_anchor((1, 0), 1);
    /// // This is a very small quadtree. It has an anchor at (1, 0) and dimensions 2x2.
    /// assert_eq!(qt.anchor(), (1,0));
    /// assert_eq!(qt.width(), 2);
    /// assert_eq!(qt.height(), 2);
    ///
    /// //  012
    /// // 0 ▓░ // The quadtree contains a region which is totally within it.
    /// // 1 ░░
    /// assert!(qt.contains((1, 0), (1, 1)));
    ///
    /// //  012
    /// // 0▓░░ // ...and the quadtree does not contains a region which is not totally within it.
    /// // 1 ░░
    /// assert!(!qt.contains((0, 0), (1, 1)));
    /// ```
    pub fn contains(&self, anchor: PointType<U>, dimensions: (U, U)) -> bool {
        self.inner.region.contains((anchor, dimensions).into())
    }

    /// Inserts the value at the requested region. Returns a unique `handle` representing this
    /// instance of the object in the Quadtree.
    ///   - If the requested region does not fit totally in the Quadtree, `.insert()` will fail
    ///     silently. Callsites may want to use `.contains()` first.
    ///   - If the requested region only fits partially in the Quadtree, `.insert()` will mark the
    ///     in-bounds regions and drop the rest of the requested region.
    ///
    /// The region described may have an anchor anywhere on the plane, but it
    /// must have positive, nonzero values for its width and height.
    ///
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let mut qt = Quadtree::<u32, String>::new(2);
    ///
    /// let handle_a_1 = qt.insert((0, 0), (1, 1), "a".to_string());
    /// let handle_a_2 = qt.insert((0, 0), (1, 1), "a".to_string());
    ///
    /// // Even though we inserted "a" at the same point in the Quadtree, the two handles returned
    /// // were not the same.
    /// assert_ne!(handle_a_1, handle_a_2);
    /// ```
    pub fn insert(&mut self, anchor: PointType<U>, dimensions: (U, U), val: V) -> u64 {
        self.inner
            .insert_val_at_region((anchor, dimensions).into(), val, &mut self.store)
    }

    /// Provides access to a single value in the Quadtree, given a previously known handle. This
    /// handle might have been saved by value at [`insert`].
    ///
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let handle: u64 = qt.insert((0, 1), (2, 3), 9.87);
    ///
    /// assert_eq!(qt.get(handle), Some(&9.87));
    ///
    /// ```
    ///
    /// [`insert`]: struct.Quadtree.html#method.insert
    pub fn get<'a>(&'a self, handle: u64) -> Option<&'a V> {
        self.store
            .get(&handle)
            .map_or(None, |entry| Some(entry.value_ref()))
    }

    /// A mutable variant of `.get()`.
    ///
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let handle: u64 = qt.insert((0, 1), (2, 3), 9.87);
    ///
    /// if let Some(val) = qt.get_mut(handle) {
    ///   *val += 1.0;
    /// }
    ///
    /// assert_eq!(qt.get(handle), Some(&10.87));
    ///
    /// ```
    ///
    /// [`.get()`]: struct.Quadtree.html#method.get
    pub fn get_mut<'a>(&'a mut self, handle: u64) -> Option<&'a mut V> {
        self.store
            .get_mut(&handle)
            .map_or(None, |entry| Some(entry.value_mut()))
    }

    /// Returns an iterator over [`&Entry<U, V>`] structs representing values
    /// within the query region.
    ///
    /// The default behavior of `.query()` is to return any intersecting regions or points, but
    /// the callsite could use [`.query_strict()`] instead.
    ///
    /// The query region described may have an anchor anywhere on the plane, but it
    /// must have positive, nonzero values for its width and height.
    ///
    /// ```
    /// use quadtree_rs::Quadtree;
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▒▒▒░░
    /// // 2 ░░▒▒▒░░
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░
    /// // 5 ░░░░░░░
    /// let mut qt = Quadtree::<u32, i16>::new(4);
    /// qt.insert((2, 1), (3, 2), 21);
    /// qt.insert((1, 4), (3, 1), 57);
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▓▒▒░░  <-- query
    /// // 2 ░░▒▒▒░░
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░
    /// // 5 ░░░░░░░
    /// // Query over the region anchored at (2, 1) with area 1x1.
    /// let mut query_a = qt.query((2, 1), (1, 1));
    ///
    /// // We can use the Entry API to destructure the result.
    /// let entry = query_a.next().unwrap();
    /// assert_eq!(entry.region(), ((2, 1), (3, 2)));
    /// assert_eq!(entry.value_ref(), &21);
    ///
    /// assert_eq!(query_a.next(), None);
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░▒▓▓▓▒░  <-- query
    /// // 2 ░▒▓▓▓▒░  <--
    /// // 3 ░▒▒▒▒▒░  <--
    /// // 4 ░▓▓▓▒▒░  <--
    /// // 5 ░░░░░░░
    /// // Query over the region anchored at (0, 0) with area 6x6.
    /// let query_b = qt.query((1, 1), (4, 4));
    ///
    /// // It's unclear what order the regions should return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    ///
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.query_strict()`]: struct.Quadtree.html#method.query_strict
    pub fn query(&self, anchor: PointType<U>, dimensions: (U, U)) -> Query<U, V> {
        Query::new(
            (anchor, dimensions).into(),
            &self.inner,
            &self.store,
            Traversal::Overlapping,
        )
    }

    ///  `query_strict()` behaves the same as `query()`, except that the regions returned are
    ///  guaranteed to be totally contained within the query region. (In the example above, the
    ///  first query would have been empty, since it only intersected the region in question.)
    pub fn query_strict(&self, anchor: PointType<U>, dimensions: (U, U)) -> Query<U, V> {
        Query::new(
            (anchor, dimensions).into(),
            &self.inner,
            &self.store,
            Traversal::Strict,
        )
    }

    /// Accepts a modification lambda and applies it to all elements in
    /// the Quadtree which intersecting the described region.
    ///
    /// ```
    /// use quadtree_rs::{Quadtree, entry::Entry};
    ///
    /// let mut qt = Quadtree::<u8, f64>::new(3);
    ///
    /// qt.insert((0, 0), (1, 1), 1.23);
    /// qt.modify_all(|i| *i += 2.0);
    ///
    /// let e: &Entry<u8, f64> = qt.iter().next().unwrap();
    /// assert_eq!(e.region(), ((0, 0), (1, 1)));
    /// assert_eq!(e.value_ref(), &3.23);
    /// ```
    pub fn modify<F>(&mut self, anchor: PointType<U>, dimensions: (U, U), f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        let query_region = (anchor, dimensions).into();
        self.modify_region(|a| a.intersects(query_region), f);
    }

    ///  `modify_strict()` behaves the same as `modify()`, except that the regions modified are
    ///  guaranteed to be totally contained within the query region.
    pub fn modify_strict<F>(&mut self, anchor: PointType<U>, dimensions: (U, U), f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        let query_region: Area<U> = (anchor, dimensions).into();
        self.modify_region(|a| query_region.contains(a), f);
    }

    /// Alias for [`.modify(self.anchor(), (self.width(), self.height()))`].
    ///
    /// [`.modify(self.anchor(), (self.width(), self.height()))`]: struct.Quadtree.html#method.modify
    pub fn modify_all<F>(&mut self, f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        self.modify_region(|_| true, f);
    }

    fn modify_region<F, M>(&mut self, filter: F, modify: M)
    where
        F: Fn(Area<U>) -> bool,
        M: Fn(&mut V) + Copy,
    {
        let relevant_handles: Vec<u64> = HandleIter::new(&self.inner).collect();
        for i in relevant_handles {
            if let Some(entry) = self.store.get_mut(&i) {
                if filter(entry.area()) {
                    modify(&mut entry.value_mut());
                }
            }
        }
    }

    /// Resets the quadtree to a totally empty state.
    pub fn reset(&mut self) {
        self.store.clear();
        self.inner.reset();
    }

    /// Deletes a described region in the tree, consuming along the way and returning an iterator
    /// ([`IntoIter<U, V>`]) over type [`Entry<U, V>`].
    ///
    /// The default behavior of `.delete()` is to delete and return any intersecting regions or
    /// points, but the callsite could use [`.delete_strict()`] instead.
    ///
    /// ```
    /// use quadtree_rs::{IntoIter, Quadtree, entry::Entry};
    ///
    /// let mut qt = Quadtree::<u32, f64>::new(4);
    ///
    /// qt.insert((0, 0), (2, 2), 1.23);
    /// qt.insert((1, 1), (3, 2), 4.56);
    /// //   0123
    /// // 0 ░░
    /// // 1 ░▓╳░  <-- ╳ is the deletion region
    /// // 2  ░░░
    ///
    /// let mut returned_entries: IntoIter<u32, f64> = qt.delete((2, 1), (1, 1));
    /// // We've removed one object from the Quadtree.
    /// assert_eq!(qt.len(), 1);
    ///
    /// // qt.delete() returns a struct of type IntoIter<u32, f64>.
    /// let hit: Entry<u32, f64> = returned_entries.next().unwrap();
    ///
    /// // IntoIter is an iterator over type Entry<u32, f64>, which makes accessible the returned
    /// // region and value.
    /// assert_eq!(hit.value_ref(), &4.56);
    /// assert_eq!(hit.region(), ((1, 1), (3, 2)));
    ///
    /// ```
    ///
    /// [`IntoIter<U, V>`]: struct.IntoIter.html
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.delete_strict()`]: struct.Quadtree.html#method.delete_strict
    pub fn delete(&mut self, anchor: PointType<U>, dimensions: (U, U)) -> IntoIter<U, V> {
        self.delete_handles_and_return(self.query(anchor, dimensions).map(|e| e.handle()).collect())
    }

    ///  `delete_strict()` behaves the same as `delete()`, except that the regions deleted and
    ///  returned are guaranteed to be totally contained within the delete region.
    pub fn delete_strict(&mut self, anchor: PointType<U>, dimensions: (U, U)) -> IntoIter<U, V> {
        self.delete_handles_and_return(
            self.query_strict(anchor, dimensions)
                .map(|e| e.handle())
                .collect(),
        )
    }

    fn delete_handles_and_return(&mut self, handles: HashSet<u64>) -> IntoIter<U, V> {
        let error: &'static str = "I tried to look up an handle in the store which I found in the tree, but it wasn't there!";

        let mut entries: Vec<Entry<U, V>> = vec![];

        handles.iter().for_each(|u| {
            entries.push(self.store.remove(u).expect(&error));
        });

        IntoIter { entries }
    }

    /// Given an handle, deletes a single item from the Quadtree. If that handle was found,
    /// `delete_by_handle()` returns an `Entry<U, V>` containing its former region and value. Otherwise,
    /// returns `None`.
    pub fn delete_by_handle(&mut self, handle: u64) -> Option<Entry<U, V>> {
        // Pop the Entry<U, V> out of the @store,
        if let Some(entry) = self.store.remove(&handle) {
            // Use the now-known region to descend into the tree efficiently,
            self.inner.delete_by_handle(handle, entry.area());
            // And return the Entry.
            return Some(entry);
        }
        // If the handle wasn't in the @store, we don't need to perform a descent.
        None
    }

    // TODO(ambuc): Test this fn.
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all items such that `f(&mut v)` returns `false`.
    pub fn retain<F>(&mut self, mut f: F) -> IntoIter<U, V>
    where
        F: FnMut(&mut V) -> bool,
        U: std::hash::Hash,
    {
        // TODO(ambuc): I think this is technically correct but it seems to be interweaving three
        // routines. Is there a way to simplify this?
        let mut doomed: HashSet<(u64, Area<U>)> = HashSet::new();
        for (handle, entry) in self.store.iter_mut() {
            if f(entry.value_mut()) {
                doomed.insert((*handle, entry.area()));
            }
        }
        // TODO(ambuc): There is an optimization here to do one traversal with many matches, over
        // many traversals i.e. one per match.
        let mut entries: Vec<Entry<U, V>> = vec![];
        for (handle, region) in doomed {
            entries.push(self.store.remove(&handle).unwrap());
            self.inner.delete_by_handle(handle, region);
        }

        IntoIter { entries }
    }
    // TODO(ambuc): retain_within

    /// Returns an iterator over all `(&((U, U), (U, U)), &V)` region/value associations in the
    /// Quadtree.
    pub fn iter(&self) -> Iter<U, V> {
        Iter::new(&self.inner, &self.store)
    }

    /// Returns an iterator over all `&'a ((U, U), (U, U))` regions in the Quadtree.
    pub fn regions(&self) -> Regions<U, V> {
        Regions {
            inner: Iter::new(&self.inner, &self.store),
        }
    }

    /// Returns an iterator over all `&'a V` values in the Quadtree.
    pub fn values(&self) -> Values<U, V> {
        Values {
            inner: Iter::new(&self.inner, &self.store),
        }
    }
}

// d888888b d888888b d88888b d8888b.
//   `88'   `~~88~~' 88'     88  `8D
//    88       88    88ooooo 88oobY'
//    88       88    88~~~~~ 88`8b
//   .88.      88    88.     88 `88.
// Y888888P    YP    Y88888P 88   YD

/// An iterator over all regions and values of a [`Quadtree`].
///
/// This struct is created by the [`iter`] method on [`Quadtree`].
///
/// [`iter`]: struct.Quadtree.html#method.iter
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Iter<'a, U, V>
where
    U: PrimInt,
{
    store: &'a StoreType<U, V>,
    handle_iter: HandleIter<'a, U>,
}

impl<'a, U, V> Iter<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(qt: &'a QTInner<U>, store: &'a StoreType<U, V>) -> Iter<'a, U, V> {
        Iter {
            store,
            handle_iter: HandleIter::new(qt),
        }
    }
}

impl<'a, U, V> Iterator for Iter<'a, U, V>
where
    U: PrimInt,
{
    type Item = &'a Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.handle_iter.next() {
            Some(handle) => {
                return Some(
                    self.store
                        .get(&handle)
                        .expect("Shouldn't have an handle in the tree which isn't in the store."),
                );
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Iter<'_, U, V> where U: PrimInt {}

//  .d88b.  db    db d88888b d8888b. db    db
// .8P  Y8. 88    88 88'     88  `8D `8b  d8'
// 88    88 88    88 88ooooo 88oobY'  `8bd8'
// 88    88 88    88 88~~~~~ 88`8b      88
// `8P  d8' 88b  d88 88.     88 `88.    88
//  `Y88'Y8 ~Y8888P' Y88888P 88   YD    YP

/// An iterator over the regions and values of a [`Quadtree`].
///
/// This struct is created by the [`query`] method on [`Quadtree`].
///
/// [`query`]: struct.Quadtree.html#method.query
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Query<'a, U, V>
where
    U: PrimInt,
{
    query_region: Area<U>,
    handle_iter: HandleIter<'a, U>,
    store: &'a StoreType<U, V>,
    traversal_method: Traversal,
}

impl<'a, U, V> Query<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(
        query_region: Area<U>,
        qt: &'a QTInner<U>,
        store: &'a StoreType<U, V>,
        traversal_method: Traversal,
    ) -> Query<'a, U, V>
    where
        U: PrimInt,
    {
        // Construct the HandleIter first...
        let mut handle_iter = HandleIter::new(qt);

        // ...and descend it to the appropriate level. Depending on the type of @traversal_method,
        // this will potentially collect intersecting regions along the way. Avoiding combing the
        // entire Quadtree is essential for the efficiency of a query.
        handle_iter.query_optimization(query_region, traversal_method);

        Query {
            query_region,
            handle_iter,
            store,
            traversal_method,
        }
    }
}

impl<'a, U, V> Iterator for Query<'a, U, V>
where
    U: PrimInt,
{
    type Item = &'a Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(handle) = self.handle_iter.next() {
            if let Some(entry) = self.store.get(&handle) {
                if self.traversal_method.eval(entry.area(), self.query_region) {
                    return Some(entry);
                }
            }
            return self.next();
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Query<'_, U, V> where U: PrimInt {}

// d8888b. d88888b  d888b  d888888b  .d88b.  d8b   db .d8888.
// 88  `8D 88'     88' Y8b   `88'   .8P  Y8. 888o  88 88'  YP
// 88oobY' 88ooooo 88         88    88    88 88V8o 88 `8bo.
// 88`8b   88~~~~~ 88  ooo    88    88    88 88 V8o88   `Y8b.
// 88 `88. 88.     88. ~8~   .88.   `8b  d8' 88  V888 db   8D
// 88   YD Y88888P  Y888P  Y888888P  `Y88P'  VP   V8P `8888Y'

/// An iterator over the regions held within a [`Quadtree`].
///
/// This struct is created by the [`regions`] method on [`Quadtree`].
///
/// [`regions`]: struct.Quadtree.html#method.regions
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Regions<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Regions<'a, U, V>
where
    U: PrimInt,
{
    type Item = AreaType<U>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |entry| Some(entry.region()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Regions<'_, U, V> where U: PrimInt {}

// db    db  .d8b.  db      db    db d88888b .d8888.
// 88    88 d8' `8b 88      88    88 88'     88'  YP
// Y8    8P 88ooo88 88      88    88 88ooooo `8bo.
// `8b  d8' 88~~~88 88      88    88 88~~~~~   `Y8b.
//  `8bd8'  88   88 88booo. 88b  d88 88.     db   8D
//    YP    YP   YP Y88888P ~Y8888P' Y88888P `8888Y'

/// An iterator over the values held within a [`Quadtree`].
///
/// This struct is created by the [`values`] method on [`Quadtree`].
///
/// [`values`]: struct.Quadtree.html#method.values
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Values<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Values<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map_or(None, |entry| Some(entry.value_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Values<'_, U, V> where U: PrimInt {}

// d888888b d8b   db d888888b  .d88b.  d888888b d888888b d88888b d8888b.
//   `88'   888o  88 `~~88~~' .8P  Y8.   `88'   `~~88~~' 88'     88  `8D
//    88    88V8o 88    88    88    88    88       88    88ooooo 88oobY'
//    88    88 V8o88    88    88    88    88       88    88~~~~~ 88`8b
//   .88.   88  V888    88    `8b  d8'   .88.      88    88.     88 `88.
// Y888888P VP   V8P    YP     `Y88P'  Y888888P    YP    Y88888P 88   YD

/// A consuming iterator over all region/value associations held in a [`Quadtree`].
///
/// This struct is created by the `into_iter()` method on the [`IntoIterator`] trait.
///
/// [`IntoIterator`]: struct.Quadtree.html#impl-IntoIterator
///
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Debug)]
pub struct IntoIter<U, V>
where
    U: PrimInt,
{
    entries: Vec<Entry<U, V>>,
}

impl<U, V> Iterator for IntoIter<U, V>
where
    U: PrimInt,
{
    type Item = Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for IntoIter<U, V> where U: PrimInt {}

/// `Extend<(((U, U), (U, U), V)>` will silently drop values whose coordinates do not fit in the
/// region represented by the Quadtree. It is the responsibility of the callsite to ensure these
/// points fit.
impl<U, V> Extend<(AreaType<U>, V)> for Quadtree<U, V>
where
    U: PrimInt,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (AreaType<U>, V)>,
    {
        for ((anchor, dimensions), val) in iter {
            self.insert(anchor, dimensions, val);
        }
    }
}

/// `Extend<((U, U), V)>` will silently drop values whose coordinates do not fit in the region
/// represented by the Quadtree. It is the responsibility of the callsite to ensure these points
/// fit.
impl<U, V> Extend<(PointType<U>, V)> for Quadtree<U, V>
where
    U: PrimInt,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (PointType<U>, V)>,
    {
        for (pt, val) in iter {
            self.insert(pt, (U::one(), U::one()), val);
        }
    }
}

// Immutable iterator for the Quadtree, returning by-reference.
impl<'a, U, V> IntoIterator for &'a Quadtree<U, V>
where
    U: PrimInt,
{
    type Item = &'a Entry<U, V>;
    type IntoIter = Iter<'a, U, V>;

    fn into_iter(self) -> Iter<'a, U, V> {
        Iter::new(&self.inner, &self.store)
    }
}

impl<U, V> IntoIterator for Quadtree<U, V>
where
    U: PrimInt,
{
    type Item = Entry<U, V>;
    type IntoIter = IntoIter<U, V>;

    fn into_iter(self) -> IntoIter<U, V> {
        IntoIter {
            entries: self
                .store
                .into_iter()
                .map(|(_handle, entry)| entry)
                .collect(),
        }
    }
}
