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

use crate::{area::Area, qtinner::QTInner, traversal::Traversal};
use num::PrimInt;
use std::{collections::HashSet, default::Default, iter::FusedIterator};

#[derive(Clone, Debug)]
pub(crate) struct HandleIter<'a, U>
where
    U: PrimInt + Default,
{
    search_area: Area<U>,
    handle_stack: Vec<u64>,
    qt_stack: Vec<&'a QTInner<U>>,
    visited: HashSet<u64>,
}

impl<'a, U> HandleIter<'a, U>
where
    U: PrimInt + Default,
{
    pub(crate) fn new(qt: &'a QTInner<U>, search_area: Area<U>) -> Self {
        HandleIter {
            search_area,
            handle_stack: vec![],
            qt_stack: vec![qt],
            visited: HashSet::new(),
        }
    }

    // Descent is an optimization for queries. We don't want to traverse the entire tree searching
    // for handles which (mostly) correspond to regions our @req doesn't intersect with.
    //
    // Instead, we can make a beeline for the lowest region which totally contains the @req (but no
    // lower). We then have to actually evaluate every handle below that node.
    //
    // Along the way, if our query is meant to be of type Traversal::Overlapping, we collect the
    // handles we meet along the way. They are guaranteed to intersect @req.
    pub(crate) fn query_optimization(&mut self, req: Area<U>, traversal_method: Traversal) {
        // This method expects to be called at a point in time when the HandleIter has just been
        // created but has not yet been called.
        assert!(self.qt_stack.len() == 1);
        assert!(self.handle_stack.is_empty());
        assert!(self.visited.is_empty());

        self.descend_recurse_step(req, traversal_method);
    }

    fn descend_recurse_step(&mut self, req: Area<U>, traversal_method: Traversal) {
        assert!(self.qt_stack.len() == 1);
        // Peek into the stack. We have to peek rather than pop, because if we are about to go too
        // far down we'd rather stop and return the HandleIter as-is.
        if let Some(qt) = self.qt_stack.last() {
            // If the region doesn't contain our @req, we're already too far down. Return here.
            if !qt.region().contains(req) {
                return;
            }
            assert!(qt.region().contains(req));

            if let Some(subquadrants) = qt.subquadrants().as_ref() {
                for subquadrant in subquadrants.iter() {
                    // If we find a subquadrant which totally contains the @req, we want to make
                    // that our new sole qt.
                    if subquadrant.region().contains(req) {
                        if traversal_method == Traversal::Overlapping {
                            self.handle_stack.extend(qt.handles());
                        }

                        // TODO(ambuc): Could this be done with Vec::swap() or std::mem::replace()?
                        assert!(self.qt_stack.len() == 1);
                        self.qt_stack = vec![subquadrant];

                        // Recurse on this step. It will naturally return, but we want to propogate
                        // that return rather than continue to search the other subquadrants.
                        return self.descend_recurse_step(req, traversal_method);
                    }
                }
            }
            // If there aren't any subquadrants, we're probably done.
        }
    }
}

impl<U> Iterator for HandleIter<'_, U>
where
    U: PrimInt + Default,
{
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            while let Some(handle) = self.handle_stack.pop() {
                if self.visited.insert(handle) {
                    return Some(handle);
                }
            }

            // Then check the qt_stack.
            if let Some(qt) = self.qt_stack.pop() {
                // Push my sub quadrants onto the qt_stack too.
                if let Some(sub_quadrants) = qt.subquadrants().as_ref() {
                    for sub_quadrant in &**sub_quadrants {
                        if sub_quadrant.region().intersects(self.search_area) {
                            self.qt_stack.push(sub_quadrant)
                        }
                    }
                }

                // Push my regions onto the region stack
                match qt.handles().len() {
                    0 => (),
                    1 => {
                        if self.visited.insert(qt.handles()[0]) {
                            return Some(qt.handles()[0]);
                        }
                    }
                    _ => self.handle_stack.extend(qt.handles()),
                }

                continue;
            }

            // Else there's nothing left to search.
            return None;
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U> FusedIterator for HandleIter<'_, U> where U: PrimInt + Default {}
