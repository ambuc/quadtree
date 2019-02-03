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

use {
    crate::{
        entry::Entry,
        geometry::{
            area::Area,
            point::{Point, PointType},
        },
        types::StoreType,
    },
    num::PrimInt,
    uuid::Uuid,
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct QTInner<U>
where
    U: PrimInt,
{
    // The depth of the current cell in its tree. Zero means it's at the very bottom.
    pub(crate) depth: usize,

    // The region  of the current cell.
    pub(crate) region: Area<U>,

    // The regions held at this level in the tree. (NB: That doesn't mean each value in `values`
    // is at self.region).
    pub(crate) kept_uuids: Vec<Uuid>,

    // The subquadrants under this cell. [ne, nw, se, sw]. If there are no subquadrants, this
    // entire list could be None.
    pub(crate) subquadrants: Option<[Box<QTInner<U>>; 4]>,
}

impl<U> std::fmt::Debug for QTInner<U>
where
    U: PrimInt + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.subquadrants.is_some() {
            write!(
                f,
                "{:?} :: {:?} {:#?}",
                self.region,
                self.kept_uuids,
                self.subquadrants.as_ref().unwrap()
            )
        } else {
            write!(f, "{:?} :: {:?}", self.region, self.kept_uuids,)
        }
    }
}

impl<U> QTInner<U>
where
    U: PrimInt,
{
    pub(crate) fn new(anchor: PointType<U>, depth: usize) -> QTInner<U> {
        let width: U = Self::two().pow(depth as u32);
        let height: U = width;
        Self::new_with_area((anchor, (width, height)).into(), depth)
    }

    fn new_with_area(region: Area<U>, depth: usize) -> QTInner<U> {
        QTInner {
            depth,
            region,
            kept_uuids: Vec::new(),
            subquadrants: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.kept_uuids.clear();
        self.subquadrants = None;
    }

    // Attempts to insert the value at the requested region. Returns false if the region was too
    // large.
    pub(crate) fn insert_val_at_region<V>(
        &mut self,
        req: Area<U>,
        val: V,
        store: &mut StoreType<U, V>,
    ) -> Uuid {
        let uuid = Uuid::new_v4();
        store.insert(uuid, Entry::new((req, val), uuid));
        self.insert_uuid_at_region(req, uuid, store);
        uuid.clone()
    }

    // Attempts to insert the value at the requested region. Returns false if the region was too
    // large.
    fn insert_uuid_at_region<V>(&mut self, req: Area<U>, uuid: Uuid, store: &mut StoreType<U, V>) {
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
            self.expand_subquadrants_by_pt(self.region.center_pt());
        }

        assert!(self.subquadrants.is_some()); // We should have Someified this in .split().

        if let Some(sqs) = self.subquadrants.as_mut() {
            for sq in sqs.iter_mut() {
                if sq.region.intersects(req) {
                    sq.insert_uuid_at_region(req, uuid, store);
                }
            }
        }
    }

    // a--+--+--+    +--+--+--+ // a <- self.region.anchor()
    // |        |    |     |  |
    // +     p  + => +--+--+--+ // p
    // |        |    |     |  |
    // +--+--+--+    +--+--+--+
    fn expand_subquadrants_by_pt(&mut self, p: Point<U>) {
        assert!(self.region.contains_pt(p));

        let anchor_nw: (U, U) = self.region.anchor().into();
        let anchor_ne: (U, U) = (p.x(), self.region.anchor().y());
        let anchor_sw: (U, U) = (self.region.anchor().x(), p.y());
        let anchor_se: (U, U) = p.into();

        self.subquadrants = Some([
            Box::new(Self::new(anchor_ne, self.depth - 1)),
            Box::new(Self::new(anchor_nw, self.depth - 1)),
            Box::new(Self::new(anchor_se, self.depth - 1)),
            Box::new(Self::new(anchor_sw, self.depth - 1)),
        ]);
    }

    // Strongly-typed alias for U::one() + U::One()
    fn two() -> U {
        U::one() + U::one()
    }
}
