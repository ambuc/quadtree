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

use crate::geometry::area::{Area, AreaType};
use crate::geometry::point::{Point, PointType};
use crate::types::{IntoIter, Iter, IterMut, Query, QueryMut, Regions, Values, ValuesMut};
use num::PrimInt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QTInner<U, V>
where
    U: PrimInt,
{
    // A reference to the top-level store.
    store: Rc<RefCell<HashMap<Uuid, (U, V)>>>,

    // The depth of the current cell in its tree. Zero means it's at the very bottom.
    depth: usize,

    // The region  of the current cell.
    region: Area<U>,

    // The regions held at this level in the tree. (NB: That doesn't mean each value in `values`
    // is at self.region).
    pub kept_values: Vec<(Area<U>, V)>,

    // The subquadrants under this cell. [ne, nw, se, sw]. If there are no subquadrants, this
    // entire list could be None.
    pub subquadrants: Option<[Box<QTInner<U, V>>; 4]>,
}

impl<U, V> QTInner<U, V>
where
    U: PrimInt,
{
    pub fn new(store: Rc<RefCell<HashMap<Uuid, (U, V)>>>, depth: usize) -> QTInner<U, V> {
        Self::new_with_anchor(store, Self::default_anchor(), depth)
    }

    pub fn new_with_anchor(
        store: Rc<RefCell<HashMap<Uuid, (U, V)>>>,
        anchor: PointType<U>,
        depth: usize,
    ) -> QTInner<U, V> {
        let width: U = Self::two().pow(depth as u32);
        let height: U = width;
        Self::new_with_area(store, (anchor, (width, height)).into(), depth)
    }

    pub fn anchor(&self) -> PointType<U> {
        self.region.anchor().into()
    }

    pub fn width(&self) -> U {
        self.region.width()
    }

    pub fn height(&self) -> U {
        self.region.height()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn len(&self) -> usize {
        self.kept_values.len()
            + self
                .subquadrants
                .as_ref()
                .map_or(0, |a| a.iter().map(|q| q.as_ref().len()).sum::<usize>())
    }

    pub fn is_empty(&self) -> bool {
        self.kept_values.is_empty()
            && self
                .subquadrants
                .as_ref()
                .map_or(true, |a| a.iter().all(|q| q.is_empty()))
    }

    pub fn contains(&self, anchor: PointType<U>, size: (U, U)) -> bool {
        self.contains_region((anchor, size).into())
    }

    pub fn insert(&mut self, anchor: PointType<U>, size: (U, U), val: V) -> bool {
        self.insert_region((anchor, size).into(), val)
    }

    pub fn insert_pt(&mut self, anchor: PointType<U>, val: V) -> bool {
        self.insert_region((anchor, Self::default_region_size()).into(), val)
    }

    pub fn query(&self, anchor: PointType<U>, size: (U, U)) -> Query<U, V> {
        assert!(!size.0.is_zero());
        assert!(!size.1.is_zero());
        self.query_by_area((anchor, size).into())
    }

    pub fn query_pt(&self, anchor: PointType<U>) -> Query<U, V> {
        self.query_by_area((anchor, Self::default_region_size()).into())
    }

    pub fn query_mut(&mut self, anchor: PointType<U>, size: (U, U)) -> QueryMut<U, V> {
        assert!(!size.0.is_zero());
        assert!(!size.1.is_zero());
        self.query_mut_by_area((anchor, size).into())
    }

    pub fn query_pt_mut(&mut self, anchor: PointType<U>) -> QueryMut<U, V> {
        self.query_mut_by_area((anchor, Self::default_region_size()).into())
    }

    pub fn reset(&mut self) {
        self.kept_values.clear();
        self.subquadrants = None;
    }

    pub fn iter(&self) -> Iter<U, V> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<U, V> {
        IterMut::new(self)
    }

    pub fn regions(&self) -> Regions<U, V> {
        Regions {
            inner: Iter::new(self),
        }
    }

    pub fn values(&self) -> Values<U, V> {
        Values {
            inner: Iter::new(self),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<U, V> {
        ValuesMut {
            inner: IterMut::new(self),
        }
    }

    fn new_with_area(
        store: Rc<RefCell<HashMap<Uuid, (U, V)>>>,
        region: Area<U>,
        depth: usize,
    ) -> QTInner<U, V> {
        QTInner {
            store,
            depth,
            region,
            kept_values: Vec::new(),
            subquadrants: None,
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
            assert!(req == self.region);
            self.kept_values.push((req, val));
            return true;
        }

        if req == self.region {
            self.kept_values.push((req, val));
            return true;
        }

        if self.subquadrants.is_none() {
            self.expand_subquadrants_by_center();
        }

        // For a subquadrant to totally contain the req. area, it must both (a) contain the req.
        // area's anchor and (b) contain the total area. We optimize by checking for (a) first.
        let q_index = (self.center_pt().dir_towards(req.anchor())) as usize;

        // Attempt to insert the value into the subquadrant we think it might fit in,
        assert!(self.subquadrants.is_some()); // We should have Someified this in .split().
        if let Some(sqs) = self.subquadrants.as_mut() {
            if sqs[q_index].contains_region(req) {
                sqs[q_index].insert_region(req, val);
            } else {
                self.kept_values.push((req, val));
            }
        }

        true
    }

    // +--+--+    +--+--+
    // |     |    |  |  |
    // +     + => +--+--+
    // |     |    |  |  |
    // +--+--+    +--+--+
    fn expand_subquadrants_by_center(&mut self) {
        self.expand_subquadrants_by_pt(self.center_pt());
    }

    // +--+--+--+    +--+--+--+
    // |        |    |     |  |
    // +     p  + => +--+--+--+
    // |        |    |     |  |
    // +--+--+--+    +--+--+--+
    // TODO(ambuc): Integrate this type with geometry::quadrant::Quadrant for higher type-safety.
    fn expand_subquadrants_by_pt(&mut self, p: Point<U>) {
        assert!(self.region.contains_pt(p));

        let anchor_nw = self.anchor();
        let anchor_ne = (p.x(), self.anchor_pt().y());
        let anchor_sw = (self.anchor_pt().x(), p.y());
        let anchor_se = p.into();

        self.subquadrants = Some([
            Box::new(Self::new_with_anchor(
                Rc::clone(&self.store),
                anchor_ne,
                self.depth - 1,
            )),
            Box::new(Self::new_with_anchor(
                Rc::clone(&self.store),
                anchor_nw,
                self.depth - 1,
            )),
            Box::new(Self::new_with_anchor(
                Rc::clone(&self.store),
                anchor_se,
                self.depth - 1,
            )),
            Box::new(Self::new_with_anchor(
                Rc::clone(&self.store),
                anchor_sw,
                self.depth - 1,
            )),
        ]);
    }

    fn query_by_area(&self, a: Area<U>) -> Query<U, V> {
        Query {
            query_region: a,
            inner: Iter::new(self),
        }
    }

    fn query_mut_by_area(&mut self, a: Area<U>) -> QueryMut<U, V> {
        QueryMut {
            query_region: a,
            inner: IterMut::new(self),
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
    fn default_anchor() -> PointType<U> {
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

/// `Extend<((U, U), V)>` will silently drop values whose coordinates do not fit in the region
/// represented by the Quadtree. It is the responsibility of the callsite to ensure these points
/// fit.
impl<U, V> Extend<(PointType<U>, V)> for QTInner<U, V>
where
    U: PrimInt,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (PointType<U>, V)>,
    {
        for (anchor, v) in iter {
            self.insert_pt(anchor, v);
        }
    }
}

/// `Extend<(((U, U), (U, U), V)>` will silently drop values whose coordinates do not fit in the
/// region represented by the Quadtree. It is the responsibility of the callsite to ensure these
/// points fit.
impl<U, V> Extend<(AreaType<U>, V)> for QTInner<U, V>
where
    U: PrimInt,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (AreaType<U>, V)>,
    {
        for ((anchor, dimensions), v) in iter {
            self.insert(anchor, dimensions, v);
        }
    }
}

// Immutable iterator for the Quadtree, returning by-reference.
impl<'a, U, V> IntoIterator for &'a QTInner<U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a V);
    type IntoIter = Iter<'a, U, V>;

    fn into_iter(self) -> Iter<'a, U, V> {
        self.iter()
    }
}

// Mutable iterator for the Quadtree, returning by-mutable-reference.
impl<'a, U, V> IntoIterator for &'a mut QTInner<U, V>
where
    U: PrimInt,
{
    type Item = (&'a AreaType<U>, &'a mut V);
    type IntoIter = IterMut<'a, U, V>;

    fn into_iter(self) -> IterMut<'a, U, V> {
        self.iter_mut()
    }
}

impl<U, V> IntoIterator for QTInner<U, V>
where
    U: PrimInt,
{
    type Item = (AreaType<U>, V);
    type IntoIter = IntoIter<U, V>;

    fn into_iter(self) -> IntoIter<U, V> {
        IntoIter::new(self)
    }
}
