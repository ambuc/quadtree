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
//! use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
//!
//! // Instantiate a new quadtree which associates String values with u64 coordinates.
//! let mut qt = Quadtree::<u64, String>::new(/*depth=*/4);
//!
//! // A depth of four means a square with width (and height) 2^4.
//! assert_eq!(qt.width(), 16);
//!
//! // Associate the value "foo" with a rectangle of size 2x1, anchored at (0, 0).
//! let region_a = AreaBuilder::default()
//!     .anchor(Point {x: 0, y: 0}).dimensions((2, 1))
//!     .build().unwrap();
//! qt.insert(region_a, "foo".to_string());
//!
//! // Query over a region of size 2x2, anchored at (1, 0).
//! let region_b = AreaBuilder::default()
//!     .anchor(Point {x: 1, y: 0}).dimensions((2, 2))
//!     .build().unwrap();
//! let mut query = qt.query(region_b);
//!
//! // The query region (region_b) intersects the region "foo" is associated with (region_a), so the query iterator returns "foo" by reference.
//! assert_eq!(query.next().unwrap().value_ref(), "foo");
//! ```
//!
//! # Implementation
//! ```
//! use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
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
//! let region_b = AreaBuilder::default()
//!     .anchor(Point {x: 0, y: 0}).dimensions((2, 2))
//!     .build().unwrap();
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
//! let region_c = AreaBuilder::default()
//!     .anchor(Point {x: 0, y: 0}).dimensions((3, 3))
//!     .build().unwrap();
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

#[macro_use]
extern crate derive_builder;
extern crate num;

pub mod area;
pub mod entry;
pub mod iter;
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
        iter::{IntoIter, Iter, Query, Regions, Values},
        point::Point,
        qtinner::QTInner,
        traversal::Traversal,
        types::StoreType,
    },
    num::PrimInt,
    std::{
        collections::{HashMap, HashSet},
        default::Default,
        hash::Hash,
    },
};

//   .d88b.  db    db  .d8b.  d8888b. d888888b d8888b. d88888b d88888b
//  .8P  Y8. 88    88 d8' `8b 88  `8D `~~88~~' 88  `8D 88'     88'
//  88    88 88    88 88ooo88 88   88    88    88oobY' 88ooooo 88ooooo
//  88    88 88    88 88~~~88 88   88    88    88`8b   88~~~~~ 88~~~~~
//  `8P  d8' 88b  d88 88   88 88  .8D    88    88 `88. 88.     88.
//   `Y88'Y8 ~Y8888P' YP   YP Y8888D'    YP    88   YD Y88888P Y88888P
//
// These headers are created by the *basic* style on https://www.messletters.com/en/big-text/.

/// A data structure for storing and accessing data in 2d space.
///
/// (A [Quadtree](https://en.wikipedia.org/wiki/Quadtree).)
///
/// `Quadtree<U, V>` is parameterized over
///  - `U`, the type of the coordinate, and
///  - `V`, the value being stored in the data structure.
///
// TODO(ambuc): Implement `.delete_by(anchor, dimensions, fn)`: `.retain()` is the inverse.
// TODO(ambuc): Implement `FromIterator<(K, V)>` for `Quadtree`.
#[derive(Debug, PartialEq, Eq)]
pub struct Quadtree<U, V>
where
    U: PrimInt + Default,
{
    inner: QTInner<U>,
    store: StoreType<U, V>,
}

impl<U, V> Quadtree<U, V>
where
    U: PrimInt + Default,
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
    /// // The anchor of a rectangular region is its top-left coordinate.
    /// // By default, quadtrees are anchored at (0, 0).
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
    /// // NB: Points can also be coerced, i.e. `(2, 4).into()`.
    ///
    /// assert_eq!(qt.depth(), 3);
    /// assert_eq!(qt.anchor(), Point {x: 2, y: 4});
    /// assert_eq!(qt.width(), 8);
    /// assert_eq!(qt.height(), 8);
    /// ```
    pub fn new_with_anchor(anchor: point::Point<U>, depth: usize) -> Self {
        Self {
            inner: QTInner::new(anchor, depth),
            store: HashMap::new(),
        }
    }

    /// The top-left corner of the region covered by the quadtree.
    pub fn anchor(&self) -> point::Point<U> {
        self.inner.region().anchor()
    }

    /// The width of the region covered by the quadtree.
    pub fn width(&self) -> usize {
        self.inner.region().width().to_usize().unwrap()
    }

    /// The height of the region covered by the quadtree.
    pub fn height(&self) -> usize {
        self.inner.region().height().to_usize().unwrap()
    }

    /// The depth of the quadtree.
    pub fn depth(&self) -> usize {
        self.inner.depth()
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
    pub fn contains(&self, area: Area<U>) -> bool {
        self.inner.region().contains(area)
    }

    /// Associates the passed value with the passed region.
    ///
    /// If the quadtree can contain the region, the value
    /// is inserted and a unique handle representing this
    /// instance of the object is returned.
    ///
    /// If the region cannot be wholly contained,
    /// `::insert()` will return an error.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, i8>::new(2);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 0}).dimensions((2,3)).build().unwrap();
    /// let handle_a_1 = qt.insert(region, 5).unwrap();
    /// let handle_a_2 = qt.insert(region, 5).unwrap();
    ///
    /// // Even though we inserted 5 at the same point in
    /// // the Quadtree, the two handles returned were not
    /// // the same.
    /// assert_ne!(handle_a_1, handle_a_2);
    /// ```
    pub fn insert(&mut self, region: Area<U>, val: V) -> Option<u64> {
        if self.contains(region) {
            return Some(
                self.inner
                    .insert_val_at_region(region, val, &mut self.store),
            );
        }
        None
    }

    /// Associates a value with a point. (An [`Area`] is
    /// really just a [`Point`] with dimensions `(1, 1)`,
    /// so the point still has to fit within the region.)
    ///
    /// ```
    /// use quadtree_rs::{point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, i8>::new(2);
    ///
    /// assert!(qt.insert_pt(Point { x: 1, y: 2 }, 5_i8).is_some());
    /// ```
    ///
    /// [`Area`]: area/struct.Area.html
    /// [`Point`]: point/struct.Point.html
    pub fn insert_pt(&mut self, point: Point<U>, val: V) -> Option<u64> {
        if let Ok(area) = AreaBuilder::default().anchor(point).build() {
            return self.insert(area, val);
        }
        None
    }

    /// Provides access to a single value in the Quadtree,
    /// given a previously known handle. This
    /// handle might have been saved by value at [`insert`].
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 1}).dimensions((2, 3)).build().unwrap();
    /// let handle = qt.insert(region, 9.87).unwrap();
    ///
    /// assert_eq!(qt.get(handle), Some(&9.87));
    /// ```
    ///
    /// [`insert`]: #method.insert
    pub fn get(&self, handle: u64) -> Option<&V> {
        self.store.get(&handle).and_then(|e| Some(e.value_ref()))
    }

    /// A mutable variant of [`.get()`].
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 1}).dimensions((2, 3)).build().unwrap();
    /// let handle: u64 = qt.insert(region, 9.87).unwrap();
    ///
    /// if let Some(val) = qt.get_mut(handle) {
    ///   *val += 1.0;
    /// }
    ///
    /// assert_eq!(qt.get(handle), Some(&10.87));
    ///
    /// ```
    ///
    /// [`.get()`]: #method.get
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
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▒▒▒░░    (2,1)->3x2
    /// // 2 ░░▒▒▒░░
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░    (1,4)->3x1
    /// // 5 ░░░░░░░
    /// let mut qt = Quadtree::<u32, i16>::new(4);
    /// let region_a = AreaBuilder::default()
    ///     .anchor(Point {x: 2, y: 1}).dimensions((3, 2)).build().unwrap();
    /// let region_b = AreaBuilder::default()
    ///     .anchor(Point {x: 1, y: 4}).dimensions((3, 1)).build().unwrap();
    /// assert!(qt.insert(region_a, 21).is_some());
    /// assert!(qt.insert(region_b, 57).is_some());
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▓▒▒░░  <-- Query over the region
    /// // 2 ░░▒▒▒░░      (2,1)->1x1
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░
    /// // 5 ░░░░░░░
    /// let region_c = AreaBuilder::default()
    ///     .anchor(Point {x: 2, y: 1}).build().unwrap();
    /// let mut query_a = qt.query(region_c);
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
    /// let region_d = AreaBuilder::default()
    ///     .anchor(Point {x: 1, y: 1}).dimensions((4, 4)).build().unwrap();
    /// let query_b = qt.query(region_d);
    ///
    /// // It's unspecified what order the regions should
    /// // return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    ///
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.query_strict()`]: #method.query_strict
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
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u8, f64>::new(3);
    ///
    /// // Insert 1.23 at (0,0)->1x1.
    /// let region_a = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 0}).build().unwrap();
    /// let handle = qt.insert(region_a, 1.23).unwrap();
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
    /// [`.modify()`]: #method.modify
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
    /// let region_a = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 0}).dimensions((2, 2)).build().unwrap();
    /// let region_b = AreaBuilder::default()
    ///     .anchor(Point {x: 1, y: 1}).dimensions((3, 2)).build().unwrap();
    /// assert!(qt.insert(region_a, 1.23).is_some());
    /// assert!(qt.insert(region_b, 4.56).is_some());
    ///
    /// //   0123
    /// // 0 ░░
    /// // 1 ░▓╳░  <-- ╳ is the deletion region
    /// // 2  ░░░
    ///
    /// let region_c = AreaBuilder::default()
    ///     .anchor(Point {x: 2, y: 1}).build().unwrap();
    /// let mut returned_entries = qt.delete(region_c);
    ///
    /// // We've removed one object from the Quadtree.
    /// assert_eq!(returned_entries.next().unwrap().value_ref(),
    ///            &4.56);
    ///
    /// // And left one behind.
    /// assert_eq!(qt.len(), 1);
    /// ```
    ///
    /// [`IntoIter<U, V>`]: iter/struct.IntoIter.html
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.delete_strict()`]: #method.delete_strict
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
        U: Hash,
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

    /// Returns an iterator ([`Iter<U, V>`]) over all [`&'a Entry<U, V>`]
    /// region/value associations in the Quadtree.
    ///
    /// [`Iter<U, V>`]: iter/struct.Iter.html
    /// [`&'a Entry<U, V>`]: entry/struct.Entry.html
    pub fn iter(&self) -> Iter<U, V> {
        Iter::new(&self.inner, &self.store)
    }

    /// Returns an iterator ([`Regions<U, V>`]) over all [`Area<U>`] regions
    /// in the Quadtree.
    ///
    /// [`Regions<U, V>`]: iter/struct.Regions.html
    /// [`Area<U>`]: area/struct.Area.html
    pub fn regions(&self) -> Regions<U, V> {
        Regions {
            inner: Iter::new(&self.inner, &self.store),
        }
    }

    /// Returns an iterator ([`Values<U, V>`]) over all `&'a V` values in the
    /// Quadtree.
    ///
    /// [`Values<U, V>`]: iter/struct.Values.html
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

/// `Extend<((U, U), V)>` will silently drop values whose coordinates do not fit in the region
/// represented by the Quadtree. It is the responsibility of the callsite to ensure these points
/// fit.
impl<U, V> Extend<(point::Type<U>, V)> for Quadtree<U, V>
where
    U: PrimInt + Default,
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
            );
        }
    }
}

// Immutable iterator for the Quadtree, returning by-reference.
impl<'a, U, V> IntoIterator for &'a Quadtree<U, V>
where
    U: PrimInt + Default,
{
    type Item = &'a Entry<U, V>;
    type IntoIter = Iter<'a, U, V>;

    fn into_iter(self) -> Iter<'a, U, V> {
        Iter::new(&self.inner, &self.store)
    }
}

impl<U, V> IntoIterator for Quadtree<U, V>
where
    U: PrimInt + Default,
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
