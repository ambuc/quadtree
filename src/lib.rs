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
//! use quadtree_rs::{area::AreaBuilder,
//!                   point::Point,
//!                   Quadtree};
//!
//! // A new Quadtree with u64 coordinates and String values.
//! // ::new(4) means a depth of 4 layers, i.e. sides of length 2^4.
//! let mut qt = Quadtree::<u64, String>::new(4);
//!
//! //   0  1  2
//! // 0 ░░░░░░░    Insert the string "foo"
//! //   ░░foo░░    at a region in the tree.
//! // 1 ░░░░░░░
//! assert!(
//!   qt.insert(
//!     /*region=*/
//!     AreaBuilder::default().anchor(Point {x: 0, y: 0})
//!                           .dimensions((2, 1))
//!                           .build().unwrap(),
//!     /*val=*/
//!     "foo".to_string())
//!   .is_ok());
//!
//! //
//! //   0  1  2  3
//! // 0 ░░░▓▓▓▓▒▒▒     
//! //   ░░░▓▓▓▓▒▒▒ <-- Query over a region
//! // 1 ░░░▓▓▓▓▒▒▒     which overlaps foo.
//! //   |  ▒▒▒▒▒▒▒
//! // 2 +--▒▒▒▒▒▒▒
//! let mut query = qt.query(
//!     /*region=*/
//!     AreaBuilder::default().anchor(Point {x: 1, y: 0})
//!                           .dimensions((2, 2))
//!                           .build().unwrap());
//!
//! // Take the first entry in the query results and check its' value.
//! assert_eq!(query.next().unwrap().value_ref(), "foo");
//! ```
//!
//! # Implementation
//! ```
//! use quadtree_rs::{area::AreaBuilder,
//!                   point::Point,
//!                   Quadtree};
//!
//! let mut qt = Quadtree::<u8, f32>::new(2);
//!
//! // In a quadtree, every region is subdivided lazily into four subqudrants.
//!
//! // Inserting a point, represented as a region with width and height one,
//! // means traversing the full height of the tree.
//! assert!(
//!   qt.insert(
//!     /*region=*/
//!     AreaBuilder::default().anchor(Point {x: 0, y: 0})
//!                           .build().unwrap(),
//!     /*val=*/
//!     1.23456)
//!   .is_ok());
//!
//! // (0,0)->4x4 ─── (0,0)->2x2 ─── (0,0)->1x1
//! //                               [1.23456]
//!
//! // Inserting a region means traversing only as far down
//! // the tree as necessary to fully cover that region.
//! assert!(
//!   qt.insert(
//!     /*region=*/
//!     AreaBuilder::default().anchor(Point {x: 0, y: 0})
//!                                 .dimensions((2, 2))
//!                                 .build().unwrap(),
//!     /*val=*/
//!     2.46810)
//!   .is_ok());
//!
//! // (0,0)->4x4 ─── (0,0)->2x2 ─── (0,0)->1x1
//! //                [2.46810]      [1.23456]
//!
//! // Often that means inserting the value in multiple places.
//! // (The implementation duplicates not the stored value,
//! // which need not implement Copy, but a handle to the
//! // value in a storage map.)
//!
//! assert!(
//!   qt.insert(
//!     AreaBuilder::default().anchor(Point {x: 0, y: 0})
//!                           .dimensions((3, 3))
//!                           .build().unwrap(),
//!     3.6912)
//!   .is_ok());
//!
//! // (0,0)->4x4 ─┬─ (0,0)->2x2 ─── (0,0)->1x1
//! //             │  [ 2.46810,      [1.23456]
//! //             │    3.6912 ]  
//! //             │
//! //             ├─ (0,2)->2x2 ─┬─ (0,2)->1x1
//! //             │              │  [3.6912]
//! //             │              │
//! //             │              └─ (1,2)->1x1
//! //             │                 [3.6912]
//! //             │
//! //             ├─ (2,0)->2x2 ─┬─ (2,0)->1x1
//! //             │              │  [3.6912]
//! //             │              │
//! //             │              └─ (2,1)->1x1
//! //             │                 [3.6912]
//! //             │
//! //             └─ (2,2)->2x2 ─── (2,2)->1x1
//! //                                [3.6912]
//! ```
//! Duplicating the storage handle is expensive, but allows for fast lookups and fast insertions at
//! the cost of slower deletions. This means that `quadtree_rs` is well-suited for maps which hold
//! many items with small regions.
//!
//! # Usage
//!
//! For further usage details, see [`Quadtree`].
//!
//! [`Quadtree`]: struct.Quadtree.html

// For extra-pedantic documentation tests.
#![doc(test(attr(deny(warnings))))]

#[macro_use]
extern crate derive_builder;
extern crate num;

pub mod area;
pub mod entry;
pub mod point;

mod handle_iter;
mod qtinner;
mod traversal;
mod types;

use {
    crate::{
        area::{Area, AreaBuilder},
        entry::Entry,
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
/// Points and areas are respresented by the [`point`] and [`area`] structs. Both are parameterized
/// over `U`.
///
/// [`point`]: point/struct.Point.html
/// [`area`]: area/struct.Area.html
// TODO(ambuc): Implement `.delete_by(anchor, dimensions, fn)`: `.retain()` is the inverse.
// TODO(ambuc): Implement `FromIterator<(K, V)>` for `Quadtree`.
#[derive(Debug, PartialEq, Eq)]
pub struct Quadtree<U, V>
where
    U: PrimInt + std::default::Default,
{
    depth: usize,
    inner: QTInner<U>,
    store: StoreType<U, V>,
}

impl<U, V> Quadtree<U, V>
where
    U: PrimInt + std::default::Default,
{
    // pub

    /// Creates a new, empty Quadtree with the requested depth.
    ///
    /// *NB:* A quadtree with depth `n` can only handle regions with coordinates between `0` and
    /// `2^n`.
    /// ```
    /// use quadtree_rs::{point::Point, Quadtree};
    ///
    /// let qt = Quadtree::<u32, u8>::new(/*depth=*/ 2);
    ///
    /// assert_eq!(qt.anchor(), Point {x: 0, y: 0});
    /// assert_eq!(qt.depth(), 2);
    /// assert_eq!(qt.width(), 4);
    /// assert_eq!(qt.height(), 4);
    /// ```
    pub fn new(depth: usize) -> Self {
        Self::new_with_anchor(
            point::Point {
                x: U::zero(),
                y: U::zero(),
            },
            depth,
        )
    }

    /// Creates a new Quadtree with the requested anchor and depth.
    /// ```
    /// use quadtree_rs::{point::Point, Quadtree};
    ///
    /// let qt = Quadtree::<u32, u8>::new_with_anchor(
    ///     /*anchor=*/ Point {x: 2, y: 4},
    ///     /*depth=*/ 3);
    /// // NB: Points can also be coerced, i.e. `(2,4).into()`.
    ///
    /// assert_eq!(qt.depth(), 3);
    /// assert_eq!(qt.anchor(), Point {x: 2, y: 4});
    /// assert_eq!(qt.width(), 8);
    /// assert_eq!(qt.height(), 8);
    /// ```
    pub fn new_with_anchor(anchor: point::Point<U>, depth: usize) -> Self {
        Self {
            depth,
            inner: QTInner::new(anchor, depth),
            store: HashMap::new(),
        }
    }

    /// The top-left corner of the region covered by the quadtree.
    pub fn anchor(&self) -> point::Point<U> {
        self.inner.region.anchor()
    }

    /// The width of the region covered by the quadtree.
    pub fn width(&self) -> usize {
        self.inner.region.width().to_usize().unwrap()
    }

    /// The height of the region covered by the quadtree.
    pub fn height(&self) -> usize {
        self.inner.region.height().to_usize().unwrap()
    }

    /// The depth of the quadtree.
    pub fn depth(&self) -> usize {
        self.inner.depth
    }

    /// The number of elements in the quadtree.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Whether or not the quadtree is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Test whether some region could fit in the region
    /// covered by the quadtree.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// // This is a very small quadtree.
    /// let qt = Quadtree::<u32, u32>::new_with_anchor(
    ///   /*anchor=*/Point {x: 1, y: 0},
    ///   /*depth=*/1);
    ///
    /// // It is anchored at (1, 0) and has height and width 2.
    /// assert_eq!(qt.anchor(), Point{ x: 1, y: 0});
    /// assert_eq!(qt.width(), 2);
    /// assert_eq!(qt.height(), 2);
    ///
    /// //  012    This quadtree _does_ contain
    /// // 0.▓░    a point at (1,0).
    /// // 1.░░
    /// assert!(qt.contains(
    ///     AreaBuilder::default().anchor(Point {x: 1, y: 0})
    ///                           .build().unwrap()));
    ///
    /// //  012     This quadtree does _not_ contain
    /// // 0▓░░     a point at (0,0).
    /// // 1 ░░
    /// assert!(!qt.contains(
    ///     AreaBuilder::default().anchor(Point {x: 0, y: 0})
    ///                           .build().unwrap()));
    /// ```
    pub fn contains(&self, area: Area<U>) -> bool {
        self.inner.region.contains(area)
    }

    /// Associates the passed value with the passed region.
    ///
    /// If the quadtree can contain the region, the value
    /// is inserted and a unique handle representing this
    /// instance of the object is returned.
    ///
    /// If the region cannot be wholly contained,
    /// `::insert()` will return an `Err`.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder,
    ///                   point::Point,
    ///                   Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, i8>::new(2);
    ///
    /// let some_area = AreaBuilder::default().anchor(Point {x: 0, y: 0})
    ///                                       .build().unwrap();
    /// let some_value = 5_i8;
    ///
    /// let handle_a_1 = qt.insert(some_area, some_value).unwrap();
    /// let handle_a_2 = qt.insert(some_area, some_value).unwrap();
    ///
    /// // Even though we inserted 5 at the same point in
    /// // the Quadtree, the two handles returned were not
    /// // the same.
    /// assert_ne!(handle_a_1, handle_a_2);
    /// ```
    pub fn insert(&mut self, region: Area<U>, val: V) -> Result<u64, &'static str> {
        if self.contains(region) {
            return Ok(self
                .inner
                .insert_val_at_region(region, val, &mut self.store));
        }
        Err("The requested region does not fit in this quadtree.")
    }

    /// Provides access to a single value in the Quadtree,
    /// given a previously known handle. This
    /// handle might have been saved by value at [`insert`].
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder,
    ///                   point::Point,
    ///                   Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let handle: u64 = qt.insert(
    ///     /*region=*/
    ///     AreaBuilder::default().anchor(Point {x: 0, y: 1})
    ///                           .dimensions((2, 3))
    ///                           .build().unwrap(),
    ///     /*value=*/
    ///     9.87).expect("This insert succeeds.");
    ///
    /// assert_eq!(qt.get(handle), Some(&9.87));
    ///
    /// ```
    ///
    /// [`insert`]: struct.Quadtree.html#method.insert
    pub fn get(&self, handle: u64) -> Option<&V> {
        self.store.get(&handle).and_then(|e| Some(e.value_ref()))
    }

    /// A mutable variant of `.get()`.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder,
    ///                   point::Point,
    ///                   Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let handle: u64 = qt.insert(
    ///     AreaBuilder::default().anchor(Point {x: 0, y: 1})
    ///                           .dimensions((2, 3))
    ///                           .build().unwrap(),
    ///     9.87).unwrap();
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
    pub fn get_mut(&mut self, handle: u64) -> Option<&mut V> {
        self.store
            .get_mut(&handle)
            .and_then(|e| Some(e.value_mut()))
    }

    /// Returns an iterator over [`&Entry<U, V>`] structs
    /// representing values within the query region.
    ///
    /// The default behavior of `.query()` is to return any
    /// intersecting regions or points, but
    /// the callsite could use [`.query_strict()`] instead.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder,
    ///                   point::Point,
    ///                   Quadtree};
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▒▒▒░░    (2,1)->3x2
    /// // 2 ░░▒▒▒░░
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░    (1,4)->3x1
    /// // 5 ░░░░░░░
    /// let mut qt = Quadtree::<u32, i16>::new(4);
    /// assert!(
    ///   qt.insert(AreaBuilder::default().anchor(Point {x: 2, y: 1})
    ///                                   .dimensions((3, 2))
    ///                                   .build().unwrap(),
    ///             21)
    ///   .is_ok());
    /// assert!(
    ///   qt.insert(AreaBuilder::default().anchor(Point {x: 1, y: 4})
    ///                                   .dimensions((3, 1))
    ///                                   .build().unwrap(),
    ///             57)
    ///   .is_ok());
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▓▒▒░░  <-- Query over the region
    /// // 2 ░░▒▒▒░░      (2,1)->1x1
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░
    /// // 5 ░░░░░░░
    /// let mut query_a = qt.query(
    ///     AreaBuilder::default().anchor(Point {x: 2, y: 1})
    ///                           .build().unwrap());
    ///
    /// // We can use the Entry API to destructure the result.
    /// let entry = query_a.next().unwrap();
    /// assert_eq!(entry.area().height(), 2);
    /// assert_eq!(entry.value_ref(), &21);
    ///
    /// // But that was the only result.
    /// assert!(query_a.next().is_none());
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░▒▓▓▓▒░  <-- query over the region
    /// // 2 ░▒▓▓▓▒░      (0,0)->6x6.
    /// // 3 ░▒▒▒▒▒░
    /// // 4 ░▓▓▓▒▒░
    /// // 5 ░░░░░░░
    /// let query_b = qt.query(
    ///     AreaBuilder::default().anchor(Point {x: 1, y: 1})
    ///                           .dimensions((4, 4))
    ///                           .build().unwrap());
    ///
    /// // It's unspecified what order the regions should
    /// // return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    ///
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.query_strict()`]: struct.Quadtree.html#method.query_strict
    // TODO(ambuc): Settle on a stable return order to avoid breaking callers.
    pub fn query(&self, area: Area<U>) -> Query<U, V> {
        Query::new(area, &self.inner, &self.store, Traversal::Overlapping)
    }

    ///  `query_strict()` behaves the same as `query()`,
    ///  except that the regions returned are guaranteed
    ///  to be totally contained within the query region.
    ///  (In the example above, the first query would
    ///  have been empty, since it only intersected the
    ///  region in question.)
    pub fn query_strict(&self, area: Area<U>) -> Query<U, V> {
        Query::new(area, &self.inner, &self.store, Traversal::Strict)
    }

    /// Accepts a modification lambda and applies it to all
    /// elements in the Quadtree which intersecting the
    /// described region.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder,
    ///                   point::Point,
    ///                   Quadtree};
    ///
    /// let mut qt = Quadtree::<u8, f64>::new(3);
    ///
    /// // Insert 1.23 at (0,0)->1x1.
    /// let handle = qt.insert(
    ///     AreaBuilder::default().anchor(Point {x: 0, y: 0})
    ///                           .build().unwrap(),
    ///     1.23).unwrap();
    ///
    /// // Run a modification lambda over all points,
    /// qt.modify_all(|i| *i += 2.0);
    ///
    /// // ...and verify that the value was applied.
    /// assert_eq!(qt.get(handle), Some(&3.23));
    /// ```
    pub fn modify<F>(&mut self, area: Area<U>, f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        self.modify_region(|a| a.intersects(area), f);
    }

    ///  `modify_strict()` behaves the same as `modify()`,
    ///  except that the regions modified are guaranteed to
    ///  be totally contained within the query region.
    pub fn modify_strict<F>(&mut self, area: Area<U>, f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        self.modify_region(|a| area.contains(a), f);
    }

    /// Alias for [`.modify()`] which runs over the entire
    /// quadtree.
    ///
    /// [`.modify()`]: struct.Quadtree.html#method.modify
    pub fn modify_all<F>(&mut self, f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        for entry in self.store.values_mut() {
            f(&mut entry.value_mut());
        }
    }

    /// Resets the quadtree to a totally empty state.
    pub fn reset(&mut self) {
        self.store.clear();
        self.inner.reset();
    }

    /// Deletes a described region in the tree, consuming
    /// along the way and returning an iterator
    /// ([`IntoIter<U, V>`]) over type [`Entry<U, V>`].
    ///
    /// The default behavior of `.delete()` is to delete
    /// and return any intersecting regions or points, but
    /// the callsite could use [`.delete_strict()`] instead.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f64>::new(4);
    ///
    /// assert!(
    ///   qt.insert(AreaBuilder::default().anchor(Point {x: 0, y: 0})
    ///                                   .dimensions((2, 2))
    ///                                   .build().unwrap(),
    ///             1.23)
    ///   .is_ok());
    /// assert!(
    ///   qt.insert(AreaBuilder::default().anchor(Point {x: 1, y: 1})
    ///                                   .dimensions((3, 2))
    ///                                   .build().unwrap(),
    ///             4.56)
    ///   .is_ok());
    ///
    /// //   0123
    /// // 0 ░░
    /// // 1 ░▓╳░  <-- ╳ is the deletion region
    /// // 2  ░░░
    ///
    /// let mut returned_entries = qt.delete(
    ///     AreaBuilder::default().anchor(Point {x: 2, y: 1})
    ///                           .build().unwrap());
    ///
    /// // We've removed one object from the Quadtree.
    /// assert_eq!(returned_entries.next().unwrap().value_ref(),
    ///            &4.56);
    ///
    /// // And left one behind.
    /// assert_eq!(qt.len(), 1);
    /// ```
    ///
    /// [`IntoIter<U, V>`]: struct.IntoIter.html
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.delete_strict()`]: struct.Quadtree.html#method.delete_strict
    pub fn delete(&mut self, area: Area<U>) -> IntoIter<U, V> {
        self.delete_handles_and_return(self.query(area).map(|e| e.handle()).collect())
    }

    ///  `delete_strict()` behaves the same as `delete()`,
    ///  except that the regions deleted and returned are
    ///  guaranteed to be totally contained within the
    ///  delete region.
    pub fn delete_strict(&mut self, area: Area<U>) -> IntoIter<U, V> {
        self.delete_handles_and_return(self.query_strict(area).map(|e| e.handle()).collect())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn delete_handles_and_return(&mut self, handles: HashSet<u64>) -> IntoIter<U, V> {
        let error: &'static str = "I tried to look up an handle in the store which I found in the tree, but it wasn't there!";

        let mut entries: Vec<Entry<U, V>> = vec![];

        handles.iter().for_each(|u| {
            // We were just passed a hashset of handles taken from this quadtree, so it is safe to
            // assume they all still exist.
            entries.push(self.store.remove(u).expect(&error));
        });

        IntoIter { entries }
    }

    /// Given an handle, deletes a single item from the
    /// Quadtree. If that handle was found,
    /// `delete_by_handle()` returns an `Entry<U, V>`
    /// containing its former region and value. Otherwise,
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
        for (handle, entry) in &mut self.store {
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

    // fn

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
    U: PrimInt + std::default::Default,
{
    store: &'a StoreType<U, V>,
    handle_iter: HandleIter<'a, U>,
}

impl<'a, U, V> Iter<'a, U, V>
where
    U: PrimInt + std::default::Default,
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
    U: PrimInt + std::default::Default,
{
    type Item = &'a Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.handle_iter.next() {
            Some(handle) => Some(
                self.store
                    .get(&handle)
                    .expect("Shouldn't have an handle in the tree which isn't in the store."),
            ),
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.store.len()))
    }
}

impl<U, V> FusedIterator for Iter<'_, U, V> where U: PrimInt + std::default::Default {}

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
    U: PrimInt + std::default::Default,
{
    query_region: Area<U>,
    handle_iter: HandleIter<'a, U>,
    store: &'a StoreType<U, V>,
    traversal_method: Traversal,
}

impl<'a, U, V> Query<'a, U, V>
where
    U: PrimInt + std::default::Default,
{
    pub(crate) fn new(
        query_region: Area<U>,
        qt: &'a QTInner<U>,
        store: &'a StoreType<U, V>,
        traversal_method: Traversal,
    ) -> Query<'a, U, V>
    where
        U: PrimInt + std::default::Default,
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
    U: PrimInt + std::default::Default,
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
        (0, Some(self.store.len()))
    }
}

impl<U, V> FusedIterator for Query<'_, U, V> where U: PrimInt + std::default::Default {}

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
    U: PrimInt + std::default::Default,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Regions<'a, U, V>
where
    U: PrimInt + std::default::Default,
{
    type Item = Area<U>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|e| Some(e.area()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Regions<'_, U, V> where U: PrimInt + std::default::Default {}

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
    U: PrimInt + std::default::Default,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Values<'a, U, V>
where
    U: PrimInt + std::default::Default,
{
    type Item = (&'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|e| Some(e.value_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Values<'_, U, V> where U: PrimInt + std::default::Default {}

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
    U: PrimInt + std::default::Default,
{
    entries: Vec<Entry<U, V>>,
}

impl<U, V> Iterator for IntoIter<U, V>
where
    U: PrimInt + std::default::Default,
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

impl<U, V> FusedIterator for IntoIter<U, V> where U: PrimInt + std::default::Default {}

/// `Extend<((U, U), V)>` will silently drop values whose coordinates do not fit in the region
/// represented by the Quadtree. It is the responsibility of the callsite to ensure these points
/// fit.
impl<U, V> Extend<(point::Type<U>, V)> for Quadtree<U, V>
where
    U: PrimInt + std::default::Default,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (point::Type<U>, V)>,
    {
        for ((x, y), val) in iter {
            // Ignore errors.
            self.insert(
                AreaBuilder::default()
                    .anchor(point::Point { x, y })
                    .build()
                    .unwrap(),
                val,
            )
            .ok();
        }
    }
}

// Immutable iterator for the Quadtree, returning by-reference.
impl<'a, U, V> IntoIterator for &'a Quadtree<U, V>
where
    U: PrimInt + std::default::Default,
{
    type Item = &'a Entry<U, V>;
    type IntoIter = Iter<'a, U, V>;

    fn into_iter(self) -> Iter<'a, U, V> {
        Iter::new(&self.inner, &self.store)
    }
}

impl<U, V> IntoIterator for Quadtree<U, V>
where
    U: PrimInt + std::default::Default,
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
