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
        area::Area, entry::Entry, handle_iter::HandleIter, qtinner::QTInner, traversal::Traversal,
        types::StoreType,
    },
    num::PrimInt,
    std::iter::FusedIterator,
};

/// An iterator over all regions and values of a [`Quadtree`].
///
/// This struct is created by the [`iter`] method on [`Quadtree`].
///
/// [`iter`]: ../struct.Quadtree.html#method.iter
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Iter<'a, U, V>
where
    U: PrimInt + Default,
{
    store: &'a StoreType<U, V>,
    handle_iter: HandleIter<'a, U>,
}

impl<'a, U, V> Iter<'a, U, V>
where
    U: PrimInt + Default,
{
    pub(crate) fn new(qt: &'a QTInner<U>, store: &'a StoreType<U, V>) -> Iter<'a, U, V> {
        Iter {
            store,
            handle_iter: HandleIter::new(qt, qt.region()),
        }
    }
}

impl<'a, U, V> Iterator for Iter<'a, U, V>
where
    U: PrimInt + Default,
{
    type Item = &'a Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.handle_iter.next() {
            Some(handle) => Some(
                self.store
                    .get(&handle)
                    .expect("Shouldn't have an handle in the tree which isn't in the store."),
            ),
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.store.len()))
    }
}

impl<U, V> FusedIterator for Iter<'_, U, V> where U: PrimInt + Default {}

/// A consuming iterator over all region/value associations held in a [`Quadtree`].
///
/// This struct is created by the `into_iter()` method on the [`IntoIterator`] trait.
///
/// [`IntoIterator`]: ../struct.Quadtree.html#impl-IntoIterator
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Debug)]
pub struct IntoIter<U, V>
where
    U: PrimInt + Default,
{
    pub(crate) entries: Vec<Entry<U, V>>,
}

impl<U, V> Iterator for IntoIter<U, V>
where
    U: PrimInt + Default,
{
    type Item = Entry<U, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for IntoIter<U, V> where U: PrimInt + Default {}

/// An iterator over the regions and values of a [`Quadtree`].
///
/// This struct is created by the [`query`] method on [`Quadtree`].
///
/// [`query`]: ../struct.Quadtree.html#method.query
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Query<'a, U, V>
where
    U: PrimInt + Default,
{
    query_region: Area<U>,
    handle_iter: HandleIter<'a, U>,
    store: &'a StoreType<U, V>,
    traversal_method: Traversal,
}

impl<'a, U, V> Query<'a, U, V>
where
    U: PrimInt + Default,
{
    pub(crate) fn new(
        query_region: Area<U>,
        qt: &'a QTInner<U>,
        store: &'a StoreType<U, V>,
        traversal_method: Traversal,
    ) -> Query<'a, U, V>
    where
        U: PrimInt + Default,
    {
        // Construct the HandleIter first...
        let mut handle_iter = HandleIter::new(qt, query_region);

        // ...and descend it to the appropriate level. Depending on the type of @traversal_method,
        // this will potentially collect intersecting regions along the way. Avoiding combing the
        // entire Quadtree is essential for the efficiency of a query.
        handle_iter.query_optimization(query_region, traversal_method);

        Query {
            query_region,
            handle_iter,
            store,
            traversal_method,
        }
    }
}

impl<'a, U, V> Iterator for Query<'a, U, V>
where
    U: PrimInt + Default,
{
    type Item = &'a Entry<U, V>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for handle in self.handle_iter.by_ref() {
            if let Some(entry) = self.store.get(&handle) {
                if self.traversal_method.eval(entry.area(), self.query_region) {
                    return Some(entry);
                }
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.store.len()))
    }
}

impl<U, V> FusedIterator for Query<'_, U, V> where U: PrimInt + Default {}

/// An iterator over the values held within a [`Quadtree`].
///
/// This struct is created by the [`values`] method on [`Quadtree`].
///
/// [`values`]: ../struct.Quadtree.html#method.values
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Values<'a, U, V>
where
    U: PrimInt + Default,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Values<'a, U, V>
where
    U: PrimInt + Default,
{
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| e.value_ref())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Values<'_, U, V> where U: PrimInt + Default {}

/// An iterator over the regions held within a [`Quadtree`].
///
/// This struct is created by the [`regions`] method on [`Quadtree`].
///
/// [`regions`]: ../struct.Quadtree.html#method.regions
/// [`Quadtree`]: ../struct.Quadtree.html
#[derive(Clone, Debug)]
pub struct Regions<'a, U, V>
where
    U: PrimInt + Default,
{
    pub(crate) inner: Iter<'a, U, V>,
}

impl<'a, U, V> Iterator for Regions<'a, U, V>
where
    U: PrimInt + Default,
{
    type Item = Area<U>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| e.area())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U, V> FusedIterator for Regions<'_, U, V> where U: PrimInt + Default {}
