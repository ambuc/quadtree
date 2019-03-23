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

//! An entry in a Quadtree.

use {
    crate::{area::Area, point::Point},
    num::PrimInt,
};

/// Lightweight encapsulation representing a region/value
/// association being returned by value from the [`Quadtree`].
///
/// ```
/// use quadtree_rs::{Quadtree, entry::Entry, area::AreaBuilder};
///
/// let mut qt = Quadtree::<u32, f64>::new(4);
/// qt.insert(AreaBuilder::default().anchor((1, 1).into())
///                                 .dimensions((3, 2))
///                                 .build()
///                                 .unwrap(),
///           4.56);
///
/// // @returned_entries is of type IntoIter<u32, f64>.
/// let mut returned_entries  = qt.delete(
///     AreaBuilder::default().anchor((2, 1).into())
///                           .dimensions((1, 1))
///                           .build()
///                           .unwrap());
///
/// let hit: Entry<u32, f64> = returned_entries.next().unwrap();
/// assert_eq!(hit.anchor().x(), 1);
/// assert_eq!(hit.anchor().y(), 1);
/// assert_eq!(hit.width(), 3);
/// assert_eq!(hit.height(), 2);
///
/// // The held value can be accessed by reference.
/// assert_eq!(hit.value_ref(), &4.56);
/// ```
///
/// [`Quadtree`]: ../struct.Quadtree.html
// TODO(ambuc): Entry should hold Box<V> for better return-by-value semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry<U, V>
where
    U: PrimInt + std::default::Default,
{
    region: Area<U>,
    value: V,
    handle: u64,
}
impl<U, V> Entry<U, V>
where
    U: PrimInt + std::default::Default,
{
    pub(crate) fn new(inner: (Area<U>, V), handle: u64) -> Self {
        Self {
            region: inner.0,
            value: inner.1,
            handle,
        }
    }

    pub fn area(&self) -> Area<U> {
        self.region
    }

    /// The top-left coordinate of the held region.
    pub fn anchor(&self) -> Point<U> {
        self.area().anchor()
    }

    fn dimensions(&self) -> (U, U) {
        self.area().dimensions()
    }

    /// The width of the held region.
    pub fn width(&self) -> U {
        self.dimensions().0
    }

    /// The height of the held region.
    pub fn height(&self) -> U {
        self.dimensions().1
    }

    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    /// The held value, returned by-reference.
    pub fn value_ref(&self) -> &V {
        &self.value
    }

    pub(crate) fn handle(&self) -> u64 {
        self.handle
    }
}
