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
    std::default::Default,
};

/// A region/value association being returned (by value) from the [`Quadtree`].
///
/// ```
/// use quadtree_rs::{area::AreaBuilder,
///                   entry::Entry,
///                   point::Point,
///                   Quadtree};
///
/// let mut qt = Quadtree::<u32, f64>::new(4);
/// assert!(
///   qt.insert(
///     /*region=*/AreaBuilder::default().anchor(Point {x: 1, y: 1})
///                                      .dimensions((3, 2))
///                                      .build()
///                                      .unwrap(),
///     /*val=*/4.56)
///   .is_ok());
///
/// // Calling Quadtree::delete() on a region in the tree
/// // clears that region of the tree and returns the
/// // region/value associations which were deleted.
///
/// let mut returned_entries = qt.delete(
///     /*region=*/AreaBuilder::default().anchor(Point {x: 2, y: 1})
///                                      .build()
///                                      .unwrap());
///
/// // The iterator contains Entry<U, V> structs.
/// let entry: Entry<u32, f64> = returned_entries.next().unwrap();
///
/// assert_eq!(entry.anchor().x(), 1);
/// assert_eq!(entry.anchor().y(), 1);
/// assert_eq!(entry.width(), 3);
/// assert_eq!(entry.height(), 2);
///
/// assert_eq!(entry.value_ref(), &4.56);
/// ```
///
/// [`Quadtree`]: ../struct.Quadtree.html
// TODO(ambuc): Entry should hold Box<V> for better return-by-value semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry<U, V>
where
    U: PrimInt + Default,
{
    region: Area<U>,
    value: V,
    handle: u64,
}
impl<U, V> Entry<U, V>
where
    U: PrimInt + Default,
{
    // pub

    /// The returned region.
    pub fn area(&self) -> Area<U> {
        self.region
    }

    /// The top-left coordinate of the returned region.
    pub fn anchor(&self) -> Point<U> {
        self.area().anchor()
    }

    /// The width of the returned region.
    pub fn width(&self) -> U {
        self.dimensions().0
    }

    /// The height of the returned region.
    pub fn height(&self) -> U {
        self.dimensions().1
    }

    /// A mutable accessor to the returned value.
    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    /// A reference to the returned value.
    pub fn value_ref(&self) -> &V {
        &self.value
    }

    // pub(crate)

    pub(crate) fn new(inner: (Area<U>, V), handle: u64) -> Self {
        Self {
            region: inner.0,
            value: inner.1,
            handle,
        }
    }

    pub(crate) fn dimensions(&self) -> (U, U) {
        self.area().dimensions()
    }

    pub(crate) fn handle(&self) -> u64 {
        self.handle
    }
}
