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
//! let mut query = q.get((1, 0), (2, 2));
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

extern crate failure;
extern crate num;
extern crate tool;

mod area;
mod point;

use crate::area::{Area, AreaType};
use crate::point::Point;

/// A data structure for storing and accessing data by x/y coordinates.
/// (A [Quadtree](https://en.wikipedia.org/wiki/Quadtree).)
///
/// `Quadtree<U, V>` is parameterized over
///  - `U`, where `U` is the index type of the x/y coordinate (an unsigned
/// primitive int), and
///  - `V`, where `V` is the value being stored in the data structure.
///
/// Both points and regions are represented by the type
/// ```
/// type U = u64; // Or any primitive unsigned integer.
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
///   - TODO(ambuc): Implement `.keys()`.
///   - TODO(ambuc): Implement `.values()`.
///   - TODO(ambuc): Implement `.values_mut()`.
///   - TODO(ambuc): Implement strictly inclusive getters.
///   - TODO(ambuc): Implement `.clear(anchor, size)`.
///   - TODO(ambuc): Implement `.delete(anchor, size)`.
///   - TODO(ambuc): Implement `.delete_by(anchor, size, fn)`.
///   - TODO(ambuc): Implement `.retain(anchor, size, fn)`.
/// - Traits
///   - TODO(ambuc): Implement `Eq` for `Quadtree`.
///   - TODO(ambuc): Implement `Extend<(K, V)>` for `Quadtree`.
///   - TODO(ambuc): Implement `FromIterator<(K, V)>` for `Quadtree`.
///   - TODO(ambuc): Implement `Intoiterator` for `Quadtree`.
///   - TODO(ambuc): Implement `Clone` for `Quadtree`.
///   - TODO(ambuc): Implement `Default` for `Quadtree`.
/// - Other
///   - TODO(ambuc): Parameterized `Quadtree<U, V>`, where `U` need not be `sign::Unsigned`.
pub struct Quadtree<U, V> {
    // The depth of the current cell in its tree. Zero means it's at the very bottom.
    depth: usize,
    // The region  of the current cell.
    region: Area<U>,
    // The regions held at this level in the tree.
    values: Vec<(Area<U>, V)>,
    // The subquadrants under this cell. [ne, nw, se, sw]
    subquadrants: [Option<Box<Quadtree<U, V>>>; 4],
}

impl<U, V> Quadtree<U, V>
where
    U: num::PrimInt + num_traits::sign::Unsigned,
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
        Quadtree::new_with_anchor(Self::default_anchor(), depth)
    }

    /// Creates a new Quadtree with the requested anchor and depth.
    /// ```
    /// let q = quadtree_impl::Quadtree::<u32, u8>::new_with_anchor(/*anchor=*/ (2, 4),
    ///                                                        /* depth=*/ 3);
    ///
    /// assert_eq!(q.depth(), 3);
    /// assert_eq!(q.anchor(), (2, 4));
    /// assert_eq!(q.width(), 8);
    /// assert_eq!(q.height(), 8);
    /// ```
    pub fn new_with_anchor(anchor: (U, U), depth: usize) -> Quadtree<U, V> {
        let width: U = Self::two().pow(depth as u32);
        let height: U = width;
        Quadtree::new_with_area((anchor, (width, height)).into(), depth)
    }

    // Accessors //

    /// The coordinate of the top-left corner of the represented region.
    pub fn anchor(&self) -> (U, U) {
        self.region.anchor().into()
    }

    /// The width of the represented region.
    pub fn width(&self) -> U {
        self.region.width()
    }

    /// The height of the represented region.
    pub fn height(&self) -> U {
        self.region.height()
    }

    /// The depth of the quadtree.
    /// - A quadtree created with depth 0 will have one node and no possibility for subdivision;
    /// - a quadtree created with depth 1 will have one node and four
    /// potential subquadrants.
    ///
    /// Thus both the width and height of a quadtree with depth `n` are `2^n`.
    pub fn depth(&self) -> usize {
        self.depth
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
        self.values.len()
            + self
                .subquadrants
                .iter()
                .map(|q| q.as_ref().map_or(0, |sq| sq.len()))
                .sum::<usize>()
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
        self.values.is_empty()
            && self
                .subquadrants
                .iter()
                .all(|q| q.as_ref().map_or(true, |sq| sq.is_empty()))
    }

    /// Whether or not the region represented by this quadtree could contain the given region.
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
    pub fn contains(&self, anchor: (U, U), size: (U, U)) -> bool {
        self.contains_region((anchor, size).into())
    }

    /// Attempts to insert the value at the requested anchor and size. Returns false if the region
    /// was too large.
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i64>::new(2);
    ///
    /// // Returns true when the region fits within the represented region,
    /// assert!(q.insert((0, 0), (1, 1), 500000));
    ///
    /// // but returns false when it doesn't.
    /// assert!(!q.insert((0, 0), (5, 4), 27500));
    /// ```
    pub fn insert(&mut self, anchor: (U, U), size: (U, U), val: V) -> bool {
        self.insert_region((anchor, size).into(), val)
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
    pub fn insert_pt(&mut self, anchor: (U, U), val: V) -> bool {
        self.insert_region((anchor, Self::default_region_size()).into(), val)
    }

    /// Returns an iterator over `(&'a ((U, U), (U, U)), &'a V)` tuples representing values
    /// within the query region.
    ///  - Values returned may either partially intersect or be wholly within the query region.
    ///  - Neither the height nor the width of the requested region can be zero. (i.e. the query
    ///  region must have area.)
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i16>::new(4);
    /// assert!(q.insert((0, 5), (7, 7), 21));
    /// assert!(q.insert((1, 3), (1, 3), 57));
    ///
    /// // Query over the region anchored at (0, 5) with area 1x1.
    /// let mut query_a = q.get((0, 5), (1, 1));
    /// assert_eq!(query_a.next(), Some((&((0, 5), (7, 7)), &21)));
    /// assert_eq!(query_a.next(), None);
    ///
    /// // Query over the region anchored at (0, 0) with area 6x6.
    /// let query_b = q.get((0, 0), (6, 6));
    ///
    /// // It's unclear what order the regions should return in, but there will be two of them.
    /// assert_eq!(query_b.count(), 2);
    /// ```
    pub fn get(&self, anchor: (U, U), size: (U, U)) -> Iter<U, V> {
        assert!(!size.0.is_zero());
        assert!(!size.1.is_zero());
        Iter::new(
            /*query_region=*/ (anchor, size).into(),
            /*len=*/ self.len(),
            /*qt=*/ self,
        )
    }

    /// Returns an iterator (of type [`Iter`]) over `(&'a ((U, U), (U, U)), &'a V)` tuples
    /// representing values intersecting the query point.
    ///
    /// Alias for [`.get(anchor, (1, 1))`].
    ///
    /// [`Iter`]: struct.Iter.html
    /// [`.get(anchor, (1, 1))`]: struct.Quadtree.html#method.get
    pub fn get_pt(&self, anchor: (U, U)) -> Iter<U, V> {
        self.get(anchor, Self::default_region_size())
    }

    /// Returns a mutable iterator (of type [`IterMut`]) over
    /// `(&'a ((U, U), (U, U)), &'a mut V)` tuples representing values either
    /// (a) wholly within or (b) intersecting the query region.
    ///
    ///  - The requested region must have area.
    ///  - Neither the height nor the width can be zero.
    /// ```
    /// let mut q = quadtree_impl::Quadtree::<u32, i16>::new(4);
    /// assert!(q.insert((0, 5), (7, 7), 21));
    /// assert!(q.insert((1, 3), (1, 3), 57));
    ///
    /// // We can verify that the region at (0, 5)->7x7 has the value 21.
    /// assert_eq!(q.get((0, 5), (1, 1)).next().unwrap().1, &21);
    ///
    /// // A mutable iterator lets us access the value in-place:
    /// for (_, val) in q.get_mut((0, 5), (1, 1)) {
    ///     *val = 1;
    /// }
    ///
    /// // And we can verify that the changes took effect.
    /// assert_eq!(q.get((0, 5), (1, 1)).next().unwrap().1, &1);
    /// ```
    ///
    /// [`IterMut`]: struct.IterMut.html
    pub fn get_mut(&mut self, anchor: (U, U), size: (U, U)) -> IterMut<U, V> {
        assert!(!size.0.is_zero());
        assert!(!size.1.is_zero());
        IterMut::new(
            /*query_region=*/ (anchor, size).into(),
            /*len=*/ self.len(),
            /*qt=*/ self,
        )
    }

    /// Returns a mutable iterator over `(&'a ((U, U), (U, U)), &'a mut V)` tuples
    /// representing values intersecting the query point.
    ///
    /// Alias for [`.get(anchor, (1, 1))`].
    ///
    /// [`.get(anchor, (1, 1))`]: struct.Quadtree.html#method.get
    pub fn get_pt_mut(&mut self, anchor: (U, U)) -> IterMut<U, V> {
        self.get_mut(anchor, Self::default_region_size())
    }

    /// Resets the quadtree to a totally empty state.
    pub fn reset(&mut self) {
        self.values.clear();
        self.subquadrants = [None, None, None, None];
    }

    // // // // // // // // // // // // // // // // // // // // // // // // // // // // // // // //
    // Private functions // // // // // // // // // // // // // // // // // // // // // // // // //
    // // // // // // // // // // // // // // // // // // // // // // // // // // // // // // // //

    fn new_with_area(region: Area<U>, depth: usize) -> Quadtree<U, V> {
        Quadtree {
            depth,
            region,
            values: Vec::new(),
            subquadrants: [None, None, None, None],
        }
    }

    // Attempts to insert the value at the requested region. Returns false if the region was too
    // large.
    fn insert_region(&mut self, req: Area<U>, val: V) -> bool {
        // If the requested region is larger than the region this cell represents, return false.
        if !self.region.contains(req) {
            return false;
        }

        // If we're at the bottom depth, it had better fit.
        if self.depth == 0 {
            self.values.push((req, val));
            return true;
        }

        // Otherwise we might attempt to insert the region one layer down instead.
        assert!(self.depth > 0);

        // For a subquadrant to totally contain the req. area, it must both (a) contain the req.
        // area's anchor and (b) contain the total area. We optimize by checking for (a) first.
        let q_index: usize = self.center_pt().dir_towards(req.anchor());

        self.expand_subquadrant_if_necessary(q_index);

        // Attempt to insert the value into the subquadrant we think it might fit in,
        match &mut self.subquadrants[q_index] {
            Some(subquadrant) => {
                // But if it doesn't fit in that subquadrant, insert it into our own @values vec.
                if subquadrant.contains_region(req) {
                    subquadrant.insert_region(req, val);
                } else {
                    self.values.push((req, val));
                }
            }
            _ => panic!("But you promised!"),
        }

        true
    }

    // For the given direction index, attempt the expand the subquadrant, if it's not already
    // expanded.
    fn expand_subquadrant_if_necessary(&mut self, dir: usize) {
        if self.subquadrants[dir].is_none() {
            let subquadrant_anchor: Point<U> = self.mk_subquadrant_anchor_pt(dir);
            self.subquadrants[dir] = Some(Box::new(Quadtree::new_with_anchor(
                (subquadrant_anchor.x(), subquadrant_anchor.y()),
                self.depth - 1,
            )));
        }
    }

    // Constructs the anchor of the requested subquadrant.
    // Accepts @direction as 0 (ne) 1 (nw) 2 (se) or 3 (sw).
    fn mk_subquadrant_anchor_pt(&mut self, direction: usize) -> Point<U> {
        // Construct cardinal anchor point:
        match direction {
            0 => self.anchor_pt() + (self.width() / Self::two(), U::zero()).into(),
            1 => self.anchor_pt() + (U::zero(), U::zero()).into(),
            2 => {
                self.anchor_pt() + (self.width() / Self::two(), self.height() / Self::two()).into()
            }
            3 => self.anchor_pt() + (U::zero(), self.height() / Self::two()).into(),
            _ => panic!("Don't send me a direction greater than 3."),
        }
    }

    fn contains_region(&self, a: Area<U>) -> bool {
        self.region.contains(a)
    }

    fn anchor_pt(&self) -> Point<U> {
        self.region.anchor()
    }

    fn center_pt(&self) -> Point<U> {
        self.anchor_pt()
            + Point::<U>::from((self.width() / Self::two(), self.height() / Self::two()))
    }

    // Strongly-typed alias for (zero(), zero()).
    fn default_anchor() -> (U, U) {
        (U::zero(), U::zero())
    }

    // Strongly-typed alias for (one(), one()).
    fn default_region_size() -> (U, U) {
        (U::one(), U::one())
    }

    // Strongly-typed alias for U::one() + U::One()
    fn two() -> U {
        U::one() + U::one()
    }
}

impl<U, V> std::fmt::Debug for Quadtree<U, V>
where
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// An iterator over the values of a [`Quadtree`].
///
/// This struct is created by the [`get`] or [`get_pt`] methods on [`Quadtree`].
///
/// # TODOs:
/// - Traits
///   - TODO(ambuc): Implement `FusedIterator` for `Iter<'a, V>`.
///   - TODO(ambuc): Implement `ExactSizeIterator` for `Iter<'a, V>`.
///   - TODO(ambuc): Implement `Clone` for `Iter<'a, V>`.
///
/// [`get`]: struct.Quadtree.html#method.get
/// [`get_pt`]: struct.Quadtree.html#method.get_pt
/// [`Quadtree`]: struct.Quadtree.html
#[derive(Clone)]
pub struct Iter<'a, U, V> {
    region_stack: Vec<(&'a Area<U>, &'a V)>,
    qt_stack: Vec<&'a Quadtree<U, V>>,
    query_region: Area<U>,
    upper_bound: usize,
    consumed: usize,
    exhausted: bool,
}

impl<'a, U, V> Iter<'a, U, V> {
    fn new(query_region: Area<U>, len: usize, qt: &'a Quadtree<U, V>) -> Iter<U, V> {
        Iter {
            region_stack: vec![],
            qt_stack: vec![qt],
            query_region,
            upper_bound: len,
            consumed: 0,
            exhausted: false,
        }
    }
}

impl<'a, U, V> Iterator for Iter<'a, U, V>
where
    U: num::PrimInt + num_traits::sign::Unsigned,
{
    type Item = (&'a AreaType<U>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the region_stack.
        if let Some((region, val)) = self.region_stack.pop() {
            self.consumed += 1;
            return Some((region.inner(), val));
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for (k, v) in qt.values.iter() {
                if k.intersects(self.query_region) {
                    self.region_stack.push((k, v));
                }
            }
            // Push my subquadrants onto the qt_stack too.
            for sq in qt.subquadrants.iter() {
                if let Some(sub_qt) = sq {
                    self.qt_stack.push(sub_qt);
                }
            }
            return self.next();
        }

        // Else there's nothing left to search.
        self.exhausted = true;
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.exhausted {
            (0, Some(0))
        } else {
            (0, Some(self.upper_bound - self.consumed))
        }
    }
}

impl<'a, U, V> std::fmt::Debug for Iter<'a, U, V>
where
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A mutable iterator over the values of a [`Quadtree`].
///
/// This struct is created by the [`get_mut`] or [`get_pt_mut`] methods on [`Quadtree`].
///
/// # TODOs:
/// - Traits
///  - TODO(ambuc): Implement `FusedIterator` for `IterMut<'a, V>`.
///  - TODO(ambuc): Implement `ExactSizeIterator` for `IterMut<'a, V>`.
///  - TODO(ambuc): Implement `Clone` for `IterMut<'a, V>`.
///
/// [`get_mut`]: struct.Quadtree.html#method.get_mut
/// [`get_pt_mut`]: struct.Quadtree.html#method.get_pt_mut
/// [`Quadtree`]: struct.Quadtree.html
pub struct IterMut<'a, U, V> {
    region_stack: Vec<(&'a Area<U>, &'a mut V)>,
    qt_stack: Vec<&'a mut Quadtree<U, V>>,
    query_region: Area<U>,
    upper_bound: usize,
    consumed: usize,
    exhausted: bool,
}

impl<'a, U, V> IterMut<'a, U, V> {
    fn new(query_region: Area<U>, len: usize, qt: &'a mut Quadtree<U, V>) -> IterMut<U, V> {
        IterMut {
            region_stack: vec![],
            qt_stack: vec![qt],
            query_region,
            upper_bound: len,
            consumed: 0,
            exhausted: false,
        }
    }
}

impl<'a, U, V> Iterator for IterMut<'a, U, V>
where
    U: num::PrimInt + num_traits::sign::Unsigned,
{
    type Item = (&'a AreaType<U>, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the region_stack.
        if let Some((region, val)) = self.region_stack.pop() {
            self.consumed += 1;
            return Some((region.inner(), val));
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            for (k, v) in qt.values.iter_mut() {
                if k.intersects(self.query_region) {
                    self.region_stack.push((k, v));
                }
            }
            // Push my subquadrants onto the qt_stack too.
            for sq in qt.subquadrants.iter_mut() {
                if let Some(sub_qt) = sq {
                    self.qt_stack.push(sub_qt);
                }
            }
            return self.next();
        }

        // Else there's nothing left to search.
        self.exhausted = true;
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.exhausted {
            (0, Some(0))
        } else {
            (0, Some(self.upper_bound - self.consumed))
        }
    }
}

impl<'a, U, V> std::fmt::Debug for IterMut<'a, U, V>
where
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
