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
use crate::geometry::point::PointType;
use num::PrimInt;

/// Lightweight reference type representing a region/value pair in the [`Quadtree`].
///
/// ```
/// use quadtree_impl::Quadtree;
/// use quadtree_impl::entry::Entry;
///
/// let mut qt = Quadtree::<u32, f32>::new(4);
///
/// qt.insert((0, 5), (7, 7), 21.01);
///
/// // We can use the Entry API to destructure the result.
/// let entry: Entry<u32, f32> = qt.query((0, 5), (1, 1)).next().unwrap();
///
/// assert_eq!(entry.region(), &((0, 5), (7, 7)));
/// assert_eq!(entry.anchor(), &(0, 5));
/// assert_eq!(entry.width(), &7);
/// assert_eq!(entry.height(), &7);
/// assert_eq!(entry.value(), &21.01);
///
/// ```
///
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Entry<'a, U, V>
where
    U: PrimInt,
{
    inner: &'a (Area<U>, V),
}

impl<'a, U, V> Entry<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(inner: &'a (Area<U>, V)) -> Entry<'a, U, V> {
        Entry { inner }
    }

    /// The region held in this `Entry`, in standard `&'a ((U, U), (U, U))` form, where the first
    /// tuple are the x/y coordinates of the region's anchor point (in its top-left corner), and
    /// the second tuple are the width/height of the region.
    pub fn region(&self) -> &'a AreaType<U> {
        self.inner.0.inner()
    }
    /// The top-left coordinate of the region held in this `Entry`.
    pub fn anchor(&self) -> &'a PointType<U> {
        &self.region().0
    }
    fn dimensions(&self) -> &'a (U, U) {
        &self.region().1
    }
    pub fn width(&self) -> &'a U {
        &self.dimensions().0
    }
    pub fn height(&self) -> &'a U {
        &self.dimensions().1
    }
    /// The value held in this `Entry`.
    pub fn value(&self) -> &'a V {
        &self.inner.1
    }
    /// The `(region, value)` tuple held in this `Entry`.
    pub fn inner(&self) -> (&'a AreaType<U>, &'a V) {
        (self.region(), self.value())
    }
}
