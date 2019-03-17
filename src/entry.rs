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
    crate::geometry::{
        area::{Area, AreaType},
        point::PointType,
    },
    num::PrimInt,
};

/// Lightweight encapsulation representing a region/value association being returned by value from
/// the [`Quadtree`].
///
/// ```
/// use quadtree_rs::{Quadtree, entry::Entry};
///
/// let mut qt = Quadtree::<u32, f64>::new(4);
/// qt.insert((1, 1), (3, 2), 4.56);
///
/// // @returned_entries is of type IntoIter<u32, f64>.
/// let mut returned_entries  = qt.delete((2, 1), (1, 1));
///
/// let hit: Entry<u32, f64> = returned_entries.next().unwrap();
/// assert_eq!(hit.region(), ((1, 1), (3, 2)));
/// assert_eq!(hit.anchor(), (1, 1));
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
    U: PrimInt,
{
    region: Area<U>,
    value: V,
    handle: u64,
}
impl<U, V> Entry<U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(inner: (Area<U>, V), handle: u64) -> Entry<U, V> {
        Entry {
            region: inner.0,
            value: inner.1,
            handle,
        }
    }

    /// The held region, in standard (anchor, dimensions) form.
    pub fn region(&self) -> AreaType<U> {
        self.region.into()
    }

    pub(crate) fn area(&self) -> Area<U> {
        self.region
    }

    /// The top-left coordinate of the held region.
    pub fn anchor(&self) -> PointType<U> {
        self.region().0
    }

    fn dimensions(&self) -> (U, U) {
        self.region().1
    }

    /// The width of the held region.
    pub fn width(&self) -> U {
        self.dimensions().0
    }

    /// The height of the held region.
    pub fn height(&self) -> U {
        self.dimensions().1
    }

    pub fn value_mut<'a>(&'a mut self) -> &'a mut V {
        &mut self.value
    }

    /// The held value, returned by-reference.
    pub fn value_ref<'a>(&'a self) -> &'a V {
        &self.value
    }

    pub(crate) fn handle(&self) -> u64 {
        self.handle
    }
}
