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

//! [Point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree) implementation
//! for Rust.
//!
//! # Quick Start
//!
//! Add `quadtree_impl` to your `Cargo.toml`, and then add it to your main.
//! ```
//! extern crate quadtree_impl;
//!
//! // Create a new Quadtree with (u16, u16) x/y coordinates, String values, and a depth of four
//! // layers. Since 2^4 = 16, this grid will be of width and height 16.
//! let mut q = quadtree_impl::Quadtree::<u64, String>::new(4);
//!
//! // Insert "foo" in the coordinate system such that it occupies a rectangle with top-left
//! // "anchor" (0, 0), and width/height 2x1.
//! //
//! //   0  1  2  3
//! // 0 ░░░░░░░--+
//! //   ░░░░░░░ <--foo
//! // 1 ░░░░░░░--+
//! //   |  |  |  |
//! // 2 +--+--+--+
//! q.insert((0, 0), (2, 1), "foo".to_string());
//!
//! // Perform a query over a region with anchor (1, 0) and width/height 1x1...
//! //
//! //   0  1  2  3
//! // 0 ░░░▓▓▓▓▒▒▒
//! //   ░░░▓▓▓▓▒▒▒ <--query region
//! // 1 ░░░▓▓▓▓▒▒▒
//! //   |  ▒▒▒▒▒▒▒
//! // 2 +--▒▒▒▒▒▒▒
//! let mut query = q.query((1, 0), (2, 2));
//!
//! // There is an overlap between our query region and the region holding "foo",
//! // so we expect that iterator to return the `(coordinate, value)` pair containing "foo".
//! assert_eq!(query.next()
//!                 .map_or("", |(_coordinate, value)| value),
//!            "foo");
//! ```
//!
//! # Usage
//!
//! For usage details, see [`Quadtree`].
//!
//! [`Quadtree`]: struct.Quadtree.html

// For extra-pedantic documentation tests.
#![doc(test(attr(deny(warnings))))]

extern crate num;

mod geometry;
mod qtinner;

use crate::geometry::area::{Area, AreaType};
use crate::geometry::point::PointType;
use crate::qtinner::QTInner;
use num::PrimInt;
use std::collections::HashMap;
use std::iter::FusedIterator;
use uuid::Uuid;

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
/// Both points and regions are represented by the type
/// ```
/// type U = u64; // Or any primitive integer, signed or unsigned.
///
/// let _region: (/*    anchor=*/ (U, U),
///               /*dimensions=*/ (U, U)) = ((1, 2), (3, 4)); // (for example)
/// ```
/// where
///  - `anchor` is the x/y coordinate of the top-left corner, and
///  - `dimensions` is a tuple containing the width and height of the region.
///
/// Points have dimensions `(1, 1)`.
///
/// ## TODOs:
/// - Methods
///   - TODO(ambuc): Implement strictly inclusive getters.
///   - TODO(ambuc): Implement `.clear(anchor, size)`.
///   - TODO(ambuc): Implement `.delete(anchor, size)`.
///   - TODO(ambuc): Implement `.delete_by(anchor, size, fn)`.
///   - TODO(ambuc): Implement `.retain(anchor, size, fn)`.
/// - Traits
///   - TODO(ambuc): Implement `FromIterator<(K, V)>` for `Quadtree`.
/// - Other
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quadtree<U, V>
where
    U: PrimInt,
{
    inner: QTInner<U>,
    store: std::collections::HashMap<Uuid, (Area<U>, V)>,
}

impl<U, V> Quadtree<U, V>
where
    U: PrimInt,
{
    // Constructors //

    /// Creates a new, empty Quadtree with the requested depth.
    /// - The default anchor is `(0, 0)`, and the default width and height are both `2^depth`.
    /// - The Quadtree must be explicitly typed, since will contain items of a type.
    /// ```
    /// let q = quadtree_impl::Quadtree::<u32, u8>::new(/*depth=*/ 2);
    ///
    /// assert_eq!(q.depth(), 2);
    /// assert_eq!(q.anchor(), (0, 0));
    /// assert_eq!(q.width(), 4);
    /// assert_eq!(q.height(), 4);
    /// ```
    pub fn new(depth: usize) -> Quadtree<U, V> {
        Quadtree {
            inner: QTInner::new(Self::default_anchor(), depth),
            store: std::collections::HashMap::new(),
        }
    }

    /// Creates a new Quadtree with the requested anchor and depth.
    /// ```
    /// let q = quadtree_impl::Quadtree::<u32, u8>::new_with_anchor(/*anchor=*/ (2, 4),
    ///                                                             /* depth=*/ 3);
    ///
    /// assert_eq!(q.depth(), 3);
    /// assert_eq!(q.anchor(), (2, 4));
    /// assert_eq!(q.width(), 8);
    /// assert_eq!(q.height(), 8);
    /// ```
    pub fn new_with_anchor(anchor: PointType<U>, depth: usize) -> Quadtree<U, V> {
        Quadtree {
            inner: QTInner::new(anchor, depth),
            store: std::collections::HashMap::new(),
        }
    }

    // Accessors //

    /// The coordinate of the top-left corner of the represented region.
    pub fn anchor(&self) -> PointType<U> {
        self.inner.region.anchor().into()
    }

    /// The width of the represented region.
    pub fn width(&self) -> U {
        self.inner.region.width()
    }

    /// The height of the represented region.
    pub fn height(&self) -> U {
        self.inner.region.height()
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
    /// let mut q = quadtree_impl::Quadtree::<u32, f32>::new(4);
    /// assert_eq!(q.len(), 0);
    ///
    /// assert!(q.insert_pt((3, 1), 3.14159));
    /// assert_eq!(q.len(), 1);
    ///
    /// assert!(q.insert_pt((2, 7), 2.71828));
    /// assert_eq!(q.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        // self.inner.len()
        self.store.len()
    }

    /// Whether or not the quadtree is empty.
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, f64>::new(3);
    /// assert!(q.is_empty());
    ///
    /// q.insert((1, 4), (1, 4), 1.4142135);
    /// assert!(!q.is_empty());
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
    /// //  0  1  2  3  4
    /// // 0+--*******--+
    /// //  |  *******  |
    /// // 1+--*******--+
    /// //  |  *******  |
    /// // 2+--*******--+
    /// let q = quadtree_impl::Quadtree::<u32, u32>::new_with_anchor((1, 0), 1);
    ///
    /// assert!(q.contains((1, 0), (2, 2))); // q contains itself.
    ///
    /// // q contains a 1x1 region within it.
    /// //
    /// //  0  1  2  3  4
    /// // 0+--XXXX***--+
    /// //  |  XXXX***  |
    /// // 1+--XXXX***--+
    /// //  |  *******  |
    /// // 2+--*******--+
    /// assert!(q.contains((1, 0), (1, 1)));
    ///
    /// // But, q does not contain regions which are not totally within it.
    /// //
    /// //  0  1  2  3  4
    /// // 0XXXX******--+
    /// //  XXXX******  |
    /// // 1XXXX******--+
    /// //  |  *******  |
    /// // 2+--*******--+
    /// assert!(!q.contains((0, 0), (1, 1)));
    /// ```
    pub fn contains(&self, anchor: PointType<U>, size: (U, U)) -> bool {
        self.inner.region.contains((anchor, size).into())
    }

    /// Attempts to insert the value at the requested anchor and size. Returns false if the region
    /// was too large.
    ///
    /// The region described may have an anchor anywhere on the plane, but it
    /// must have positive, nonzero values for its width and height.
    ///
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i64>::new(2);
    ///
    /// // Returns true when the region fits within the represented region,
    /// assert!(q.insert((0, 0), (1, 1), 500000));
    ///
    /// // but returns false when it doesn't.
    /// assert!(!q.insert((0, 0), (5, 4), 27500));
    /// ```
    pub fn insert(&mut self, anchor: PointType<U>, size: (U, U), val: V) -> bool {
        self.inner
            .insert_val_at_region((anchor, size).into(), val, &mut self.store)
    }

    /// Attempts to insert the value at the given point. Returns false if the point was out of
    /// bounds.
    ///  - Expect the behavior of `.insert_pt(_, _)` to be the same as [`.insert(_, (1, 1), _)`].
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i64>::new(2);
    ///
    /// // Returns true when the point is within the represented region,
    /// assert!(q.insert_pt((0, 0),  8675309));
    ///
    /// // but returns false when it doesn't.
    /// assert!(!q.insert_pt((5, 4),  6060842));
    /// ```
    ///
    /// [`.insert(_, (1, 1), _)`]: struct.Quadtree.html#method.insert
    pub fn insert_pt(&mut self, anchor: PointType<U>, val: V) -> bool {
        self.inner.insert_val_at_region(
            (anchor, Self::default_region_size()).into(),
            val,
            &mut self.store,
        )
    }

    /// Returns an iterator over `(&'a ((U, U), (U, U)), &'a V)` tuples representing values
    /// within the query region.
    ///  - Values returned may either partially intersect or be wholly within the query region.
    ///
    /// The query region described may have an anchor anywhere on the plane, but it
    /// must have positive, nonzero values for its width and height.
    ///
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i16>::new(4);
    /// assert!(q.insert((0, 5), (7, 7), 21));
    /// assert!(q.insert((1, 3), (1, 3), 57));
    ///
    /// // Query over the region anchored at (0, 5) with area 1x1.
    /// let mut query_a = q.query((0, 5), (1, 1));
    /// assert_eq!(query_a.next(), Some((&((0, 5), (7, 7)), &21)));
    /// assert_eq!(query_a.next(), None);
    ///
    /// // Query over the region anchored at (0, 0) with area 6x6.
    /// let query_b = q.query((0, 0), (6, 6));
    ///
    /// // It's unclear what order the regions should return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    pub fn query(&self, anchor: PointType<U>, size: (U, U)) -> Query<U, V> {
        let a = (anchor, size).into();
        Query {
            query_region: a,
            inner: Iter::new(self.inner.lowest_node_totally_containing(a), &self.store),
        }
    }

    /// Returns an iterator (of type [`Query`]) over `(&'a ((U, U), (U, U)), &'a V)` tuples
    /// representing values intersecting the query point.
    ///
    /// Alias for [`.query(anchor, (1, 1))`].
    ///
    /// [`Query`]: struct.Query.html
    /// [`.query(anchor, (1, 1))`]: struct.Quadtree.html#method.query
    pub fn query_pt(&self, anchor: PointType<U>) -> Query<U, V> {
        let a = (anchor, Self::default_region_size()).into();
        Query {
            query_region: a,
            inner: Iter::new(self.inner.lowest_node_totally_containing(a), &self.store),
        }
    }

    // /// Returns a mutable iterator (of type [`QueryMut`]) over
    // /// `(&'a ((U, U), (U, U)), &'a mut V)` tuples representing values either
    // /// (a) wholly within or (b) intersecting the query region.
    // ///
    // /// The query region described may have an anchor anywhere on the plane, but it
    // /// must have positive, nonzero values for its width and height.
    // ///
    // /// ```
    // /// let mut q = quadtree_impl::Quadtree::<u32, i16>::new(4);
    // /// assert!(q.insert((0, 5), (7, 7), 21));
    // /// assert!(q.insert((1, 3), (1, 3), 57));
    // ///
    // /// // We can verify that the region at (0, 5)->7x7 has the value 21.
    // /// assert_eq!(q.query((0, 5), (1, 1)).next().unwrap().1, &21);
    // ///
    // /// // A mutable iterator lets us access the value in-place:
    // /// for (_, val) in q.query_mut((0, 5), (1, 1)) {
    // ///     *val = 1;
    // /// }
    // ///
    // /// // And we can verify that the changes took effect.
    // /// assert_eq!(q.query((0, 5), (1, 1)).next().unwrap().1, &1);
    // /// ```
    // ///
    // /// [`QueryMut`]: struct.QueryMut.html
    // pub fn query_mut(&mut self, anchor: PointType<U>, size: (U, U)) -> QueryMut<U, V> {
    //     self.inner.query_mut(anchor, size, &mut self.store)
    // }

    /// Resets the quadtree to a totally empty state.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.store.clear();
    }

    /// Returns an iterator over all `(&((U, U), (U, U)), &V)` region/value pairs in the
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

    // Strongly-typed alias for (one(), one()).
    fn default_region_size() -> (U, U) {
        (U::one(), U::one())
    }

    fn default_anchor() -> PointType<U> {
        (U::zero(), U::zero())
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
            self.insert_pt(pt, val);
        }
    }
}

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
        for ((anchor, size), val) in iter {
            self.insert(anchor, size, val);
        }
    }
}

// Immutable iterator for the Quadtree, returning by-reference.
impl<'a, U, V> IntoIterator for &'a Quadtree<U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);
    type IntoIter = Iter<'a, U, V>;

    fn into_iter(self) -> Iter<'a, U, V> {
        Iter::new(&self.inner, &self.store)
    }
}

impl<U, V> IntoIterator for Quadtree<U, V>
where
    U: PrimInt,
{
    type Item = (AreaType<U>, V);
    type IntoIter = IntoIter<U, V>;

    fn into_iter(self) -> IntoIter<U, V> {
        IntoIter::new(self.inner, self.store)
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
/// [`iter`]: ../struct.Quadtree.html#method.iter
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Iter<'a, U, V>
where
    U: PrimInt,
{
    uuid_stack: Vec<Uuid>,
    qt_stack: Vec<&'a QTInner<U>>,
    remaining: usize,
    store: &'a HashMap<Uuid, (Area<U>, V)>,
}

impl<'a, U, V> Iter<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(
        qt: &'a QTInner<U>,
        store: &'a HashMap<Uuid, (Area<U>, V)>,
    ) -> Iter<'a, U, V> {
        Iter {
            uuid_stack: vec![],
            qt_stack: vec![qt],
            remaining: qt.len(),
            store,
        }
    }
}

impl<'a, U, V> Iterator for Iter<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            self.remaining -= 1;
            let result = self.store.get(&uuid).unwrap();
            return Some((&result.0.inner(), &result.1));
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for uuid in qt.kept_uuids.iter() {
                self.uuid_stack.push(uuid.clone());
            }
            // Push my subquadrants onto the qt_stack too.
            if let Some(sqs) = qt.subquadrants.as_ref() {
                for sq in sqs.iter() {
                    self.qt_stack.push(sq);
                }
            }
            return self.next();
        }

        // Else there's nothing left to search.
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, U, V> FusedIterator for Iter<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Iter<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.remaining
    }
}

// d888888b d8b   db d888888b  .d88b.  d888888b d888888b d88888b d8888b.
//   `88'   888o  88 `~~88~~' .8P  Y8.   `88'   `~~88~~' 88'     88  `8D
//    88    88V8o 88    88    88    88    88       88    88ooooo 88oobY'
//    88    88 V8o88    88    88    88    88       88    88~~~~~ 88`8b
//   .88.   88  V888    88    `8b  d8'   .88.      88    88.     88 `88.
// Y888888P VP   V8P    YP     `Y88P'  Y888888P    YP    Y88888P 88   YD

/// A consuming iterator over all region/value pairs held in a [`Quadtree`].
///
/// TODO(ambuc): How is this created? `.into_iter()`? Find the right URL for it, if it's part of
/// IntoIterator.
///
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct IntoIter<U, V>
where
    U: PrimInt,
{
    uuid_stack: Vec<Uuid>,
    qt_stack: Vec<QTInner<U>>,
    remaining: usize,
    store: HashMap<Uuid, (Area<U>, V)>,
}

impl<U, V> IntoIter<U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(qt: QTInner<U>, store: HashMap<Uuid, (Area<U>, V)>) -> IntoIter<U, V> {
        let len = qt.len();
        IntoIter {
            uuid_stack: Vec::new(),
            qt_stack: vec![qt],
            remaining: len,
            store,
        }
    }
}

impl<U, V> Iterator for IntoIter<U, V>
where
    U: PrimInt,
{
    type Item = (AreaType<U>, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            self.remaining -= 1;
            let (_uuid, kv) = self.store.remove_entry(&uuid).unwrap();
            return Some((*kv.0.inner(), kv.1));
        }
        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for uuid in qt.kept_uuids.into_iter() {
                self.uuid_stack.push(uuid.clone());
            }
            // Push my subquadrants onto the qt_stack too.
            if let Some([a, b, c, d]) = qt.subquadrants {
                self.qt_stack.push(*a);
                self.qt_stack.push(*b);
                self.qt_stack.push(*c);
                self.qt_stack.push(*d);
            }
            return self.next();
        }

        // Else there's nothing left to search.
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<U, V> ExactSizeIterator for IntoIter<U, V>
where
    U: PrimInt,
{
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<U, V> FusedIterator for IntoIter<U, V> where U: PrimInt {}

//  .d88b.  db    db d88888b d8888b. db    db
// .8P  Y8. 88    88 88'     88  `8D `8b  d8'
// 88    88 88    88 88ooooo 88oobY'  `8bd8'
// 88    88 88    88 88~~~~~ 88`8b      88
// `8P  d8' 88b  d88 88.     88 `88.    88
//  `Y88'Y8 ~Y8888P' Y88888P 88   YD    YP

/// An iterator over the regions and values of a [`Quadtree`].
///
/// This struct is created by the [`query`] or [`query_pt`] methods on [`Quadtree`].
///
/// [`query`]: ../struct.Quadtree.html#method.query
/// [`query_pt`]: ../struct.Quadtree.html#method.query_pt
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Query<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) query_region: Area<U>,
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Query<'a, U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |(k, v)| {
            if self.query_region.intersects(k.into()) {
                Some((k, v))
            } else {
                self.next()
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Query<'a, U, V> where U: PrimInt {}

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
/// [`regions`]: ../struct.Quadtree.html#method.regions
/// [`Quadtree`]: ../struct.Quadtree.html
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
    type Item = (&'a AreaType<U>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map_or(None, |(k, _v)| Some(k))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Regions<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Regions<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

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
/// [`values`]: ../struct.Quadtree.html#method.values
/// [`Quadtree`]: ../struct.Quadtree.html
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
        self.inner.next().map_or(None, |(_k, v)| Some(v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, U, V> FusedIterator for Values<'a, U, V> where U: PrimInt {}

impl<'a, U, V> ExactSizeIterator for Values<'a, U, V>
where
    U: PrimInt,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}
