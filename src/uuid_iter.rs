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

use crate::qtinner::QTInner;
use num::PrimInt;
use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::FusedIterator;
use std::ops::Deref;
use uuid::Uuid;

// db    db db    db d888888b d8888b. d888888b d888888b d88888b d8888b.
// 88    88 88    88   `88'   88  `8D   `88'   `~~88~~' 88'     88  `8D
// 88    88 88    88    88    88   88    88       88    88ooooo 88oobY'
// 88    88 88    88    88    88   88    88       88    88~~~~~ 88`8b
// 88b  d88 88b  d88   .88.   88  .8D   .88.      88    88.     88 `88.
// ~Y8888P' ~Y8888P' Y888888P Y8888D' Y888888P    YP    Y88888P 88   YD

#[derive(Clone, Debug)]
pub struct UuidIter<'a, U>
where
    U: PrimInt + Debug,
{
    uuid_stack: Vec<&'a Uuid>,
    qt_stack: Vec<&'a QTInner<U>>,
    // TODO(ambuc): Fix @remaining.
    remaining: usize,
    visited: HashSet<Uuid>,
}
impl<'a, U> UuidIter<'a, U>
where
    U: PrimInt + Debug,
{
    pub(crate) fn new(qt: &'a QTInner<U>) -> UuidIter<'a, U> {
        UuidIter {
            uuid_stack: vec![],
            qt_stack: vec![qt],
            remaining: qt.len(),
            visited: HashSet::new(),
        }
    }
}

impl<'a, U> Iterator for UuidIter<'a, U>
where
    U: PrimInt + Debug,
{
    type Item = Uuid;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            if !self.visited.insert(uuid.clone()) {
                return self.next();
            }
            return Some(uuid.clone());
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            self.uuid_stack.extend(&qt.kept_uuids);

            // Push my subquadrants onto the qt_stack too.
            if let Some(sqs) = qt.subquadrants.as_ref() {
                self.qt_stack.extend(sqs.iter().map(|x| x.deref()));
            }
            return self.next();
        }

        // Else there's nothing left to search.
        return None;
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, U> FusedIterator for UuidIter<'a, U> where U: PrimInt + Debug {}

impl<'a, U> ExactSizeIterator for UuidIter<'a, U>
where
    U: PrimInt + Debug,
{
    fn len(&self) -> usize {
        self.remaining
    }
}
