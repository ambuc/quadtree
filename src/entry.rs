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
use uuid::Uuid;

/// Lightweight encapsulation representing a region/value pair being returned by value from the
/// [`Quadtree`].
///
/// ```
/// use quadtree_impl::{IntoIter, Quadtree, entry::Entry};
///
/// let mut qt = Quadtree::<u32, f64>::new(4);
/// qt.insert((1, 1), (3, 2), 4.56);
///
/// let mut returned_entries: IntoIter<u32, f64> = qt.delete((2, 1), (1, 1));
///
/// let mut hit: Entry<u32, f64> = returned_entries.next().unwrap();
/// assert_eq!(hit.region(), ((1, 1), (3, 2)));
/// assert_eq!(hit.anchor(), (1, 1));
/// assert_eq!(hit.width(), 3);
/// assert_eq!(hit.height(), 2);
///
/// // The held value can be accessed by reference.
/// assert_eq!(hit.value_ref(), &4.56);
///
/// // The held value can be transferred out once:
/// let value: f64 = hit.value();
/// assert_eq!(value, 4.56);
///
/// // But the next time, it will have reverted to the default.
/// assert_ne!(hit.value_ref(), &4.56);
/// // TODO(ambuc): Entry should hold Box<V> for better return-by-value semantics.
///
/// ```
///
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Entry<U, V>
where
    U: PrimInt,
{
    region: Area<U>,
    value: V,
    uuid: Uuid,
}
impl<U, V> Entry<U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(inner: (Area<U>, V), uuid: Uuid) -> Entry<U, V> {
        Entry {
            region: inner.0,
            value: inner.1,
            uuid,
        }
    }

    /// The held region, in standard (anchor, dimensions) form.
    pub fn region(&self) -> AreaType<U> {
        *self.region.inner()
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

    /// The held value, returned by-value. `V` must implement `std::default::Default` (for now).
    pub fn value(&mut self) -> V
    where
        V: std::default::Default,
    {
        let elem = std::mem::replace(&mut self.value, V::default());
        elem
    }

    /// The held value, returned by-reference.
    pub fn value_ref<'a>(&'a self) -> &'a V {
        &self.value
    }

    /// The held (region, value) tuple.
    pub fn inner(&mut self) -> (AreaType<U>, V)
    where
        V: std::default::Default,
    {
        (self.region(), self.value())
    }
}

/// Lightweight reference type representing a region/value pair in the [`Quadtree`].
///
/// ```
/// use quadtree_impl::Quadtree;
/// use quadtree_impl::entry::EntryRef;
///
/// let mut qt = Quadtree::<u32, f32>::new(4);
///
/// qt.insert((0, 5), (7, 7), 21.01);
///
/// // We can use the Entry API to destructure the result.
/// let entry: EntryRef<u32, f32> = qt.query((0, 5), (1, 1)).next().unwrap();
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
pub struct EntryRef<'a, U, V>
where
    U: PrimInt,
{
    inner: &'a (Area<U>, V),
    uuid: Uuid,
}

impl<'a, U, V> EntryRef<'a, U, V>
where
    U: PrimInt,
{
    pub(crate) fn new(inner: &'a (Area<U>, V), uuid: Uuid) -> EntryRef<'a, U, V> {
        EntryRef { inner, uuid }
    }

    /// The held region, in standard (anchor, dimensions) form.
    pub fn region(&self) -> &'a AreaType<U> {
        self.inner.0.inner()
    }

    /// The top-left coordinate of the held region.
    pub fn anchor(&self) -> &'a PointType<U> {
        &self.region().0
    }

    fn dimensions(&self) -> &'a (U, U) {
        &self.region().1
    }

    /// The width of the held region.
    pub fn width(&self) -> &'a U {
        &self.dimensions().0
    }

    /// The height of the held region.
    pub fn height(&self) -> &'a U {
        &self.dimensions().1
    }

    /// The held value, returned by-reference.
    pub fn value(&self) -> &'a V {
        &self.inner.1
    }

    /// The held (region, value) tuple, returned by-reference.
    pub fn inner(&self) -> (&'a AreaType<U>, &'a V) {
        (self.region(), self.value())
    }

    pub(crate) fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }
}
