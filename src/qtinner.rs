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

use crate::geometry::area::Area;
use crate::geometry::point::{Point, PointType};
use num::PrimInt;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QTInner<U>
where
    U: PrimInt + std::fmt::Debug,
{
    // The depth of the current cell in its tree. Zero means it's at the very bottom.
    pub(crate) depth: usize,

    // The region  of the current cell.
    pub(crate) region: Area<U>,

    // The regions held at this level in the tree. (NB: That doesn't mean each value in `values`
    // is at self.region).
    pub kept_uuids: Vec<Uuid>,

    // The subquadrants under this cell. [ne, nw, se, sw]. If there are no subquadrants, this
    // entire list could be None.
    pub subquadrants: Option<[Box<QTInner<U>>; 4]>,
}

impl<U> QTInner<U>
where
    U: PrimInt + std::fmt::Debug,
{
    pub fn new(anchor: PointType<U>, depth: usize) -> QTInner<U> {
        let width: U = Self::two().pow(depth as u32);
        let height: U = width;
        Self::new_with_area((anchor, (width, height)).into(), depth)
    }

    pub fn len(&self) -> usize {
        self.kept_uuids.len()
            + self
                .subquadrants
                .as_ref()
                .map_or(0, |a| a.iter().map(|q| q.as_ref().len()).sum::<usize>())
    }

    pub fn reset(&mut self) {
        self.kept_uuids.clear();
        self.subquadrants = None;
    }

    fn new_with_area(region: Area<U>, depth: usize) -> QTInner<U> {
        QTInner {
            depth,
            region,
            kept_uuids: Vec::new(),
            subquadrants: None,
        }
    }

    // Attempts to insert the value at the requested region. Returns false if the region was too
    // large.
    pub fn insert_val_at_region<V>(
        &mut self,
        req: Area<U>,
        val: V,
        store: &mut HashMap<Uuid, (Area<U>, V)>,
    ) where
        U: std::fmt::Debug,
    {
        let uuid = Uuid::new_v4();
        store.insert(uuid, (req, val));

        self.insert_uuid_at_region(req, uuid, store)
    }

    // Attempts to insert the value at the requested region. Returns false if the region was too
    // large.
    fn insert_uuid_at_region<V>(
        &mut self,
        req: Area<U>,
        uuid: Uuid,
        store: &mut HashMap<Uuid, (Area<U>, V)>,
    ) where
        U: std::fmt::Debug,
    {
        // If we're at the bottom depth, it had better fit.
        if self.depth == 0 {
            self.kept_uuids.push(uuid);
            return;
        }

        if req.contains(self.region) {
            self.kept_uuids.push(uuid);
            return;
        }

        if req == self.region {
            self.kept_uuids.push(uuid);
            return;
        }

        if self.subquadrants.is_none() {
            self.expand_subquadrants_by_center();
        }

        // For a subquadrant to totally contain the req. area, it must both (a) contain the req.
        // area's anchor and (b) contain the total area. We optimize by checking for (a) first.
        // let q_index = (self.center_pt().dir_towards(req.anchor())) as usize;

        // Attempt to insert the uuid into the subquadrant we think it might fit in,
        assert!(self.subquadrants.is_some()); // We should have Someified this in .split().
        if let Some(sqs) = self.subquadrants.as_mut() {
            for sq in sqs.iter_mut() {
                if sq.region.intersects(req) {
                    sq.insert_uuid_at_region(req, uuid, store);
                }
            }
        }
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

        let anchor_nw: (U, U) = self.region.anchor().into();
        let anchor_ne: (U, U) = (p.x(), self.anchor_pt().y());
        let anchor_sw: (U, U) = (self.anchor_pt().x(), p.y());
        let anchor_se: (U, U) = p.into();

        self.subquadrants = Some([
            Box::new(Self::new(anchor_ne, self.depth - 1)),
            Box::new(Self::new(anchor_nw, self.depth - 1)),
            Box::new(Self::new(anchor_se, self.depth - 1)),
            Box::new(Self::new(anchor_sw, self.depth - 1)),
        ]);
    }

    fn anchor_pt(&self) -> Point<U> {
        self.region.anchor()
    }

    fn center_pt(&self) -> Point<U> {
        self.anchor_pt()
            + Point::<U>::from((
                self.region.width() / Self::two(),
                self.region.height() / Self::two(),
            ))
    }

    // Strongly-typed alias for U::one() + U::One()
    fn two() -> U {
        U::one() + U::one()
    }
}
