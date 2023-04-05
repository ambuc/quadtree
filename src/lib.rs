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
//!     .anchor(Point {x: 0, y: 0})
//!     .dimensions((2, 1))
//!     .build().unwrap();
//! qt.insert(region_a, "foo".to_string());
//!
//! // Query over a region of size 2x2, anchored at (1, 0).
//! let region_b = AreaBuilder::default()
//!     .anchor(Point {x: 1, y: 0})
//!     .dimensions((2, 2))
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
//!     .anchor(Point {x: 0, y: 0})
//!     .dimensions((2, 2))
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
//!     .anchor(Point {x: 0, y: 0})
//!     .dimensions((3, 3))
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

/// A data structure for storing and accessing data in 2d space.
///
/// For historical context, other implementations, and potential uses of a
/// quadtree, see the [quadtree](https://en.wikipedia.org/wiki/Quadtree)
/// article on Wikipedia.
///
/// ## Parameterization
///
/// `Quadtree<U, V>` is parameterized over
///  - `U`, the type of the coordinate, and
///  - `V`, the value being stored.
///
/// `U` must implement `num::PrimInt` and a set of arithmetic operations necessary for coordinate
/// insertion and comparison. `U` must also implement `std::default` for [`derive_builder`]
/// semantics.
///
/// ## Strictness
///
/// Some methods ([`.query()`], [`.modify()`], and [`.delete()`]) have strict variants. While the
/// default behavior is for any operation to apply to all regions which _intersect_ some
/// operational region, the strict behavior is for the operation to apply only to those regions
/// which are _totally contained by_ the operational region.
///
/// [`derive_builder`]: https://docs.rs/derive_builder/0.7.0/derive_builder/
/// [`.query()`]: #method.query
/// [`.modify()`]: #method.modify
/// [`.delete()`]: #method.delete
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

    /// Creates a new, empty quadtree with some depth.
    /// A quadtree with depth `n` will accept coordinates in the range `[0, 2^n]`.
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

    /// Creates a new, empty quadtree with some depth and an explicit anchor.
    ///
    /// The anchor of a rectangular region is its upper-left coordinate. The
    /// anchor argument is of type [`point::Point`], and can either be
    /// explicit (`Point {x: 2, y: 4}`) or implicit (`(2, 4).into()`).
    ///
    /// [`point::Point`]: point/struct.Point.html
    /// ```
    /// use quadtree_rs::{point::Point, Quadtree};
    ///
    /// let anchor = Point {x: 2, y: 4};
    /// let depth = 3_usize;
    /// let qt = Quadtree::<u32, u8>::new_with_anchor(anchor, depth);
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

    /// The top-left corner (anchor) of the region which this quadtree represents.
    pub fn anchor(&self) -> point::Point<U> {
        self.inner.region().anchor()
    }

    /// The width of the region which this quadtree represents.
    pub fn width(&self) -> usize {
        self.inner.region().width().to_usize().unwrap()
    }

    /// The height of the region which this quadtree represents.
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

    /// Whether or not some trial region could fit in the region which this quadtree represents.
    pub fn contains(&self, area: Area<U>) -> bool {
        self.inner.region().contains(area)
    }

    /// Associate some value with a region in the quadtree.
    ///
    /// If insertion is successful, returns a unique handle to the value.
    ///
    /// If the region is too large for, or doesn't overlap with, the region which this quadtree
    /// represents, returns `None`.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, i8>::new(8);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 4, y: 5})
    ///     .dimensions((2,3))
    ///     .build().unwrap();
    ///
    /// let handle_a_1 = qt.insert(region, 5).unwrap();
    /// let handle_a_2 = qt.insert(region, 5).unwrap();
    ///
    /// // Even though we inserted 5 at the same point in the quadtree, the
    /// // two handles returned were not the same.
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

    /// Alias for [`.insert()`] which expects a [`Point`] instead of an [`Area`].
    ///
    /// (An [`Area`] is really just a [`Point`] with dimensions `(1, 1)`, so
    /// the point still has to fit within the region.)
    ///
    /// ```
    /// use quadtree_rs::{point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, i8>::new(2);
    ///
    /// assert!(qt.insert_pt(Point { x: 1, y: 2 }, 5_i8).is_some());
    /// ```
    ///
    /// [`.insert()`]: #method.insert
    /// [`Area`]: area/struct.Area.html
    /// [`Point`]: point/struct.Point.html
    pub fn insert_pt(&mut self, point: Point<U>, val: V) -> Option<u64> {
        if let Ok(area) = AreaBuilder::default().anchor(point).build() {
            return self.insert(area, val);
        }
        None
    }

    /// Given the handle from an [`.insert()`] operation, provides read-only
    /// access to the associated [`Entry<U, V>`] struct.
    ///
    /// Handles are unique and never re-used, so lookup of a handle to a now-deleted entry can
    /// fail and return `None`.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 1})
    ///     .dimensions((2, 3))
    ///     .build().unwrap();
    /// let handle = qt.insert(region, 9.87).unwrap();
    ///
    /// let entry = qt.get(handle).unwrap();
    /// assert_eq!(entry.value_ref(), &9.87);
    /// ```
    ///
    /// [`.insert()`]: #method.insert
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    pub fn get(&self, handle: u64) -> Option<&Entry<U, V>> {
        self.store.get(&handle)
    }

    /// A mutable variant of [`.get()`] which provides mutable access to the
    /// associated [`Entry<U, V>`] struct.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f32>::new(4);
    ///
    /// let region = AreaBuilder::default()
    ///     .anchor(Point {x: 0, y: 1})
    ///     .dimensions((2, 3))
    ///     .build().unwrap();
    /// let handle: u64 = qt.insert(region, 9.87).unwrap();
    ///
    /// if let Some(entry) = qt.get_mut(handle) {
    ///   *entry.value_mut() += 1.0;
    /// }
    ///
    /// assert_eq!(qt.get(handle).unwrap().value_ref(), &10.87);
    ///
    /// ```
    ///
    /// [`.get()`]: #method.get
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    pub fn get_mut(&mut self, handle: u64) -> Option<&mut Entry<U, V>> {
        self.store.get_mut(&handle)
    }

    /// Returns an iterator over [`&Entry<U, V>`] structs representing values
    /// within the query region.
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, Quadtree};
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▒▒▒░░    (2,1)->3x2
    /// // 2 ░░▒▒▒░░
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░    (1,4)->3x1
    /// // 5 ░░░░░░░
    /// let mut qt = Quadtree::<u32, char>::new(4);
    ///
    /// let region_a = AreaBuilder::default()
    ///     .anchor((2, 1).into())
    ///     .dimensions((3, 2))
    ///     .build().unwrap();
    /// qt.insert(region_a, 'a');
    ///
    /// let region_b = AreaBuilder::default()
    ///     .anchor((1, 4).into())
    ///     .dimensions((3, 1))
    ///     .build().unwrap();
    /// qt.insert(region_b, 'b');
    ///
    /// //   0123456
    /// // 0 ░░░░░░░
    /// // 1 ░░▓▒▒░░  <-- Query over the region
    /// // 2 ░░▒▒▒░░      (2,1)->1x1
    /// // 3 ░░░░░░░
    /// // 4 ░▒▒▒░░░
    /// // 5 ░░░░░░░
    /// let region_c = AreaBuilder::default()
    ///     .anchor((2, 1).into()).build().unwrap();
    /// let mut query_a = qt.query(region_c);
    ///
    /// // We can use the Entry API to destructure the result.
    /// let entry = query_a.next().unwrap();
    /// assert_eq!(entry.area().height(), 2);
    /// assert_eq!(entry.value_ref(), &'a');
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
    ///     .anchor((1, 1).into())
    ///     .dimensions((4, 4))
    ///     .build().unwrap();
    /// let query_b = qt.query(region_d);
    ///
    /// // It's unspecified what order the regions should
    /// // return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    ///
    /// [`&Entry<U, V>`]: entry/struct.Entry.html
    /// [`.query()`]: #method.query
    // TODO(ambuc): Settle on a stable return order to avoid breaking callers.
    pub fn query(&self, area: Area<U>) -> Query<U, V> {
        Query::new(area, &self.inner, &self.store, Traversal::Overlapping)
    }

    /// A strict variant of [`.query()`].
    ///
    /// [`.query()`]: #method.query
    pub fn query_strict(&self, area: Area<U>) -> Query<U, V> {
        Query::new(area, &self.inner, &self.store, Traversal::Strict)
    }

    /// Accepts a modification lambda and applies it to all elements in the
    /// quadtree which intersecting the described region.
    ///
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, Quadtree};
    ///
    /// let mut qt = Quadtree::<u8, bool>::new(3);
    ///
    /// let region_a = AreaBuilder::default()
    ///     .anchor((0, 0).into())
    ///     .build().unwrap();
    /// let handle = qt.insert(region_a, true).unwrap();
    ///
    /// // Run a modification lambda over all values in region_a...
    /// qt.modify(region_a, |i| *i = false);
    ///
    /// // ...and verify that the value was applied.
    /// assert_eq!(qt.get(handle).unwrap().value_ref(), &false);
    /// ```
    pub fn modify<F>(&mut self, area: Area<U>, f: F)
    where
        F: Fn(&mut V) + Copy,
    {
        self.modify_region(|a| a.intersects(area), f);
    }

    /// A strict variant of [`.modify()`].
    ///
    /// [`.modify()`]: #method.modify
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
            f(entry.value_mut());
        }
    }

    /// Resets the quadtree to a totally empty state.
    pub fn reset(&mut self) {
        self.store.clear();
        self.inner.reset();
    }

    /// Deletes all value associations which overlap a region in the tree.
    ///
    /// Along the way, consumed [`Entry<U, V>`] entries are collected and returned in an iterator
    /// [`IntoIter<U, V>`].
    /// ```
    /// use quadtree_rs::{area::AreaBuilder, Quadtree};
    ///
    /// let mut qt = Quadtree::<u32, f64>::new(4);
    ///
    /// let region_a = AreaBuilder::default()
    ///     .anchor((0, 0).into())
    ///     .dimensions((2, 2))
    ///     .build().unwrap();
    /// qt.insert(region_a, 1.23);
    ///
    /// let region_b = AreaBuilder::default()
    ///     .anchor((1, 1).into())
    ///     .dimensions((3, 2))
    ///     .build().unwrap();
    /// qt.insert(region_b, 4.56);
    ///
    /// //   0123
    /// // 0 ░░
    /// // 1 ░▓╳░  <-- ╳ is the deletion region
    /// // 2  ░░░
    ///
    /// let region_c = AreaBuilder::default()
    ///     .anchor((2, 1).into()).build().unwrap();
    /// let mut returned_entries = qt.delete(region_c);
    ///
    /// // We've removed one object from the quadtree.
    /// assert_eq!(returned_entries.next().unwrap().value_ref(),
    ///            &4.56);
    ///
    /// // And left one behind.
    /// assert_eq!(qt.len(), 1);
    /// ```
    ///
    /// [`IntoIter<U, V>`]: iter/struct.IntoIter.html
    /// [`Entry<U, V>`]: entry/struct.Entry.html
    /// [`.delete()`]: #method.delete
    pub fn delete(&mut self, area: Area<U>) -> IntoIter<U, V> {
        self.delete_handles_and_return(self.query(area).map(|e| e.handle()).collect())
    }

    /// A strict variant of [`.delete()`].
    ///
    /// [`.delete()`]: #method.delete
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
            entries.push(self.store.remove(u).expect(error));
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
        let relevant_handles: Vec<u64> =
            HandleIter::new(&self.inner, self.inner.region()).collect();
        for i in relevant_handles {
            if let Some(entry) = self.store.get_mut(&i) {
                if filter(entry.area()) {
                    modify(entry.value_mut());
                }
            }
        }
    }
}

/// `Extend<((U, U), V)>` will silently drop values whose coordinates do not fit in the region
/// represented by the Quadtree. It is the responsibility of the callsite to ensure these points
/// fit.
impl<U, V> Extend<((U, U), V)> for Quadtree<U, V>
where
    U: PrimInt + Default,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = ((U, U), V)>,
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
            entries: self.store.into_values().collect(),
        }
    }
}
